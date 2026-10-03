use super::*;

#[test]
fn journal_text_anchors_preserve_unity_vertical_semantics() {
    assert_eq!(mission_glyph_vertical_bounds(10.0, 8.0), (6.0, 14.0));

    let row_label = mission_gui_style("FusionFallMissionSkinR", "label").unwrap();
    assert_eq!(row_label.alignment, 3);
    assert_eq!(
        mission_text_vertical_anchor(
            row_label.alignment,
            Some(MissionUiView::JournalMissionRowSelectedObjective(0)),
        ),
        Some(MissionTextVerticalAnchor::Middle)
    );
    assert_eq!(
        mission_text_vertical_offset(MissionTextVerticalAnchor::Middle, 50.0, 0.0, 14.0),
        18.0
    );
    assert_eq!(
        mission_text_vertical_offset(MissionTextVerticalAnchor::Middle, 27.0, 3.0, 8.0),
        6.5,
        "button placement must center visible glyph ink, not the replacement line box"
    );
    let scaled_center_correction =
        mission_text_vertical_offset(MissionTextVerticalAnchor::Middle, 14.0, 4.0, 7.0)
            * CENTERED_MENU_JEFFE_VERTICAL_SCALE;
    assert!((scaled_center_correction + 0.35).abs() < f32::EPSILON);

    let dpi_100_translation = mission_text_translation_px(
        MissionTextVerticalAnchor::Middle,
        85.0,
        3.0,
        8.0,
        1.0,
        1.0,
    );
    let dpi_150_translation = mission_text_translation_px(
        MissionTextVerticalAnchor::Middle,
        127.5,
        4.5,
        12.0,
        2.0 / 3.0,
        1.0,
    );
    assert_eq!(dpi_100_translation, 35.5);
    assert!((dpi_150_translation - dpi_100_translation).abs() < f32::EPSILON);

    assert_eq!(
        mission_text_vertical_anchor(
            row_label.alignment,
            Some(MissionUiView::JournalDescription),
        ),
        None,
        "the fixed Journal description node is a GUILayout scroll viewport"
    );

    let button = mission_gui_style("FusionFallMissionSkin", "acceptbut").unwrap();
    assert_eq!(button.alignment, 4);
    assert_eq!(button.padding, MissionGuiInsets::default());
    assert_eq!(
        mission_text_vertical_anchor(button.alignment, Some(MissionUiView::JournalPrimaryText),),
        Some(MissionTextVerticalAnchor::Middle)
    );
    assert_eq!(
        mission_text_vertical_offset(MissionTextVerticalAnchor::Lower, 30.0, 0.0, 14.0),
        16.0
    );

    let offer = MissionJournalUi::Allow(mission(2248, 100));
    assert_eq!(journal_primary_rect(&offer), JOURNAL_ACCEPT_RECT);
    assert_eq!(
        journal_primary_text_rect(&offer),
        MissionUiRect::new(0.0, 0.0, 218.0, 85.0)
    );
    let reward = MissionJournalUi::Reward {
        mission: mission(2248, 100),
        box1_choice: 0,
        box2_choice: 0,
    };
    assert_eq!(journal_primary_rect(&reward), JOURNAL_COMPLETE_RECT);
    assert_eq!(
        journal_primary_text_rect(&reward),
        MissionUiRect::new(0.0, 0.0, 233.0, 70.0)
    );
}
