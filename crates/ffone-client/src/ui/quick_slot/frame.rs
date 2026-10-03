use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QuickSlotInputFrame {
    pub macro_chat_modifier_held: bool,
    pub slot_just_pressed: [bool; QUICK_SLOT_COUNT],
}

#[must_use]
pub fn quick_slot_input_frame_from_keys(keys: &ButtonInput<KeyCode>) -> QuickSlotInputFrame {
    QuickSlotInputFrame {
        // Clean localized.MacroChatKey is Unity KeyCode 304 (LeftShift).
        macro_chat_modifier_held: keys.pressed(KeyCode::ShiftLeft),
        slot_just_pressed: [
            keys.just_pressed(KeyCode::Digit5),
            keys.just_pressed(KeyCode::Digit6),
            keys.just_pressed(KeyCode::Digit7),
            keys.just_pressed(KeyCode::Digit8),
            keys.just_pressed(KeyCode::Digit9),
            keys.just_pressed(KeyCode::Digit0),
            keys.just_pressed(KeyCode::Numpad5),
            keys.just_pressed(KeyCode::Numpad8),
        ],
    }
}
