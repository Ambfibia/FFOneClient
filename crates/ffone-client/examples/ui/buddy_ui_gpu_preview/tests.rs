use super::*;
use ffone_client::buddy_ui::buddy_ui_view;

#[test]
fn preview_sample_exercises_verified_fallback_selected_and_freechat_rows() {
    let mut model = BuddyUiModel::default();
    model.set_visible(true);
    model
        .set_entry(
            0,
            Some(sample_entry(
                4_002,
                "Gaia",
                "Roundbreath",
                1,
                BuddyPresence::Online,
                true,
            )),
        )
        .unwrap();
    model
        .set_entry(
            4,
            Some(sample_entry(
                8_198,
                "Hidden",
                "LegacyName",
                0,
                BuddyPresence::Offline,
                false,
            )),
        )
        .unwrap();
    model.select_slot(4).unwrap();
    let view = buddy_ui_view(&model);
    assert_eq!(view.rows[0].display_name, "Gaia Roundbreath");
    assert!(view.rows[0].free_chat);
    assert_eq!(view.rows[1].display_name, "Player 8198");
    assert!(view.rows[1].selected);
}
