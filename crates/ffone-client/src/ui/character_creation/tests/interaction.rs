use super::*;

#[test]
fn five_name_rows_and_scroll_wrap_are_stable() {
    let names = names();
    let mut model = CharacterCreationUiModel::default();
    model.name_indices[0] = 1;
    assert_eq!(
        model.visible_name_indices(&names, CharacterNamePart::First),
        Some([2, 3, 1, 2, 3])
    );
    assert!(model.scroll_name(&names, CharacterNamePart::First, -1));
    assert_eq!(model.name_indices[0], 3);
    assert!(model.scroll_name(&names, CharacterNamePart::First, 1));
    assert_eq!(model.name_indices[0], 1);
}
