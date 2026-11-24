//! `OxvmmEngine`: runs each machine in its own `oxvmm` process (the Rust VMM on KVM).
//!
//! Guest serial output goes to `<folder>/logs/serial.log` and VMM diagnostics to
//! `<folder>/logs/oxvmm.log`. A guest that powers off or reboots ends its process and the machine
//! becomes `powered_off`; an unexpected process exit makes it `aborted`. Machines do not outlive the
//! daemon yet: their processes are killed when `oxboxd` exits.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use futures::stream::BoxStream;
use oxbox_model::{Action, Arch, EngineKind, MachineManifest, MachineState, OxeCode};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    ApplyMode, Engine, EngineCaps, EngineError, EngineEvent, HostInfo, OpHandle, StartOpts,
    StopMode, apply_mode_for,
};

/// The line `oxvmm` prints on stderr once the guest is about to run (see `apps/oxvmm`).
const READY_LINE: &str = "oxvmm: guest running";
const READY_TIMEOUT: Duration = Duration::from_secs(30);

struct Machine {
    manifest: MachineManifest,
    folder: PathBuf,
    state: MachineState,
    /// Present while a process runs: cancel it to kill the VM.
    kill: Option<CancellationToken>,
    /// Resolved when the running process has exited and the state is final.
    exited: Option<tokio::sync::watch::Receiver<bool>>,
}

struct Inner {
    binary: PathBuf,
    machines: Mutex<HashMap<Uuid, Machine>>,
    events: broadcast::Sender<EngineEvent>,
}

impl Inner {
    fn set_state(&self, id: Uuid, state: MachineState) {
        if let Some(m) = self.machines.lock().unwrap().get_mut(&id) {
            m.state = state;
        }
        let _ = self.events.send(EngineEvent::StateChanged { id, state });
    }
}

pub struct OxvmmEngine {
    inner: Arc<Inner>,
}

fn failed(code: OxeCode, message: impl Into<String>) -> EngineError {
    EngineError::Failed {
        code: code.code.into(),
        message: message.into(),
    }
}

fn resolve(folder: &Path, path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        folder.join(p)
    }
}

/// The `oxvmm` command line for a machine, or why it cannot run.
fn vmm_args(m: &MachineManifest, folder: &Path) -> Result<Vec<String>, EngineError> {
    let boot = m.boot.as_ref().ok_or_else(|| {
        failed(
            OxeCode::ENG_VM_CONFIG,
            "no kernel to boot: set one with `oxctl vm set <vm> --kernel <path>`",
        )
    })?;
    if m.cpu.count != 1 {
        return Err(failed(
            OxeCode::ENG_VM_CONFIG,
            format!(
                "oxvmm boots one vCPU for now; the machine has {} (`oxctl vm set <vm> --cpus 1`)",
                m.cpu.count
            ),
        ));
    }
    if m.platform.arch != Arch::X86_64 {
        return Err(failed(
            OxeCode::ENG_VM_CONFIG,
            "oxvmm runs x86_64 guests only for now",
        ));
    }
    let kernel = resolve(folder, &boot.kernel);
    if !kernel.is_file() {
        return Err(failed(
            OxeCode::ENG_VM_CONFIG,
            format!("kernel {} not found", kernel.display()),
        ));
    }
    let mut args = vec![
        "--kernel".into(),
        kernel.to_string_lossy().into_owned(),
        "--memory-mib".into(),
        m.memory.size_mib.to_string(),
        "--cpus".into(),
        m.cpu.count.to_string(),
        "--serial".into(),
        folder
            .join("logs")
            .join("serial.log")
            .to_string_lossy()
            .into_owned(),
    ];
    if let Some(initrd) = &boot.initrd {
        let initrd = resolve(folder, initrd);
        if !initrd.is_file() {
            return Err(failed(
                OxeCode::ENG_VM_CONFIG,
                format!("initrd {} not found", initrd.display()),
            ));
        }
        args.extend(["--initrd".into(), initrd.to_string_lossy().into_owned()]);
    }
    if let Some(cmdline) = &boot.cmdline {
        args.extend(["--cmdline".into(), cmdline.clone()]);
    }
    Ok(args)
}

impl OxvmmEngine {
    /// `binary` is the `oxvmm` executable.
    pub fn new(binary: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                binary,
                machines: Mutex::new(HashMap::new()),
                events: broadcast::channel(1024).0,
            }),
        }
    }

    /// Whether this host can run oxvmm machines, with the reason when it cannot.
    pub fn host_check(binary: &Path) -> Result<(), String> {
        if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            return Err("oxvmm runs on Linux x86_64 hosts only for now".into());
        }
        if !binary.is_file() {
            return Err(format!("oxvmm binary not found at {}", binary.display()));
        }
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/kvm")
            .map(|_| ())
            .map_err(|e| format!("cannot open /dev/kvm: {e} (is virtualization enabled and the user in the `kvm` group?)"))
    }

    fn get(&self, id: Uuid) -> Result<MachineState, EngineError> {
        self.inner
            .machines
            .lock()
            .unwrap()
            .get(&id)
            .map(|m| m.state)
            .ok_or(EngineError::NotFound(id))
    }

    /// Check and enter the transient state atomically.
    fn begin(
        &self,
        id: Uuid,
        action: Action,
        transient: MachineState,
    ) -> Result<MachineState, EngineError> {
        let mut machines = self.inner.machines.lock().unwrap();
        let m = machines.get_mut(&id).ok_or(EngineError::NotFound(id))?;
        if !m.state.allows(action) {
            return Err(EngineError::InvalidState(m.state));
        }
        let from = m.state;
        m.state = transient;
        drop(machines);
        let _ = self.inner.events.send(EngineEvent::StateChanged {
            id,
            state: transient,
        });
        Ok(from)
    }

    async fn spawn(&self, id: Uuid, op: &OpHandle) -> Result<(), EngineError> {
        let (manifest, folder) = {
            let machines = self.inner.machines.lock().unwrap();
            let m = machines.get(&id).ok_or(EngineError::NotFound(id))?;
            (m.manifest.clone(), m.folder.clone())
        };
        let args = vmm_args(&manifest, &folder)?;
        let logs = folder.join("logs");
        tokio::fs::create_dir_all(&logs)
            .await
            .map_err(|e| failed(OxeCode::ENG_FAILED, format!("{}: {e}", logs.display())))?;
        let mut vmm_log = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs.join("oxvmm.log"))
            .await
            .map_err(|e| failed(OxeCode::ENG_FAILED, e.to_string()))?;

        op.progress(10, "launching oxvmm");
        let mut child: Child = Command::new(&self.inner.binary)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                failed(
                    OxeCode::ENG_FAILED,
                    format!("cannot run {}: {e}", self.inner.binary.display()),
                )
            })?;
        let mut stderr = BufReader::new(child.stderr.take().expect("piped")).lines();

        // Wait for the ready line, keeping the log and the last line for error reports.
        op.progress(30, "booting");
        let mut last = String::new();
        let ready = tokio::time::timeout(READY_TIMEOUT, async {
            loop {
                tokio::select! {
                    _ = op.cancel.cancelled() => return Err(EngineError::Cancelled),
                    line = stderr.next_line() => match line {
                        Ok(Some(line)) => {
                            let _ = vmm_log.write_all(format!("{line}\n").as_bytes()).await;
                            if line == READY_LINE {
                                return Ok(());
                            }
                            last = line;
                        }
                        _ => return Err(failed(OxeCode::ENG_FAILED, format!("oxvmm exited during setup: {last}"))),
                    },
                }
            }
        })
        .await
        .unwrap_or_else(|_| Err(failed(OxeCode::ENG_FAILED, "oxvmm did not become ready within 30 s")));
        if let Err(e) = ready {
            let _ = child.kill().await;
            return Err(e);
        }

        // Supervise the running VM.
        let kill = CancellationToken::new();
        let (exited_tx, exited_rx) = tokio::sync::watch::channel(false);
        {
            let mut machines = self.inner.machines.lock().unwrap();
            if let Some(m) = machines.get_mut(&id) {
                m.kill = Some(kill.clone());
                m.exited = Some(exited_rx);
            }
        }
        let inner = Arc::clone(&self.inner);
        let (started_tx, started_rx) = oneshot::channel::<()>();
        tokio::spawn(async move {
            let _ = started_tx.send(());
            let mut killed = false;
            let status = loop {
                tokio::select! {
                    _ = kill.cancelled(), if !killed => {
                        killed = true;
                        let _ = child.start_kill();
                    }
                    line = stderr.next_line() => {
                        if let Ok(Some(line)) = line {
                            let _ = vmm_log.write_all(format!("{line}\n").as_bytes()).await;
                        }
                    }
                    status = child.wait() => break status,
                }
            };
            while let Ok(Some(line)) = stderr.next_line().await {
                let _ = vmm_log.write_all(format!("{line}\n").as_bytes()).await;
            }
            let state = match status {
                Ok(s) if s.success() || killed => MachineState::PoweredOff,
                _ => MachineState::Aborted,
            };
            if let Some(m) = inner.machines.lock().unwrap().get_mut(&id) {
                m.kill = None;
                m.exited = None;
            }
            inner.set_state(id, state);
            let _ = exited_tx.send(true);
        });
        let _ = started_rx.await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Engine for OxvmmEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::Oxvmm
    }

    async fn capabilities(&self, _host: &HostInfo) -> EngineCaps {
        match Self::host_check(&self.inner.binary) {
            Ok(()) => EngineCaps {
                available: true,
                reason: None,
                guest_archs: vec![Arch::X86_64],
            },
            Err(reason) => EngineCaps {
                available: false,
                reason: Some(reason),
                guest_archs: vec![],
            },
        }
    }

    async fn register(&self, manifest: &MachineManifest, folder: &Path) -> Result<(), EngineError> {
        let mut machines = self.inner.machines.lock().unwrap();
        let entry = machines.entry(manifest.id).or_insert_with(|| Machine {
            manifest: manifest.clone(),
            folder: folder.to_path_buf(),
            state: MachineState::PoweredOff,
            kill: None,
            exited: None,
        });
        entry.manifest = manifest.clone();
        entry.folder = folder.to_path_buf();
        Ok(())
    }

    async fn unregister(&self, id: Uuid) -> Result<(), EngineError> {
        let state = self.get(id)?;
        if !state.allows(Action::Delete) {
            return Err(EngineError::InvalidState(state));
        }
        self.inner.machines.lock().unwrap().remove(&id);
        Ok(())
    }

    async fn state(&self, id: Uuid) -> Result<MachineState, EngineError> {
        self.get(id)
    }

    async fn apply_mode(&self, id: Uuid, field_path: &str) -> Result<ApplyMode, EngineError> {
        Ok(apply_mode_for(self.get(id)?, field_path))
    }

    async fn apply(
        &self,
        manifest: &MachineManifest,
        changed: &[String],
    ) -> Result<(), EngineError> {
        let state = self.get(manifest.id)?;
        if changed
            .iter()
            .any(|p| apply_mode_for(state, p) == ApplyMode::NeedsPowerOff)
        {
            return Err(EngineError::InvalidState(state));
        }
        if let Some(m) = self.inner.machines.lock().unwrap().get_mut(&manifest.id) {
            m.manifest = manifest.clone();
        }
        Ok(())
    }

    async fn start(&self, id: Uuid, _opts: StartOpts, op: OpHandle) -> Result<(), EngineError> {
        let from = self.begin(id, Action::Start, MachineState::Starting)?;
        match self.spawn(id, &op).await {
            Ok(()) => {
                // The guest may already have exited; only report Running if it has not.
                let still_starting = self.get(id)? == MachineState::Starting;
                if still_starting {
                    self.inner.set_state(id, MachineState::Running);
                }
                op.progress(100, "running");
                Ok(())
            }
            Err(e) => {
                self.inner.set_state(id, from);
                Err(e)
            }
        }
    }

    async fn stop(&self, id: Uuid, mode: StopMode, op: OpHandle) -> Result<(), EngineError> {
        if mode == StopMode::AcpiShutdown {
            return Err(failed(
                OxeCode::ENG_UNSUPPORTED,
                "ACPI shutdown needs ACPI tables, which oxvmm does not provide yet",
            ));
        }
        let state = self.get(id)?;
        if !state.allows(Action::Stop) {
            return Err(EngineError::InvalidState(state));
        }
        let (kill, exited) = {
            let machines = self.inner.machines.lock().unwrap();
            let m = machines.get(&id).ok_or(EngineError::NotFound(id))?;
            (m.kill.clone(), m.exited.clone())
        };
        let (Some(kill), Some(mut exited)) = (kill, exited) else {
            // Aborted with no process left: just record that it is off.
            self.inner.set_state(id, MachineState::PoweredOff);
            return Ok(());
        };
        self.inner.set_state(id, MachineState::Stopping);
        op.progress(50, "powering off");
        kill.cancel();
        while !*exited.borrow() {
            if exited.changed().await.is_err() {
                break;
            }
        }
        op.progress(100, "done");
        Ok(())
    }

    fn events(&self) -> BoxStream<'static, EngineEvent> {
        BroadcastStream::new(self.inner.events.subscribe())
            .filter_map(|e| async move { e.ok() })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vmm_args_require_kernel_and_one_vcpu() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = MachineManifest::new("v", "Linux_64", EngineKind::Oxvmm);
        m.platform.arch = Arch::X86_64;
        let err = vmm_args(&m, dir.path()).unwrap_err();
        assert!(matches!(err, EngineError::Failed { ref code, .. } if code == "OXE-ENG-011"));

        std::fs::write(dir.path().join("vmlinuz"), b"k").unwrap();
        m.boot = Some(oxbox_model::Boot {
            kernel: "vmlinuz".into(),
            initrd: None,
            cmdline: Some("quiet".into()),
        });
        assert!(
            vmm_args(&m, dir.path()).is_err(),
            "default manifest has 2 vCPUs"
        );
        m.cpu.count = 1;
        let args = vmm_args(&m, dir.path()).unwrap();
        assert_eq!(args[1], dir.path().join("vmlinuz").to_string_lossy());
        assert!(args.windows(2).any(|w| w == ["--cmdline", "quiet"]));
        assert!(args.iter().any(|a| a.ends_with("logs/serial.log")));
    }

    #[tokio::test]
    async fn a_failing_vmm_leaves_the_machine_off() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("vmlinuz"), b"k").unwrap();
        // `false` exits at once without printing the ready line.
        let engine = OxvmmEngine::new(PathBuf::from("/bin/false"));
        let mut m = MachineManifest::new("v", "Linux_64", EngineKind::Oxvmm);
        m.platform.arch = Arch::X86_64;
        m.cpu.count = 1;
        m.boot = Some(oxbox_model::Boot {
            kernel: "vmlinuz".into(),
            initrd: None,
            cmdline: None,
        });
        engine.register(&m, dir.path()).await.unwrap();
        let err = engine
            .start(m.id, StartOpts::default(), OpHandle::detached())
            .await
            .unwrap_err();
        assert!(
            matches!(err, EngineError::Failed { ref message, .. } if message.contains("exited during setup")),
            "{err:?}"
        );
        assert_eq!(engine.state(m.id).await.unwrap(), MachineState::PoweredOff);
        let acpi = engine
            .stop(m.id, StopMode::AcpiShutdown, OpHandle::detached())
            .await
            .unwrap_err();
        assert!(matches!(acpi, EngineError::Failed { ref code, .. } if code == "OXE-ENG-010"));
    }
}
