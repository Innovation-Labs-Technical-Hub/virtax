//! Virtax control-plane API `virtax.v1`: generated messages and gRPC stubs, plus the per-user socket
//! location and a client connector for Unix domain sockets.

pub mod v1 {
    tonic::include_proto!("virtax.v1");
}

use std::path::{Path, PathBuf};

use hyper_util::rt::TokioIo;
use tonic::transport::{Channel, Endpoint};

pub const API_VERSION: &str = "virtax.v1";
/// Response metadata key carrying the VTX error code.
pub const VTX_CODE_KEY: &str = "vtx-code";
/// Response metadata key carrying the current etag on a conflict.
pub const VTX_ETAG_KEY: &str = "vtx-etag";

/// Default daemon socket: `$VIRTAX_SOCKET`, else `$XDG_RUNTIME_DIR/virtax/virtaxd.sock` on Linux,
/// else `<local data dir>/Virtax/virtaxd.sock`.
pub fn default_socket_path() -> PathBuf {
    if let Some(p) = std::env::var_os("VIRTAX_SOCKET") {
        return PathBuf::from(p);
    }
    if cfg!(target_os = "linux")
        && let Some(rt) = dirs::runtime_dir()
    {
        return rt.join("virtax").join("virtaxd.sock");
    }
    let base = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    base.join(if cfg!(target_os = "linux") {
        "virtax"
    } else {
        "Virtax"
    })
    .join("virtaxd.sock")
}

/// Open a gRPC channel to the daemon over its Unix domain socket.
#[cfg(unix)]
pub async fn connect(socket: &Path) -> Result<Channel, tonic::transport::Error> {
    let socket = socket.to_path_buf();
    // The URI is required by the API but unused: the connector always dials the socket.
    Endpoint::from_static("http://virtaxd.local")
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_with_connector(tower::service_fn(move |_| {
            let socket = socket.clone();
            async move {
                Ok::<_, std::io::Error>(TokioIo::new(
                    tokio::net::UnixStream::connect(socket).await?,
                ))
            }
        }))
        .await
}
