use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::error::CoreError;
use crate::event::CoreEvent;
use crate::ids::*;
use crate::model::*;

/// The single seam between the UI and the world. `MockBackend` implements this
/// for the prototype; a `MatrixBackend` (matrix-rust-sdk) implements the exact
/// same trait for the real client. Swapping which one is constructed at startup
/// changes nothing in the UI.
#[async_trait]
pub trait ChatBackend: Send + Sync {
    // Session
    async fn login(&self, req: LoginRequest) -> Result<SessionInfo, CoreError>;
    async fn restore_session(&self) -> Result<Option<SessionInfo>, CoreError>;
    async fn logout(&self) -> Result<(), CoreError>;

    // Spaces and rooms
    async fn list_spaces(&self) -> Result<Vec<Space>, CoreError>;
    async fn list_rooms(&self) -> Result<Vec<Room>, CoreError>;
    async fn get_members(&self, room: RoomId) -> Result<Vec<Member>, CoreError>;

    // Timeline
    async fn load_timeline(&self, room: RoomId, limit: u32) -> Result<TimelineChunk, CoreError>;
    async fn load_older(
        &self,
        room: RoomId,
        before: MessageId,
        limit: u32,
    ) -> Result<TimelineChunk, CoreError>;
    async fn send_message(
        &self,
        room: RoomId,
        content: OutgoingContent,
    ) -> Result<MessageId, CoreError>;
    async fn edit_message(
        &self,
        room: RoomId,
        target: MessageId,
        content: OutgoingContent,
    ) -> Result<(), CoreError>;
    async fn redact_message(&self, room: RoomId, target: MessageId) -> Result<(), CoreError>;
    async fn toggle_reaction(
        &self,
        room: RoomId,
        target: MessageId,
        key: String,
    ) -> Result<(), CoreError>;
    async fn mark_read(&self, room: RoomId, up_to: MessageId) -> Result<(), CoreError>;
    async fn set_typing(&self, room: RoomId, typing: bool) -> Result<(), CoreError>;

    // Media (encrypt then upload; download/decrypt is served via custom protocol)
    async fn upload_media(&self, path: String) -> Result<MediaRef, CoreError>;

    // E2EE device verification: no key material crosses the boundary, only SAS emoji.
    async fn request_verification(&self, user: UserId) -> Result<(), CoreError>;
    async fn confirm_sas(&self, flow: String) -> Result<(), CoreError>;
    async fn cancel_verification(&self, flow: String) -> Result<(), CoreError>;

    /// Reactive stream the Tauri layer pumps into the webview as events.
    fn subscribe(&self) -> broadcast::Receiver<CoreEvent>;
}
