use super::*;

#[test]
fn full_settings_defaults_and_clean_input_schema_are_typed() {
    let settings = OptionSettings::default();
    assert_eq!(
        (settings.graphics.width, settings.graphics.height),
        (1_024, 768)
    );
    assert!(settings.graphics.windowed);
    assert_eq!(settings.graphics.detail, GraphicsDetail::Balanced);
    assert!(settings.graphics.soft_vegetation);
    assert_eq!(settings.graphics.texture, TextureQuality::High);
    assert_eq!(settings.sound.music.volume, 0.5);
    assert!(settings.sound.master.enabled);
    assert_eq!(settings.sound.master.volume, 0.5);
    assert!(settings.display.scale_ui);
    assert!(settings.display.current_objective);
    assert!(settings.display.guide_email);
    assert!(settings.display.waypoint);
    assert!(settings.display.npc_messages_in_chat);
    assert!(!settings.display.combat_in_chat);
    assert!(settings.display.animated_nanos);
    assert_eq!(settings.text_colors, TextColorSettings::default());
    assert_eq!(settings.social, SocialRequestSettings::default());

    let input = InputSettings::default();
    assert!(input.has_complete_clean_schema());
    assert_eq!(input.mappings.len(), OPTION_INPUT_ACTION_COUNT);
    assert_eq!(LegacyOptionAction::ALL.len(), OPTION_INPUT_ACTION_COUNT);
    assert_eq!(input.camera_sensitivity, 5.0);
    assert_eq!(input.pad_profile, LegacyPadProfile::Xbox360);
    let help = input.mappings.iter()
        .find(|row| row.action == LegacyOptionAction::Help).unwrap();
    assert_eq!(help.primary, LegacyInputBinding::Unbound);
    let email = input.mappings.iter()
        .find(|row| row.action == LegacyOptionAction::Email).unwrap();
    assert_eq!(email.primary, LegacyInputBinding::Key(LegacyPhysicalKey::P));
}

#[test]
fn apply_b_then_edit_c_then_cancel_restores_option_a_but_keeps_input_b() {
    let options_a = OptionSettings::default();
    let input_a = InputSettings::default();
    let mut model = OptionUiModel::default();
    let mut outbox = OptionUiOutbox::default();
    model.open(
        options_a.clone(),
        input_a,
        OptionOpenAudioRoute::default(),
        &mut outbox,
    );
    outbox.clear();

    model.draft_options.graphics.width = 1_280;
    model.draft_input.camera_sensitivity = 7.0;
    model.mark_options_edited();
    assert!(model.apply(&mut outbox));
    assert_eq!(model.opening_options, options_a);
    assert_eq!(model.persisted_options.graphics.width, 1_280);
    assert_eq!(model.persisted_input.camera_sensitivity, 7.0);

    model.draft_options.graphics.width = 1_600;
    model.draft_input.camera_sensitivity = 9.0;
    model.mark_input_edited();
    outbox.clear();
    assert!(model.cancel(
        OptionCloseTrigger::Shortcut,
        OptionCloseAudioRoute::default(),
        &mut outbox
    ));
    assert_eq!(model.persisted_options.graphics.width, 1_024);
    assert_eq!(model.draft_options.graphics.width, 1_024);
    assert_eq!(model.persisted_input.camera_sensitivity, 7.0);
    assert_eq!(model.draft_input.camera_sensitivity, 7.0);
}

#[test]
fn clean_cancel_is_save_plus_success_and_dirty_shortcut_has_neither_success_nor_click() {
    let mut clean_model = OptionUiModel::default();
    let mut clean_outbox = OptionUiOutbox::default();
    clean_model.open(
        OptionSettings::default(),
        InputSettings::default(),
        OptionOpenAudioRoute::default(),
        &mut clean_outbox,
    );
    clean_outbox.clear();
    assert!(clean_model.cancel(
        OptionCloseTrigger::Shortcut,
        OptionCloseAudioRoute {
            main_game_transition: true,
            inventory_transition: true,
        },
        &mut clean_outbox,
    ));
    assert_eq!(
        cue_timeline(&mut clean_outbox),
        vec![
            OptionUiAudioCue::CloseScreen,
            OptionUiAudioCue::CloseScreen,
            OptionUiAudioCue::ActionSuccess,
        ]
    );

    let mut dirty_model = OptionUiModel::default();
    let mut dirty_outbox = OptionUiOutbox::default();
    dirty_model.open(
        OptionSettings::default(),
        InputSettings::default(),
        OptionOpenAudioRoute::default(),
        &mut dirty_outbox,
    );
    dirty_model.mark_options_edited();
    dirty_outbox.clear();
    assert!(dirty_model.cancel(
        OptionCloseTrigger::Shortcut,
        OptionCloseAudioRoute {
            main_game_transition: true,
            inventory_transition: false,
        },
        &mut dirty_outbox,
    ));
    assert_eq!(
        cue_timeline(&mut dirty_outbox),
        vec![OptionUiAudioCue::CloseScreen]
    );
}

#[test]
fn open_close_and_pointer_boundaries_preserve_possible_double_cues() {
    let mut model = OptionUiModel::default();
    let mut outbox = OptionUiOutbox::default();
    model.open(
        OptionSettings::default(),
        InputSettings::default(),
        OptionOpenAudioRoute {
            main_game_transition: true,
            inventory_transition: true,
        },
        &mut outbox,
    );
    assert_eq!(
        cue_timeline(&mut outbox),
        vec![OptionUiAudioCue::OpenScreen, OptionUiAudioCue::OpenScreen]
    );

    model.mark_options_edited();
    model.cancel(
        OptionCloseTrigger::CloseButton,
        OptionCloseAudioRoute {
            main_game_transition: true,
            inventory_transition: true,
        },
        &mut outbox,
    );
    assert_eq!(
        cue_timeline(&mut outbox),
        vec![
            OptionUiAudioCue::CloseScreen,
            OptionUiAudioCue::CloseScreen,
            OptionUiAudioCue::ButtonClick {
                clip_index: 0,
                gain: 1.0,
            },
        ]
    );

    model.open(
        OptionSettings::default(),
        InputSettings::default(),
        OptionOpenAudioRoute::default(),
        &mut outbox,
    );
    outbox.clear();
    assert!(!model.apply(&mut outbox));
    assert!(outbox.is_empty());
    model.mark_options_edited();
    assert!(model.apply(&mut outbox));
    assert_eq!(
        cue_timeline(&mut outbox),
        vec![
            OptionUiAudioCue::ActionSuccess,
            OptionUiAudioCue::ButtonClick {
                clip_index: 1,
                gain: 1.0,
            },
        ]
    );
}

#[test]
fn blocked_scroll_stays_within_the_fifty_slot_projection() {
    let mut model = OptionUiModel {
        visible: true,
        selected_tab: OptionTab::Social,
        ..default()
    };
    for (slot, buddy) in model.buddy_slots.iter_mut().enumerate() {
        buddy.pc_uid = slot as i64 + 1;
        buddy.blocked = true;
    }
    assert!(model.scroll_blocked_rows(10_000));
    assert_eq!(
        model.blocked_scroll_row,
        OPTION_BLOCKED_SLOT_CAPACITY - OPTION_BLOCKED_VISIBLE_ROWS
    );
    assert!(!model.scroll_blocked_rows(1));
    assert!(model.scroll_blocked_rows(-10_000));
    assert_eq!(model.blocked_scroll_row, 0);
}

#[test]
fn old_saved_pad_mappings_survive_new_camera_settings_defaults() {
    let mut saved = InputSettings::default();
    saved.mappings[0].pad = LegacyInputBinding::PadButton(5);
    let mut json = serde_json::to_value(&saved).unwrap();
    json.as_object_mut().unwrap().remove("pad_invert_y");
    json.as_object_mut().unwrap().remove("pad_camera_sensitivity");
    let loaded: InputSettings = serde_json::from_value(json).unwrap();
    assert_eq!(loaded.mappings, saved.mappings);
    assert_eq!(loaded.pad_camera_sensitivity, 5.0);
    assert!(!loaded.pad_invert_y);
    let mut new_settings = loaded;
    new_settings.pad_camera_sensitivity = 2.0;
    new_settings.pad_invert_y = true;
    let roundtrip: InputSettings = serde_json::from_str(&serde_json::to_string(&new_settings).unwrap()).unwrap();
    assert_eq!(roundtrip, new_settings);
}
