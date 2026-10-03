use super::*;

#[test]
fn normal_world_target_hud_ecs_query_accepts_network_npc_appearance() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(&asset_root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let mut world = World::new();
    world.insert_resource(content);
    let entity = world
        .spawn((
            GlobalTransform::from_translation(Vec3::new(2.0, 10.0, 4.0)),
            normal_world_npc_appearance(1, 630),
        ))
        .id();
    let mut state = bevy::ecs::system::SystemState::<GameplayHudNpcQuery>::new(&mut world);
    let targets = state.get(&world).unwrap();
    let (transform, target) = targets.get(entity).unwrap();

    assert_eq!(transform.translation(), Vec3::new(2.0, 10.0, 4.0));
    assert_eq!(target.name.fallback(), "Fusion Eduardo");
    assert_eq!(target.hp_fraction, Some(0.5));
    assert_eq!(target.level, Some(3));
    assert_eq!(target.affinity_style, Some(0));
    assert_eq!(
        target.portrait_icon_path,
        Some("icons/entities/mobs/mobicon_223.png")
    );
}

#[test]
fn normal_world_target_hud_fails_closed_on_mismatched_or_zero_hp_table_rows() {
    let mut definition = normal_world_npc_definition();
    let appearance = normal_world_npc_appearance(definition.npc_type, 630);
    definition.npc_type += 1;
    assert!(GameplayHudNpc::network(&appearance, &definition, None).is_none());

    definition.npc_type = appearance.0.npc_type;
    definition.max_hp = 0;
    let target = GameplayHudNpc::network(&appearance, &definition, None).unwrap();
    assert_eq!(target.hp_fraction, None);
    assert_eq!(target.level, Some(3));
    assert_eq!(target.affinity_style, Some(0));
}

#[test]
fn tutorial_target_hud_retains_its_checked_level_affinity_and_icon_contract() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(&asset_root).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let actor = TutorialActor {
        id: 27,
        npc_type: 2676,
        team: 2,
        hp: 650,
        max_hp: 1300,
        damaged: true,
        interacting: false,
        invulnerable: false,
    };
    let target =
        GameplayHudNpc::tutorial(&actor, Some(content.gameplay_npc(actor.npc_type).unwrap()))
            .unwrap();

    assert_eq!(target.name.fallback(), "Oil Ogre");
    assert_eq!(target.name.localized().key, "content.npc.2676.name");
    assert_eq!(target.hp_fraction, Some(0.5));
    assert_eq!(target.height, 3.5);
    assert_eq!(target.projection_origin_y, 0.0);
    assert_eq!(target.level, Some(1));
    assert_eq!(target.affinity_style, Some(1));
    assert_eq!(
        target.portrait_icon_path,
        Some("icons/entities/mobs/mobicon_11.png")
    );
}

#[test]
fn nano_cooldown_reproduces_source_coarse_and_fine_wedges() {
    assert_eq!(
        nano_cooldown_layers(true, Some(1.0)),
        Some(NanoCooldownLayers {
            base_texture_index: Some(6),
            rotating_texture_index: Some(6),
            rotating_degrees: 45.0,
            fine_step_count: 0,
        })
    );
    assert_eq!(
        nano_cooldown_layers(true, Some(0.5)),
        Some(NanoCooldownLayers {
            base_texture_index: Some(2),
            rotating_texture_index: Some(3),
            rotating_degrees: 45.0,
            fine_step_count: 0,
        })
    );
    assert_eq!(
        nano_cooldown_layers(true, Some(0.75)),
        Some(NanoCooldownLayers {
            base_texture_index: Some(4),
            rotating_texture_index: Some(5),
            rotating_degrees: 45.0,
            fine_step_count: 0,
        }),
        "the mask consumes normalized time remaining, not elapsed time"
    );
    assert_eq!(
        nano_cooldown_layers(true, Some(0.25)),
        Some(NanoCooldownLayers {
            base_texture_index: Some(1),
            rotating_texture_index: Some(1),
            rotating_degrees: 45.0,
            fine_step_count: 0,
        })
    );
    assert_eq!(
        nano_cooldown_layers(true, Some(0.125)),
        Some(NanoCooldownLayers {
            fine_step_count: 18,
            ..default()
        })
    );
    assert_eq!(
        nano_cooldown_layers(true, Some(0.14)),
        Some(NanoCooldownLayers {
            fine_step_count: 19,
            ..default()
        })
    );
}

#[test]
fn nano_stamina_stretches_from_the_fixed_left_edge_until_the_source_threshold() {
    assert_eq!(nano_stamina_fill_width(true, false, 0.0), Some(0.0));
    assert_eq!(nano_stamina_fill_width(true, false, 0.5), Some(20.0));
    assert_eq!(
        nano_stamina_fill_width(true, false, 0.989),
        Some(40.0 * 0.989)
    );
    assert_eq!(nano_stamina_fill_width(true, false, 0.99), None);
    assert_eq!(nano_stamina_fill_width(true, false, 1.0), None);
    assert_eq!(nano_stamina_fill_width(true, true, 0.5), None);
    assert_eq!(nano_stamina_fill_width(false, false, 0.5), None);
    assert_eq!(nano_stamina_fill_width(true, false, f32::NAN), None);
    assert_eq!(active_nano_stamina_fill_width(0.0), 0.0);
    assert_eq!(active_nano_stamina_fill_width(0.5), 52.0);
    assert_eq!(active_nano_stamina_fill_width(1.0), 104.0);
    assert_eq!(active_nano_stamina_fill_width(f32::NAN), 0.0);
}

#[test]
fn nano_stamina_and_cooldown_use_the_primary_linear_repeat_sampler() {
    let mut settings = ImageLoaderSettings::default();
    configure_nano_wheel_image(&mut settings);
    let ImageSampler::Descriptor(sampler) = settings.sampler else {
        panic!("Nano HUD load callback must install an explicit sampler descriptor");
    };
    assert_eq!(sampler.address_mode_u, ImageAddressMode::Repeat);
    assert_eq!(sampler.address_mode_v, ImageAddressMode::Repeat);
    assert_eq!(sampler.address_mode_w, ImageAddressMode::ClampToEdge);
    assert_eq!(sampler.mag_filter, ImageFilterMode::Linear);
    assert_eq!(sampler.min_filter, ImageFilterMode::Linear);
    assert_eq!(sampler.mipmap_filter, ImageFilterMode::Nearest);
    assert_eq!(sampler.anisotropy_clamp, 1);
    assert_eq!(NANO_WHEEL_EXACT_SAMPLER_PATHS.len(), 10);
    assert_eq!(
        &NANO_WHEEL_EXACT_SAMPLER_PATHS[8..],
        &[
            "ui/en/gameplay/nano/stamina/nano_st_bar.png",
            "ui/en/gameplay/nano/stamina/HP_BAR.png",
        ]
    );
}

#[test]
fn every_quick_chat_and_emote_row_has_matching_en_ru_keys() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let load_entries = |locale: &str| {
        let path = asset_root.join(format!("localization/{locale}.json"));
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
        let document: serde_json::Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("parse {path:?}: {error}"));
        document["entries"]
            .as_object()
            .expect("localization entries object")
            .clone()
    };
    let en = load_entries("en");
    let ru = load_entries("ru");
    let items = MENU_CHAT_ITEMS
        .iter()
        .chain(EMOTE_CHAT_ITEMS.iter())
        .collect::<Vec<_>>();
    assert_eq!(items.len(), 147);
    let unique_keys = items.iter().map(|item| item.key).collect::<BTreeSet<_>>();
    assert_eq!(unique_keys.len(), items.len());
    for item in items {
        assert_eq!(en[item.key].as_str(), Some(item.fallback), "{}", item.key);
        assert!(ru.contains_key(item.key), "RU is missing {}", item.key);
    }
    assert_eq!(
        en["ui.hud.chat.empty.buddy"].as_str(),
        Some(
            "Click on a player and select MAKE A BUDDY from the menu, or select ADD in your BUDDY LIST."
        )
    );
    assert_eq!(
        en["ui.hud.chat.empty.group"].as_str(),
        Some("Click on a player and select INVITE TO GROUP from the menu.")
    );
    assert!(ru.contains_key("ui.hud.chat.empty.buddy"));
    assert!(ru.contains_key("ui.hud.chat.empty.group"));
    // Opening production RU also enforces complete canonical key parity and
    // placeholder parity, including {mission} and {location}.
    Localization::open(&asset_root, "ru").expect("quick-chat localization parity");
}

#[test]
fn chat_history_retains_fifty_measured_rows_and_exact_source_colors() {
    let lines = (0..53)
        .map(|index| ChatLineUi::normal(format!("line {index}")))
        .collect::<Vec<_>>();
    let retained = retained_chat_lines(&lines);
    assert_eq!(retained.len(), CHAT_HISTORY_CAPACITY);
    assert_eq!(retained.first().unwrap().text, "line 3");
    assert_eq!(retained.last().unwrap().text, "line 52");

    let settings = TextColorSettings::default();
    let normal = chat_line_color(ChatLineKind::Normal, settings).to_srgba();
    assert_eq!(
        [normal.red, normal.green, normal.blue, normal.alpha],
        [1.0, 0.6, 0.0, 1.0]
    );
    let buddy = chat_line_color(ChatLineKind::Buddy, settings).to_srgba();
    assert_eq!([buddy.red, buddy.green, buddy.blue], [0.5, 0.66, 0.0]);
    let group = chat_line_color(ChatLineKind::Group, settings).to_srgba();
    assert_eq!([group.red, group.green, group.blue], [0.0, 0.39, 0.0]);
    let receive = chat_line_color(ChatLineKind::Receive, settings).to_srgba();
    assert_eq!([receive.red, receive.green, receive.blue], [1.0, 0.5, 0.0]);
    let attack = chat_line_color(ChatLineKind::Attack, settings).to_srgba();
    assert_eq!([attack.red, attack.green, attack.blue], [0.0, 0.0, 1.0]);
    let tutorial = chat_line_color(ChatLineKind::Tutorial, settings).to_srgba();
    assert_eq!(
        [tutorial.red, tutorial.green, tutorial.blue, tutorial.alpha,],
        [1.0, 0.0, 0.0, 1.0]
    );
    assert_eq!(CHAT_LOG_LINE_GAP, 4.0);
    assert_eq!(CHAT_LOG_BOTTOM_PADDING, 5.0);
    assert_eq!(
        chat_scroll_position(CHAT_SCROLL_TO_LATEST, 100.0, -15.0),
        85.0
    );
    assert_eq!(chat_scroll_position(0.0, 100.0, -15.0), 0.0);
    assert_eq!(chat_scroll_position(95.0, 100.0, 15.0), 100.0);
}

#[test]
fn combat_target_icons_follow_exact_npc_and_icon_table_routes() {
    assert_eq!(
        combat_target_icon_path(2674),
        Some("icons/entities/mobs/mobicon_00.png")
    );
    assert_eq!(
        combat_target_icon_path(2675),
        Some("icons/entities/mobs/mobicon_24.png")
    );
    assert_eq!(
        combat_target_icon_path(2676),
        Some("icons/entities/mobs/mobicon_11.png")
    );
    assert_eq!(
        combat_target_icon_path(2677),
        Some("icons/entities/mobs/mobicon_21.png")
    );
    assert_eq!(
        combat_target_icon_path(2678),
        Some("icons/entities/mobs/mobicon_235.png")
    );
    assert_eq!(
        combat_target_icon_path(2671),
        Some("icons/entities/npc/npcicon_87.png")
    );
    assert_eq!(
        combat_target_icon_path(2672),
        Some("icons/entities/npc/npcicon_88.png")
    );
    assert_eq!(combat_target_icon_path(2374), None);
}

#[test]
fn normal_world_player_focus_uses_the_shared_talk_target_hud() {
    let player = Entity::from_bits(77);
    let focused = LegacyFocusedTarget {
        entity: player,
        kind: LegacyTargetKind::Player,
        distance: 4.0,
        talk_enabled: true,
    };
    assert_eq!(
        primary_target_icon(&LegacyTargetSelection {
            focused_player: Some(focused),
            check_attack_target: false,
            ..default()
        }),
        Some((player, PrimaryTargetIconKind::Talk))
    );

    assert_eq!(
        primary_target_icon(&LegacyTargetSelection {
            focused_npc: Some(LegacyFocusedTarget {
                entity: Entity::from_bits(78),
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 3.0,
                talk_enabled: false,
            }),
            focused_player: Some(focused),
            check_attack_target: false,
            ..default()
        }),
        None,
        "the player marker must not contradict TalkPlayer's NPC-first branch"
    );

    assert_eq!(
        primary_target_icon(&LegacyTargetSelection {
            focused_player: Some(LegacyFocusedTarget {
                talk_enabled: false,
                ..focused
            }),
            check_attack_target: false,
            ..default()
        }),
        None
    );
}

#[test]
fn combat_target_icon_keeps_priority_over_talk_icon() {
    let friendly = Entity::from_bits(42);
    let hostile = Entity::from_bits(43);
    assert_eq!(
        primary_target_icon(&LegacyTargetSelection {
            focused_npc: Some(LegacyFocusedTarget {
                entity: friendly,
                kind: LegacyTargetKind::Npc { team: 1 },
                distance: 3.0,
                talk_enabled: true,
            }),
            check_attack_target: false,
            attack_targets: vec![LegacyAttackTarget {
                entity: hostile,
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 1.0,
            }],
            ..default()
        }),
        Some((hostile, PrimaryTargetIconKind::Combat))
    );
}

#[test]
fn all_twelve_tutorial_assets_have_exact_paths_and_sizes() {
    assert_eq!(
        TutorialIllustration::ALL.map(TutorialIllustration::asset_path),
        [
            "ui/en/gameplay/tutorial/tut_mouse.png",
            "ui/en/gameplay/tutorial/tut_lmouse.png",
            "ui/en/gameplay/tutorial/tut_rmouse.png",
            "ui/en/gameplay/tutorial/tut_move.png",
            "ui/en/gameplay/tutorial/tut_move_s.png",
            "ui/en/gameplay/tutorial/tut_move_w.png",
            "ui/en/gameplay/tutorial/tut_jump.png",
            "ui/en/gameplay/tutorial/tut_one.png",
        ]
    );
    assert_eq!(
        TutorialIllustration::ALL.map(TutorialIllustration::size),
        [
            (116.0, 154.0),
            (116.0, 154.0),
            (116.0, 154.0),
            (182.0, 131.0),
            (182.0, 131.0),
            (182.0, 131.0),
            (298.0, 91.0),
            (88.0, 91.0),
        ]
    );
    assert_eq!(
        TutorialArrowDirection::ALL.map(TutorialArrowDirection::asset_path),
        [
            "ui/en/gameplay/tutorial/tut_left.png",
            "ui/en/gameplay/tutorial/tut_right.png",
            "ui/en/gameplay/tutorial/tut_up.png",
            "ui/en/gameplay/tutorial/tut_down.png",
        ]
    );
    assert_eq!(
        TutorialArrowDirection::ALL.map(TutorialArrowDirection::size),
        [(90.0, 69.0), (88.0, 69.0), (69.0, 89.0), (69.0, 89.0)]
    );
}

#[test]
fn tutorial_legacy_screen_pivot_ids_are_explicit_and_fail_closed() {
    assert_eq!(
        TutorialCueScalePivot::from_legacy(0),
        Some(TutorialCueScalePivot::TopRight)
    );
    assert_eq!(
        TutorialCueScalePivot::from_legacy(1),
        Some(TutorialCueScalePivot::TopLeft)
    );
    assert_eq!(
        TutorialCueScalePivot::from_legacy(2),
        Some(TutorialCueScalePivot::BottomRight)
    );
    assert_eq!(
        TutorialCueScalePivot::from_legacy(3),
        Some(TutorialCueScalePivot::BottomLeft)
    );
    assert_eq!(
        TutorialCueScalePivot::from_legacy(4),
        Some(TutorialCueScalePivot::Center)
    );
    assert_eq!(
        TutorialCueScalePivot::from_legacy(5),
        Some(TutorialCueScalePivot::CenterTop)
    );
    assert_eq!(TutorialCueScalePivot::from_legacy(255), None);
}

#[test]
fn health_fraction_is_fail_closed_and_clamped() {
    let mut status = PlayerStatusUi::default();
    status.max_hp = 0;
    assert_eq!(status.health_fraction(), 0.0);
    status.max_hp = 100;
    status.hp = 250;
    assert_eq!(status.health_fraction(), 1.0);
    status.hp = -5;
    assert_eq!(status.health_fraction(), 0.0);
}

#[test]
fn fusion_matter_fraction_is_fail_closed_and_clamped() {
    let mut minimap = MinimapUi::default();
    minimap.max_fusion_matter = 220;
    assert_eq!(minimap.fusion_matter_fraction(), 0.0);
    minimap.fusion_matter = 110;
    assert_eq!(minimap.fusion_matter_fraction(), 0.5);
    minimap.fusion_matter = 500;
    assert_eq!(minimap.fusion_matter_fraction(), 1.0);
    minimap.fusion_matter = -1;
    assert_eq!(minimap.fusion_matter_fraction(), 0.0);
    minimap.max_fusion_matter = 0;
    assert_eq!(minimap.fusion_matter_fraction(), 0.0);
}

#[test]
fn minimap_marker_icons_and_projection_match_retrobution() {
    assert_eq!(
        MinimapMarkerIcon::ALL.map(MinimapMarkerIcon::asset_path),
        [
            "ui/en/gameplay/minimap/map_icon_15.png",
            "ui/en/gameplay/minimap/map_icon_16.png",
            "ui/en/world-map/markers/map_icon_17.png",
            "ui/en/world-map/markers/map_icon_18.png",
            "ui/en/world-map/markers/map_icon_19.png",
            "ui/en/world-map/markers/map_icon_20.png",
            "ui/en/world-map/markers/map_icon_26.png",
        ]
    );
    let player = Vec3::new(4_096.0, 10.0, 4_096.0);
    let marker = minimap_marker(
        player,
        Vec3::new(4_160.0, 10.0, 4_096.0),
        8.0,
        MinimapMarkerIcon::New,
    )
    .unwrap();
    assert_eq!(marker.icon, MinimapMarkerIcon::New);
    assert_eq!(marker.left, 103.0);
    assert_eq!(marker.top, 66.0);
    assert_eq!((marker.width, marker.height), (16.0, 16.0));

    let table_icon = MinimapTableMarkerIcon::from_table_data(1).unwrap();
    assert_eq!(table_icon.index(), 1);
    assert_eq!(table_icon.asset_path(), WORLD_MAP_MARKER_PATHS[1]);
    assert_eq!(table_icon.dimensions(), Vec2::splat(32.0));
    let large_marker = minimap_marker(
        player,
        Vec3::new(4_160.0, 10.0, 4_096.0),
        8.0,
        MinimapMarkerIcon::TableData(table_icon),
    )
    .unwrap();
    assert_eq!((large_marker.left, large_marker.top), (95.0, 58.0));
    assert_eq!((large_marker.width, large_marker.height), (32.0, 32.0));
    assert!(MinimapTableMarkerIcon::from_table_data(-1).is_none());
    assert!(MinimapTableMarkerIcon::from_table_data(34).is_some());
    assert!(MinimapTableMarkerIcon::from_table_data(35).is_none());
    assert!(
        minimap_marker_sized(
            player,
            player,
            8.0,
            MinimapMarkerIcon::New,
            Vec2::new(f32::NAN, 16.0),
        )
        .is_none()
    );
    assert!(
        minimap_marker(
            player,
            Vec3::new(8_192.0, 10.0, 8_192.0),
            8.0,
            MinimapMarkerIcon::Advance,
        )
        .is_none()
    );
    assert!(
        minimap_marker(
            player,
            Vec3::new(4_224.0, 10.0, 4_096.0),
            8.0,
            MinimapMarkerIcon::ShowNpc,
        )
        .is_none(),
        "RenderMinimap uses a strict < radius test"
    );
    assert!(
        minimap_marker(
            player,
            Vec3::new(4_223.0, 10.0, 4_096.0),
            8.0,
            MinimapMarkerIcon::ShowNpc,
        )
        .is_some()
    );
    assert!(
        MINIMAP_MARKER_CAPACITY > 8,
        "the original iterates every visible NPC rather than truncating at eight"
    );
}

#[test]
fn minimap_sample_stitches_at_most_four_tiles() {
    let samples = minimap_tiles(4096.0, 4096.0, 8.0);
    assert_eq!(samples.len(), 4);
    assert!(
        samples
            .iter()
            .all(|sample| (1..=16).contains(&sample.tile_number))
    );
}

#[test]
fn waypoint_radius_is_inclusive_and_clamps_to_the_148_pixel_edge() {
    let player = Vec3::new(4096.0, 0.0, 4096.0);
    // ratio/256/2 * 8192 = 128 source-world units at ratio 8.
    let sample = minimap_waypoint(
        player,
        Vec3::new(player.x + 128.0, player.y, player.z),
        8.0,
        false,
        0.0,
    )
    .unwrap();
    assert_eq!(sample.icon, MinimapWaypointIcon::OutOfRange);
    assert!((sample.left - 139.5).abs() < 0.001, "{sample:?}");
    assert!((sample.top - 65.5).abs() < 0.001, "{sample:?}");
    assert!((sample.rotation_degrees - 90.0).abs() < 0.001);

    let farther = minimap_waypoint(
        player,
        Vec3::new(player.x + 1024.0, player.y, player.z),
        8.0,
        false,
        0.0,
    )
    .unwrap();
    assert!((farther.left - sample.left).abs() < 0.001);
    assert!((farther.top - sample.top).abs() < 0.001);
}

#[test]
fn waypoint_icon_routes_and_independent_pulses_are_exact() {
    assert_eq!(
        MinimapWaypointIcon::ALL.map(MinimapWaypointIcon::asset_path),
        [
            "ui/en/gameplay/minimap/map_icon_02.png",
            "ui/en/gameplay/minimap/map_icon_03.png",
            "ui/en/gameplay/minimap/plus_but_1.png",
            "ui/en/gameplay/minimap/minus_but_1.png",
        ]
    );
    assert_eq!(minimap_marker_alpha(false, 0.5), 1.0);
    assert_eq!(minimap_marker_alpha(true, 0.0), 0.0);
    assert!((minimap_marker_alpha(true, 0.5) - 1.0).abs() < 0.000_001);
    assert!(minimap_marker_alpha(true, 1.0) < 0.000_001);

    let player_alpha = minimap_marker_alpha(true, 0.0);
    let waypoint = minimap_waypoint(Vec3::ZERO, Vec3::ZERO, 8.0, false, 0.0)
        .expect("finite exact center waypoint");
    assert_eq!(player_alpha, 0.0);
    assert_eq!(waypoint.alpha, 1.0);
}
