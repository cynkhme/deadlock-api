//! Assets routes: heroes, items and ranks as the game client describes them.
//!
//! The responses are large: `items` is about 6.4 MB and `heroes` about
//! 1.9 MB uncompressed (2026-10-25 sample). On a slow link `items` may not
//! fit the 60 s default timeout; the `gzip` feature asks the server for a
//! compressed body. Assets change with the game client, so they are never
//! cached through [`Cache`](crate::Cache).

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{HeroId, ItemId};

/// A string-valued enum with a catch-all variant: decoded from and encoded
/// as the server's spelling, with a value this crate does not know kept
/// verbatim in the catch-all.
macro_rules! string_enum {
    (
        $(#[$doc:meta])*
        $name:ident, $other:ident { $($variant:ident = $text:literal),* $(,)? }
    ) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(from = "String", into = "String")]
        #[non_exhaustive]
        pub enum $name {
            $(
                #[doc = concat!("`", $text, "`.")]
                $variant,
            )*
            /// A value this crate does not know, kept verbatim and sent as is.
            $other(String),
        }

        impl $name {
            /// The value as the server spells it.
            pub fn as_str(&self) -> &str {
                match self {
                    $($name::$variant => $text,)*
                    $name::$other(s) => s,
                }
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                match s.as_str() {
                    $($text => $name::$variant,)*
                    _ => $name::$other(s),
                }
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.as_str().to_owned()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

string_enum! {
    /// The language of the localised strings in an Assets response, as the
    /// OpenAPI document of 2026-10-24 lists it (the game's
    /// `localization/<lang>.json` keys).
    Language, Other {
        Brazilian = "brazilian",
        Bulgarian = "bulgarian",
        Czech = "czech",
        Danish = "danish",
        Dutch = "dutch",
        English = "english",
        Finnish = "finnish",
        French = "french",
        German = "german",
        Greek = "greek",
        Hungarian = "hungarian",
        Indonesian = "indonesian",
        Italian = "italian",
        Japanese = "japanese",
        Koreana = "koreana",
        Latam = "latam",
        Norwegian = "norwegian",
        Polish = "polish",
        Portuguese = "portuguese",
        Romanian = "romanian",
        Russian = "russian",
        Schinese = "schinese",
        Spanish = "spanish",
        Swedish = "swedish",
        Tchinese = "tchinese",
        Thai = "thai",
        Turkish = "turkish",
        Ukrainian = "ukrainian",
        Vietnamese = "vietnamese",
    }
}

string_enum! {
    /// The shop column of an upgrade: `item_slot_type`, and the path segment
    /// of `GET /v1/assets/items/by-slot-type/{slot_type}`.
    ItemSlot, Unknown {
        Weapon = "weapon",
        Vitality = "vitality",
        Spirit = "spirit",
    }
}

string_enum! {
    /// The kind of an item: its `type` field, and the path segment of
    /// `GET /v1/assets/items/by-type/{type}`.
    ItemKind, Unknown {
        Ability = "ability",
        Upgrade = "upgrade",
        Weapon = "weapon",
    }
}

/// Parameters every Assets route takes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AssetsQuery {
    /// The language of names and descriptions; `None` is the server's default.
    pub language: Option<Language>,
    /// The game client version to describe; `None` is the current one. The
    /// server takes an `int32`, so a value above `i32::MAX` is refused with
    /// [`Error::InvalidRequest`](crate::Error::InvalidRequest) before a
    /// request is sent.
    pub client_version: Option<u32>,
}

impl AssetsQuery {
    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut q = Vec::new();
        if let Some(language) = &self.language {
            q.push(("language", language.as_str().to_owned()));
        }
        if let Some(version) = self.client_version {
            q.push(("client_version", version.to_string()));
        }
        q
    }

    /// Why the query cannot be sent, if it cannot.
    pub(crate) fn invalid(&self) -> Option<String> {
        self.client_version
            .filter(|&v| i32::try_from(v).is_err())
            .map(|v| format!("client_version {v} is above the server's int32 range"))
    }
}

/// Parameters of `GET /v1/assets/heroes`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct HeroesQuery {
    /// The parameters every Assets route takes.
    pub assets: AssetsQuery,
    /// The server's `only_active` filter: `true` keeps the heroes it marks
    /// active, `false` the others; `None` sends no filter.
    pub only_active: Option<bool>,
}

impl HeroesQuery {
    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut q = self.assets.query_pairs();
        if let Some(only_active) = self.only_active {
            q.push(("only_active", only_active.to_string()));
        }
        q
    }

    pub(crate) fn invalid(&self) -> Option<String> {
        self.assets.invalid()
    }
}

/// A hero, from `GET /v1/assets/heroes` and the single-hero routes.
///
/// Field presence counts below are from the 65 heroes of the 2026-10-25
/// sample. Objects whose meaning the sample does not explain are kept as
/// [`Value`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Hero {
    /// The hero id.
    pub id: HeroId,
    /// The game's internal name, e.g. `hero_inferno`.
    pub class_name: String,
    /// The display name in the requested language.
    pub name: String,
    /// The name the in-game search matches.
    pub search_name: Option<String>,
    /// Whether players can pick the hero.
    #[serde(default)]
    pub player_selectable: bool,
    /// Whether the hero is disabled.
    #[serde(default)]
    pub disabled: bool,
    /// Whether the hero is still in development.
    #[serde(default)]
    pub in_development: bool,
    /// Whether the hero is flagged for testing.
    #[serde(default)]
    pub needs_testing: bool,
    /// Whether only assigned players can pick the hero.
    #[serde(default)]
    pub assigned_players_only: bool,
    /// Whether the hero is in limited testing.
    #[serde(default)]
    pub limited_testing: bool,
    /// Whether the hero is in prerelease only; 44 of 65 heroes carry it.
    pub prerelease_only: Option<bool>,
    /// The development state; 44 of 65.
    pub development_state: Option<String>,
    /// The gender; 49 of 65.
    pub gender: Option<String>,
    /// The gun tag; 45 of 65.
    pub gun_tag: Option<String>,
    /// The hero's role; observed values in the sample: `assassin`, `brawler`,
    /// `marksman`, `mystic` (45 of 65).
    pub hero_type: Option<String>,
    /// The rich-presence text in the hideout; 44 of 65.
    pub hideout_rich_presence: Option<String>,
    /// How hard the hero is to play; observed values 1 to 4.
    pub complexity: Option<u32>,
    /// The skin id.
    pub skin: Option<u32>,
    /// Tags such as roles.
    #[serde(default)]
    pub tags: Vec<String>,
    /// The texts describing the hero.
    #[serde(default)]
    pub description: HeroDescription,
    /// Image URLs by image name; up to 17 in the sample.
    #[serde(default)]
    pub images: BTreeMap<String, String>,
    /// The `class_name` of each of the hero's items, by slot.
    #[serde(default)]
    pub items: BTreeMap<String, String>,
    /// Starting stats; not typed.
    pub starting_stats: Option<Value>,
    /// Popular items; 44 of 65, not typed.
    pub popular_items: Option<Value>,
    /// Item slot info; not typed.
    pub item_slot_info: Option<Value>,
    /// Physics settings; not typed.
    pub physics: Option<Value>,
    /// UI colours; not typed.
    pub colors: Option<Value>,
    /// Shop stat display; not typed.
    pub shop_stat_display: Option<Value>,
    /// Cost bonuses; not typed.
    pub cost_bonuses: Option<Value>,
    /// Stat display; not typed.
    pub stats_display: Option<Value>,
    /// Hero stats UI; not typed.
    pub hero_stats_ui: Option<Value>,
    /// Level info; not typed.
    pub level_info: Option<Value>,
    /// Scaling stats; not typed.
    pub scaling_stats: Option<Value>,
    /// Purchase bonuses; not typed.
    pub purchase_bonuses: Option<Value>,
    /// Standard level-up upgrades; not typed.
    pub standard_level_up_upgrades: Option<Value>,
    /// Item draft bucketing; not typed.
    pub item_draft_bucketing: Option<Value>,
    /// Model references; not typed.
    pub models: Option<Value>,
}

/// The texts of a [`Hero`]; a hero in the sample has none, one or all three.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HeroDescription {
    /// The lore text.
    pub lore: Option<String>,
    /// The playstyle text.
    pub playstyle: Option<String>,
    /// The role text.
    pub role: Option<String>,
}

/// An item from `GET /v1/assets/items` and the other item routes: an
/// ability, a shop upgrade or a weapon, told apart by its `type` field.
///
/// A list of items decodes strictly: an item of a `type` this crate does not
/// know is kept whole as [`AssetItem::Other`], but a known item with a field
/// of the wrong type fails the call with
/// [`Error::Decode`](crate::Error::Decode). In the 2026-10-25 sample of 781
/// items, 436 are abilities, 250 upgrades and 95 weapons.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum AssetItem {
    /// `type: "ability"`.
    Ability(AssetAbility),
    /// `type: "upgrade"`, an item of the shop.
    Upgrade(AssetUpgrade),
    /// `type: "weapon"`.
    Weapon(AssetWeapon),
    /// An item of a type this crate does not know, as the server sent it.
    Other(Value),
}

impl AssetItem {
    /// The item id; for [`AssetItem::Other`], its `id` field if that is one.
    pub fn id(&self) -> Option<ItemId> {
        match self {
            AssetItem::Ability(a) => Some(a.id),
            AssetItem::Upgrade(u) => Some(u.id),
            AssetItem::Weapon(w) => Some(w.id),
            AssetItem::Other(v) => v
                .get("id")
                .and_then(Value::as_u64)
                .and_then(|id| ItemId::try_from(id).ok()),
        }
    }

    /// The game's internal name; for [`AssetItem::Other`], its `class_name`
    /// field if that is a string.
    pub fn class_name(&self) -> Option<&str> {
        match self {
            AssetItem::Ability(a) => Some(&a.class_name),
            AssetItem::Upgrade(u) => Some(&u.class_name),
            AssetItem::Weapon(w) => Some(&w.class_name),
            AssetItem::Other(v) => v.get("class_name").and_then(Value::as_str),
        }
    }

    /// The display name; for [`AssetItem::Other`], its `name` field if that
    /// is a string.
    pub fn name(&self) -> Option<&str> {
        match self {
            AssetItem::Ability(a) => Some(&a.name),
            AssetItem::Upgrade(u) => Some(&u.name),
            AssetItem::Weapon(w) => Some(&w.name),
            AssetItem::Other(v) => v.get("name").and_then(Value::as_str),
        }
    }

    /// The item's `type`; for [`AssetItem::Other`], [`ItemKind::Unknown`]
    /// with its `type` field, or `None` when it has no string `type`.
    pub fn kind(&self) -> Option<ItemKind> {
        match self {
            AssetItem::Ability(_) => Some(ItemKind::Ability),
            AssetItem::Upgrade(_) => Some(ItemKind::Upgrade),
            AssetItem::Weapon(_) => Some(ItemKind::Weapon),
            AssetItem::Other(v) => v
                .get("type")
                .and_then(Value::as_str)
                .map(|t| ItemKind::from(t.to_owned())),
        }
    }
}

impl<'de> Deserialize<'de> for AssetItem {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;
        let value = Value::deserialize(deserializer)?;
        let Some(object) = value.as_object() else {
            return Err(D::Error::custom("expected an item object"));
        };
        let decode = |e: serde_json::Error| D::Error::custom(e.to_string());
        Ok(match object.get("type").and_then(Value::as_str) {
            Some("ability") => AssetItem::Ability(serde_json::from_value(value).map_err(decode)?),
            Some("upgrade") => AssetItem::Upgrade(serde_json::from_value(value).map_err(decode)?),
            Some("weapon") => AssetItem::Weapon(serde_json::from_value(value).map_err(decode)?),
            _ => AssetItem::Other(value),
        })
    }
}

impl Serialize for AssetItem {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::Error as _;
        let (kind, fields) = match self {
            AssetItem::Ability(a) => ("ability", serde_json::to_value(a)),
            AssetItem::Upgrade(u) => ("upgrade", serde_json::to_value(u)),
            AssetItem::Weapon(w) => ("weapon", serde_json::to_value(w)),
            AssetItem::Other(v) => return v.serialize(serializer),
        };
        let mut fields = fields.map_err(S::Error::custom)?;
        if let Value::Object(map) = &mut fields {
            map.insert("type".into(), Value::String(kind.into()));
        }
        fields.serialize(serializer)
    }
}

/// An ability: `type: "ability"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AssetAbility {
    /// The item id.
    pub id: ItemId,
    /// The game's internal name.
    pub class_name: String,
    /// The display name in the requested language.
    pub name: String,
    /// Observed values in the sample: `held`, `innate`, `item`, `melee`,
    /// `signature`, `ultimate`.
    pub ability_type: Option<String>,
    /// The hero the ability belongs to; 322 of 436 abilities carry it.
    pub hero: Option<HeroId>,
    /// The heroes that can use the ability.
    #[serde(default)]
    pub heroes: Vec<HeroId>,
    /// Whether the ability starts trained.
    #[serde(default)]
    pub start_trained: bool,
    /// The image URL; 28 of 436 abilities have none.
    pub image: Option<String>,
    /// The WebP image URL.
    pub image_webp: Option<String>,
    /// The ability's properties by name; values not typed.
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
    /// The ability's upgrades; not typed.
    #[serde(default)]
    pub upgrades: Vec<Value>,
    /// Behaviour flags; not typed.
    #[serde(default)]
    pub behaviours: Vec<Value>,
    /// The damage scale against bosses; 315 of 436.
    pub boss_damage_scale: Option<f64>,
    /// Abilities this one depends on; not typed.
    pub dependent_abilities: Option<Value>,
    /// Tooltip details; not typed.
    pub tooltip_details: Option<Value>,
    /// Videos; one ability in the sample has them, not typed.
    pub videos: Option<Value>,
    /// One ability in the sample carries it.
    pub grant_ammo_on_cast: Option<bool>,
    /// The description texts; not typed.
    #[serde(default)]
    pub description: Value,
    /// Weapon info; not typed.
    pub weapon_info: Option<Value>,
    /// The last update, as the server's integer timestamp.
    pub update_time: Option<u64>,
}

/// A shop upgrade: `type: "upgrade"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AssetUpgrade {
    /// The item id.
    pub id: ItemId,
    /// The game's internal name.
    pub class_name: String,
    /// The display name in the requested language.
    pub name: String,
    /// The shop column.
    pub item_slot_type: ItemSlot,
    /// The shop tier; observed values 1 to 5.
    pub item_tier: Option<u32>,
    /// The price in souls.
    pub cost: Option<u32>,
    /// How the item is used; observed values in the sample: `instant_cast`,
    /// `instant_cast_toggle`, `passive`, `press`.
    pub activation: Option<String>,
    /// Whether the item has an active ability.
    #[serde(default)]
    pub is_active_item: bool,
    /// Whether the shop sells the item.
    #[serde(default)]
    pub shopable: bool,
    /// Whether the item is disabled; 98 of 250 upgrades carry it.
    pub disabled: Option<bool>,
    /// The `class_name` of each item this one is built from.
    #[serde(default)]
    pub component_items: Vec<String>,
    /// The shop version; 0 throughout the sample.
    pub shop_version: Option<u32>,
    /// The shop image URL; 184 of 250.
    pub shop_image: Option<String>,
    /// The WebP shop image URL.
    pub shop_image_webp: Option<String>,
    /// The imbue target; 13 of 250.
    pub imbue: Option<String>,
    /// Shop filters; not typed.
    #[serde(default)]
    pub shop_filters: Vec<Value>,
    /// Disabled shop filters; not typed.
    #[serde(default)]
    pub disabled_shop_filters: Vec<Value>,
    /// The item's upgrades; not typed.
    #[serde(default)]
    pub upgrades: Vec<Value>,
    /// The corrupted variant; 95 of 250, not typed.
    pub corrupted_info: Option<Value>,
    /// Whether the item starts trained.
    #[serde(default)]
    pub start_trained: bool,
    /// The heroes the item is limited to.
    #[serde(default)]
    pub heroes: Vec<HeroId>,
    /// The image URL; 19 of 250 upgrades have none.
    pub image: Option<String>,
    /// The WebP image URL.
    pub image_webp: Option<String>,
    /// The item's properties by name; values not typed.
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
    /// The description texts; not typed.
    #[serde(default)]
    pub description: Value,
    /// The tooltip sections; not typed.
    #[serde(default)]
    pub tooltip_sections: Value,
    /// Weapon info; not typed.
    pub weapon_info: Option<Value>,
    /// The last update, as the server's integer timestamp.
    pub update_time: Option<u64>,
}

/// A weapon: `type: "weapon"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AssetWeapon {
    /// The item id.
    pub id: ItemId,
    /// The game's internal name.
    pub class_name: String,
    /// The display name in the requested language.
    pub name: String,
    /// The hero the weapon belongs to; 68 of 95 weapons carry it.
    pub hero: Option<HeroId>,
    /// The heroes that can use the weapon.
    #[serde(default)]
    pub heroes: Vec<HeroId>,
    /// Whether the weapon starts trained.
    #[serde(default)]
    pub start_trained: bool,
    /// The image URL; 5 of 95 weapons have none.
    pub image: Option<String>,
    /// The WebP image URL.
    pub image_webp: Option<String>,
    /// The weapon's properties by name; values not typed.
    #[serde(default)]
    pub properties: BTreeMap<String, Value>,
    /// Weapon info; not typed.
    pub weapon_info: Option<Value>,
    /// The crosshair's CSS class; 2 of 95.
    pub crosshair_css_class: Option<String>,
    /// Custom crosshair settings; 7 of 95, not typed.
    pub custom_crosshair_settings: Option<Value>,
    /// Whether the custom crosshair settings apply; 6 of 95.
    pub use_custom_crosshair_settings: Option<bool>,
    /// The last update, as the server's integer timestamp.
    pub update_time: Option<u64>,
}

/// A rank, from `GET /v1/assets/ranks` and `GET /v1/assets/ranks/{tier}`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Rank {
    /// The tier, 0 to 11 in the sample; tier 0 is `Obscurus`, the unranked tier.
    pub tier: u32,
    /// The display name in the requested language.
    pub name: String,
    /// The rank colour as the server sends it.
    pub color: String,
    /// Badge image URLs by name, per subrank and format: 4 for tier 0, 40
    /// for the others in the sample. The images themselves are served by
    /// `GET /v1/assets/ranks/{tier}/{subrank}/image` as PNG or WebP, which
    /// `get_raw` fetches.
    #[serde(default)]
    pub images: BTreeMap<String, String>,
}
