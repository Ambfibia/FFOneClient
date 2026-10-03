use std::path::Path;

use bevy::asset::AssetPlugin;
use ffone_protocol::{TimeBuff0104, WirePayload};
use tempfile::tempdir;

use crate::localization::Localization;
use crate::skill_buff_ui::*;

fn real_catalog() -> SkillBuffUiCatalog {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let assets = AssetLocator::open(root).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    SkillBuffUiCatalog::open(&content, &assets).unwrap()
}

#[test]
fn real_table_projection_resolves_buff_rows_through_icon_elements() {
    let catalog = real_catalog();
    assert_eq!(catalog.definition(1).map(|row| row.icon_number), Some(14));
    assert_eq!(catalog.definition(19).map(|row| row.icon_number), Some(32));
    assert_eq!(catalog.definition(21).map(|row| row.icon_number), Some(66));
    assert_eq!(catalog.definition(22).map(|row| row.icon_number), Some(65));
    assert_eq!(catalog.definition(23).map(|row| row.icon_number), Some(64));
    assert_eq!(catalog.definition(24).map(|row| row.icon_number), Some(36));
}

#[test]
fn source_owner_and_reached_label_style_match_clean_serialization() {
    assert_eq!(SKILL_BUFF_GAME_HUD_PATH_ID, 1_352);
    assert_eq!(SKILL_BUFF_COMPONENT_PATH_ID, 1_566);
    assert_eq!(SKILL_BUFF_SCRIPT_PATH_ID, 909);
    assert_eq!(SKILL_BUFF_BACK_PATH_ID, 166);

    let spec = SkillBuffTextStyle::HudLabel.spec();
    assert_eq!(spec.source_style, "label");
    assert_eq!(spec.source_skin_path_id, 1_372);
    assert_eq!(spec.source_font_path_id, 1_018);
    assert_eq!(spec.font_path, "fonts/chaletbook-regular.ttf");
    assert_eq!(spec.font_size, 12.0);
    assert_eq!(spec.line_height, 12.071_999_55);
    assert_eq!(spec.padding, [0.0; 4]);
    assert_eq!(spec.anchor, SkillBuffTextAnchor::MiddleLeft);
    assert_eq!(spec.justify, Justify::Left);
    assert_eq!(spec.linebreak, LineBreak::WordBoundary);
    assert_eq!(spec.normal_color, [0.9, 0.9, 0.9, 1.0]);
    assert!(spec.word_wrap);
    assert!(spec.text_clipping);
    assert_eq!(spec.y_offset, 0.0);
}

#[test]
fn production_bundles_resolve_cash_time() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    let asset_root = project_root.join("assets/game");
    let (localization, mut language) = Localization::open(&asset_root, "en").unwrap();
    let localized = skill_buff_cash_time_localized("14m");
    assert_eq!(localization.text(&language, &localized), "14m");
    localization.select(&mut language, "ru");
    assert_eq!(localization.text(&language, &localized), "14m");
}

#[test]
fn every_spawned_cash_timer_is_key_first_and_keeps_the_source_label_style() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(SkillBuffUiPlugin);
    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<(
        Entity,
        &SkillBuffUiElement,
        &SkillBuffTextStyle,
        &Text,
        &LocalizedText,
        (&TextFont, &LineHeight),
        &TextColor,
        &TextLayout,
        &ChildOf,
    ), With<Text>>();
    let rows = query.iter(world).collect::<Vec<_>>();
    assert_eq!(rows.len(), SKILL_BUFF_MAX_ICONS);
    let spec = SkillBuffTextStyle::HudLabel.spec();
    for (entity, element, style, text, localized, font, color, layout, parent) in rows {
        assert!(matches!(element, SkillBuffUiElement::CashText(_)));
        assert_eq!(*style, SkillBuffTextStyle::HudLabel);
        assert!(
            text.0.is_empty(),
            "{entity:?} must await localization apply"
        );
        assert_eq!(localized.key, "ui.skill_buff.cash_time");
        assert_eq!(localized.fallback, "{time}");
        assert_eq!(localized.args.get("time").map(String::as_str), Some(""));
        assert_eq!(font.0.font_size.eval(Vec2::ZERO, 16.0), spec.font_size);
        assert_eq!((*font.1), LineHeight::Px(spec.line_height));
        assert_eq!(layout.justify, spec.justify);
        assert_eq!(layout.linebreak, spec.linebreak);
        assert_eq!(
            color.0,
            Color::srgba(
                spec.normal_color[0],
                spec.normal_color[1],
                spec.normal_color[2],
                spec.normal_color[3]
            )
        );

        let parent_node = world
            .get::<Node>(parent.parent())
            .unwrap_or_else(|| panic!("{entity:?} has no cash label Rect parent"));
        assert_eq!(parent_node.left, px(2));
        assert_eq!(parent_node.top, px(19.0 + spec.y_offset));
        assert_eq!(parent_node.width, px(SKILL_BUFF_ICON_SIZE));
        assert_eq!(parent_node.height, px(SKILL_BUFF_ICON_SIZE));
        assert_eq!(parent_node.align_items, AlignItems::Center);
        assert_eq!(parent_node.justify_content, JustifyContent::FlexStart);
        assert_eq!(parent_node.overflow, Overflow::clip());
    }
}

#[test]
fn exact_order_excludes_local_stim_icons_but_keeps_cash_and_target_stims() {
    let all = SKILL_BUFF_DEBUFFS
        .iter()
        .chain(SKILL_BUFF_BUFFS.iter())
        .fold(0, |condition, (flag, _)| condition | flag);
    assert_eq!(
        local_buff_ids(all),
        vec![
            8, 9, 10, 11, 17, 19, 1, 3, 4, 5, 6, 7, 13, 14, 15, 16, 18, 20, 24
        ]
    );
    assert_eq!(
        cash_buff_ids(all),
        vec![1, 3, 4, 5, 6, 7, 13, 14, 15, 16, 18, 20, 24, 21, 22, 23]
    );
    assert_eq!(
        target_buff_ids(all, [Some(0), Some(1), Some(2)]),
        vec![
            8, 9, 10, 11, 17, 19, 1, 3, 4, 5, 6, 7, 13, 14, 15, 16, 18, 20, 24, 21, 22, 23
        ]
    );
    assert_eq!(
        target_buff_ids(
            1_048_576 | 2_097_152 | 4_194_304,
            [Some(2), Some(0), Some(1)]
        ),
        vec![23, 21, 22]
    );
    assert_eq!(
        target_buff_ids(
            1_048_576 | 2_097_152 | 4_194_304,
            [Some(0), Some(0), Some(0)]
        ),
        vec![21, 21, 21]
    );
    assert_eq!(
        target_buff_ids(1_048_576 | 2_097_152 | 4_194_304, [None, None, None]),
        Vec::<i32>::new()
    );
}

#[test]
fn geometry_matches_clean_serialized_rects_and_integer_centering() {
    let catalog = real_catalog();
    let all = SKILL_BUFF_DEBUFFS
        .iter()
        .chain(SKILL_BUFF_BUFFS.iter())
        .fold(0, |condition, (flag, _)| condition | flag);
    let mut model = SkillBuffUiModel {
        visible: true,
        local_condition_bit_flag: all,
        cash_condition_bit_flag: all,
        target: Some(SkillBuffTargetUi {
            character_type: 2,
            character_id: 77,
            condition_bit_flag: all,
        }),
        nano_styles: [Some(0), Some(1), Some(2)],
        ..default()
    };
    model.set_cash_remaining_ms(1, 60_000);
    let view = skill_buff_ui_view(1_264, &model, &catalog);
    assert_eq!(view.local.len(), 19);
    assert_eq!(
        view.local[0].rect,
        SkillBuffUiRect::new(-117.0, 72.0, 26.0, 26.0)
    );
    assert_eq!(
        view.cash[0].rect,
        SkillBuffUiRect::new(630.0, 40.0, 26.0, 26.0)
    );
    assert_eq!(view.cash[0].cash_time.as_deref(), Some("1m"));
    assert_eq!(view.target.len(), 22);
    assert_eq!(
        view.target[0].rect,
        SkillBuffUiRect::new(466.0, 40.0, 26.0, 26.0)
    );
}

#[test]
fn pivot_scaling_matches_ffguiutility_for_each_row() {
    let catalog = real_catalog();
    let mut model = SkillBuffUiModel {
        visible: true,
        ui_scale: 2.0,
        local_condition_bit_flag: 1,
        cash_condition_bit_flag: 1,
        target: Some(SkillBuffTargetUi {
            character_type: 2,
            character_id: 7,
            condition_bit_flag: 1,
        }),
        ..default()
    };
    model.set_cash_remaining_ms(1, 1_000);
    let view = skill_buff_ui_view(1_280, &model, &catalog);
    assert_eq!(
        view.local[0].rect,
        SkillBuffUiRect::new(234.0, 144.0, 52.0, 52.0)
    );
    assert_eq!(
        view.cash[0].rect,
        SkillBuffUiRect::new(-20.0, 80.0, 52.0, 52.0)
    );
    assert_eq!(
        view.target[0].rect,
        SkillBuffUiRect::new(749.0, 80.0, 52.0, 52.0)
    );
}

#[test]
fn cash_countdown_preserves_clean_coarse_tick_and_day_divisor_bug() {
    assert_eq!(format_cash_time(59_999), "59s");
    assert_eq!(format_cash_time(60_000), "1m");
    assert_eq!(format_cash_time(3_600_000), "1h");
    assert_eq!(format_cash_time(86_400_000), "0d");
    let localized = skill_buff_cash_time_localized(format_cash_time(60_000));
    assert_eq!(localized.key, "ui.skill_buff.cash_time");
    assert_eq!(localized.fallback, "{time}");
    assert_eq!(localized.args.get("time").map(String::as_str), Some("1m"));

    let mut model = SkillBuffUiModel::default();
    model.cash_condition_bit_flag = 1;
    model.set_cash_remaining_ms(1, 1_500);
    model.advance_cash_timer(0.75);
    assert_eq!(model.cash_remaining_ms(1), 1_500);
    model.advance_cash_timer(3.0);
    assert_eq!(model.cash_remaining_ms(1), 500);
    model.advance_cash_timer(1.0);
    assert_eq!(model.cash_remaining_ms(1), u64::MAX - 499);
}

#[test]
fn every_cash_packet_rebuilds_all_active_times_from_server_duration_cache() {
    let mut model = SkillBuffUiModel::default();
    model.apply_cash_update(PcCashBuffUpdate0104 {
        buff_id: 1,
        update_kind: 1,
        time_buff: TimeBuff0104 {
            time_duration: 5_000,
            ..default()
        },
        condition_bit_flag: 1,
    });
    model.apply_cash_update(PcCashBuffUpdate0104 {
        buff_id: 14,
        update_kind: 1,
        time_buff: TimeBuff0104 {
            time_duration: 9_000,
            ..default()
        },
        condition_bit_flag: 1 | 8_192,
    });
    model.advance_cash_timer(1.0);
    assert_eq!(model.cash_remaining_ms(1), 4_000);
    assert_eq!(model.cash_remaining_ms(14), 8_000);
    assert_eq!(model.cash_remaining_ms(20), 0);

    model.apply_cash_update(PcCashBuffUpdate0104 {
        buff_id: 14,
        update_kind: 3,
        time_buff: TimeBuff0104 {
            time_duration: 7_000,
            ..default()
        },
        condition_bit_flag: 1 | 8_192,
    });
    assert_eq!(model.cash_remaining_ms(1), 5_000);
    assert_eq!(model.cash_remaining_ms(14), 7_000);
}

#[test]
fn infection_tick_refreshes_only_a_protected_selected_owner_never_the_local_row() {
    let mut model = SkillBuffUiModel {
        local_character_id: Some(77),
        local_condition_bit_flag: 0x80,
        target: Some(SkillBuffTargetUi {
            character_type: 2,
            character_id: 88,
            condition_bit_flag: 0x400,
        }),
        ..default()
    };
    let tick = TimeBuffDotDamageTick0104 {
        character_type: 4,
        character_id: 88,
        time_buff_id: 17,
        result_character_type: 1,
        result_character_id: 77,
        protected: false,
        damage: 100,
        hp: 500,
        stamina: 80,
        nano_deactivated: false,
        condition_bit_flag: 0x10080,
    };
    model.apply_dot_damage_tick(tick);
    assert_eq!(model.local_condition_bit_flag, 0x80);
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x400)
    );

    model.apply_dot_damage_tick(TimeBuffDotDamageTick0104 {
        protected: true,
        ..tick
    });
    assert_eq!(model.local_condition_bit_flag, 0x80);
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x10080)
    );
}

#[test]
fn local_timeout_releases_both_radars_without_clearing_unrelated_buffs() {
    let mut model = SkillBuffUiModel {local_character_id: Some(77),
        local_condition_bit_flag: 0x3001, ..default()};
    model.apply_timeout(CharTimeBuffTimeout0104 {character_type: 1,
        character_id: 88, condition_bit_flag: 0});
    assert!(model.reveals_mobs() && model.reveals_shinies());
    model.apply_timeout(CharTimeBuffTimeout0104 {character_type: 1,
        character_id: 77, condition_bit_flag: 1});
    assert!(!model.reveals_mobs() && !model.reveals_shinies());
    assert_eq!(model.local_condition_bit_flag, 1);
}

#[test]
fn off_target_condition_updates_survive_selection_and_despawn_prunes_recycled_ids() {
    let mut model = SkillBuffUiModel {
        target: Some(SkillBuffTargetUi {
            character_type: 2,
            character_id: 7,
            condition_bit_flag: 1,
        }),
        ..default()
    };
    model.apply_timeout(CharTimeBuffTimeout0104 {
        character_type: 2,
        character_id: 88,
        condition_bit_flag: 0x400,
    });
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(1)
    );

    model.select_target_from_appearance(2, 88, 0, true);
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x400),
        "a newly selected owner must retain its newer off-target packet"
    );

    model.apply_dot_damage_tick(TimeBuffDotDamageTick0104 {
        character_type: 4,
        character_id: 99,
        time_buff_id: 17,
        result_character_type: 1,
        result_character_id: 77,
        protected: true,
        damage: 100,
        hp: 500,
        stamina: 80,
        nano_deactivated: false,
        condition_bit_flag: 0x10080,
    });
    model.select_target_from_appearance(2, 99, 0, false);
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x10080)
    );

    model.select_target_from_appearance(2, 99, 0x200, true);
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x200),
        "a changed appearance for the same owner remains authoritative"
    );

    model.retain_target_conditions(&BTreeSet::from([(2, 99)]));
    model.clear_target_selection();
    model.select_target_from_appearance(2, 88, 0x20, false);
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x20),
        "a despawned owner ID must not inherit its prior cached mask"
    );
}

#[test]
fn server_buff_updates_drive_senses_and_control_without_stale_membership() {
    let mut model = SkillBuffUiModel::default();
    for (flags, control, mobs, shinies) in [
        (0x1000, 0, true, false),
        (0x2000, 0, false, true),
        (0x3200, 5, true, true),
        (0x3400, 6, true, true),
        (0x600, 6, false, false),
        (0, 0, false, false),
    ] {
        let wire = PcBuffUpdate0104 {
            buff_id: 13,
            update_kind: if flags == 0 { 2 } else { 1 },
            buff_type: 0,
            time_buff: TimeBuff0104::default(),
            condition_bit_flag: flags,
        }
        .encode();
        model.apply_pc_update(PcBuffUpdate0104::decode(&wire).unwrap());
        assert_eq!(
            (
                model.local_control_condition(),
                model.reveals_mobs(),
                model.reveals_shinies()
            ),
            (control, mobs, shinies)
        );
    }
}

#[test]
fn packets_and_pc_load_drive_only_the_clean_icon_state() {
    let mut load = PcLoadData0104::zeroed();
    load.as_bytes_mut()[PcLoadData0104::CONDITION_BIT_FLAG_OFFSET
        ..PcLoadData0104::CONDITION_BIT_FLAG_OFFSET + 4]
        .copy_from_slice(&0x10080i32.to_le_bytes());
    let mut model = SkillBuffUiModel::default();
    model.seed_from_pc_load(77, &load);
    assert_eq!(model.local_condition_bit_flag, 0x10080);

    model.apply_packet(SkillBuffPacket0104::Pc(PcBuffUpdate0104 {
        buff_id: 17,
        update_kind: 1,
        buff_type: 0,
        time_buff: TimeBuff0104::default(),
        condition_bit_flag: 0x200,
    }));
    assert_eq!(model.local_condition_bit_flag, 0x200);

    model.apply_packet(SkillBuffPacket0104::Cash(PcCashBuffUpdate0104 {
        buff_id: 14,
        update_kind: 3,
        time_buff: TimeBuff0104 {
            time_duration: 90_000,
            ..default()
        },
        condition_bit_flag: 8_192,
    }));
    assert_eq!(model.cash_condition_bit_flag, 8_192);
    assert_eq!(model.cash_remaining_ms(14), 90_000);

    model.target = Some(SkillBuffTargetUi {
        character_type: 2,
        character_id: 88,
        condition_bit_flag: 0,
    });
    model.apply_packet(SkillBuffPacket0104::Timeout(CharTimeBuffTimeout0104 {
        character_type: 2,
        character_id: 88,
        condition_bit_flag: 0x400,
    }));
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x400)
    );
    // OpenFusion rewrites MOB(4) to NPC(2); the clean client ignores 4.
    model.apply_packet(SkillBuffPacket0104::Timeout(CharTimeBuffTimeout0104 {
        character_type: 4,
        character_id: 88,
        condition_bit_flag: 0,
    }));
    assert_eq!(
        model.target.map(|target| target.condition_bit_flag),
        Some(0x400)
    );

    assert_eq!(
        PcBuffUpdate0104::decode(
            &PcBuffUpdate0104 {
                buff_id: 1,
                update_kind: 1,
                buff_type: 0,
                time_buff: TimeBuff0104::default(),
                condition_bit_flag: 1,
            }
            .encode()
        )
        .unwrap()
        .condition_bit_flag,
        1
    );
}
