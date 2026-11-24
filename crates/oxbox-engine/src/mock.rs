//! `MockEngine`: deterministic, in-memory, no hypervisor. Latency and failures are scriptable so the
//! daemon, CLI and UI can be tested on any machine. Machine state does not survive a restart.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use futures::StreamExt;
use futures::stream::BoxStream;
use oxbox_model::{Action, Arch, EngineKind, MachineManifest, MachineState};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use uuid::Uuid;

use crate::{
    ApplyMode, Engine, EngineCaps, EngineError, EngineEvent, HostInfo, OpHandle, StartOpts,
    StopMode,
};

const STEPS: u32 = 4;

/// Which call a scripted failure applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MockOp {
    Start,
    Stop,
}

pub struct MockEngine {
    latency: Duration,
    machines: Mutex<HashMap<Uuid, MachineState>>,
    failures: Mutex<HashMap<MockOp, String>>,
    events: broadcast::Sender<EngineEvent>,
}

impl MockEngine {
    /// `latency` is the total duration of start/stop, split across progress steps.
    pub fn new(latency: Duration) -> Self {
        Self {
            latency,
            machines: Mutex::new(HashMap::new()),
            failures: Mutex::new(HashMap::new()),
            events: broadcast::channel(1024).0,
        }
    }

    /// Make the next call of `op` fail with the given OXE code.
    pub fn fail_next(&self, op: MockOp, code: impl Into<String>) {
        self.failures.lock().unwrap().insert(op, code.into());
    }

    fn set(&self, id: Uuid, state: MachineState) {
        self.machines.lock().unwrap().insert(id, state);
        let _ = self.events.send(EngineEvent::StateChanged { id, state });
    }

    fn get(&self, id: Uuid) -> Result<MachineState, EngineError> {
        self.machines
            .lock()
            .unwrap()
            .get(&id)
            .copied()
            .ok_or(EngineError::NotFound(id))
    }

    /// Check and enter the transient state atomically so two callers cannot both start a machine.
    fn begin(
        &self,
        id: Uuid,
        action: Action,
        transient: MachineState,
    ) -> Result<MachineState, EngineError> {
        let mut machines = self.machines.lock().unwrap();
        let state = *machines.get(&id).ok_or(EngineError::NotFound(id))?;
        if !state.allows(action) {
            return Err(EngineError::InvalidState(state));
        }
        machines.insert(id, transient);
        drop(machines);
        let _ = self.events.send(EngineEvent::StateChanged {
            id,
            state: transient,
        });
        Ok(state)
    }

    async fn run(
        &self,
        id: Uuid,
        op: &OpHandle,
        mock_op: MockOp,
        step: &str,
        from: MachineState,
        done: MachineState,
    ) -> Result<(), EngineError> {
        for i in 0..STEPS {
            op.progress(i * 100 / STEPS, step);
            tokio::select! {
                _ = op.cancel.cancelled() => {
                    self.set(id, from);
                    return Err(EngineError::Cancelled);
                }
                _ = tokio::time::sleep(self.latency / STEPS) => {}
            }
        }
        if let Some(code) = self.failures.lock().unwrap().remove(&mock_op) {
            self.set(
                id,
                if mock_op == MockOp::Start {
                    MachineState::Aborted
                } else {
                    from
                },
            );
            return Err(EngineError::Failed {
                code,
                message: format!("scripted {mock_op:?} failure"),
            });
        }
        self.set(id, done);
        op.progress(100, "done");
        Ok(())
    }
}

#[async_trait::async_trait]
impl Engine for MockEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::Mock
    }

    async fn capabilities(&self, host: &HostInfo) -> EngineCaps {
        EngineCaps {
            available: true,
            reason: None,
            guest_archs: vec![
                host.arch,
                if host.arch == Arch::X86_64 {
                    Arch::Aarch64
                } else {
                    Arch::X86_64
                },
            ],
        }
    }

    async fn register(
        &self,
        manifest: &MachineManifest,
        _folder: &std::path::Path,
    ) -> Result<(), EngineError> {
        self.machines
            .lock()
            .unwrap()
            .entry(manifest.id)
            .or_insert(MachineState::PoweredOff);
        Ok(())
    }

    async fn unregister(&self, id: Uuid) -> Result<(), EngineError> {
        let state = self.get(id)?;
        if !state.allows(Action::Delete) {
            return Err(EngineError::InvalidState(state));
        }
        self.machines.lock().unwrap().remove(&id);
        Ok(())
    }

    async fn state(&self, id: Uuid) -> Result<MachineState, EngineError> {
        self.get(id)
    }

    async fn apply_mode(&self, id: Uuid, field_path: &str) -> Result<ApplyMode, EngineError> {
        Ok(crate::apply_mode_for(self.get(id)?, field_path))
    }

    async fn apply(
        &self,
        manifest: &MachineManifest,
        changed: &[String],
    ) -> Result<(), EngineError> {
        for path in changed {
            if self.apply_mode(manifest.id, path).await? == ApplyMode::NeedsPowerOff {
                return Err(EngineError::InvalidState(self.get(manifest.id)?));
            }
        }
        Ok(())
    }

    async fn start(&self, id: Uuid, _opts: StartOpts, op: OpHandle) -> Result<(), EngineError> {
        let from = self.begin(id, Action::Start, MachineState::Starting)?;
        self.run(
            id,
            &op,
            MockOp::Start,
            "booting",
            from,
            MachineState::Running,
        )
        .await
    }

    async fn stop(&self, id: Uuid, mode: StopMode, op: OpHandle) -> Result<(), EngineError> {
        let from = self.begin(id, Action::Stop, MachineState::Stopping)?;
        let step = match mode {
            StopMode::AcpiShutdown => "shutting down",
            StopMode::PowerOff => "powering off",
        };
        self.run(id, &op, MockOp::Stop, step, from, MachineState::PoweredOff)
            .await
    }

    fn events(&self) -> BoxStream<'static, EngineEvent> {
        BroadcastStream::new(self.events.subscribe())
            .filter_map(|e| async move { e.ok() })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::sync::CancellationToken;

    fn vm() -> MachineManifest {
        MachineManifest::new("t", "Linux_64", EngineKind::Mock)
    }

    #[tokio::test]
    async fn start_stop_emits_events() {
        let e = MockEngine::new(Duration::from_millis(8));
        let m = vm();
        e.register(&m, std::path::Path::new("/tmp")).await.unwrap();
        let mut events = e.events();
        e.start(m.id, StartOpts::default(), OpHandle::detached())
            .await
            .unwrap();
        assert_eq!(e.state(m.id).await.unwrap(), MachineState::Running);
        assert_eq!(
            events.next().await,
            Some(EngineEvent::StateChanged {
                id: m.id,
                state: MachineState::Starting
            })
        );
        assert_eq!(
            events.next().await,
            Some(EngineEvent::StateChanged {
                id: m.id,
                state: MachineState::Running
            })
        );
        assert_eq!(
            e.start(m.id, StartOpts::default(), OpHandle::detached())
                .await,
            Err(EngineError::InvalidState(MachineState::Running))
        );
        assert_eq!(
            e.apply_mode(m.id, "memory.size_mib").await.unwrap(),
            ApplyMode::NeedsPowerOff
        );
        assert_eq!(e.apply_mode(m.id, "name").await.unwrap(), ApplyMode::Live);
        e.stop(m.id, StopMode::PowerOff, OpHandle::detached())
            .await
            .unwrap();
        assert_eq!(
            e.apply_mode(m.id, "memory.size_mib").await.unwrap(),
            ApplyMode::Live
        );
    }

    #[tokio::test]
    async fn cancel_restores_previous_state() {
        let e = MockEngine::new(Duration::from_secs(10));
        let m = vm();
        e.register(&m, std::path::Path::new("/tmp")).await.unwrap();
        let cancel = CancellationToken::new();
        let op = OpHandle::new(|_, _| {}, cancel.clone());
        cancel.cancel();
        assert_eq!(
            e.start(m.id, StartOpts::default(), op).await,
            Err(EngineError::Cancelled)
        );
        assert_eq!(e.state(m.id).await.unwrap(), MachineState::PoweredOff);
    }

    #[tokio::test]
    async fn scripted_failure_aborts() {
        let e = MockEngine::new(Duration::ZERO);
        let m = vm();
        e.register(&m, std::path::Path::new("/tmp")).await.unwrap();
        e.fail_next(MockOp::Start, "OXE-HV-002");
        let err = e
            .start(m.id, StartOpts::default(), OpHandle::detached())
            .await
            .unwrap_err();
        assert!(matches!(err, EngineError::Failed { ref code, .. } if code == "OXE-HV-002"));
        assert_eq!(e.state(m.id).await.unwrap(), MachineState::Aborted);
    }
}
