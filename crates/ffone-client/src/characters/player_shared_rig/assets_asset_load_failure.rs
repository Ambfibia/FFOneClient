use super::*;

pub(super) const PLAYER_ITEM_SET_CATALOG_PATH: &str = "characters/player/items/catalog.json";

pub(super) const PLAYER_ITEM_SET_CATALOG_SCHEMA: &str = "ffone.player-item-set-catalog.v1";

pub(super) const SKINNED_BACK_CLOTHES_INDEX: u8 = 5;

pub(super) const fn actor_skin_combiner_clothes_index(category: CharacterAppearanceCategory) -> u8 {
    match category {
        CharacterAppearanceCategory::Shoes => 0,
        CharacterAppearanceCategory::Pants => 1,
        CharacterAppearanceCategory::Shirt => 2,
        CharacterAppearanceCategory::Face => 3,
        CharacterAppearanceCategory::Hair => 4,
    }
}

pub(super) fn asset_uri(source: Option<&str>, relative: &str) -> String {
    source.map_or_else(
        || relative.to_owned(),
        |source| format!("{source}://{relative}"),
    )
}

pub(super) fn asset_load_failure<A: Asset>(asset_server: &AssetServer, handle: &Handle<A>) -> Option<String> {
    match asset_server.load_state(handle.id()) {
        LoadState::Failed(error) => Some(format!("asset load failed: {error}")),
        _ => asset_server
            .get_recursive_dependency_load_state(handle.id())
            .and_then(|state| match state {
                RecursiveDependencyLoadState::Failed(error) => {
                    Some(format!("dependency load failed: {error}"))
                }
                _ => None,
            }),
    }
}

pub(super) fn flattened_renderer_index(renderer_base: u16, local_renderer_index: usize) -> Option<u16> {
    let local_renderer_index = u16::try_from(local_renderer_index).ok()?;
    renderer_base.checked_add(local_renderer_index)
}
