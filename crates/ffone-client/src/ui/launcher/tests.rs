use std::{fs, path::Path};

use bevy::asset::AssetPlugin;
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::launcher_ui::*;

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.000_1,
        "expected {expected}, got {actual}"
    );
}

fn trigger() -> LauncherTriggerSpec {
    LauncherTriggerSpec {
        trigger_position: Vec3::new(100.0, 200.0, 300.0),
        trigger_euler_degrees: Vec3::new(0.0, 350.0, 0.0),
        min_power: 10.0,
        max_power: 30.0,
        initial_rotation_degrees: Vec3::new(0.0, 25.0, 0.0),
        maximum_rotation_degrees: Vec3::new(20.0, 45.0, 0.0),
    }
}

fn open_model() -> (LauncherUiModel, LauncherUiOutbox) {
    let mut model = LauncherUiModel::default();
    let mut outbox = LauncherUiOutbox::default();
    model
        .open(trigger(), Vec3::new(1.0, 2.0, 3.0), 7.0, &mut outbox)
        .unwrap();
    (model, outbox)
}

#[test]
fn launcher_aim_and_shot_follow_native_heading_at_cardinal_and_oblique_yaws() {
    use crate::coordinates::{LegacyUnityHeadingDegrees, ProtocolScaledVelocity};
    for yaw in [0.0, 45.0, 90.0, 180.0, 270.0, 315.0] {
        let mut spec = trigger();
        spec.trigger_euler_degrees.y = yaw - 180.0;
        spec.initial_rotation_degrees = Vec3::ZERO;
        let mut model = LauncherUiModel::default();
        let mut outbox = LauncherUiOutbox::default();
        model.open(spec, Vec3::ZERO, 7.0, &mut outbox).unwrap();
        for _ in 0..10 {
            model.fixed_update_aim(1.0, 0.0);
        }
        model.press_fire(false, &mut outbox);
        let shot = model.release_fire(false, &mut outbox).unwrap();
        let horizontal = Vec3::new(shot.forward.x, 0.0, shot.forward.z).normalize();
        assert!(horizontal.abs_diff_eq(
            LegacyUnityHeadingDegrees::new(shot.facing_yaw_degrees).native_forward(), 0.0001));
        assert!(shot.forward.y > 0.0, "up input must launch upward");
        assert!(shot.velocity.abs_diff_eq(shot.forward * shot.power, 0.0001));
        let wire = ProtocolScaledVelocity::from_native(shot.velocity).raw();
        assert_eq!(wire[0], (-shot.velocity.x * 100.0) as i32);
    }
}

#[test]
fn clean_root_components_skin_and_serialized_overrides_are_explicit() {
    assert_eq!(LAUNCHER_UI_GAME_MODE_SLOT, 13);
    assert_eq!(LAUNCHER_UI_ROOT_NAME, "LuncherMode");
    assert_eq!(LAUNCHER_UI_ROOT_PATH_ID, 1_315);
    assert_eq!(LAUNCHER_UI_TRANSFORM_PATH_ID, 1_236);
    assert_eq!(LAUNCHER_UI_GUI_COMPONENT_PATH_ID, 1_546);
    assert_eq!(LAUNCHER_UI_LOGIC_COMPONENT_PATH_ID, 1_547);
    assert_eq!(LAUNCHER_UI_GUI_SCRIPT_PATH_ID, 1_076);
    assert_eq!(LAUNCHER_UI_LOGIC_SCRIPT_PATH_ID, 922);
    assert_eq!(LAUNCHER_UI_SKIN_PATH_ID, 1_387);
    assert!(!LAUNCHER_UI_ROOT_INITIALLY_ACTIVE);
    assert_close(LAUNCHER_UI_ROTATE_SPEED_PER_FIXED_UPDATE, 1.6);
    assert_eq!(LAUNCHER_UI_CAMERA_LAUNCH_OFFSET, 5.0);
    assert_eq!(LAUNCHER_UI_BACKDROP_BORDER, BorderRect::all(2.0));
    assert_eq!(LAUNCHER_REQUEST_PACKET_ID, 318_767_166);
    assert_eq!(LAUNCHER_REQUEST_PACKET_SIZE, 40);
    assert_eq!(LAUNCHER_BROADCAST_PACKET_ID, 822_083_702);
    assert_eq!(LAUNCHER_BROADCAST_PACKET_SIZE, 52);
    assert_eq!(LAUNCHER_UI_PARITY_STATUS, "partial");
    assert!(LAUNCHER_UI_PARITY_CAVEAT.contains("RequestEscapeCloseGate resolution"));
    assert!(LAUNCHER_UI_PARITY_CAVEAT.contains("live HP/system-popup synchronization"));
    assert!(LAUNCHER_UI_PARITY_CAVEAT.contains("CnGuiEUALA belongs to LoginMode"));
}

#[test]
fn label_templates_keep_clean_copy_semantic_and_dynamic_copy_in_args() {
    let labels = LauncherUiLabels::default();
    let power = labels.power_localized();
    assert_eq!(power.key, LAUNCHER_UI_POWER_LOCALIZATION_KEY);
    assert_eq!(power.fallback, LAUNCHER_UI_POWER_LABEL_KEY);
    assert!(power.args.is_empty());
    let tip = labels.tip_localized();
    assert_eq!(tip.key, LAUNCHER_UI_TIP_LOCALIZATION_KEY);
    assert_eq!(tip.fallback, LAUNCHER_UI_TIP_LABEL_KEY);
    assert!(tip.args.is_empty());

    let labels = LauncherUiLabels {
        power: "Server power".to_owned(),
        tip: "Player-authored instruction".to_owned(),
    };
    let power = labels.power_localized();
    assert_eq!(power.key, "ui.content.passthrough");
    assert_eq!(power.fallback, "{text}");
    assert_eq!(
        power.args.get("text").map(String::as_str),
        Some("Server power")
    );
    let tip = labels.tip_localized();
    assert_eq!(tip.key, "ui.content.passthrough");
    assert_eq!(tip.fallback, "{text}");
    assert_eq!(
        tip.args.get("text").map(String::as_str),
        Some("Player-authored instruction")
    );
}

#[test]
fn presentation_ecs_is_key_first_and_preserves_serialized_font_metrics() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(LauncherUiPlugin);
    app.update();

    let expected_font = app.world().resource::<LauncherUiAssets>().font.clone();
    let world = app.world_mut();
    let mut text_query = world.query::<(
        &LauncherUiTextRole,
        &Text,
        (&TextFont, &LineHeight),
        Option<&LocalizedText>,
    )>();
    let rows = text_query.iter(world).collect::<Vec<_>>();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|(_, _, _, localized)| localized.is_some()));
    for (role, text, font, localized) in rows {
        assert_eq!(
            font.0.font,
            bevy::text::FontSource::Handle(expected_font.clone())
        );
        let localized = localized.unwrap();
        match role {
            LauncherUiTextRole::Power => {
                assert_eq!(text.0, LAUNCHER_UI_POWER_LABEL_KEY);
                assert_eq!(localized.key, LAUNCHER_UI_POWER_LOCALIZATION_KEY);
                assert_eq!(localized.fallback, LAUNCHER_UI_POWER_LABEL_KEY);
                assert_eq!(
                    font.0.font_size.eval(Vec2::ZERO, 16.0),
                    LAUNCHER_UI_DEFAULT_FONT_SIZE
                );
                assert_eq!((*font.1), LineHeight::Px(LAUNCHER_UI_DEFAULT_LINE_HEIGHT));
            }
            LauncherUiTextRole::Tip => {
                assert_eq!(text.0, LAUNCHER_UI_TIP_LABEL_KEY);
                assert_eq!(localized.key, LAUNCHER_UI_TIP_LOCALIZATION_KEY);
                assert_eq!(localized.fallback, LAUNCHER_UI_TIP_LABEL_KEY);
                assert_eq!(
                    font.0.font_size.eval(Vec2::ZERO, 16.0),
                    LAUNCHER_UI_SMALL_FONT_SIZE
                );
                assert_eq!((*font.1), LineHeight::Px(LAUNCHER_UI_SMALL_LINE_HEIGHT));
            }
        }
    }

    {
        let mut labels = app.world_mut().resource_mut::<LauncherUiLabels>();
        labels.power = "Server power".to_owned();
        labels.tip = "Player-authored instruction".to_owned();
    }
    app.update();
    let world = app.world_mut();
    let mut localized_query = world.query::<(&LauncherUiTextRole, &LocalizedText)>();
    for (role, localized) in localized_query.iter(world) {
        let expected = match role {
            LauncherUiTextRole::Power => "Server power",
            LauncherUiTextRole::Tip => "Player-authored instruction",
        };
        assert_eq!(localized.key, "ui.content.passthrough");
        assert_eq!(localized.fallback, "{text}");
        assert_eq!(
            localized.args.get("text").map(String::as_str),
            Some(expected)
        );
    }

    let production = concat!(
        include_str!("constants.rs"),
        "\n",
        include_str!("state.rs"),
        "\n",
        include_str!("assets.rs"),
        "\n",
        include_str!("systems.rs"),
        "\n",
        include_str!("audio.rs"),
        "\n",
        include_str!("localization_launcher_ui_power_localization_k.rs"),
        "\n",
        include_str!("layout.rs"),
        "\n",
        include_str!("operations.rs"),
        "\n",
        include_str!("types.rs"),
        "\n",
        include_str!("validation.rs"),
        "\n",
        include_str!("frame.rs"),
        "\n",
        include_str!("interaction.rs"),
        "\n",
        include_str!("models.rs"),
        "\n",
        include_str!("view.rs"),
        "\n",
        include_str!("mod.rs")
    )
        .split("#[cfg(test)]")
        .next()
        .unwrap();
    assert!(production.contains("before(LocalizationSet::Apply)"));
    assert!(!production.contains("Query<(&LauncherUiTextRole, &mut Text)>"));
}

#[test]
fn clean_1264x681_geometry_and_source_bugs_are_preserved() {
    let layout = LauncherUiLayout::from_viewport(Vec2::new(1_264.0, 681.0), 0.0);
    assert_eq!(layout.viewport, Vec2::new(1_264.0, 681.0));
    assert_eq!(
        layout.crosshair,
        LauncherUiRect::new(302.0, 10.0, 660.0, 660.0)
    );
    assert_eq!(
        layout.backdrop[0],
        LauncherUiRect::new(0.0, 0.0, 1_264.0, 10.0)
    );
    assert_eq!(
        layout.backdrop[1],
        LauncherUiRect::new(0.0, 10.0, 302.0, 671.0)
    );
    assert_eq!(
        layout.backdrop[2],
        LauncherUiRect::new(302.0, 670.0, 962.0, 1_331.0)
    );
    assert_eq!(
        layout.backdrop[3],
        LauncherUiRect::new(962.0, 10.0, 302.0, 660.0)
    );
    assert_eq!(layout.gauge, LauncherUiRect::new(123.0, 40.0, 169.0, 595.0));
    assert_eq!(
        layout.power_label,
        LauncherUiRect::new(123.0, -129.0, 169.0, 30.0)
    );
    assert_eq!(
        layout.tip_label,
        LauncherUiRect::new(133.0, 600.0, 150.0, 30.0)
    );
    assert_eq!(
        layout.gauge_bar,
        LauncherUiRect::new(165.0, 539.0, 87.0, 31.0)
    );

    let full = LauncherUiLayout::from_viewport(Vec2::new(1_264.0, 681.0), 1.0);
    assert_eq!(full.gauge_bar, LauncherUiRect::new(165.0, 70.0, 87.0, 31.0));
    let middle = LauncherUiLayout::from_viewport(Vec2::new(1_264.0, 681.0), 0.5);
    assert_close(middle.gauge_bar.y, 304.5);
}

#[test]
fn entry_normalizes_only_start_y_and_emits_clean_side_effect_order() {
    let (model, mut outbox) = open_model();
    assert_eq!(model.phase, LauncherUiPhase::Aiming);
    assert_eq!(model.start_rotation_degrees, Vec3::new(0.0, 15.0, 0.0));
    assert_eq!(model.current_rotation_degrees, Vec3::ZERO);
    assert_eq!(model.current_power, 10.0);
    assert_eq!(model.normalized_power(), 0.0);
    assert!(!model.charging);
    assert!(model.power_rising);
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            LauncherUiEffect::Audio(LauncherUiAudioCue::ClickOn),
            LauncherUiEffect::SetNameVisible(false),
            LauncherUiEffect::SetCameraPosition(Vec3::new(100.0, 207.0, 300.0)),
            LauncherUiEffect::SetCameraCustomControl(true),
            LauncherUiEffect::SetTriggerRenderersVisible(false),
            LauncherUiEffect::SetAvatarRenderersVisible(false),
        ]
    );
}

#[test]
fn fixed_aim_uses_serialized_speed_and_exact_clamps() {
    let (mut model, _) = open_model();
    for _ in 0..100 {
        model.fixed_update_aim(1.0, 1.0);
    }
    assert_eq!(model.current_rotation_degrees.x, -20.0);
    assert_eq!(model.current_rotation_degrees.y, 45.0);
    for _ in 0..100 {
        model.fixed_update_aim(-1.0, -1.0);
    }
    assert_eq!(model.current_rotation_degrees.x, 0.0);
    assert_eq!(model.current_rotation_degrees.y, -45.0);
}

#[test]
fn power_is_a_one_range_per_second_triangle_and_pulses_only_after_crossing_bounds() {
    let (mut model, mut outbox) = open_model();
    outbox.clear();
    assert!(model.press_fire(false, &mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            LauncherUiEffect::Audio(LauncherUiAudioCue::StartPower),
            LauncherUiEffect::Audio(LauncherUiAudioCue::PowerPulse),
        ]
    );
    model.advance_power(0.5, false, &mut outbox);
    assert_eq!(model.current_power, 20.0);
    assert_eq!(model.normalized_power(), 0.5);
    assert!(outbox.is_empty());
    model.advance_power(0.5, false, &mut outbox);
    assert_eq!(model.current_power, 30.0);
    assert!(
        model.power_rising,
        "clean code uses strict > at the endpoint"
    );
    assert!(outbox.is_empty());
    model.advance_power(0.01, false, &mut outbox);
    assert_eq!(model.current_power, 30.0);
    assert!(!model.power_rising);
    assert_eq!(
        outbox.pop_front(),
        Some(LauncherUiEffect::Audio(LauncherUiAudioCue::PowerPulse))
    );
    model.advance_power(1.01, false, &mut outbox);
    assert_eq!(model.current_power, 10.0);
    assert!(model.power_rising);
    assert_eq!(
        outbox.pop_front(),
        Some(LauncherUiEffect::Audio(LauncherUiAudioCue::PowerPulse))
    );
}

#[test]
fn system_popup_freezes_update_after_combat_icon_but_fixed_aim_remains_separate() {
    let (mut model, mut outbox) = open_model();
    outbox.clear();
    model.press_fire(false, &mut outbox);
    outbox.clear();
    model.update(
        LauncherUiFrameInput {
            delta_seconds: 5.0,
            fire_released: true,
            escape_pressed: true,
            current_hp: 0,
            system_popup_active: true,
            ..default()
        },
        &mut outbox,
    );
    assert_eq!(model.current_power, 10.0);
    assert!(model.visible());
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![LauncherUiEffect::SetCombatIcon(-1)]
    );
    model.fixed_update_aim(1.0, 1.0);
    assert_close(model.current_rotation_degrees.x, -1.6);
    assert_close(model.current_rotation_degrees.y, 1.6);
}

#[test]
fn release_restores_renderers_starts_launcher_and_exits_in_clean_order() {
    let (mut model, mut outbox) = open_model();
    outbox.clear();
    model.current_rotation_degrees = Vec3::new(-10.0, 5.0, 0.0);
    model.press_fire(false, &mut outbox);
    model.advance_power(0.25, false, &mut outbox);
    outbox.clear();
    let shot = model.release_fire(false, &mut outbox).unwrap();
    assert_close(shot.power, 15.0);
    assert_close(shot.facing_yaw_degrees, 200.0);
    assert_close(shot.forward.length(), 1.0);
    assert_eq!(shot.request_packet_id, LAUNCHER_REQUEST_PACKET_ID);
    assert_eq!(shot.request_packet_size, LAUNCHER_REQUEST_PACKET_SIZE);
    assert_eq!(model.phase, LauncherUiPhase::Hidden);
    assert_eq!(model.dismissal, Some(LauncherUiDismissalSource::Fired));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            LauncherUiEffect::SetTriggerRenderersVisible(true),
            LauncherUiEffect::SetAvatarRenderersVisible(true),
            LauncherUiEffect::Audio(LauncherUiAudioCue::StopPower),
            LauncherUiEffect::SetCameraCustomControl(false),
            LauncherUiEffect::StartLauncher(shot),
            LauncherUiEffect::SetCameraRotationY(200.0),
            LauncherUiEffect::Audio(LauncherUiAudioCue::Firing),
            LauncherUiEffect::ExitMode {
                source: LauncherUiDismissalSource::Fired,
                event_group: 2,
                event_function: 1,
            },
            LauncherUiEffect::SetNameVisible(true),
        ]
    );
}

#[test]
fn escape_uses_computress_gate_and_death_alone_restores_old_position() {
    let (mut model, mut outbox) = open_model();
    outbox.clear();
    assert!(model.request_escape_close(false, &mut outbox));
    assert_eq!(model.phase, LauncherUiPhase::AwaitingEscapeGate);
    assert_eq!(
        outbox.pop_front(),
        Some(LauncherUiEffect::RequestEscapeCloseGate {
            event_group: 2,
            event_function: 24,
        })
    );
    assert!(model.resolve_escape_close_gate(false, &mut outbox));
    assert_eq!(model.phase, LauncherUiPhase::Aiming);
    assert!(outbox.is_empty());
    assert!(model.request_escape_close(false, &mut outbox));
    outbox.clear();
    assert!(model.resolve_escape_close_gate(true, &mut outbox));
    let accepted = outbox.drain().collect::<Vec<_>>();
    assert!(
        !accepted
            .iter()
            .any(|effect| matches!(effect, LauncherUiEffect::RestoreAvatarPosition(_)))
    );
    assert_eq!(model.dismissal, Some(LauncherUiDismissalSource::EscapeGate));

    let (mut model, mut outbox) = open_model();
    outbox.clear();
    assert!(model.cancel_for_death(false, &mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            LauncherUiEffect::SetTriggerRenderersVisible(true),
            LauncherUiEffect::SetCameraCustomControl(false),
            LauncherUiEffect::SetAvatarRenderersVisible(true),
            LauncherUiEffect::ExitMode {
                source: LauncherUiDismissalSource::Death,
                event_group: 2,
                event_function: 1,
            },
            LauncherUiEffect::RestoreAvatarPosition(Vec3::new(1.0, 2.0, 3.0)),
            LauncherUiEffect::SetNameVisible(true),
        ]
    );
}

#[test]
fn audio_names_paths_and_mode_boundary_are_exact() {
    let cues = [
        (
            LauncherUiAudioCue::ClickOn,
            "Launcher_ClickOn",
            LAUNCHER_UI_CLICK_ON_AUDIO_PATH,
        ),
        (
            LauncherUiAudioCue::StartPower,
            "Launcher_StartPower",
            LAUNCHER_UI_START_POWER_AUDIO_PATH,
        ),
        (
            LauncherUiAudioCue::PowerPulse,
            "Launcher_PowerPulse",
            LAUNCHER_UI_POWER_PULSE_AUDIO_PATH,
        ),
        (
            LauncherUiAudioCue::StopPower,
            "Launcher_StopPower",
            LAUNCHER_UI_STOP_POWER_AUDIO_PATH,
        ),
        (
            LauncherUiAudioCue::Firing,
            "Launcher_Firing",
            LAUNCHER_UI_FIRING_AUDIO_PATH,
        ),
    ];
    for (cue, name, path) in cues {
        assert_eq!(cue.legacy_name(), name);
        assert_eq!(cue.path(), path);
    }
    let (model, _) = open_model();
    assert_eq!(
        model.input_boundary(false),
        LauncherUiInputBoundary {
            blocks_lower_ui: true,
            blocks_gameplay_input: true,
            requires_pointer: false,
            fire_enabled: true,
            escape_enabled: true,
        }
    );
    assert!(!model.input_boundary(true).fire_enabled);
    assert!(!model.input_boundary(true).escape_enabled);
}

#[test]
fn semantic_pngs_are_present_and_exact() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for asset in LAUNCHER_UI_ASSET_CONTRACTS {
        let path = workspace.join("assets/game").join(asset.path);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        assert_eq!(bytes.len() as u64, asset.png_size);
        assert_eq!(format!("{:X}", Sha256::digest(&bytes)), asset.png_sha256);
        let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .unwrap_or_else(|error| panic!("failed to decode {}: {error}", path.display()));
        assert_eq!(image.width(), asset.source_width);
        assert_eq!(image.height(), asset.source_height);
        assert_eq!(asset.source_texture_format, "DXT5");
        assert_eq!(asset.source_payload_sha256.len(), 64);
    }
}
