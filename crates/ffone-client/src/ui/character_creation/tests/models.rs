use super::*;

#[test]
fn default_model_fails_closed_without_native_dependencies() {
    let model = CharacterCreationUiModel::default();
    assert!(!model.name_table.enabled());
    assert!(!model.custom_name_filter.enabled());
    assert!(!model.creation_items.enabled());
    assert!(!model.starter_icons.enabled());
    assert!(!model.reserve_name.enabled());
    assert!(!model.save_appearance.enabled());
    assert_eq!(
        model.preview,
        CharacterCreationPreviewStatus::PlayerAssemblyPending
    );
}
