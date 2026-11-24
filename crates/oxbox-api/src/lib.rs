//! OxBox control-plane API `oxbox.v1`: generated messages and gRPC stubs, plus the per-user socket
//! location and a client connector for Unix domain sockets.

pub mod v1 {
    tonic::include_proto!("oxbox.v1");
}

use std::path::{Path, PathBuf};

use hyper_util::rt::TokioIo;
use tonic::transport::{Channel, Endpoint};

pub const API_VERSION: &str = "oxbox.v1";
/// Response metadata key carrying the OXE error code.
pub const OXE_CODE_KEY: &str = "oxe-code";
/// Response metadata key carrying the current etag on a conflict.
pub const OXE_ETAG_KEY: &str = "oxe-etag";

/// Default daemon socket: `$OXBOX_SOCKET`, else `$XDG_RUNTIME_DIR/oxbox/oxboxd.sock` on Linux,
/// else `<local data dir>/OxBox/oxboxd.sock`.
pub fn default_socket_path() -> PathBuf {
    if let Some(p) = std::env::var_os("OXBOX_SOCKET") {
        return PathBuf::from(p);
    }
    if cfg!(target_os = "linux")
        && let Some(rt) = dirs::runtime_dir()
    {
        return rt.join("oxbox").join("oxboxd.sock");
    }
    let base = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    base.join(if cfg!(target_os = "linux") {
        "oxbox"
    } else {
        "OxBox"
    })
    .join("oxboxd.sock")
}

/// Open a gRPC channel to the daemon over its Unix domain socket.
#[cfg(unix)]
pub async fn connect(socket: &Path) -> Result<Channel, tonic::transport::Error> {
    let socket = socket.to_path_buf();
    // The URI is required by the API but unused: the connector always dials the socket.
    Endpoint::from_static("http://oxboxd.local")
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
