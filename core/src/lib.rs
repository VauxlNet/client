//! vauxl-core: the protocol-agnostic chat contract plus a MockBackend.
//!
//! This crate deliberately has no Tauri and no Matrix dependency. The same
//! `ChatBackend` trait is implemented by `MockBackend` (here, for the prototype)
//! and later by a matrix-rust-sdk backend. The UI depends only on these types,
//! so the shell and the protocol stay swappable.

pub mod backend;
pub mod error;
pub mod event;
pub mod ids;
pub mod mock;
pub mod model;

pub use backend::ChatBackend;
pub use error::CoreError;
pub use event::{CoreEvent, SasEmoji, SyncState, TimelineChange, VerificationUpdate};
pub use ids::{DeviceId, MessageId, RoomId, SpaceId, UserId};
pub use mock::MockBackend;
pub use model::*;
