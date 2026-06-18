use serde::{Deserialize, Serialize};
use specta::Type;

use crate::ids::*;
use crate::model::*;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum SyncState {
    Offline,
    Connecting,
    Syncing,
    Live,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "op")]
pub enum TimelineChange {
    Added { message: Message },
    Updated { message: Message },
    Removed { id: MessageId },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SasEmoji {
    pub symbol: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum VerificationUpdate {
    Requested { from: UserId, flow: String },
    ShowSas { flow: String, emoji: Vec<SasEmoji> },
    Done { flow: String },
    Cancelled { flow: String, reason: String },
}

/// Reactive updates pushed from the core to the UI. The Tauri layer forwards
/// each of these to the webview as an event.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum CoreEvent {
    Session { state: SessionState },
    Sync { state: SyncState },
    RoomUpserted { room: Room },
    RoomRemoved { room: RoomId },
    SpaceUpserted { space: Space },
    Timeline { room: RoomId, change: TimelineChange },
    Typing { room: RoomId, users: Vec<UserId> },
    Receipt { room: RoomId, user: UserId, up_to: MessageId },
    Presence { user: User },
    Verification { update: VerificationUpdate },
}
