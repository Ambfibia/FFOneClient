use bevy::asset::AssetPlugin;
use tempfile::tempdir;

use crate::group_ui::*;

fn pc(uid: i64, first: &str, last: &str) -> GroupPcMemberUi {
    GroupPcMemberUi {
        pc_uid: uid,
        first_name: first.to_owned(),
        last_name: last.to_owned(),
        level: 12,
        hp: 75,
        max_hp: 100,
        ..default()
    }
}

fn npc(name: &str) -> GroupNpcMemberUi {
    GroupNpcMemberUi {
        npc_type: 0,
        name: name.to_owned(),
        hp: 30,
        max_hp: 60,
    }
}

#[test]
fn geometry_matches_cngui_group_info_at_1280_by_720() {
    assert_eq!(GROUP_UI_REFERENCE_WIDTH, 1_280.0);
    assert_eq!(GROUP_UI_REFERENCE_HEIGHT, 720.0);
    assert_eq!(
        GROUP_PC_BACKGROUND_RECT,
        GroupUiRect::new(2.0, 0.0, 198.0, 48.0)
    );
    assert_eq!(
        GROUP_PC_NAME_RECT,
        GroupUiRect::new(10.0, -4.0, 180.0, 20.0)
    );
    assert_eq!(
        GROUP_PC_LEVEL_RECT,
        GroupUiRect::new(40.0, 25.0, 25.0, 20.0)
    );
    assert_eq!(GROUP_PC_HP_RECT, GroupUiRect::new(7.0, 17.0, 186.0, 4.0));
    assert_eq!(
        GROUP_NANO_NAME_RECT,
        GroupUiRect::new(60.0, 15.0, 96.0, 20.0)
    );
    assert_eq!(
        GROUP_NANO_FRAME_RECT,
        GroupUiRect::new(60.0, 35.0, 111.0, 6.0)
    );
    assert_eq!(
        GROUP_NANO_FILL_RECT,
        GroupUiRect::new(61.0, 36.0, 109.0, 4.0)
    );
    assert_eq!(
        GROUP_NANO_SKILL_ICON_RECT,
        GroupUiRect::new(172.0, 21.0, 26.0, 26.0)
    );
    assert_eq!(
        GROUP_FREECHAT_ICON_RECT,
        GroupUiRect::new(3.0, 5.0, 18.0, 14.0)
    );
    assert_eq!(
        GROUP_NPC_BACKGROUND_RECT,
        GroupUiRect::new(5.0, 0.0, 192.0, 28.0)
    );
    assert_eq!(
        GROUP_NPC_NAME_RECT,
        GroupUiRect::new(10.0, -4.0, 180.0, 20.0)
    );
    assert_eq!(GROUP_NPC_HP_RECT, GroupUiRect::new(10.0, 17.0, 180.0, 4.0));
    assert_eq!(
        group_pc_row_rect(0),
        GroupUiRect::new(2.0, 100.0, 198.0, 48.0)
    );
    assert_eq!(
        group_pc_row_rect(3),
        GroupUiRect::new(2.0, 250.0, 198.0, 48.0)
    );
}

#[test]
fn remote_rows_compact_but_npc_retains_legacy_group_size_gap() {
    let model = GroupUiModel {
        local_pc_uid: Some(20),
        pc_members: vec![
            pc(10, "One", "Remote"),
            pc(20, "Local", "Player"),
            pc(30, "Two", "Remote"),
            pc(40, "Three", "Remote"),
        ],
        npc_members: vec![npc("Co-op NPC")],
    };

    let view = group_ui_view(&model);

    assert_eq!(view.legacy_group_size, 4);
    assert_eq!(view.pc_rows.len(), 3);
    assert_eq!(view.pc_rows[0].rect.top, 100.0);
    assert_eq!(view.pc_rows[1].rect.top, 150.0);
    assert_eq!(view.pc_rows[2].rect.top, 200.0);
    assert_eq!(
        view.npc_row.unwrap().rect,
        GroupUiRect::new(5.0, 300.0, 192.0, 28.0)
    );
}

#[test]
fn fractions_clamp_and_fail_closed() {
    assert_eq!(group_ui_fraction(-1, 100), 0.0);
    assert_eq!(group_ui_fraction(50, 0), 0.0);
    assert_eq!(group_ui_fraction(50, -1), 0.0);
    assert_eq!(group_ui_fraction(0, 100), 0.0);
    assert_eq!(group_ui_fraction(50, 100), 0.5);
    assert_eq!(group_ui_fraction(150, 100), 1.0);

    let model = GroupUiModel {
        pc_members: vec![GroupPcMemberUi {
            pc_uid: 7,
            hp: 10,
            max_hp: 0,
            nano: Some(GroupNanoUi {
                stamina: 50,
                max_stamina: 0,
                ..default()
            }),
            ..default()
        }],
        npc_members: vec![GroupNpcMemberUi {
            hp: 10,
            max_hp: -5,
            ..default()
        }],
        ..default()
    };
    let view = group_ui_view(&model);
    assert_eq!(view.pc_rows[0].hp_fraction, 0.0);
    assert_eq!(view.pc_rows[0].nano.as_ref().unwrap().stamina_fraction, 0.0);
    assert_eq!(view.npc_row.unwrap().hp_fraction, 0.0);
}

#[test]
fn local_member_is_skipped_and_unnamed_remote_uses_uid_fallback() {
    let model = GroupUiModel {
        local_pc_uid: Some(2),
        pc_members: vec![
            pc(1, "  ", ""),
            pc(2, "Local", "Player"),
            pc(3, "", "Lastname"),
        ],
        ..default()
    };

    let view = group_ui_view(&model);

    assert_eq!(view.legacy_group_size, 3);
    assert_eq!(view.pc_rows.len(), 2);
    assert_eq!(view.pc_rows[0].pc_uid, 1);
    assert_eq!(view.pc_rows[0].name, "Player 1");
    assert_eq!(view.pc_rows[1].name, "Lastname");
    assert_eq!(view.pc_rows[1].rect.top, 150.0);
}

#[test]
fn protocol_closure_limits_pc_and_npc_rows_and_uses_first_npc() {
    let model = GroupUiModel {
        pc_members: (1..=6).map(|uid| pc(uid, "Player", "")).collect(),
        npc_members: (1..=7).map(|index| npc(&format!("NPC {index}"))).collect(),
        ..default()
    };

    let view = group_ui_view(&model);

    assert_eq!(view.legacy_group_size, GROUP_PROTOCOL_MAX_PC_MEMBERS);
    assert_eq!(view.pc_rows.len(), GROUP_PROTOCOL_MAX_PC_MEMBERS);
    assert_eq!(
        GROUP_LEGACY_VISIBLE_NPC_MEMBERS,
        usize::from(view.npc_row.is_some())
    );
    assert_eq!(view.npc_row.unwrap().name, "NPC 1");
}

#[test]
fn hidden_without_renderable_roster_and_visible_with_npc_only() {
    assert!(!group_ui_view(&GroupUiModel::default()).visible);
    assert!(
        !group_ui_view(&GroupUiModel {
            local_pc_uid: Some(1),
            pc_members: vec![pc(1, "Local", "Player")],
            ..default()
        })
        .visible
    );
    assert!(
        group_ui_view(&GroupUiModel {
            npc_members: vec![npc("Escort")],
            ..default()
        })
        .visible
    );
}

#[test]
fn nano_view_keeps_explicit_semantic_skill_icon_path() {
    let path = "ui/en/gameplay/nano/skills/skill_01.png";
    let model = GroupUiModel {
        pc_members: vec![GroupPcMemberUi {
            pc_uid: 9,
            nano: Some(GroupNanoUi {
                name: "Belladonna".to_owned(),
                stamina: 25,
                max_stamina: 100,
                skill_icon_path: Some(path.to_owned()),
            }),
            ..default()
        }],
        ..default()
    };

    let nano = group_ui_view(&model).pc_rows.remove(0).nano.unwrap();
    assert_eq!(nano.name, "Belladonna");
    assert_eq!(nano.stamina_fraction, 0.25);
    assert_eq!(nano.skill_icon_path.as_deref(), Some(path));
}

#[test]
fn plugin_spawns_one_passive_hierarchy_and_binds_without_own_camera() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(GroupUiPlugin);
    app.update();

    assert!(app.world().contains_resource::<GroupUiModel>());
    {
        let world = app.world_mut();
        let mut roots = world.query_filtered::<&Visibility, With<GroupUiRoot>>();
        assert_eq!(roots.iter(world).count(), 1);
        assert_eq!(*roots.single(world).unwrap(), Visibility::Hidden);

        let mut pc_rows = world.query_filtered::<&GroupUiElement, With<Node>>();
        assert_eq!(
            pc_rows
                .iter(world)
                .filter(|element| matches!(element, GroupUiElement::PcRow(_)))
                .count(),
            GROUP_PROTOCOL_MAX_PC_MEMBERS
        );

        let mut cameras = world.query::<&Camera>();
        assert_eq!(cameras.iter(world).count(), 0);
    }

    app.world_mut().resource_mut::<GroupUiModel>().pc_members = vec![GroupPcMemberUi {
        pc_uid: 77,
        first_name: "Remote".to_owned(),
        last_name: "Player".to_owned(),
        hp: 25,
        max_hp: 100,
        free_chat: true,
        nano: Some(GroupNanoUi {
            name: "Nano".to_owned(),
            stamina: 50,
            max_stamina: 100,
            ..default()
        }),
        ..default()
    }];
    app.update();

    let world = app.world_mut();
    let mut roots = world.query_filtered::<&Visibility, With<GroupUiRoot>>();
    assert_eq!(*roots.single(world).unwrap(), Visibility::Visible);
    let mut elements = world.query::<(
        &GroupUiElement,
        &Node,
        Option<&Text>,
        Option<&LocalizedText>,
    )>();
    for (element, node, text, localized) in elements.iter(world) {
        match element {
            GroupUiElement::PcRow(0) => assert_eq!(node.display, Display::Flex),
            GroupUiElement::PcName(0) => {
                assert_eq!(text.unwrap().0, "");
                let localized = localized.expect("player name localization component");
                assert_eq!(localized.key, "ui.content.passthrough");
                assert_eq!(
                    localized.args.get("text").map(String::as_str),
                    Some("Remote Player")
                );
            }
            GroupUiElement::PcHp(0) => {
                assert_eq!(node.width, px(GROUP_PC_HP_RECT.width * 0.25));
            }
            GroupUiElement::NanoFill(0) => {
                assert_eq!(node.width, px(GROUP_NANO_FILL_RECT.width * 0.5));
            }
            _ => {}
        }
    }
    let mut text_nodes = world.query_filtered::<Option<&LocalizedText>, With<Text>>();
    assert!(
        text_nodes.iter(world).all(|localized| localized.is_some()),
        "every spawned Group HUD Text must carry LocalizedText"
    );

    let mut styled_text = world.query::<(
        &Text,
        &LocalizedText,
        &GroupUiTextStyle,
        (&TextFont, &LineHeight),
        &TextLayout,
        &TextColor,
        &Node,
        &UiTransform,
    )>();
    let mut count = 0usize;
    for (_, localized, style, font, layout, color, node, transform) in styled_text.iter(world) {
        count += 1;
        assert!(!localized.key.is_empty());
        assert_eq!(*style, GroupUiTextStyle::HudLabel);
        let spec = style.spec();
        assert_eq!(spec.source_game_object_path_id, 1_352);
        assert_eq!(spec.source_component_path_id, 1_563);
        assert_eq!(spec.source_script_path_id, 1_135);
        assert_eq!(spec.source_skin_path_id, 1_372);
        assert_eq!(spec.source_style, "label");
        assert_eq!(spec.source_font_path_id, 1_018);
        assert_eq!(font.0.font_size.eval(Vec2::ZERO, 16.0), spec.font_size);
        assert_eq!((*font.1), LineHeight::Px(spec.line_height));
        assert_eq!(layout.justify, spec.justify);
        assert_eq!(layout.linebreak, spec.linebreak);
        assert_eq!(color.0, Color::srgb(0.9, 0.9, 0.9));
        assert_eq!(node.padding, UiRect::ZERO);
        assert_eq!(node.justify_content, JustifyContent::FlexStart);
        assert_eq!(node.align_items, AlignItems::Center);
        assert_eq!(node.overflow, Overflow::clip());
        assert_eq!(transform.translation, Val2::px(0.0, 0.0));
        assert_eq!(spec.y_offset, 0.0);
    }
    assert_eq!(count, GROUP_PROTOCOL_MAX_PC_MEMBERS * 3 + 1);
}
