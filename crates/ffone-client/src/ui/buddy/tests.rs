use bevy::asset::AssetPlugin;
use tempfile::tempdir;

use crate::buddy_ui::*;

#[test]
fn unchanged_buddy_view_does_not_dirty_hidden_panel_layout() {
    #[derive(Resource, Default)]
    struct ChangedNodeCount(usize);

    fn count_changed_nodes(
        nodes: Query<(), (With<BuddyPanel>, Changed<Node>)>,
        mut count: ResMut<ChangedNodeCount>,
    ) {
        count.0 = nodes.iter().count();
    }

    let mut app = App::new();
    app.init_resource::<BuddyUiModel>()
        .init_resource::<ChangedNodeCount>()
        .add_systems(
            Update,
            (bind_buddy_ui, count_changed_nodes.after(bind_buddy_ui)),
        );
    app.world_mut().spawn((BuddyPanel, Node::default()));

    app.update();
    assert_eq!(app.world().resource::<ChangedNodeCount>().0, 1);

    // The real timer system changes this resource every frame even while
    // the panel is hidden. It must not invalidate an identical view.
    app.world_mut()
        .resource_mut::<BuddyUiModel>()
        .refresh_elapsed += 0.1;
    app.update();
    assert_eq!(
        app.world().resource::<ChangedNodeCount>().0,
        0,
        "timer-only model changes must not rebuild the hidden buddy tree"
    );
}

fn entry(uid: i64, flag: i8, presence: BuddyPresence) -> BuddyEntry {
    BuddyEntry {
        runtime_pc_id: uid as i32,
        pc_uid: uid,
        presence,
        first_name: "Gaia".to_owned(),
        last_name: "Roundbreath".to_owned(),
        name_check_flag: flag,
        ..default()
    }
}

#[test]
fn clean_geometry_and_labels_are_fixed() {
    assert_eq!(BUDDY_BOX_PATH, "ui/en/gameplay/buddy/window.png");
    assert_eq!(BUDDY_SELECT_PATH, "ui/en/gameplay/buddy/selection.png");
    assert_eq!(BUDDY_FREECHAT_PATH, "ui/en/gameplay/buddy/freechat.png");
    assert_eq!(
        BUDDY_LIST_BACKGROUND_PATH,
        "ui/en/gameplay/buddy/list-background.png"
    );
    assert_eq!(
        BUDDY_LARGE_LIST_BACKGROUND_PATH,
        "ui/en/gameplay/chat/darkenedChatArea.png"
    );
    assert_eq!(BUDDY_LARGE_LIST_BACKGROUND_SOURCE_TEXTURE_PATH_ID, 610);
    assert_eq!(
        BUDDY_LARGE_LIST_BACKGROUND_BORDER,
        BorderRect {
            min_inset: Vec2::new(8.0, 4.0),
            max_inset: Vec2::new(8.0, 4.0)
        }
    );
    assert_eq!(BUDDY_WINDOW_RECT, BuddyUiRect::new(0.0, 15.0, 293.0, 135.0));
    assert_eq!(
        BUDDY_CONTENT_RECT,
        BuddyUiRect::new(-1.0, 32.0, 290.0, 91.0)
    );
    assert_eq!(
        BUDDY_INNER_LIST_RECT,
        BuddyUiRect::new(4.0, 37.0, 280.0, 81.0)
    );
    assert_eq!(BUDDY_DELETE_RECT, BuddyUiRect::new(30.0, 127.0, 80.0, 20.0));
    assert_eq!(BUDDY_WARP_RECT, BuddyUiRect::new(115.0, 127.0, 80.0, 20.0));
    assert_eq!(BUDDY_ADD_RECT, BuddyUiRect::new(200.0, 127.0, 80.0, 20.0));
    assert_eq!(
        (BUDDY_DELETE_LABEL, BUDDY_WARP_LABEL, BUDDY_ADD_LABEL),
        ("DELETE", "WARP", "ADD")
    );
    assert_eq!(BUDDY_JEFFE_14_SOURCE_FONT_PATH_ID, 903);
    assert_eq!(BUDDY_JEFFE_14_FONT_SIZE, 12.0);
    assert_eq!(BUDDY_JEFFE_14_LINE_HEIGHT, 13.710_000_04);
    assert_eq!(BUDDY_JEFFE_12_SOURCE_FONT_PATH_ID, 953);
    assert_eq!(BUDDY_JEFFE_12_FONT_SIZE, 10.0);
    assert_eq!(BUDDY_JEFFE_12_LINE_HEIGHT, 12.338_999_75);
    assert_eq!(BUDDY_CHALET_SMALL_SOURCE_FONT_PATH_ID, 1018);
    assert_eq!(BUDDY_CHALET_SMALL_FONT_SIZE, 12.0);
    assert_eq!(BUDDY_CHALET_SMALL_LINE_HEIGHT, 12.071_999_55);
    assert_eq!(BUDDY_CHAT_SKIN_PATH_ID, 1368);
    assert_eq!(BUDDY_POP_SKIN_PATH_ID, 1382);
    assert_eq!(BUDDY_GUI_COMPONENT_PATH_ID, 1564);
    assert_eq!(BUDDY_GAME_OBJECT_PATH_ID, 1352);
    assert_eq!(BUDDY_UI_PRIMARY_MAIN_BYTES, 7_000_415);
    assert_eq!(
        BUDDY_UI_PRIMARY_MAIN_SHA256,
        "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F"
    );
    assert_eq!(BUDDY_SCROLL_TRACK_SOURCE_TEXTURE_PATH_ID, 374);
    assert_eq!(BUDDY_SCROLL_THUMB_SOURCE_TEXTURE_PATH_ID, 324);
    assert_eq!(BUDDY_SCROLL_UP_SOURCE_TEXTURE_PATH_ID, 63);
    assert_eq!(BUDDY_SCROLL_DOWN_SOURCE_TEXTURE_PATH_ID, 415);
    assert_eq!(
        BUDDY_SCROLL_UP_RECT,
        BuddyUiRect::new(264.0, 0.0, 16.0, 12.0)
    );
    assert_eq!(
        BUDDY_SCROLL_TRACK_RECT,
        BuddyUiRect::new(264.0, 12.0, 16.0, 57.0)
    );
    assert_eq!(
        BUDDY_SCROLL_DOWN_RECT,
        BuddyUiRect::new(264.0, 69.0, 16.0, 12.0)
    );

    let buddy_window = BuddyTextStyle::BuddyWindow.spec();
    assert_eq!(buddy_window.source_skin_path_id, BUDDY_CHAT_SKIN_PATH_ID);
    assert_eq!(buddy_window.padding, [10.0, 2.0, 1.0, 2.0]);
    assert_eq!(buddy_window.anchor, BuddyTextAnchor::UpperLeft);
    assert_eq!(buddy_window.source_font_path_id, 903);
    let buddy_item = BuddyTextStyle::BuddyItem.spec();
    assert_eq!(buddy_item.padding, [2.0, 2.0, 0.0, 2.0]);
    assert_eq!(buddy_item.anchor, BuddyTextAnchor::MiddleLeft);
    assert_eq!(buddy_item.source_font_path_id, 1018);
    let blue = BuddyTextStyle::BlueButton.spec();
    assert_eq!(blue.padding, [2.0; 4]);
    assert_eq!(blue.hover_color, [0.0, 0.342_741_94, 0.528_225_8, 1.0]);
    assert!(!blue.word_wrap);
    let red = BuddyTextStyle::RedButton2.spec();
    assert_eq!(red.padding, [0.0; 4]);
    assert_eq!(red.active_color, [1.0, 1.0, 1.0, 0.0]);
    let delete_text = BuddyTextStyle::DeleteText.spec();
    assert_eq!(delete_text.source_skin_path_id, BUDDY_POP_SKIN_PATH_ID);
    assert_eq!(delete_text.padding, [10.0, 6.0, 4.0, 6.0]);
    assert_eq!(delete_text.normal_color, [0.8, 1.0, 1.0, 1.0]);
    assert_eq!(BuddyTextStyle::Cancel.spec().padding, [0.0, 0.0, 4.0, 7.0]);
}

#[test]
fn semantic_static_labels_and_dynamic_copy_are_key_first() {
    for (fallback, key) in [
        (BUDDY_LIST_TITLE, "ui.buddy.title"),
        (BUDDY_DELETE_LABEL, "ui.common.delete"),
        (BUDDY_WARP_LABEL, "ui.buddy.warp"),
        (BUDDY_ADD_LABEL, "ui.buddy.add"),
        (BUDDY_CANCEL_LABEL, "ui.common.cancel"),
        (BUDDY_ADD_TITLE, "ui.buddy.add_title"),
        (BUDDY_ADD_INSTRUCTION, "ui.buddy.add_instruction"),
    ] {
        let localized = buddy_static_localized(fallback);
        assert_eq!(localized.key, key);
        assert_eq!(localized.fallback, fallback);
        assert!(localized.args.is_empty());
    }

    let dynamic = buddy_add_name_localized("Gaia Roundbreath");
    assert_eq!(dynamic.key, "ui.buddy.add_name_input");
    assert_eq!(dynamic.fallback, "{name}");
    assert_eq!(
        dynamic.args.get("name").map(String::as_str),
        Some("Gaia Roundbreath")
    );

    let mut model = BuddyUiModel::default();
    model
        .set_entry(0, Some(entry(8_198, 0, BuddyPresence::Offline)))
        .unwrap();
    let view = buddy_ui_view(&model);
    let unknown = buddy_row_localized(view.rows.first());
    assert_eq!(unknown.key, "ui.buddy.row.player");
    assert_eq!(unknown.fallback, "    Player {pc_uid}");
    assert_eq!(unknown.args.get("pc_uid").map(String::as_str), Some("8198"));

    let verified_entry = entry(4_002, 1, BuddyPresence::Online);
    model.set_entry(0, Some(verified_entry)).unwrap();
    let verified_view = buddy_ui_view(&model);
    let verified = buddy_row_localized(verified_view.rows.first());
    assert_eq!(verified.key, "ui.buddy.row.verified");
    assert_eq!(verified.fallback, "    {display_name}");
    assert_eq!(
        verified.args.get("display_name").map(String::as_str),
        Some("Gaia Roundbreath")
    );
    let empty = buddy_row_localized(None);
    assert_eq!(empty.key, "ui.buddy.row.empty");
    assert_eq!(empty.fallback, "");
    assert!(empty.args.is_empty());
}

#[test]
fn plugin_gives_every_text_a_localization_spec_and_exact_font_metrics() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(BuddyUiPlugin);
    app.update();

    {
        let mut model = app.world_mut().resource_mut::<BuddyUiModel>();
        model.set_visible(true);
        model
            .set_entry(0, Some(entry(4_002, 1, BuddyPresence::Online)))
            .unwrap();
        model
            .set_entry(1, Some(entry(8_198, 0, BuddyPresence::Offline)))
            .unwrap();
        model.open_add_dialog();
        model.set_add_name_input("Dexter Hero");
    }
    app.update();

    let world = app.world_mut();
    let mut text_nodes = world.query_filtered::<(
        &Text,
        &LocalizedText,
        &BuddyTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &Node,
        &UiTransform,
    ), With<Text>>();
    let texts = text_nodes.iter(world).collect::<Vec<_>>();
    assert_eq!(texts.len(), 59);
    assert!(
        texts
            .iter()
            .all(|(_, localized, _, _, _, _, _)| !localized.key.is_empty())
    );
    assert!(
        texts
            .iter()
            .all(|(_, localized, style, _, _, _, transform)| {
                localized.key != "ui.content.passthrough"
                    && transform.translation == Val2::px(0.0, style.spec().y_offset)
            })
    );

    for (style, expected) in [
        (BuddyTextStyle::BuddyWindow, 1),
        (BuddyTextStyle::BuddyItem, 50),
        (BuddyTextStyle::BlueButton, 2),
        (BuddyTextStyle::RedButton2, 1),
        (BuddyTextStyle::Transparent3, 1),
        (BuddyTextStyle::DeleteText, 2),
        (BuddyTextStyle::Cancel, 1),
        (BuddyTextStyle::QuitButton, 1),
    ] {
        assert_eq!(
            texts
                .iter()
                .filter(|(_, _, actual, _, _, _, _)| **actual == style)
                .count(),
            expected,
            "{style:?} Text count"
        );
    }

    let (_, _, _, title_font, _, title_node, _) = texts
        .iter()
        .find(|(_, localized, _, _, _, _, _)| localized.key == "ui.buddy.title")
        .copied()
        .expect("Buddy_Window title");
    assert_eq!(
        title_font.0.font_size.eval(Vec2::ZERO, 16.0),
        BUDDY_JEFFE_14_FONT_SIZE
    );
    assert_eq!((*title_font.1), LineHeight::Px(BUDDY_JEFFE_14_LINE_HEIGHT));
    assert_eq!(title_node.padding.left, px(10));
    assert_eq!(title_node.padding.right, px(2));
    assert_eq!(title_node.padding.top, px(1));
    assert_eq!(title_node.padding.bottom, px(2));

    let (_, _, _, instruction_font, _, _, _) = texts
        .iter()
        .find(|(_, localized, _, _, _, _, _)| localized.key == "ui.buddy.add_instruction")
        .copied()
        .expect("DeleteText instruction");
    assert_eq!(
        instruction_font.0.font_size.eval(Vec2::ZERO, 16.0),
        BUDDY_CHALET_SMALL_FONT_SIZE
    );
    assert_eq!(
        (*instruction_font.1),
        LineHeight::Px(BUDDY_CHALET_SMALL_LINE_HEIGHT)
    );

    let (_, _, _, cancel_font, _, _, _) = texts
        .iter()
        .find(|(_, localized, _, _, _, _, _)| localized.key == "ui.common.cancel")
        .copied()
        .expect("cancel control");
    assert_eq!(
        cancel_font.0.font_size.eval(Vec2::ZERO, 16.0),
        BUDDY_JEFFE_12_FONT_SIZE
    );
    assert_eq!((*cancel_font.1), LineHeight::Px(BUDDY_JEFFE_12_LINE_HEIGHT));

    assert!(texts.iter().any(|(_, localized, _, _, _, _, _)| {
        localized.key == "ui.buddy.row.verified"
            && localized.args.get("display_name").map(String::as_str)
                == Some("Gaia Roundbreath")
    }));
    assert!(texts.iter().any(|(_, localized, _, _, _, _, _)| {
        localized.key == "ui.buddy.add_name_input"
            && localized.args.get("name").map(String::as_str) == Some("Dexter Hero")
    }));
    assert!(texts.iter().any(|(_, localized, _, _, _, _, _)| {
        localized.key == "ui.buddy.row.player"
            && localized.args.get("pc_uid").map(String::as_str) == Some("8198")
    }));
}

#[test]
fn display_name_requires_name_check_flag_exactly_one() {
    assert_eq!(
        entry(4_002, 1, BuddyPresence::Online).display_name(),
        "Gaia Roundbreath"
    );
    assert_eq!(
        entry(4_002, 0, BuddyPresence::Online).display_name(),
        "Player 4002"
    );
    assert_eq!(
        entry(4_002, 2, BuddyPresence::Online).display_name(),
        "Player 4002"
    );
    assert_eq!(
        BuddyInvite {
            invite_id: 1,
            requester_pc_id: 2,
            requester_pc_uid: 4_002,
            first_name: "Gaia".to_owned(),
            last_name: "Roundbreath".to_owned(),
            name_check_flag: -1,
        }
        .display_name(),
        "Player 4002"
    );
}

#[test]
fn pending_buddy_invites_are_resolved_fifo() {
    let invite = |invite_id, requester_pc_id, requester_pc_uid, first_name: &str| BuddyInvite {
        invite_id,
        requester_pc_id,
        requester_pc_uid,
        first_name: first_name.to_owned(),
        last_name: "Hero".to_owned(),
        name_check_flag: 1,
    };
    let first = invite(1, 10, 100, "Dexter");
    let second = invite(2, 20, 200, "Ben");
    let mut model = BuddyUiModel::default();

    assert_eq!(
        model.receive_invite(first.clone()),
        BuddyInviteDisposition::Queued
    );
    assert_eq!(
        model.receive_invite(second.clone()),
        BuddyInviteDisposition::Queued
    );
    assert_eq!(model.current_invite(), Some(&first));
    assert_eq!(
        model.respond_to_invite(second.invite_id, true),
        Err(BuddyUiError::InviteNotCurrent(second.invite_id))
    );
    assert_eq!(
        model.respond_to_invite(first.invite_id, true),
        Ok(invite_response(&first, true))
    );
    assert_eq!(model.current_invite(), Some(&second));
    assert_eq!(
        model.respond_to_invite(second.invite_id, false),
        Ok(invite_response(&second, false))
    );
    assert!(model.current_invite().is_none());
}

#[test]
fn default_panel_is_clean_small_bottom_left_layout() {
    let layout = buddy_panel_layout(
        Vec2::new(1_264.0, 681.0),
        BuddyChatWindowStyle::Small,
        false,
        1.0,
    );
    assert_eq!(layout.painted_left, 300.0);
    assert_eq!(layout.painted_top, 531.0);
    assert_eq!(layout.painted_width, 293.0);
    assert_eq!(layout.painted_height, 150.0);

    let large = buddy_panel_layout(
        Vec2::new(1_264.0, 681.0),
        BuddyChatWindowStyle::Large,
        false,
        1.0,
    );
    assert_eq!(large.painted_left, 440.0);
    let resized_fresh_chat = buddy_panel_layout_for_chat_width(
        Vec2::new(1_264.0, 681.0),
        BuddyChatWindowStyle::Large,
        false,
        1.0,
        525.0,
    );
    assert_eq!(resized_fresh_chat.painted_left, 525.0);

    let small_quick = buddy_panel_layout(
        Vec2::new(1_264.0, 681.0),
        BuddyChatWindowStyle::Small,
        true,
        1.0,
    );
    assert_eq!(small_quick.painted_top, 571.0);
    assert_eq!(small_quick.painted_height, 110.0);

    let large_quick_setting = buddy_panel_layout(
        Vec2::new(1_264.0, 681.0),
        BuddyChatWindowStyle::Large,
        true,
        1.0,
    );
    assert_eq!(large_quick_setting.painted_top, 531.0);
    assert_eq!(large_quick_setting.painted_height, 150.0);
}

#[test]
fn begin_scroll_view_scrollbar_is_conditional_and_tracks_clamped_scroll() {
    let five_rows = buddy_scrollbar_layout(5, 0.0);
    assert!(!five_rows.visible);
    assert_eq!(five_rows.thumb.height, BUDDY_SCROLL_TRACK_RECT.height);

    let six_rows_top = buddy_scrollbar_layout(6, 0.0);
    assert!(six_rows_top.visible);
    assert_eq!(six_rows_top.thumb.y, BUDDY_SCROLL_TRACK_RECT.y);
    assert_eq!(six_rows_top.thumb.width, 15.0);
    assert!((six_rows_top.thumb.height - 48.093_75).abs() < f32::EPSILON);

    let six_rows_bottom = buddy_scrollbar_layout(6, 15.0);
    assert!(
        (six_rows_bottom.thumb.y + six_rows_bottom.thumb.height
            - (BUDDY_SCROLL_TRACK_RECT.y + BUDDY_SCROLL_TRACK_RECT.height))
            .abs()
            < f32::EPSILON
    );
}

#[test]
fn roster_is_bounded_and_blocked_entries_are_not_rendered() {
    let mut model = BuddyUiModel::default();
    model.set_visible(true);
    model
        .set_entry(0, Some(entry(10, 1, BuddyPresence::Online)))
        .unwrap();
    let mut blocked = entry(20, 1, BuddyPresence::Offline);
    blocked.blocked = true;
    model.set_entry(49, Some(blocked)).unwrap();

    assert!(matches!(
        model.set_entry(50, Some(entry(30, 1, BuddyPresence::Online))),
        Err(BuddyUiError::SlotOutOfRange(50))
    ));
    let view = buddy_ui_view(&model);
    assert_eq!(view.rows.len(), 1);
    assert_eq!(view.rows[0].slot, 0);
    assert!(model.select_visible_row(1).is_err());
}

#[test]
fn state_change_clears_selection_and_emits_exact_fallback_notice() {
    let mut model = BuddyUiModel::default();
    model
        .set_entry(3, Some(entry(4_002, 2, BuddyPresence::Offline)))
        .unwrap();
    model.select_slot(3).unwrap();
    let notices = model
        .apply_state_snapshot(&[BuddyStateUpdate {
            slot: 3,
            runtime_pc_id: 91,
            legacy_state: 1,
        }])
        .unwrap();

    assert_eq!(model.selected_slot(), None);
    assert_eq!(
        notices,
        vec![BuddyUiNotice::PresenceChanged {
            target: BuddyTarget {
                slot: 3,
                pc_uid: 4_002,
            },
            display_name: "Player 4002".to_owned(),
            presence: BuddyPresence::Online,
        }]
    );
}

#[test]
fn remove_confirmation_is_semantic_and_stale_targets_fail_closed() {
    let mut model = BuddyUiModel::default();
    model
        .set_entry(2, Some(entry(22, 1, BuddyPresence::Online)))
        .unwrap();
    model.select_slot(2).unwrap();
    let requested = model.request_remove().unwrap();
    let BuddyUiAction::ConfirmationRequested(confirmation) = requested else {
        panic!("expected confirmation intent");
    };

    model
        .set_entry(2, Some(entry(23, 1, BuddyPresence::Online)))
        .unwrap();
    assert!(matches!(
        model.resolve_confirmation(confirmation, true),
        Err(BuddyUiError::StaleConfirmation(_))
    ));
}

#[test]
fn warp_checks_presence_movement_cooldown_and_group_leave_stage() {
    let mut model = BuddyUiModel::default();
    model
        .set_entry(4, Some(entry(44, 1, BuddyPresence::Online)))
        .unwrap();
    model.select_slot(4).unwrap();
    model.set_group(2, [1, 2]);

    let BuddyUiAction::ConfirmationRequested(BuddyConfirmation::Warp(target)) =
        model.request_warp().unwrap()
    else {
        panic!("expected warp confirmation");
    };
    let next = model
        .resolve_confirmation(BuddyConfirmation::Warp(target), true)
        .unwrap();
    assert_eq!(
        next,
        Some(BuddyUiAction::ConfirmationRequested(
            BuddyConfirmation::LeaveGroupForWarp(target)
        ))
    );
    let warp = model
        .resolve_confirmation(BuddyConfirmation::LeaveGroupForWarp(target), true)
        .unwrap();
    assert_eq!(
        warp,
        Some(BuddyUiAction::WarpRequested {
            target,
            leave_group: true,
        })
    );
    assert_eq!(
        model.request_warp().unwrap(),
        BuddyUiAction::WarpCooldownNotice {
            remaining_seconds: 60
        }
    );

    model.set_warp_cooldown_seconds(0);
    model.set_player_moving(true);
    assert_eq!(model.request_warp(), Err(BuddyUiError::PlayerMoving));
}

#[test]
fn add_dialog_splits_at_first_space_and_rejects_incomplete_names() {
    let mut model = BuddyUiModel::default();
    model.open_add_dialog();
    model.set_add_name_input("Gaia Round Breath");
    assert_eq!(
        model.submit_add_dialog(),
        BuddyUiAction::AddByNameRequested {
            first_name: "Gaia".to_owned(),
            last_name: "Round Breath".to_owned(),
        }
    );
    assert!(!model.add_dialog_open());

    model.open_add_dialog();
    model.set_add_name_input("Gaia ");
    assert_eq!(
        model.submit_add_dialog(),
        BuddyUiAction::AddNameRejected {
            message: BUDDY_ADD_NAME_ERROR.to_owned()
        }
    );
}

#[test]
fn buddy_audio_buttons_emit_clean_button_and_modal_cues() {
    let mut app = App::new();
    app.init_resource::<BuddyUiModel>()
        .init_resource::<BuddyUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .add_systems(Update, handle_buddy_interactions);
    app.world_mut().spawn((
        Interaction::Pressed,
        BuddyControlMarker(BuddyControl::Delete),
    ));

    app.update();

    assert!(app.world().resource::<BuddyUiOutbox>().is_empty());
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAudioCue::ButtonSound]
    );

    app.world_mut()
        .resource_mut::<BuddyUiModel>()
        .open_add_dialog();
    app.world_mut().spawn((
        Interaction::Pressed,
        BuddyControlMarker(BuddyControl::ModalCancel),
    ));
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<GameplayUiAudioOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![GameplayUiAudioCue::NoButton]
    );
}

#[test]
fn blocked_and_disabled_social_invites_never_enter_the_modal_stack() {
    let invite = BuddyInvite {
        invite_id: 7,
        requester_pc_id: 70,
        requester_pc_uid: 700,
        first_name: "Dexter".to_owned(),
        last_name: "Hero".to_owned(),
        name_check_flag: 1,
    };
    let mut model = BuddyUiModel::default();
    let mut blocked = entry(700, 1, BuddyPresence::Online);
    blocked.blocked = true;
    model.set_entry(0, Some(blocked)).unwrap();
    assert!(model.is_blocked_runtime_pc_id(700));
    assert!(!model.is_blocked_runtime_pc_id(701));
    assert_eq!(
        model.receive_invite(invite.clone()),
        BuddyInviteDisposition::IgnoredBlocked
    );
    assert!(model.current_invite().is_none());

    model.set_entry(0, None).unwrap();
    model.set_social_buddy_enabled(false);
    assert!(!model.social_buddy_enabled());
    assert_eq!(
        model.receive_invite(invite),
        BuddyInviteDisposition::AutoDeclined(BuddyUiAction::InviteResponse {
            invite_id: 7,
            requester_pc_id: 70,
            requester_pc_uid: 700,
            accepted: false,
        })
    );
    assert!(model.current_invite().is_none());
}

#[test]
fn refresh_and_warp_cooldown_are_typed_time_state() {
    let mut model = BuddyUiModel::default();
    model.set_visible(true);
    model
        .set_entry(0, Some(entry(10, 1, BuddyPresence::Online)))
        .unwrap();
    model.set_warp_cooldown_seconds(2);

    assert!(model.advance_time(1.0).is_empty());
    assert_eq!(model.warp_cooldown_seconds(), 2);
    assert!(model.advance_time(0.01).is_empty());
    assert_eq!(model.warp_cooldown_seconds(), 1);
    assert_eq!(
        model.advance_time(9.1),
        vec![BuddyUiAction::RefreshStatesRequested]
    );
    assert_eq!(model.warp_cooldown_seconds(), 0);
}
