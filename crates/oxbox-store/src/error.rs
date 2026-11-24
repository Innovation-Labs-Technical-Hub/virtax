use oxbox_model::OxeError;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error(transparent)]
    Oxe(#[from] OxeError),
    #[error("config revision conflict: current revision is {current}")]
    Conflict { current: u64 },
    #[error("not found: {0}")]
    NotFound(String),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
