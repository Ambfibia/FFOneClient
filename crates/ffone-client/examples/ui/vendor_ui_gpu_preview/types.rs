use super::*;

#[derive(Resource)]
pub(super) struct PreviewOutput(pub(super) PathBuf);

#[derive(Resource)]
pub(super) struct PreviewAssets {
    pub(super) images: Vec<Handle<Image>>,
    pub(super) font: Handle<Font>,
    pub(super) service_font: Handle<Font>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PreviewCli {
    pub(super) language: String,
    pub(super) output: PathBuf,
}

#[derive(Resource, Default)]
pub(super) struct TryOnChecked(pub(super) bool);

#[derive(Resource)]
pub(super) struct TryOnFixture(pub(super) ffone_client::character_creation_data::CharacterCreationData);

pub(super) struct PreviewEligibility;

impl VendorEquipEligibility0104 for PreviewEligibility {
    fn enable_equip(&self, value: ItemBase0104) -> Option<bool> {
        Some(value.item_id != 107)
    }
}
