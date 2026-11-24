//! `oxboxd`: the per-user OxBox control-plane daemon. Serves `oxbox.v1` over a Unix domain socket
//! (mode 0600 in a 0700 directory) and accepts only connections from its own user.

mod daemon;
mod error;
mod grpc;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, bail};
use clap::Parser;
use oxbox_api::v1::daemon_server::DaemonServer;
use oxbox_api::v1::machines_server::MachinesServer;
use oxbox_api::v1::operations_server::OperationsServer;
use oxbox_api::v1::runtime_server::RuntimeServer;
use oxbox_engine::{Engine, MockEngine};
use oxbox_model::{EngineKind, OxeCode};
use oxbox_store::{FolderLock, Registry, StoreError};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

#[derive(Parser, Debug)]
#[command(version, about = "OxBox control-plane daemon")]
struct Args {
    /// Socket path (default: $XDG_RUNTIME_DIR/oxbox/oxboxd.sock).
    #[arg(long, env = "OXBOX_SOCKET")]
    socket: Option<PathBuf>,
    /// Data directory holding registry.db and machine folders.
    #[arg(long, env = "OXBOX_DATA_DIR")]
    data_dir: Option<PathBuf>,
    /// Enable the in-memory MockEngine (development and tests; it never runs a guest).
    #[arg(long, env = "OXBOX_MOCK_ENGINE")]
    mock_engine: bool,
    /// Duration of MockEngine start/stop operations.
    #[arg(long, env = "OXBOX_MOCK_LATENCY_MS", default_value_t = 400)]
    mock_latency_ms: u64,
    /// Log as JSON lines.
    #[arg(long)]
    log_json: bool,
}

fn init_logging(json: bool) {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr);
    if json {
        builder.json().init()
    } else {
        builder.init()
    }
}

/// Bind the socket after making sure no other daemon is serving it.
async fn bind_socket(path: &Path) -> anyhow::Result<UnixListener> {
    // sockaddr_un.sun_path is 108 bytes on Linux (104 on macOS), including the trailing NUL.
    if path.as_os_str().len() >= 104 {
        bail!(
            "socket path {} is too long for a Unix socket (max 103 bytes); pass a shorter --socket",
            path.display()
        );
    }
    let dir = path
        .parent()
        .context("socket path has no parent directory")?;
    std::fs::create_dir_all(dir)?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
    if path.exists() {
        if tokio::net::UnixStream::connect(path).await.is_ok() {
            bail!(
                "{}: another oxboxd is serving {}",
                OxeCode::API_DAEMON_RUNNING.code,
                path.display()
            );
        }
        std::fs::remove_file(path)?; // stale socket from a daemon that exited
    }
    let listener = UnixListener::bind(path)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

async fn shutdown_signal() {
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
    }
    tracing::info!("shutting down");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_logging(args.log_json);
    let data_dir = args.data_dir.unwrap_or_else(oxbox_store::default_data_dir);
    let socket = args.socket.unwrap_or_else(oxbox_api::default_socket_path);
    std::fs::create_dir_all(&data_dir)?;

    let _instance = match FolderLock::acquire_file(&data_dir.join("oxboxd.lock")) {
        Ok(lock) => lock,
        Err(StoreError::Oxe(e)) if e.code == OxeCode::CFG_VM_LOCKED => {
            bail!(
                "{}: {} ({})",
                OxeCode::API_DAEMON_RUNNING.code,
                OxeCode::API_DAEMON_RUNNING.en,
                e.detail.unwrap_or_default()
            )
        }
        Err(e) => return Err(e.into()),
    };

    let registry = Registry::open(&data_dir.join("registry.db"))?;
    let mut engines: Vec<Arc<dyn Engine>> = Vec::new();
    if args.mock_engine {
        tracing::warn!("MockEngine enabled: machines are simulated and no guest code runs");
        engines.push(Arc::new(MockEngine::new(Duration::from_millis(
            args.mock_latency_ms,
        ))));
    }
    if engines.is_empty() {
        tracing::warn!("no engine enabled; machines can be listed but not created or started");
    }
    let daemon = daemon::Daemon::new(
        data_dir.clone(),
        registry,
        engines,
        args.mock_engine.then_some(EngineKind::Mock),
    );
    daemon
        .reconcile()
        .await
        .map_err(|e| anyhow::anyhow!("reconcile failed: {}", e.code))?;
    daemon.spawn_event_pumps();

    let listener = bind_socket(&socket).await?;
    tracing::info!(socket = %socket.display(), data_dir = %data_dir.display(), "oxboxd listening");
    let api = grpc::Api(daemon);
    Server::builder()
        .add_service(MachinesServer::with_interceptor(
            api.clone(),
            grpc::check_peer,
        ))
        .add_service(RuntimeServer::with_interceptor(
            api.clone(),
            grpc::check_peer,
        ))
        .add_service(OperationsServer::with_interceptor(
            api.clone(),
            grpc::check_peer,
        ))
        .add_service(DaemonServer::with_interceptor(api, grpc::check_peer))
        .serve_with_incoming_shutdown(UnixListenerStream::new(listener), shutdown_signal())
        .await?;
    let _ = std::fs::remove_file(&socket);
    Ok(())
}
