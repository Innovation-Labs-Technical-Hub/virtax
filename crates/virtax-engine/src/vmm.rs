//! `VirtaxVmmEngine`: runs each machine in its own `virtax-vmm` process (the Rust VMM on KVM).
//!
//! Guest serial output goes to `<folder>/logs/serial.log` and VMM diagnostics to
//! `<folder>/logs/virtax-vmm.log`. A guest that powers off or reboots ends its process and the machine
//! becomes `powered_off`; an unexpected process exit makes it `aborted`. Machines do not outlive the
//! daemon yet: their processes are killed when `virtaxd` exits.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::StreamExt;
use futures::stream::BoxStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, oneshot};
use tokio_stream::wrappers::BroadcastStream;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
use virtax_model::{
    Action, Arch, Bus, DeviceKind, EngineKind, MachineManifest, MachineState, VtxCode,
};

use crate::{
    ApplyMode, Engine, EngineCaps, EngineError, EngineEvent, HostInfo, OpHandle, StartOpts,
    StopMode, apply_mode_for,
};

/// The line `virtax-vmm` prints on stderr once the guest is about to run (see `apps/virtax-vmm`).
const READY_LINE: &str = "virtax-vmm: guest running";
const READY_TIMEOUT: Duration = Duration::from_secs(30);
/// How long an ACPI shutdown request waits for the guest to power off.
const ACPI_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(120);
/// Matches `virtax-vmm-core`'s ACPI MADT limit.
const MAX_VCPUS: u32 = 254;

struct Machine {
    manifest: MachineManifest,
    folder: PathBuf,
    state: MachineState,
    /// Present while a process runs: cancel it to kill the VM.
    kill: Option<CancellationToken>,
    /// Resolved when the running process has exited and the state is final.
    exited: Option<tokio::sync::watch::Receiver<bool>>,
    /// Process ID of the running `virtax-vmm`, for the power-button signal.
    pid: Option<u32>,
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

pub struct VirtaxVmmEngine {
    inner: Arc<Inner>,
}

fn failed(code: VtxCode, message: impl Into<String>) -> EngineError {
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

/// `--disk` arguments for every attachment on the machine's `virtio` controllers, in controller then
/// port order, or why one cannot be attached. Disks are read-write unless they are DVDs or the user
/// cannot write the image.
fn disk_args(m: &MachineManifest, folder: &Path) -> Result<Vec<String>, EngineError> {
    let config = |message: String| failed(VtxCode::ENG_VM_CONFIG, message);
    let mut args = Vec::new();
    for controller in &m.storage.controllers {
        if controller.attachments.is_empty() {
            continue;
        }
        if controller.bus != Bus::Virtio {
            return Err(config(format!(
                "storage controller `{}` is on the {} bus; virtax-vmm attaches disks on `virtio` \
                 controllers only (change its `bus` in machine.toml)",
                controller.name,
                format!("{:?}", controller.bus).to_lowercase()
            )));
        }
        let mut attachments: Vec<_> = controller.attachments.iter().collect();
        attachments.sort_by_key(|a| (a.port, a.device));
        for a in attachments {
            let label = format!("`{}` port {}", controller.name, a.port);
            let read_only = match a.kind {
                DeviceKind::Disk => false,
                DeviceKind::Dvd => true,
                DeviceKind::Floppy => {
                    return Err(config(format!(
                        "attachment {label} is a floppy; virtax-vmm attaches disks and DVDs only"
                    )));
                }
            };
            let Some(path) = &a.path else {
                if a.kind == DeviceKind::Dvd {
                    continue; // an empty drive
                }
                return Err(config(format!(
                    "attachment {label} has no image `path`; virtax-vmm attaches image files only"
                )));
            };
            let path = resolve(folder, path);
            let file = std::fs::File::open(&path).map_err(|e| {
                config(format!(
                    "attachment {label}: image {} cannot be opened: {e}",
                    path.display()
                ))
            })?;
            if !file.metadata().map(|md| md.is_file()).unwrap_or(false) {
                return Err(config(format!(
                    "attachment {label}: {} is not a regular file",
                    path.display()
                )));
            }
            match virtax_vmm_core::disk::detect_format(&file) {
                Ok(None) => {}
                Ok(Some(format)) => {
                    return Err(config(format!(
                        "attachment {label}: {} is in {format} format; virtax-vmm attaches raw images \
                         only (convert it with `qemu-img convert -O raw`)",
                        path.display()
                    )));
                }
                Err(e) => {
                    return Err(config(format!(
                        "attachment {label}: cannot read {}: {e}",
                        path.display()
                    )));
                }
            }
            let writable = std::fs::OpenOptions::new().write(true).open(&path).is_ok();
            let mode = if read_only || !writable { "ro" } else { "rw" };
            args.extend(["--disk".into(), format!("{},{mode}", path.display())]);
        }
    }
    Ok(args)
}

/// The `virtax-vmm` command line for a machine, or why it cannot run.
fn vmm_args(m: &MachineManifest, folder: &Path) -> Result<Vec<String>, EngineError> {
    let boot = m.boot.as_ref().ok_or_else(|| {
        failed(
            VtxCode::ENG_VM_CONFIG,
            "no kernel to boot: set one with `virtaxctl vm set <vm> --kernel <path>`",
        )
    })?;
    if m.cpu.count > MAX_VCPUS {
        return Err(failed(
            VtxCode::ENG_VM_CONFIG,
            format!(
                "virtax-vmm supports up to {MAX_VCPUS} vCPUs; the machine has {}",
                m.cpu.count
            ),
        ));
    }
    if m.platform.arch != Arch::X86_64 {
        return Err(failed(
            VtxCode::ENG_VM_CONFIG,
            "virtax-vmm runs x86_64 guests only for now",
        ));
    }
    let kernel = resolve(folder, &boot.kernel);
    if !kernel.is_file() {
        return Err(failed(
            VtxCode::ENG_VM_CONFIG,
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
                VtxCode::ENG_VM_CONFIG,
                format!("initrd {} not found", initrd.display()),
            ));
        }
        args.extend(["--initrd".into(), initrd.to_string_lossy().into_owned()]);
    }
    if let Some(cmdline) = &boot.cmdline {
        args.extend(["--cmdline".into(), cmdline.clone()]);
    }
    args.extend(disk_args(m, folder)?);
    Ok(args)
}

async fn wait_exited(exited: &mut tokio::sync::watch::Receiver<bool>) {
    while !*exited.borrow() {
        if exited.changed().await.is_err() {
            break;
        }
    }
}

/// `virtax-vmm` turns SIGTERM into an ACPI power-button press.
fn press_power_button(pid: Option<u32>) -> std::io::Result<()> {
    let pid = pid
        .and_then(|p| i32::try_from(p).ok())
        .and_then(rustix::process::Pid::from_raw)
        .ok_or_else(|| std::io::Error::other("the VM process ID is unknown"))?;
    rustix::process::kill_process(pid, rustix::process::Signal::TERM)?;
    Ok(())
}

impl VirtaxVmmEngine {
    /// `binary` is the `virtax-vmm` executable.
    pub fn new(binary: PathBuf) -> Self {
        Self {
            inner: Arc::new(Inner {
                binary,
                machines: Mutex::new(HashMap::new()),
                events: broadcast::channel(1024).0,
            }),
        }
    }

    /// Whether this host can run virtax-vmm machines, with the reason when it cannot.
    pub fn host_check(binary: &Path) -> Result<(), String> {
        if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
            return Err("virtax-vmm runs on Linux x86_64 hosts only for now".into());
        }
        if !binary.is_file() {
            return Err(format!(
                "virtax-vmm binary not found at {}",
                binary.display()
            ));
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
            .map_err(|e| failed(VtxCode::ENG_FAILED, format!("{}: {e}", logs.display())))?;
        let mut vmm_log = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs.join("virtax-vmm.log"))
            .await
            .map_err(|e| failed(VtxCode::ENG_FAILED, e.to_string()))?;

        op.progress(10, "launching virtax-vmm");
        let mut child: Child = Command::new(&self.inner.binary)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                failed(
                    VtxCode::ENG_FAILED,
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
                        _ => return Err(failed(VtxCode::ENG_FAILED, format!("virtax-vmm exited during setup: {last}"))),
                    },
                }
            }
        })
        .await
        .unwrap_or_else(|_| Err(failed(VtxCode::ENG_FAILED, "virtax-vmm did not become ready within 30 s")));
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
                m.pid = child.id();
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
                m.pid = None;
            }
            inner.set_state(id, state);
            let _ = exited_tx.send(true);
        });
        let _ = started_rx.await;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Engine for VirtaxVmmEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::VirtaxVmm
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
            pid: None,
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
        let state = self.get(id)?;
        if !state.allows(Action::Stop) {
            return Err(EngineError::InvalidState(state));
        }
        let (kill, exited, pid) = {
            let machines = self.inner.machines.lock().unwrap();
            let m = machines.get(&id).ok_or(EngineError::NotFound(id))?;
            (m.kill.clone(), m.exited.clone(), m.pid)
        };
        let (Some(kill), Some(mut exited)) = (kill, exited) else {
            // Aborted with no process left: just record that it is off.
            self.inner.set_state(id, MachineState::PoweredOff);
            return Ok(());
        };
        self.inner.set_state(id, MachineState::Stopping);
        match mode {
            StopMode::PowerOff => {
                op.progress(50, "powering off");
                kill.cancel();
                wait_exited(&mut exited).await;
            }
            StopMode::AcpiShutdown => {
                op.progress(20, "pressing the power button");
                if let Err(e) = press_power_button(pid) {
                    self.inner.set_state(id, state);
                    return Err(failed(
                        VtxCode::ENG_FAILED,
                        format!("cannot signal virtax-vmm: {e}"),
                    ));
                }
                op.progress(50, "waiting for the guest to shut down");
                tokio::select! {
                    _ = wait_exited(&mut exited) => {}
                    _ = tokio::time::sleep(ACPI_SHUTDOWN_TIMEOUT) => {
                        self.inner.set_state(id, state);
                        return Err(failed(
                            VtxCode::ENG_SHUTDOWN_TIMEOUT,
                            format!("no shutdown within {} s of the power button", ACPI_SHUTDOWN_TIMEOUT.as_secs()),
                        ));
                    }
                    _ = op.cancel.cancelled() => {
                        self.inner.set_state(id, state);
                        return Err(EngineError::Cancelled);
                    }
                }
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
    fn vmm_args_require_a_kernel_and_a_supported_vcpu_count() {
        let dir = tempfile::tempdir().unwrap();
        let mut m = MachineManifest::new("v", "Linux_64", EngineKind::VirtaxVmm);
        m.platform.arch = Arch::X86_64;
        let err = vmm_args(&m, dir.path()).unwrap_err();
        assert!(matches!(err, EngineError::Failed { ref code, .. } if code == "VTX-ENG-011"));

        std::fs::write(dir.path().join("vmlinuz"), b"k").unwrap();
        m.boot = Some(virtax_model::Boot {
            kernel: "vmlinuz".into(),
            initrd: None,
            cmdline: Some("quiet".into()),
        });
        m.cpu.count = 300;
        assert!(vmm_args(&m, dir.path()).is_err(), "too many vCPUs");
        m.cpu.count = 2;
        let args = vmm_args(&m, dir.path()).unwrap();
        assert_eq!(args[1], dir.path().join("vmlinuz").to_string_lossy());
        assert!(args.windows(2).any(|w| w == ["--cmdline", "quiet"]));
        assert!(args.iter().any(|a| a.ends_with("logs/serial.log")));
    }

    fn attach(
        m: &mut MachineManifest,
        controller: &str,
        bus: Bus,
        attachments: &[(u8, DeviceKind, Option<&str>)],
    ) {
        m.storage.controllers.push(virtax_model::StorageController {
            name: controller.into(),
            bus,
            attachments: attachments
                .iter()
                .map(|(port, kind, path)| virtax_model::Attachment {
                    port: *port,
                    device: 0,
                    kind: *kind,
                    medium: None,
                    path: path.map(str::to_string),
                    ssd: false,
                    discard: false,
                })
                .collect(),
        });
    }

    fn config_error(m: &MachineManifest, folder: &Path) -> String {
        match disk_args(m, folder).unwrap_err() {
            EngineError::Failed { code, message } => {
                assert_eq!(code, "VTX-ENG-011", "{message}");
                message
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn disks_follow_controller_then_port_order() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["a.img", "b.img", "c.img", "cd.iso"] {
            std::fs::write(dir.path().join(name), [0u8; 1024]).unwrap();
        }
        let absolute = dir.path().join("c.img");
        let mut m = MachineManifest::new("v", "Linux_64", EngineKind::VirtaxVmm);
        attach(&mut m, "empty", Bus::Sata, &[]);
        attach(
            &mut m,
            "vd0",
            Bus::Virtio,
            &[
                (1, DeviceKind::Disk, Some("b.img")),
                (0, DeviceKind::Disk, Some("a.img")),
                (2, DeviceKind::Dvd, None),
            ],
        );
        attach(
            &mut m,
            "vd1",
            Bus::Virtio,
            &[
                (0, DeviceKind::Disk, Some(absolute.to_str().unwrap())),
                (1, DeviceKind::Dvd, Some("cd.iso")),
            ],
        );
        let at = |name: &str, mode: &str| format!("{},{mode}", dir.path().join(name).display());
        assert_eq!(
            disk_args(&m, dir.path()).unwrap(),
            [
                "--disk".to_string(),
                at("a.img", "rw"),
                "--disk".into(),
                at("b.img", "rw"),
                "--disk".into(),
                at("c.img", "rw"),
                "--disk".into(),
                at("cd.iso", "ro"),
            ],
            "an empty SATA controller and an empty DVD drive are ignored"
        );
    }

    #[test]
    fn an_image_the_user_cannot_write_is_attached_read_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let image = dir.path().join("ro.img");
        std::fs::write(&image, [0u8; 512]).unwrap();
        std::fs::set_permissions(&image, std::fs::Permissions::from_mode(0o444)).unwrap();
        let mut m = MachineManifest::new("v", "Linux_64", EngineKind::VirtaxVmm);
        attach(
            &mut m,
            "vd0",
            Bus::Virtio,
            &[(0, DeviceKind::Disk, Some("ro.img"))],
        );
        let args = disk_args(&m, dir.path()).unwrap();
        // Root can write any file, so only check the read-only case as a normal user.
        if !rustix::process::geteuid().is_root() {
            assert_eq!(args[1], format!("{},ro", image.display()));
        }
    }

    #[test]
    fn disks_that_cannot_be_attached_are_refused_before_start() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("disk.img"), [0u8; 512]).unwrap();
        std::fs::write(
            dir.path().join("disk.qcow2"),
            [&b"QFI\xfb"[..], &[0; 508]].concat(),
        )
        .unwrap();
        let manifest = |controller: &str, bus, kind, path: Option<&str>| {
            let mut m = MachineManifest::new("v", "Linux_64", EngineKind::VirtaxVmm);
            attach(&mut m, controller, bus, &[(0, kind, path)]);
            m
        };
        for bus in [Bus::Sata, Bus::Ide, Bus::Scsi, Bus::Nvme] {
            let m = manifest("ctl0", bus, DeviceKind::Disk, Some("disk.img"));
            let e = config_error(&m, dir.path());
            assert!(
                e.contains("`ctl0`") && e.contains("`virtio` controllers only"),
                "{e}"
            );
        }
        let e = config_error(
            &manifest("vd0", Bus::Virtio, DeviceKind::Disk, Some("missing.img")),
            dir.path(),
        );
        assert!(
            e.contains("missing.img") && e.contains("`vd0` port 0"),
            "{e}"
        );
        let e = config_error(
            &manifest("vd0", Bus::Virtio, DeviceKind::Disk, Some("disk.qcow2")),
            dir.path(),
        );
        assert!(e.contains("qcow2 format"), "{e}");
        let e = config_error(
            &manifest("vd0", Bus::Virtio, DeviceKind::Disk, None),
            dir.path(),
        );
        assert!(e.contains("no image `path`"), "{e}");
        let e = config_error(
            &manifest("vd0", Bus::Virtio, DeviceKind::Floppy, Some("disk.img")),
            dir.path(),
        );
        assert!(e.contains("floppy"), "{e}");
    }

    #[tokio::test]
    async fn a_failing_vmm_leaves_the_machine_off() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("vmlinuz"), b"k").unwrap();
        // `false` (found on PATH) exits at once without printing the ready line.
        let engine = VirtaxVmmEngine::new(PathBuf::from("false"));
        let mut m = MachineManifest::new("v", "Linux_64", EngineKind::VirtaxVmm);
        m.platform.arch = Arch::X86_64;
        m.boot = Some(virtax_model::Boot {
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
        assert_eq!(acpi, EngineError::InvalidState(MachineState::PoweredOff));
    }
}
