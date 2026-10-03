use super::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CharacterRegistryModel {
    pub(super) id: String,
    pub(super) logical_name: String,
    pub(super) category: String,
    pub(super) glb: String,
    #[serde(default)]
    pub(super) animations: Vec<String>,
}

#[derive(Debug, Default, Resource)]
pub(super) struct ModelPreview {
    pub(super) current_index: Option<usize>,
    pub(super) generation: u64,
    pub(super) root: Option<Entity>,
    pub(super) gltf: Option<Handle<Gltf>>,
    pub(super) preserve_camera: bool,
}
