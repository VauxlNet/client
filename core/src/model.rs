use serde::{Deserialize, Serialize};
use specta::Type;

use crate::ids::*;

// ---------- Session and identity ----------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum SessionState {
    LoggedOut,
    Authenticating,
    Recovering,
    Ready { user: User },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct LoginRequest {
    pub homeserver: String,
    pub username: String,
    /// Crosses the boundary once, into the core. The core persists the session,
    /// never the password. OIDC/SSO (where the password never touches our code)
    /// is the more secure path to add later.
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SessionInfo {
    pub user: User,
    pub device_id: DeviceId,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum Presence {
    Online,
    Idle,
    DoNotDisturb,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct User {
    pub id: UserId,
    pub display_name: String,
    pub avatar: Option<MediaRef>,
    pub presence: Presence,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum Membership {
    Joined,
    Invited,
    Left,
    Banned,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Member {
    pub user: User,
    pub membership: Membership,
    /// Canonical permission weight (a Matrix power level maps in here).
    pub power_level: i32,
    pub roles: Vec<String>,
}

// ---------- Spaces and rooms (Discord on Matrix) ----------

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Space {
    pub id: SpaceId,
    pub name: String,
    pub avatar: Option<MediaRef>,
    pub rooms: Vec<RoomId>,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum RoomKind {
    Text,
    Voice,
    Announcement,
    DirectMessage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum EncryptionState {
    Unencrypted,
    Encrypted,
    /// Encrypted, but unverified devices are present (show a shield warning).
    EncryptedUnverified,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct UnreadInfo {
    pub unread: u32,
    pub highlights: u32,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MessagePreview {
    pub sender_name: String,
    pub snippet: String,
    #[specta(type = f64)]
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Room {
    pub id: RoomId,
    pub space: Option<SpaceId>,
    pub kind: RoomKind,
    pub name: String,
    pub topic: Option<String>,
    pub avatar: Option<MediaRef>,
    pub unread: UnreadInfo,
    pub encryption: EncryptionState,
    pub member_count: u32,
    pub last_message: Option<MessagePreview>,
}

// ---------- Media ----------

/// Opaque handle. The UI never fetches from the homeserver and never holds media
/// keys: it renders via a custom protocol the Rust side serves (for example
/// `vauxl-media://{id}`), where the core fetches the ciphertext, decrypts in
/// Rust, and streams plaintext bytes to the webview.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct MediaRef {
    pub id: String,
    pub thumbnail: Option<String>,
}

// ---------- Messages and content ----------

/// HTML the core only constructs after sanitizing (for example with `ammonia`).
/// The UI can render it without re-sanitizing. Raw HTML never crosses the boundary.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(transparent)]
pub struct SafeHtml(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum SenderTrust {
    Verified,
    Unverified,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum SendState {
    /// Optimistic local echo, not yet acknowledged.
    Local,
    Sending,
    Sent,
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Reaction {
    pub key: String,
    pub count: u32,
    pub me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum MessageContent {
    Text {
        body: String,
        formatted: Option<SafeHtml>,
    },
    Image {
        media: MediaRef,
        alt: Option<String>,
        blurhash: Option<String>,
        width: u32,
        height: u32,
    },
    File {
        media: MediaRef,
        filename: String,
        #[specta(type = f64)]
        size: u64,
        mime: String,
    },
    Audio {
        media: MediaRef,
        duration_ms: u32,
    },
    Redacted,
    /// Graceful fallback for unknown or extension content.
    Unsupported {
        kind: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Message {
    pub id: MessageId,
    pub room: RoomId,
    pub sender: UserId,
    #[specta(type = f64)]
    pub timestamp: u64,
    pub content: MessageContent,
    pub reactions: Vec<Reaction>,
    pub reply_to: Option<MessageId>,
    pub edited: bool,
    pub send_state: SendState,
    pub sender_trust: SenderTrust,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum OutgoingContent {
    Text {
        body: String,
        formatted: Option<String>,
        reply_to: Option<MessageId>,
    },
    Media {
        upload: MediaRef,
        caption: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TimelineChunk {
    pub messages: Vec<Message>,
    pub reached_start: bool,
}
