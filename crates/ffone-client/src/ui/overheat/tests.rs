use std::{fs, path::Path};

use bevy::{
    asset::AssetPlugin,
    window::{PrimaryWindow, WindowResolution},
};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::gui_skin::{GuiInsets, gui_style};

use crate::overheat_ui::*;

fn ready_model(heat: f32) -> OverheatUiModel {
    OverheatUiModel {
        ready_for_play: true,
        main: LegacyOverheatWeaponSlot::equipped(100, 1, heat),
        ..default()
    }
}

#[test]
fn clean_localized_default_is_dormant_and_preview_is_explicit() {
    let table = LegacyClassWeaponOverheatTable::default();
    let config = OverheatUiConfig::default();
    let model = ready_model(50.0);

    assert_eq!(config.localized_mode, LegacyWpnOverheatMode::None);
    assert!(
        !overheat_ui_view(
            1_264,
            681,
            config,
            OverheatParityPreview::default(),
            model,
            &table
        )
        .visible
    );
    assert!(
        overheat_ui_view(
            1_264,
            681,
            config,
            OverheatParityPreview::enabled(),
            model,
            &table
        )
        .visible
    );
    assert!(
        overheat_ui_view(
            1_264,
            681,
            OverheatUiConfig {
                localized_mode: LegacyWpnOverheatMode::Use,
                ..default()
            },
            OverheatParityPreview::default(),
            model,
            &table
        )
        .visible
    );
    assert!(
        !overheat_ui_view(
            1_264,
            681,
            OverheatUiConfig {
                localized_mode: LegacyWpnOverheatMode::UseOriginal,
                ..default()
            },
            OverheatParityPreview::default(),
            model,
            &table
        )
        .visible
    );
    assert!(
        !overheat_ui_view(
            1_264,
            681,
            config,
            OverheatParityPreview::enabled(),
            OverheatUiModel {
                ready_for_play: false,
                ..model
            },
            &table
        )
        .visible
    );
    assert!(
        !overheat_ui_view(
            1_264,
            681,
            config,
            OverheatParityPreview::enabled(),
            ready_model(0.0),
            &table
        )
        .visible
    );
}

#[test]
fn exact_1264_by_681_integer_half_rectangles_are_stable() {
    let table = LegacyClassWeaponOverheatTable::default();
    let config = OverheatUiConfig::default();
    let preview = OverheatParityPreview::enabled();

    let normal = overheat_ui_view(1_264, 681, config, preview, ready_model(50.0), &table);
    assert_eq!(normal.fraction, 0.5);
    assert_eq!(
        normal.background,
        Some(OverheatUiRect::new(572.0, 340.0, 17.0, 86.0))
    );
    assert_eq!(
        normal.normal_fill,
        Some(OverheatUiRect::new(579.0, 384.0, 6.0, 40.0))
    );
    assert_eq!(normal.maximum, None);

    let maximum = overheat_ui_view(1_264, 681, config, preview, ready_model(100.0), &table);
    assert_eq!(maximum.fraction, 1.0);
    assert_eq!(
        maximum.background,
        Some(OverheatUiRect::new(572.0, 340.0, 17.0, 86.0))
    );
    assert_eq!(maximum.normal_fill, None);
    assert_eq!(
        maximum.maximum,
        Some(OverheatUiRect::new(571.0, 336.0, 22.0, 95.0))
    );
}

#[test]
fn exact_1280_by_720_rectangles_and_center_scale_are_stable() {
    let table = LegacyClassWeaponOverheatTable::default();
    let preview = OverheatParityPreview::enabled();
    let unscaled = overheat_ui_view(
        1_280,
        720,
        OverheatUiConfig::default(),
        preview,
        ready_model(25.0),
        &table,
    );
    assert_eq!(
        unscaled.background,
        Some(OverheatUiRect::new(580.0, 360.0, 17.0, 86.0))
    );
    assert_eq!(
        unscaled.normal_fill,
        Some(OverheatUiRect::new(587.0, 424.0, 6.0, 20.0))
    );

    let scaled = overheat_ui_view(
        1_280,
        720,
        OverheatUiConfig {
            ui_scale: 2.0,
            ..default()
        },
        preview,
        ready_model(25.0),
        &table,
    );
    assert_eq!(
        scaled.background,
        Some(OverheatUiRect::new(520.0, 360.0, 34.0, 172.0))
    );
    assert_eq!(
        scaled.normal_fill,
        Some(OverheatUiRect::new(534.0, 488.0, 12.0, 40.0))
    );
}

#[test]
fn clean_class_weapon_table_rows_are_exact() {
    let table = LegacyClassWeaponOverheatTable::default();
    let expected_use = [0, 10, 12, 13, 16, 20, 35];
    let expected_cool_time = [0, 20, 20, 30, 30, 40, 40];
    for weapon_type in 0..=6 {
        let row = table.row(weapon_type).unwrap();
        assert_eq!(row.weapon_type, weapon_type);
        assert_eq!(row.overheat_use, expected_use[weapon_type as usize]);
        assert_eq!(
            row.cooldown_tenths,
            expected_cool_time[weapon_type as usize]
        );
        if weapon_type == 0 {
            assert_eq!(*row, LegacyClassWeaponOverheatRow::default());
        } else {
            assert_eq!(row.rate_of_fire, 75);
            assert_eq!(row.overheat_max, 100);
            assert_eq!(row.unuse_cool_per_second, -35);
            assert_eq!(row.use_cool_per_second, -5);
            assert_eq!(row.overheat_effect, 1);
        }
    }
}

#[test]
fn accepted_attack_uses_frame_delta_and_starts_channel_17_window() {
    let table = LegacyClassWeaponOverheatTable::default();
    let mut model = ready_model(10.0);

    assert!(model.allows_attack(&table));
    model.record_accepted_attack(0.25, &table);

    assert_eq!(model.main.heat, 10.25);
    assert_eq!(model.main_use_cooldown_remaining, 2.0);
    assert_ne!(
        model.main.heat,
        10.0 + table.row(1).unwrap().overheat_use as f32 * 0.25
    );
}

#[test]
fn main_and_backup_cool_at_their_distinct_clean_rates() {
    let table = LegacyClassWeaponOverheatTable::default();
    let mut model = OverheatUiModel {
        ready_for_play: true,
        main: LegacyOverheatWeaponSlot::equipped(10, 1, 50.0),
        backup: LegacyOverheatWeaponSlot::equipped(11, 2, 50.0),
        main_use_cooldown_remaining: 2.0,
    };

    model.advance(1.0, &table);
    assert_eq!(model.main.heat, 45.0);
    assert_eq!(model.backup.heat, 15.0);
    assert_eq!(model.main_use_cooldown_remaining, 1.0);

    model.advance(1.0, &table);
    assert_eq!(model.main.heat, 40.0);
    assert_eq!(model.backup.heat, 0.0);
    assert_eq!(model.main_use_cooldown_remaining, 0.0);

    model.advance(1.0, &table);
    assert_eq!(model.main.heat, 5.0);
}

#[test]
fn equipped_weapon_without_a_class_row_resets_its_heat() {
    let table = LegacyClassWeaponOverheatTable::default();
    let mut model = OverheatUiModel {
        main: LegacyOverheatWeaponSlot::equipped(10, 99, 80.0),
        backup: LegacyOverheatWeaponSlot {
            item_id: 11,
            weapon_type: None,
            heat: 60.0,
        },
        main_use_cooldown_remaining: 2.0,
        ..default()
    };

    model.advance(0.25, &table);

    assert_eq!(model.main.heat, 0.0);
    assert_eq!(model.backup.heat, 0.0);
    assert_eq!(model.main_use_cooldown_remaining, 0.0);
}

#[test]
fn max_heat_blocks_attack_and_switch_codes_transfer_heat_exactly() {
    let table = LegacyClassWeaponOverheatTable::default();
    let mut model = OverheatUiModel {
        main: LegacyOverheatWeaponSlot::equipped(10, 1, 100.0),
        backup: LegacyOverheatWeaponSlot::equipped(11, 2, 25.0),
        ..default()
    };
    assert!(!model.allows_attack(&table));

    model.apply_weapon_switch(LegacyOverheatWeaponSwitch::SwapMainAndBackup);
    assert_eq!((model.main.heat, model.backup.heat), (25.0, 100.0));
    model.apply_weapon_switch(LegacyOverheatWeaponSwitch::MoveMainToBackup);
    assert_eq!((model.main.heat, model.backup.heat), (0.0, 25.0));
    model.apply_weapon_switch(LegacyOverheatWeaponSwitch::MoveBackupToMain);
    assert_eq!((model.main.heat, model.backup.heat), (25.0, 0.0));
}

#[test]
fn source_skin_pointers_and_borders_match_the_converted_clean_contract() {
    for (style_name, path_id) in [
        ("OverheatBG", OVERHEAT_BACKGROUND_PATH_ID),
        ("OverheatMax", OVERHEAT_MAXIMUM_PATH_ID),
    ] {
        let style = gui_style("FusionFallHUDSkin", style_name).unwrap();
        assert_eq!(
            style.border,
            GuiInsets {
                left: 8,
                right: 8,
                top: 2,
                bottom: 4,
            }
        );
        assert_eq!(style.states["normal"].background.path_id, path_id);
    }
}

#[test]
fn runtime_texture_bytes_and_dimensions_match_the_acceptance_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for contract in OVERHEAT_TEXTURE_CONTRACTS {
        let bytes = fs::read(root.join(contract.runtime_path)).unwrap();
        let digest = format!("{:X}", Sha256::digest(&bytes));
        assert_eq!(digest, contract.sha256);
        let image = image::load_from_memory(&bytes).unwrap();
        assert_eq!(
            (image.width(), image.height()),
            (contract.source_width, contract.source_height)
        );
    }
}

#[test]
fn plugin_spawns_without_a_camera_and_stays_hidden_until_preview_opt_in() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .add_plugins(OverheatUiPlugin);
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(1_264, 681),
            ..default()
        },
        PrimaryWindow,
    ));
    *app.world_mut().resource_mut::<OverheatUiModel>() = ready_model(50.0);
    app.update();

    {
        let world = app.world_mut();
        let mut roots = world.query_filtered::<(&Node, &Visibility), With<OverheatUiRoot>>();
        let (node, visibility) = roots.single(world).unwrap();
        assert_eq!(*visibility, Visibility::Hidden);
        assert_eq!((node.width, node.height), (px(1_264), px(681)));
        let mut cameras = world.query::<&Camera>();
        assert_eq!(cameras.iter(world).count(), 0);
    }

    *app.world_mut().resource_mut::<OverheatParityPreview>() = OverheatParityPreview::enabled();
    app.update();

    let world = app.world_mut();
    let mut roots = world.query_filtered::<&Visibility, With<OverheatUiRoot>>();
    assert_eq!(*roots.single(world).unwrap(), Visibility::Visible);
    let mut elements = world.query::<(&OverheatUiElement, &Node)>();
    for (element, node) in elements.iter(world) {
        match element {
            OverheatUiElement::Background => {
                assert_eq!(
                    (node.left, node.top, node.width, node.height),
                    (px(572), px(340), px(17), px(86))
                );
            }
            OverheatUiElement::NormalFill => {
                assert_eq!(
                    (node.left, node.top, node.width, node.height),
                    (px(579), px(384), px(6), px(40))
                );
            }
            OverheatUiElement::Maximum => assert_eq!(node.display, Display::None),
        }
    }
}
