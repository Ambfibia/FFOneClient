use super::*;

#[test]
fn four_slot_model_preserves_uid_selection_and_rejects_empty_slots() {
    let mut model = CharacterSelectionUiModel::default();
    model.set_slots(
        [
            occupied(10, CharacterLocationBackground::Future),
            CharacterSlotUi::Empty,
            occupied(30, CharacterLocationBackground::Wilds),
            CharacterSlotUi::SubscriptionLocked,
        ],
        Some(30),
    );
    assert_eq!(model.selected_slot, Some(2));
    assert_eq!(model.selected_character().unwrap().pc_uid, 30);
    assert_eq!(
        model.selected_background(),
        CharacterLocationBackground::Wilds
    );
    assert!(!model.select_slot(1));
    assert_eq!(model.selected_slot, Some(2));
    assert!(model.select_slot(0));
    assert_eq!(model.selected_character().unwrap().pc_uid, 10);
}
