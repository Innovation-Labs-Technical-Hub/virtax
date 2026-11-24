//! The `Engine` trait is the strangler seam: `oxboxd` drives every hypervisor backend through it and
//! never knows whether a VM runs on the VirtualBox legacy engine, `oxvmm`, or the `MockEngine`.

pub mod mock;

use std::sync::Arc;

use futures::stream::BoxStream;
use oxbox_model::{Arch, EngineKind, MachineManifest, MachineState};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub use mock::MockEngine;

#[derive(Debug, Clone)]
pub struct HostInfo {
    pub arch: Arch,
}

impl HostInfo {
    pub fn current() -> Self {
        Self { arch: Arch::host() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EngineCaps {
    pub available: bool,
    /// Why the engine is unavailable, when it is.
    pub reason: Option<String>,
    pub guest_archs: Vec<Arch>,
}

/// When a configuration change takes effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyMode {
    Live,
    NextBoot,
    NeedsPowerOff,
}

#[derive(Debug, Clone, Default)]
pub struct StartOpts {
    pub headless: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopMode {
    AcpiShutdown,
    PowerOff,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    StateChanged { id: Uuid, state: MachineState },
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EngineError {
    #[error("machine {0} is not registered with the engine")]
    NotFound(Uuid),
    #[error("not allowed while the machine is {}", .0.as_str())]
    InvalidState(MachineState),
    #[error("operation cancelled")]
    Cancelled,
    #[error("{code}: {message}")]
    Failed { code: String, message: String },
}

type ProgressFn = Arc<dyn Fn(u32, &str) + Send + Sync>;

/// Progress reporting and cancellation for a long-running engine call.
#[derive(Clone)]
pub struct OpHandle {
    progress: ProgressFn,
    pub cancel: CancellationToken,
}

impl OpHandle {
    pub fn new(
        progress: impl Fn(u32, &str) + Send + Sync + 'static,
        cancel: CancellationToken,
    ) -> Self {
        Self {
            progress: Arc::new(progress),
            cancel,
        }
    }

    /// A handle that discards progress and is never cancelled.
    pub fn detached() -> Self {
        Self::new(|_, _| {}, CancellationToken::new())
    }

    pub fn progress(&self, percent: u32, step: &str) {
        (self.progress)(percent.min(100), step);
    }
}

#[async_trait::async_trait]
pub trait Engine: Send + Sync + 'static {
    fn kind(&self) -> EngineKind;

    async fn capabilities(&self, host: &HostInfo) -> EngineCaps;

    /// Register a machine with the engine (also used at startup for machines already in the registry).
    async fn register(&self, manifest: &MachineManifest) -> Result<(), EngineError>;

    async fn unregister(&self, id: Uuid) -> Result<(), EngineError>;

    async fn state(&self, id: Uuid) -> Result<MachineState, EngineError>;

    /// When a change to `field_path` (manifest path such as `memory.size_mib`) would take effect now.
    async fn apply_mode(&self, id: Uuid, field_path: &str) -> Result<ApplyMode, EngineError>;

    /// Push an updated configuration to the engine; `changed` lists the manifest paths that changed.
    async fn apply(
        &self,
        manifest: &MachineManifest,
        changed: &[String],
    ) -> Result<(), EngineError>;

    async fn start(&self, id: Uuid, opts: StartOpts, op: OpHandle) -> Result<(), EngineError>;

    async fn stop(&self, id: Uuid, mode: StopMode, op: OpHandle) -> Result<(), EngineError>;

    /// By-value state events for every machine the engine manages.
    fn events(&self) -> BoxStream<'static, EngineEvent>;
}
