//! Identifier newtypes.
//!
//! Each wraps the integer the server sends and serializes transparently, so a
//! typed id costs nothing on the wire and cannot be mixed up with another kind.

use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! id_newtype {
    ($(#[$doc:meta])* $name:ident($inner:ty)) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub $inner);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

id_newtype! {
    /// A Deadlock match id, e.g. `122539002`.
    MatchId(u64)
}

id_newtype! {
    /// A player's Steam account id in 32-bit form (`steam32`), as the server sends it.
    AccountId(u32)
}

id_newtype! {
    /// A hero id as used by the game and the API.
    HeroId(u32)
}

id_newtype! {
    /// An item id (ability, upgrade or weapon) as used by the game and the
    /// Assets routes. Match metadata carries item ids as `u64`; convert one
    /// with `ItemId::try_from`.
    ItemId(u32)
}

impl TryFrom<u64> for ItemId {
    type Error = std::num::TryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        u32::try_from(value).map(ItemId)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_transparent_in_json() {
        assert_eq!(serde_json::to_string(&MatchId(42)).unwrap(), "42");
        assert_eq!(
            serde_json::from_str::<AccountId>("7").unwrap(),
            AccountId(7)
        );
        assert_eq!(MatchId::from(5u64).to_string(), "5");
        assert_eq!(HeroId::from(3u32), HeroId(3));
    }

    #[test]
    fn item_id_converts_from_u64_within_u32() {
        assert_eq!(
            ItemId::try_from(4_284_855_775u64),
            Ok(ItemId(4_284_855_775))
        );
        assert!(ItemId::try_from(u64::from(u32::MAX) + 1).is_err());
    }
}
