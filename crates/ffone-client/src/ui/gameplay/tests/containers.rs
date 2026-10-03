use super::*;

#[test]
fn tutorial_instruction_uses_integer_screen_quarter_like_unity_imgui() {
    assert_eq!(legacy_tutorial_instruction_top(720.0), 180.0);
    assert_eq!(legacy_tutorial_instruction_top(681.0), 170.0);
    assert_eq!(legacy_tutorial_instruction_top(1_080.0), 270.0);
    assert_eq!(
        legacy_tutorial_instruction_top(f32::NAN),
        TUTORIAL_TEXT_RECT.y
    );
}
