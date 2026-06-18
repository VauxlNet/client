use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio::sync::broadcast;

use crate::backend::ChatBackend;
use crate::error::CoreError;
use crate::event::*;
use crate::ids::*;
use crate::model::*;

const BASE_TS: u64 = 1_747_000_000_000;

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(BASE_TS)
}

struct MockState {
    me: User,
    spaces: Vec<Space>,
    rooms: Vec<Room>,
    timelines: HashMap<RoomId, Vec<Message>>,
    members: HashMap<RoomId, Vec<Member>>,
}

/// In-memory backend with canned data, so the entire UI can be built and felt
/// before any Matrix code exists. The only throwaway piece in the architecture.
pub struct MockBackend {
    state: Mutex<MockState>,
    tx: broadcast::Sender<CoreEvent>,
    counter: AtomicU64,
}

impl MockBackend {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(256);
        Self {
            state: Mutex::new(build_sample()),
            tx,
            counter: AtomicU64::new(1000),
        }
    }

    fn next_id(&self, prefix: &str) -> String {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}{n}")
    }

    fn make_message(&self, room: &RoomId, sender: &UserId, body: &str) -> Message {
        Message {
            id: MessageId::from(self.next_id("m")),
            room: room.clone(),
            sender: sender.clone(),
            timestamp: now_ms(),
            content: MessageContent::Text {
                body: body.to_string(),
                formatted: None,
            },
            reactions: vec![],
            reply_to: None,
            edited: false,
            send_state: SendState::Sent,
            sender_trust: SenderTrust::Unverified,
        }
    }

    /// Emit fake live traffic (typing, then an incoming message) so the prototype
    /// shows reactive updates. Spawn this onto the Tauri async runtime.
    pub async fn run_demo_traffic(self: Arc<Self>) {
        let room = {
            let st = self.state.lock().unwrap();
            st.rooms.first().map(|r| r.id.clone())
        };
        let Some(room) = room else {
            return;
        };
        let alice = UserId::from("@alice:vauxl.local");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(8)).await;
            let _ = self.tx.send(CoreEvent::Typing {
                room: room.clone(),
                users: vec![alice.clone()],
            });
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let msg = self.make_message(&room, &alice, "ping from the mock backend");
            self.state
                .lock()
                .unwrap()
                .timelines
                .entry(room.clone())
                .or_default()
                .push(msg.clone());
            let _ = self.tx.send(CoreEvent::Typing {
                room: room.clone(),
                users: vec![],
            });
            let _ = self.tx.send(CoreEvent::Timeline {
                room: room.clone(),
                change: TimelineChange::Added { message: msg },
            });
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ChatBackend for MockBackend {
    async fn login(&self, _req: LoginRequest) -> Result<SessionInfo, CoreError> {
        let st = self.state.lock().unwrap();
        Ok(SessionInfo {
            user: st.me.clone(),
            device_id: DeviceId::from("MOCKDEVICE"),
        })
    }

    async fn restore_session(&self) -> Result<Option<SessionInfo>, CoreError> {
        let st = self.state.lock().unwrap();
        Ok(Some(SessionInfo {
            user: st.me.clone(),
            device_id: DeviceId::from("MOCKDEVICE"),
        }))
    }

    async fn logout(&self) -> Result<(), CoreError> {
        Ok(())
    }

    async fn list_spaces(&self) -> Result<Vec<Space>, CoreError> {
        Ok(self.state.lock().unwrap().spaces.clone())
    }

    async fn list_rooms(&self) -> Result<Vec<Room>, CoreError> {
        Ok(self.state.lock().unwrap().rooms.clone())
    }

    async fn get_members(&self, room: RoomId) -> Result<Vec<Member>, CoreError> {
        Ok(self
            .state
            .lock()
            .unwrap()
            .members
            .get(&room)
            .cloned()
            .unwrap_or_default())
    }

    async fn load_timeline(&self, room: RoomId, limit: u32) -> Result<TimelineChunk, CoreError> {
        let st = self.state.lock().unwrap();
        let all = st.timelines.get(&room).cloned().unwrap_or_default();
        let start = all.len().saturating_sub(limit as usize);
        Ok(TimelineChunk {
            messages: all[start..].to_vec(),
            reached_start: start == 0,
        })
    }

    async fn load_older(
        &self,
        _room: RoomId,
        _before: MessageId,
        _limit: u32,
    ) -> Result<TimelineChunk, CoreError> {
        Ok(TimelineChunk {
            messages: vec![],
            reached_start: true,
        })
    }

    async fn send_message(
        &self,
        room: RoomId,
        content: OutgoingContent,
    ) -> Result<MessageId, CoreError> {
        let body = match content {
            OutgoingContent::Text { body, .. } => body,
            OutgoingContent::Media { caption, .. } => caption.unwrap_or_else(|| "[media]".into()),
        };
        let me_id = self.state.lock().unwrap().me.id.clone();
        let msg = self.make_message(&room, &me_id, &body);
        self.state
            .lock()
            .unwrap()
            .timelines
            .entry(room.clone())
            .or_default()
            .push(msg.clone());
        let id = msg.id.clone();
        let _ = self.tx.send(CoreEvent::Timeline {
            room,
            change: TimelineChange::Added { message: msg },
        });
        Ok(id)
    }

    async fn edit_message(
        &self,
        _room: RoomId,
        _target: MessageId,
        _content: OutgoingContent,
    ) -> Result<(), CoreError> {
        Ok(())
    }

    async fn redact_message(&self, _room: RoomId, _target: MessageId) -> Result<(), CoreError> {
        Ok(())
    }

    async fn toggle_reaction(
        &self,
        _room: RoomId,
        _target: MessageId,
        _key: String,
    ) -> Result<(), CoreError> {
        Ok(())
    }

    async fn mark_read(&self, _room: RoomId, _up_to: MessageId) -> Result<(), CoreError> {
        Ok(())
    }

    async fn set_typing(&self, _room: RoomId, _typing: bool) -> Result<(), CoreError> {
        Ok(())
    }

    async fn upload_media(&self, _path: String) -> Result<MediaRef, CoreError> {
        Ok(MediaRef {
            id: self.next_id("media"),
            thumbnail: None,
        })
    }

    async fn request_verification(&self, _user: UserId) -> Result<(), CoreError> {
        Ok(())
    }

    async fn confirm_sas(&self, _flow: String) -> Result<(), CoreError> {
        Ok(())
    }

    async fn cancel_verification(&self, _flow: String) -> Result<(), CoreError> {
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<CoreEvent> {
        self.tx.subscribe()
    }
}

fn build_sample() -> MockState {
    let me = User {
        id: UserId::from("@you:vauxl.local"),
        display_name: "you".into(),
        avatar: None,
        presence: Presence::Online,
        status_message: Some("building Vauxl".into()),
    };
    let alice = User {
        id: UserId::from("@alice:vauxl.local"),
        display_name: "alice".into(),
        avatar: None,
        presence: Presence::Online,
        status_message: None,
    };
    let bob = User {
        id: UserId::from("@bob:vauxl.local"),
        display_name: "bob".into(),
        avatar: None,
        presence: Presence::Idle,
        status_message: None,
    };

    let general = RoomId::from("!general:vauxl.local");
    let random = RoomId::from("!random:vauxl.local");
    let voice = RoomId::from("!voice:vauxl.local");
    let friends_chat = RoomId::from("!friends:vauxl.local");
    let dm_alice = RoomId::from("!dm-alice:vauxl.local");

    let space_hq = SpaceId::from("!space-hq:vauxl.local");
    let space_friends = SpaceId::from("!space-friends:vauxl.local");

    let spaces = vec![
        Space {
            id: space_hq.clone(),
            name: "Vauxl HQ".into(),
            avatar: None,
            rooms: vec![general.clone(), random.clone(), voice.clone()],
            order: 0,
        },
        Space {
            id: space_friends.clone(),
            name: "Friends".into(),
            avatar: None,
            rooms: vec![friends_chat.clone()],
            order: 1,
        },
    ];

    let mk_room = |id: RoomId,
                   space: Option<SpaceId>,
                   kind: RoomKind,
                   name: &str,
                   topic: Option<&str>,
                   encryption: EncryptionState,
                   member_count: u32,
                   unread: u32| Room {
        id,
        space,
        kind,
        name: name.into(),
        topic: topic.map(|s| s.to_string()),
        avatar: None,
        unread: UnreadInfo {
            unread,
            highlights: 0,
            muted: false,
        },
        encryption,
        member_count,
        last_message: None,
    };

    let rooms = vec![
        mk_room(
            general.clone(),
            Some(space_hq.clone()),
            RoomKind::Text,
            "general",
            Some("Welcome to Vauxl HQ"),
            EncryptionState::Encrypted,
            3,
            0,
        ),
        mk_room(
            random.clone(),
            Some(space_hq.clone()),
            RoomKind::Text,
            "random",
            None,
            EncryptionState::Encrypted,
            3,
            2,
        ),
        mk_room(
            voice.clone(),
            Some(space_hq.clone()),
            RoomKind::Voice,
            "voice-lounge",
            None,
            EncryptionState::Unencrypted,
            3,
            0,
        ),
        mk_room(
            friends_chat.clone(),
            Some(space_friends.clone()),
            RoomKind::Text,
            "chat",
            None,
            EncryptionState::Encrypted,
            4,
            5,
        ),
        mk_room(
            dm_alice.clone(),
            None,
            RoomKind::DirectMessage,
            "alice",
            None,
            EncryptionState::Encrypted,
            2,
            1,
        ),
    ];

    let seed = |sender: &User, body: &str, offset: u64, n: u64| Message {
        id: MessageId::from(format!("m{n}")),
        room: general.clone(),
        sender: sender.id.clone(),
        timestamp: BASE_TS + offset,
        content: MessageContent::Text {
            body: body.into(),
            formatted: None,
        },
        reactions: vec![],
        reply_to: None,
        edited: false,
        send_state: SendState::Sent,
        sender_trust: SenderTrust::Verified,
    };

    let mut timelines: HashMap<RoomId, Vec<Message>> = HashMap::new();
    timelines.insert(
        general.clone(),
        vec![
            seed(&alice, "hey, welcome to the Vauxl prototype", 0, 1),
            seed(&bob, "this timeline is coming from the mock backend", 60_000, 2),
            seed(&me, "and the UI never touches a key", 120_000, 3),
        ],
    );
    timelines.insert(random.clone(), vec![]);
    timelines.insert(voice.clone(), vec![]);
    timelines.insert(friends_chat.clone(), vec![]);
    timelines.insert(dm_alice.clone(), vec![]);

    let mk_member = |u: &User, power_level: i32| Member {
        user: u.clone(),
        membership: Membership::Joined,
        power_level,
        roles: vec![],
    };
    let mut members: HashMap<RoomId, Vec<Member>> = HashMap::new();
    members.insert(
        general.clone(),
        vec![
            mk_member(&me, 100),
            mk_member(&alice, 50),
            mk_member(&bob, 0),
        ],
    );

    MockState {
        me,
        spaces,
        rooms,
        timelines,
        members,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_send_appends_and_emits() {
        let b = MockBackend::new();
        let rooms = b.list_rooms().await.unwrap();
        assert!(!rooms.is_empty());
        let room = rooms[0].id.clone();
        let before = b.load_timeline(room.clone(), 50).await.unwrap();

        let mut rx = b.subscribe();
        let id = b
            .send_message(
                room.clone(),
                OutgoingContent::Text {
                    body: "hi".into(),
                    formatted: None,
                    reply_to: None,
                },
            )
            .await
            .unwrap();

        let ev = tokio::time::timeout(std::time::Duration::from_secs(1), rx.recv())
            .await
            .expect("event timed out")
            .expect("recv failed");
        match ev {
            CoreEvent::Timeline {
                change: TimelineChange::Added { message },
                ..
            } => assert_eq!(message.id, id),
            other => panic!("unexpected event: {other:?}"),
        }

        let after = b.load_timeline(room, 50).await.unwrap();
        assert_eq!(after.messages.len(), before.messages.len() + 1);
    }
}
