use super::*;

pub(super) const PLAYER_ITEM_SET_CATALOG_SCHEMA: &str = "ffone.player-item-set-catalog.v1";

pub(super) const PLAYER_ITEM_SET_CATALOG_PATH: &str = "characters/player/items/catalog.json";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PlayerItemRouteCatalog {
    pub(super) schema: String,
    pub(super) models: Vec<PlayerItemRoute>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PlayerItemRoute {
    pub(super) category: String,
    pub(super) true_name: String,
    pub(super) source_route: String,
    pub(super) resource_set: String,
    pub(super) model: CharacterCreationAssetReference,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum NativeAssetKind {
    Model,
    Texture,
}

impl NativeAssetKind {
    pub(super) const fn extension(self) -> &'static str {
        match self {
            Self::Model => "glb",
            Self::Texture => "png",
        }
    }
}

pub(super) fn normalize_relative_path(path: &str) -> CharacterCreationDataResult<String> {
    let normalized = path.replace('\\', "/");
    if normalized.is_empty()
        || normalized.starts_with('/')
        || normalized
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return invalid(format!("unsafe project asset route {path:?}"));
    }
    Ok(normalized)
}
