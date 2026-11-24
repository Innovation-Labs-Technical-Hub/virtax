//! Daemon errors: an OXE code plus the gRPC status it maps to.

use oxbox_api::{OXE_CODE_KEY, OXE_ETAG_KEY};
use oxbox_engine::EngineError;
use oxbox_model::{OxeCode, OxeError};
use oxbox_store::StoreError;
use tonic::{Code, Status};

#[derive(Debug)]
pub struct ApiError {
    pub status: Code,
    pub code: OxeCode,
    pub detail: Option<String>,
    /// Current etag, set on a concurrency conflict.
    pub etag: Option<String>,
}

pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn new(status: Code, code: OxeCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            code,
            detail: Some(detail.into()),
            etag: None,
        }
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(Code::InvalidArgument, OxeCode::API_BAD_REQUEST, detail)
    }

    pub fn not_found(what: impl Into<String>) -> Self {
        Self::new(Code::NotFound, OxeCode::API_NOT_FOUND, what)
    }
}

fn status_for(code: OxeCode) -> Code {
    match code {
        OxeCode::CFG_INVALID_MANIFEST | OxeCode::API_BAD_REQUEST => Code::InvalidArgument,
        OxeCode::CFG_NAME_TAKEN => Code::AlreadyExists,
        OxeCode::CFG_ETAG_MISMATCH => Code::Aborted,
        OxeCode::API_NOT_FOUND => Code::NotFound,
        OxeCode::API_PERMISSION_DENIED => Code::PermissionDenied,
        OxeCode::RT_CANCELLED => Code::Cancelled,
        OxeCode::CFG_VM_LOCKED
        | OxeCode::RT_INVALID_TRANSITION
        | OxeCode::RT_NEEDS_POWER_OFF
        | OxeCode::RT_ALREADY_IN_STATE
        | OxeCode::CFG_MANIFEST_UNREADABLE
        | OxeCode::ENG_NOT_AVAILABLE => Code::FailedPrecondition,
        _ => Code::Internal,
    }
}

impl From<OxeError> for ApiError {
    fn from(e: OxeError) -> Self {
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
            StoreError::Oxe(e) => e.into(),
            StoreError::Conflict { current } => Self {
                status: Code::Aborted,
                code: OxeCode::CFG_ETAG_MISMATCH,
                detail: None,
                etag: Some(current.to_string()),
            },
            StoreError::NotFound(what) => Self::not_found(what),
            other => Self::new(Code::Internal, OxeCode::ENG_FAILED, other.to_string()),
        }
    }
}

impl From<EngineError> for ApiError {
    fn from(e: EngineError) -> Self {
        match e {
            EngineError::InvalidState(_) => {
                OxeError::with(OxeCode::RT_INVALID_TRANSITION, e.to_string()).into()
            }
            EngineError::Cancelled => OxeError::new(OxeCode::RT_CANCELLED).into(),
            EngineError::NotFound(_) => {
                Self::new(Code::Internal, OxeCode::ENG_FAILED, e.to_string())
            }
            EngineError::Failed { code, message } => match OxeCode::by_code(&code) {
                Some(c) => Self::new(status_for(c), c, message),
                None => Self::new(
                    Code::Internal,
                    OxeCode::ENG_FAILED,
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
            status.metadata_mut().insert(OXE_CODE_KEY, v);
        }
        if let Some(etag) = e.etag.and_then(|t| t.parse().ok()) {
            status.metadata_mut().insert(OXE_ETAG_KEY, etag);
        }
        status
    }
}
