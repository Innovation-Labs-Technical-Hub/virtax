//! gRPC services for `virtax.v1`, and the peer-UID check applied to every request.

use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use tonic::{Request, Response, Status};
use virtax_api::v1 as pb;
use virtax_api::v1::daemon_server::Daemon as DaemonSvc;
use virtax_api::v1::events_server::Events;
use virtax_api::v1::machines_server::Machines;
use virtax_api::v1::operations_server::Operations;
use virtax_api::v1::runtime_server::Runtime;
use virtax_model::{Action, SCHEMA_ID, VtxCode, parse_manifest};

use crate::daemon::Daemon;
use crate::error::ApiError;
use crate::events::EventFilter;

#[derive(Clone)]
pub struct Api(pub Arc<Daemon>);

/// Reject connections from any user other than the one running the daemon.
pub fn check_peer(req: Request<()>) -> Result<Request<()>, Status> {
    let me = rustix::process::getuid().as_raw();
    let peer = req
        .extensions()
        .get::<tonic::transport::server::UdsConnectInfo>()
        .and_then(|info| info.peer_cred)
        .map(|cred| cred.uid());
    if peer == Some(me) {
        Ok(req)
    } else {
        tracing::warn!(?peer, "rejected connection from another user");
        Err(ApiError::new(
            tonic::Code::PermissionDenied,
            VtxCode::API_PERMISSION_DENIED,
            format!("peer uid {peer:?}"),
        )
        .into())
    }
}

#[tonic::async_trait]
impl Machines for Api {
    async fn list_machines(
        &self,
        req: Request<pb::ListMachinesRequest>,
    ) -> Result<Response<pb::ListMachinesResponse>, Status> {
        let r = req.into_inner();
        Ok(Response::new(pb::ListMachinesResponse {
            machines: self.0.list(&r.state, &r.group)?,
        }))
    }

    async fn get_machine(
        &self,
        req: Request<pb::GetMachineRequest>,
    ) -> Result<Response<pb::Machine>, Status> {
        Ok(Response::new(self.0.get(&req.into_inner().name).await?))
    }

    async fn create_machine(
        &self,
        req: Request<pb::CreateMachineRequest>,
    ) -> Result<Response<pb::Machine>, Status> {
        Ok(Response::new(self.0.create(req.into_inner()).await?))
    }

    async fn update_machine(
        &self,
        req: Request<pb::UpdateMachineRequest>,
    ) -> Result<Response<pb::UpdateMachineResponse>, Status> {
        let out = self.0.update(req.into_inner()).await?;
        Ok(Response::new(pb::UpdateMachineResponse {
            machine: Some(out.machine),
            field_results: out.field_results,
        }))
    }

    async fn delete_machine(
        &self,
        req: Request<pb::DeleteMachineRequest>,
    ) -> Result<Response<pb::DeleteMachineResponse>, Status> {
        let r = req.into_inner();
        self.0.delete(&r.name, r.delete_files).await?;
        Ok(Response::new(pb::DeleteMachineResponse {}))
    }

    async fn cancel_pending_change(
        &self,
        req: Request<pb::CancelPendingChangeRequest>,
    ) -> Result<Response<pb::Machine>, Status> {
        let r = req.into_inner();
        Ok(Response::new(
            self.0.cancel_pending(&r.name, &r.field_path).await?,
        ))
    }

    async fn validate_manifest(
        &self,
        req: Request<pb::ValidateManifestRequest>,
    ) -> Result<Response<pb::ValidateManifestResponse>, Status> {
        let issues = match parse_manifest(&req.into_inner().manifest_toml) {
            Ok(_) => Vec::new(),
            Err(issues) => issues
                .into_iter()
                .map(|i| pb::ManifestIssue {
                    path: i.path,
                    message: i.message,
                    line: i.line.unwrap_or(0) as u32,
                    column: i.column.unwrap_or(0) as u32,
                })
                .collect(),
        };
        Ok(Response::new(pb::ValidateManifestResponse { issues }))
    }
}

#[tonic::async_trait]
impl Runtime for Api {
    async fn start(
        &self,
        req: Request<pb::StartRequest>,
    ) -> Result<Response<pb::Operation>, Status> {
        Ok(Response::new(
            self.0
                .run_action(&req.into_inner().name, Action::Start, false)
                .await?,
        ))
    }

    async fn stop(&self, req: Request<pb::StopRequest>) -> Result<Response<pb::Operation>, Status> {
        let r = req.into_inner();
        Ok(Response::new(
            self.0.run_action(&r.name, Action::Stop, r.force).await?,
        ))
    }
}

type OpStream = Pin<Box<dyn Stream<Item = Result<pb::Operation, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl Operations for Api {
    type WatchOperationStream = OpStream;

    async fn get_operation(
        &self,
        req: Request<pb::GetOperationRequest>,
    ) -> Result<Response<pb::Operation>, Status> {
        Ok(Response::new(self.0.get_op(&req.into_inner().name)?))
    }

    async fn list_operations(
        &self,
        req: Request<pb::ListOperationsRequest>,
    ) -> Result<Response<pb::ListOperationsResponse>, Status> {
        Ok(Response::new(pb::ListOperationsResponse {
            operations: self.0.list_ops(req.into_inner().page_size)?,
        }))
    }

    async fn watch_operation(
        &self,
        req: Request<pb::WatchOperationRequest>,
    ) -> Result<Response<OpStream>, Status> {
        let stream: OpStream = match self.0.watch_op(&req.into_inner().name)? {
            Err(finished) => Box::pin(futures::stream::iter([Ok(finished)])),
            Ok(mut rx) => {
                let shutdown = self.0.shutdown.clone();
                Box::pin(async_stream(move |tx| async move {
                    loop {
                        let op = rx.borrow_and_update().clone();
                        let done = op.done;
                        if tx.send(Ok(op)).await.is_err() || done {
                            break;
                        }
                        tokio::select! {
                            _ = shutdown.cancelled() => break,
                            changed = rx.changed() => if changed.is_err() { break },
                        }
                    }
                }))
            }
        };
        Ok(Response::new(stream))
    }

    async fn cancel_operation(
        &self,
        req: Request<pb::CancelOperationRequest>,
    ) -> Result<Response<pb::Operation>, Status> {
        Ok(Response::new(self.0.cancel_op(&req.into_inner().name)?))
    }
}

type EventStream = Pin<Box<dyn Stream<Item = Result<pb::Event, Status>> + Send + 'static>>;

#[tonic::async_trait]
impl Events for Api {
    type WatchStream = EventStream;

    async fn watch(
        &self,
        req: Request<pb::WatchEventsRequest>,
    ) -> Result<Response<EventStream>, Status> {
        let r = req.into_inner();
        let filter = EventFilter {
            resource_prefixes: r.resource_prefixes,
            kinds: r.kinds,
        };
        let rx = self.0.watch_events(r.since_seq, filter)?;
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(futures::StreamExt::map(
            stream,
            |item| item.map_err(Status::from),
        ))))
    }
}

#[tonic::async_trait]
impl DaemonSvc for Api {
    async fn get_version(
        &self,
        _req: Request<pb::GetVersionRequest>,
    ) -> Result<Response<pb::VersionInfo>, Status> {
        Ok(Response::new(pb::VersionInfo {
            daemon_version: env!("CARGO_PKG_VERSION").into(),
            api_version: virtax_api::API_VERSION.into(),
            manifest_schema: SCHEMA_ID.into(),
            registry_schema: self.0.registry_schema(),
            engines: self.0.engine_kinds(),
            last_event_seq: self.0.last_event_seq()?,
        }))
    }
}

/// Drive `body` on a task that feeds a bounded channel; the returned stream ends when the task does.
fn async_stream<F, Fut>(
    body: F,
) -> impl Stream<Item = Result<pb::Operation, Status>> + Send + 'static
where
    F: FnOnce(tokio::sync::mpsc::Sender<Result<pb::Operation, Status>>) -> Fut,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::channel(16);
    tokio::spawn(body(tx));
    tokio_stream::wrappers::ReceiverStream::new(rx)
}
