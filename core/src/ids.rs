use serde::{Deserialize, Serialize};
use specta::Type;

/// Opaque string identifiers. Newtypes so a `RoomId` can never be passed where a
/// `UserId` is expected. They serialize transparently to and from a string, and
/// the UI treats their contents as a black box.
macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Hash)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

string_id!(UserId);
string_id!(SpaceId);
string_id!(RoomId);
string_id!(MessageId);
string_id!(DeviceId);
