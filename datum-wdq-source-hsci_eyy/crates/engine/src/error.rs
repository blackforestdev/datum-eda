use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("object not found: {object_type} {uuid}")]
    NotFound {
        object_type: &'static str,
        uuid: Uuid,
    },

    #[error(
        "dangling reference: {source_type} {source_uuid} references missing {target_type} {target_uuid}"
    )]
    DanglingReference {
        source_type: &'static str,
        source_uuid: Uuid,
        target_type: &'static str,
        target_uuid: Uuid,
    },

    #[error("import error: {0}")]
    Import(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("operation failed: {0}")]
    Operation(String),

    #[error("unsupported rule scope: {node} (available from {available_from})")]
    UnsupportedScope {
        node: String,
        available_from: &'static str,
    },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("no project open")]
    NoProjectOpen,

    #[error("nothing to undo")]
    NothingToUndo,

    #[error("nothing to redo")]
    NothingToRedo,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
