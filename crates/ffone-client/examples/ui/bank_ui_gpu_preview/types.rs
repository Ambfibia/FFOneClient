use super::*;

#[derive(Resource)]
pub(super) struct PreviewOutput(pub(super) PathBuf);

#[derive(Resource)]
pub(super) struct PreviewAssets {
    pub(super) images: Vec<Handle<Image>>,
    pub(super) font: Handle<Font>,
    pub(super) search_font: Handle<Font>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PreviewCli {
    pub(super) language: String,
    pub(super) output: PathBuf,
}

pub(super) struct PreviewEligibility;

impl BankEquipEligibility for PreviewEligibility {
    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }
}
