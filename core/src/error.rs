use serde::{Deserialize, Serialize};
use specta::Type;

/// Canonical error surface handed to the UI. Categories are actionable; internal
/// detail is logged inside the core and never leaked through `Internal`.
#[derive(Debug, Clone, Serialize, Deserialize, Type, thiserror::Error)]
#[serde(tag = "type")]
pub enum CoreError {
    #[error("authentication failed: {message}")]
    Auth { message: String },
    #[error("network error")]
    Network,
    #[error("not found")]
    NotFound,
    #[error("forbidden")]
    Forbidden,
    #[error("rate limited")]
    RateLimited {
        #[specta(type = f64)]
        retry_after_ms: u64,
    },
    #[error("encryption error: {message}")]
    Encryption { message: String },
    #[error("internal error")]
    Internal,
}
