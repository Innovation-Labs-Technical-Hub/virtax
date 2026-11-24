//! Daemon errors: an VTX code plus the gRPC status it maps to.

use tonic::{Code, Status};
use virtax_api::{VTX_CODE_KEY, VTX_ETAG_KEY};
use virtax_engine::EngineError;
use virtax_model::{VtxCode, VtxError};
use virtax_store::StoreError;

#[derive(Debug)]
pub struct ApiError {
    pub status: Code,
    pub code: VtxCode,
    pub detail: Option<String>,
    /// Current etag, set on a concurrency conflict.
    pub etag: Option<String>,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(status: Code, code: VtxCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            code,
            detail: Some(detail.into()),
            etag: None,
        }
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(Code::InvalidArgument, VtxCode::API_BAD_REQUEST, detail)
    }

    pub fn not_found(what: impl Into<String>) -> Self {
        Self::new(Code::NotFound, VtxCode::API_NOT_FOUND, what)
    }
}

fn status_for(code: VtxCode) -> Code {
    match code {
        VtxCode::CFG_INVALID_MANIFEST | VtxCode::API_BAD_REQUEST => Code::InvalidArgument,
        VtxCode::CFG_NAME_TAKEN => Code::AlreadyExists,
        VtxCode::CFG_ETAG_MISMATCH => Code::Aborted,
        VtxCode::API_RESYNC_REQUIRED => Code::OutOfRange,
        VtxCode::API_NOT_FOUND => Code::NotFound,
        VtxCode::API_PERMISSION_DENIED => Code::PermissionDenied,
        VtxCode::RT_CANCELLED => Code::Cancelled,
        VtxCode::CFG_VM_LOCKED
        | VtxCode::RT_INVALID_TRANSITION
        | VtxCode::RT_NEEDS_POWER_OFF
        | VtxCode::RT_ALREADY_IN_STATE
        | VtxCode::CFG_MANIFEST_UNREADABLE
        | VtxCode::ENG_NOT_AVAILABLE => Code::FailedPrecondition,
        _ => Code::Internal,
    }
}

impl From<VtxError> for ApiError {
    fn from(e: VtxError) -> Self {
        Self {
            status: status_for(e.code),
            code: e.code,
            detail: e.detail,
            etag: None,
        }
    }
}

impl From<StoreError> for ApiError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::Vtx(e) => e.into(),
            StoreError::Conflict { current } => Self {
                status: Code::Aborted,
                code: VtxCode::CFG_ETAG_MISMATCH,
                detail: None,
                etag: Some(current.to_string()),
            },
            StoreError::NotFound(what) => Self::not_found(what),
            other => Self::new(Code::Internal, VtxCode::ENG_FAILED, other.to_string()),
        }
    }
}

impl From<EngineError> for ApiError {
    fn from(e: EngineError) -> Self {
        match e {
            EngineError::InvalidState(_) => {
                VtxError::with(VtxCode::RT_INVALID_TRANSITION, e.to_string()).into()
            }
            EngineError::Cancelled => VtxError::new(VtxCode::RT_CANCELLED).into(),
            EngineError::NotFound(_) => {
                Self::new(Code::Internal, VtxCode::ENG_FAILED, e.to_string())
            }
            EngineError::Failed { code, message } => match VtxCode::by_code(&code) {
                Some(c) => Self::new(status_for(c), c, message),
                None => Self::new(
                    Code::Internal,
                    VtxCode::ENG_FAILED,
                    format!("{code}: {message}"),
                ),
            },
        }
    }
}

impl From<ApiError> for Status {
    fn from(e: ApiError) -> Self {
        let message = match &e.detail {
            Some(d) => format!("{}: {} ({d})", e.code.code, e.code.en),
            None => format!("{}: {}", e.code.code, e.code.en),
        };
        let mut status = Status::new(e.status, message);
        if let Ok(v) = e.code.code.parse() {
            status.metadata_mut().insert(VTX_CODE_KEY, v);
        }
        if let Some(etag) = e.etag.and_then(|t| t.parse().ok()) {
            status.metadata_mut().insert(VTX_ETAG_KEY, etag);
        }
        status
    }
}
