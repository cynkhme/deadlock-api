//! Assets routes.

use crate::endpoint::{Endpoint, path_segment, segment_problem};
use crate::types::assets::{AssetItem, AssetsQuery, Hero, HeroesQuery, ItemKind, ItemSlot, Rank};
use crate::{HeroId, ItemId};

/// An Assets route: its path, output and query type.
macro_rules! assets_route {
    ($(#[$doc:meta])* $name:ident { $($field:ident: $ty:ty),* } -> $out:ty, $query:ty, |$s:ident| $path:expr $(, invalid: $check:expr)?) => {
        $(#[$doc])*
        pub(crate) struct $name {
            $(pub $field: $ty,)*
            pub query: $query,
        }

        impl Endpoint for $name {
            type Output = $out;
            fn path(&self) -> String {
                let $s = self;
                $path
            }
            fn query(&self) -> Vec<(&'static str, String)> {
                self.query.query_pairs()
            }
            fn invalid(&self) -> Option<String> {
                let $s = self;
                $(if let Some(problem) = $check {
                    return Some(problem);
                })?
                $s.query.invalid()
            }
        }
    };
}

assets_route! {
    /// `GET /v1/assets/heroes`.
    ListHeroes {} -> Vec<Hero>, HeroesQuery, |_s| "/v1/assets/heroes".into()
}

assets_route! {
    /// `GET /v1/assets/heroes/{hero_id}`.
    GetHero { id: HeroId } -> Hero, AssetsQuery, |s| format!("/v1/assets/heroes/{}", s.id)
}

assets_route! {
    /// `GET /v1/assets/heroes/by-name/{name}`.
    GetHeroByName { name: String } -> Hero, AssetsQuery,
        |s| format!("/v1/assets/heroes/by-name/{}", path_segment(&s.name)),
        invalid: segment_problem("the hero name", &s.name)
}

assets_route! {
    /// `GET /v1/assets/items`.
    ListItems {} -> Vec<AssetItem>, AssetsQuery, |_s| "/v1/assets/items".into()
}

assets_route! {
    /// `GET /v1/assets/items/{id_or_class_name}` with an id.
    GetItem { id: ItemId } -> AssetItem, AssetsQuery, |s| format!("/v1/assets/items/{}", s.id)
}

assets_route! {
    /// `GET /v1/assets/items/{id_or_class_name}` with a class name.
    GetItemByClassName { class_name: String } -> AssetItem, AssetsQuery,
        |s| format!("/v1/assets/items/{}", path_segment(&s.class_name)),
        invalid: segment_problem("the class name", &s.class_name)
}

assets_route! {
    /// `GET /v1/assets/items/by-hero-id/{id}`.
    ListHeroItems { hero: HeroId } -> Vec<AssetItem>, AssetsQuery,
        |s| format!("/v1/assets/items/by-hero-id/{}", s.hero)
}

assets_route! {
    /// `GET /v1/assets/items/by-slot-type/{slot_type}`.
    ListItemsBySlot { slot: ItemSlot } -> Vec<AssetItem>, AssetsQuery,
        |s| format!("/v1/assets/items/by-slot-type/{}", path_segment(s.slot.as_str())),
        invalid: segment_problem("the item slot", s.slot.as_str())
}

assets_route! {
    /// `GET /v1/assets/items/by-type/{type}`.
    ListItemsByKind { kind: ItemKind } -> Vec<AssetItem>, AssetsQuery,
        |s| format!("/v1/assets/items/by-type/{}", path_segment(s.kind.as_str())),
        invalid: segment_problem("the item kind", s.kind.as_str())
}

assets_route! {
    /// `GET /v1/assets/ranks`.
    ListRanks {} -> Vec<Rank>, AssetsQuery, |_s| "/v1/assets/ranks".into()
}

assets_route! {
    /// `GET /v1/assets/ranks/{tier}`.
    GetRank { tier: u32 } -> Rank, AssetsQuery, |s| format!("/v1/assets/ranks/{}", s.tier)
}
