use std::{fs, path::Path};

use bevy::{
    asset::AssetPlugin,
    window::{PrimaryWindow, WindowResolution},
};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use crate::gui_skin::{GuiInsets, gui_style};

use crate::quick_slot_ui::*;

fn ready_model() -> QuickSlotUiModel {
    QuickSlotUiModel {
        ready_for_play: true,
        ..default()
    }
}

fn semantic_icon() -> &'static str {
    "icons/items/general/generalitemicon_00.png"
}

#[test]
fn clean_defaults_keep_quick_slot_dormant_but_preserve_localization_modes() {
    let config = QuickSlotUiConfig::default();
    assert_eq!(config.localized_mode, LegacyQuickSlotMode::None);
    assert_eq!(config.macro_chat_mode, LegacyMacroChatMode::Use);
    assert_eq!(config.chat_window_style, LegacyChatWindowStyle::Large);
    assert!(config.scale_ui);
    assert_eq!(config.ui_scale_factor, 1.05);
    assert!(!quick_slot_component_enabled(
        config,
        QuickSlotParityPreview::default()
    ));
}

#[test]
fn preview_geometry_matches_all_nested_clean_rects_at_1264x681() {
    let view = quick_slot_ui_view(
        1_264,
        681,
        QuickSlotUiConfig::default(),
        QuickSlotParityPreview::enabled(),
        &ready_model(),
        true,
    );
    assert!(view.visible);
    assert_eq!(view.scale, 1.0);
    assert_eq!(
        view.background,
        Some(QuickSlotUiRect::new(450.0, 641.0, 290.0, 39.0))
    );
    assert_eq!(
        view.slots[0].frame,
        QuickSlotUiRect::new(456.0, 645.0, 34.0, 34.0)
    );
    assert_eq!(
        view.slots[7].frame,
        QuickSlotUiRect::new(701.0, 645.0, 34.0, 34.0)
    );
}

#[test]
fn small_chat_changes_only_the_proven_outer_group_x() {
    let view = quick_slot_ui_view(
        1_264,
        681,
        QuickSlotUiConfig {
            chat_window_style: LegacyChatWindowStyle::Small,
            ..default()
        },
        QuickSlotParityPreview::enabled(),
        &ready_model(),
        true,
    );
    assert_eq!(
        view.background,
        Some(QuickSlotUiRect::new(400.0, 641.0, 290.0, 39.0))
    );
    assert_eq!(view.slots[0].frame.left, 406.0);
    assert_eq!(view.slots[7].frame.left, 651.0);
}

#[test]
fn bottom_left_scaling_matches_ffguiutility_formula() {
    let view = quick_slot_ui_view(
        1_920,
        1_080,
        QuickSlotUiConfig::default(),
        QuickSlotParityPreview::enabled(),
        &ready_model(),
        true,
    );
    let scale = 1_080.0 / 768.0 * 1.05;
    assert_eq!(view.scale, scale);
    let background = view.background.unwrap();
    assert_eq!(background.left, 450.0 * scale);
    assert_eq!(background.top, 1_080.0 - 40.0 * scale);
    assert_eq!(background.width, 290.0 * scale);
    assert_eq!(view.slots[0].frame.top, 1_080.0 - 36.0 * scale);
}

#[test]
fn cooldown_mask_grows_upward_inside_the_exact_32_pixel_inset_rect() {
    let mut model = ready_model();
    model.slots[0] = LegacyQuickSlotEntry {
        item_id: 42,
        icon_path: Some(semantic_icon().into()),
        cooldown_remaining_fraction: 0.25,
        ..default()
    };
    let view = quick_slot_ui_view(
        1_264,
        681,
        QuickSlotUiConfig::default(),
        QuickSlotParityPreview::enabled(),
        &model,
        true,
    );
    assert_eq!(
        view.slots[0].cooldown,
        Some(QuickSlotUiRect::new(457.0, 670.0, 32.0, 8.0))
    );

    model.slots[0].cooldown_remaining_fraction = 5.0;
    let full = quick_slot_ui_view(
        1_264,
        681,
        QuickSlotUiConfig::default(),
        QuickSlotParityPreview::enabled(),
        &model,
        true,
    );
    assert_eq!(
        full.slots[0].cooldown,
        Some(QuickSlotUiRect::new(457.0, 646.0, 32.0, 32.0))
    );
}

#[test]
fn item_id_is_not_a_count_and_missing_icon_uses_the_empty_style() {
    let mut model = ready_model();
    model.slots[0].item_id = 500;
    model.slots[0].cooldown_remaining_fraction = 1.0;
    let view = quick_slot_ui_view(
        1_264,
        681,
        QuickSlotUiConfig::default(),
        QuickSlotParityPreview::enabled(),
        &model,
        true,
    );
    assert_eq!(view.slots[0].visual, QuickSlotVisual::Empty);
    assert!(!view.slots[0].icon_visible);
    assert_eq!(view.slots[0].cooldown, None);
    assert_eq!(model.slots[0].item_id, 500);
}

#[test]
fn inventory_empty_disables_pointer_but_keeps_icon_and_cooldown_visible() {
    let mut model = ready_model();
    model.slots[3] = LegacyQuickSlotEntry {
        item_type: 7,
        item_id: 81,
        icon_path: Some(semantic_icon().into()),
        inventory_empty: true,
        cooldown_remaining_fraction: 0.5,
    };
    let view = quick_slot_ui_view(
        1_264,
        681,
        QuickSlotUiConfig::default(),
        QuickSlotParityPreview::enabled(),
        &model,
        true,
    );
    assert_eq!(view.slots[3].visual, QuickSlotVisual::Occupied);
    assert!(view.slots[3].icon_visible);
    assert!(!view.slots[3].pointer_enabled);
    assert!(view.slots[3].cooldown.is_some());
}

#[test]
fn hotkeys_preserve_exclusive_priority_even_when_first_slot_is_empty() {
    let mut model = ready_model();
    model.slots[1].item_id = 77;
    let mut input = QuickSlotInputFrame::default();
    input.slot_just_pressed[0] = true;
    input.slot_just_pressed[1] = true;
    assert_eq!(
        model.hotkey_action(QuickSlotUiConfig::default(), input),
        None
    );

    input.slot_just_pressed[0] = false;
    assert_eq!(
        model.hotkey_action(QuickSlotUiConfig::default(), input),
        Some(QuickSlotUiAction {
            logical_slot: 1,
            legacy_event_index: 1,
            source: QuickSlotActivationSource::Hotkey,
        })
    );
}

#[test]
fn left_shift_macro_chat_modifier_suppresses_the_entire_hotkey_chain() {
    let mut model = ready_model();
    model.slots[0].item_id = 10;
    let input = QuickSlotInputFrame {
        macro_chat_modifier_held: true,
        slot_just_pressed: [true, false, false, false, false, false, false, false],
    };
    assert_eq!(
        model.hotkey_action(QuickSlotUiConfig::default(), input),
        None
    );
    assert!(
        model
            .hotkey_action(
                QuickSlotUiConfig {
                    macro_chat_mode: LegacyMacroChatMode::None,
                    ..default()
                },
                input
            )
            .is_some()
    );
}

#[test]
fn bevy_key_mapping_matches_all_eight_legacy_keycodes() {
    let mut keys = ButtonInput::<KeyCode>::default();
    for key in [
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
        KeyCode::Numpad5,
        KeyCode::Numpad8,
        KeyCode::ShiftLeft,
    ] {
        keys.press(key);
    }
    let input = quick_slot_input_frame_from_keys(&keys);
    assert_eq!(input.slot_just_pressed, [true; QUICK_SLOT_COUNT]);
    assert!(input.macro_chat_modifier_held);
}

#[test]
fn pointer_and_hotkey_actions_preserve_the_clean_index_asymmetry() {
    let mut model = ready_model();
    model.slots[0] = LegacyQuickSlotEntry::occupied(100, semantic_icon());
    model.slots[7] = LegacyQuickSlotEntry::occupied(107, semantic_icon());
    assert_eq!(
        model.hotkey_action_for_slot(0).unwrap().legacy_event_index,
        0
    );
    assert_eq!(
        model.pointer_action_for_slot(0).unwrap().legacy_event_index,
        1
    );
    assert_eq!(
        model.hotkey_action_for_slot(7).unwrap().legacy_event_index,
        7
    );
    assert_eq!(
        model.pointer_action_for_slot(7).unwrap().legacy_event_index,
        8
    );
}

#[test]
fn activation_is_a_pure_request_and_does_not_decrement_or_start_cooldown() {
    let mut model = ready_model();
    model.slots[2] = LegacyQuickSlotEntry {
        item_id: 200,
        icon_path: Some(semantic_icon().into()),
        cooldown_remaining_fraction: 0.3,
        ..default()
    };
    let before = model.slots[2].clone();
    assert!(model.hotkey_action_for_slot(2).is_some());
    assert!(model.pointer_action_for_slot(2).is_some());
    assert_eq!(model.slots[2], before);
}

#[test]
fn semantic_contract_rejects_hash_named_source_textures_and_traversal() {
    assert!(QuickSlotUiAssetContract::default().is_complete());
    assert!(is_semantic_png_path(semantic_icon()));
    assert!(
        QuickSlotUiAssetContract::try_from_semantic_paths(
            "ui/en/gameplay/quick-slot/quickslot_main--35d2a07b3d510eaf.png",
            "ui/en/gameplay/quick-slot/occupied.png",
            "ui/en/gameplay/quick-slot/empty.png",
            "ui/en/gameplay/quick-slot/cooldown.png",
        )
        .is_err()
    );
    assert!(
        QuickSlotUiAssetContract::try_from_semantic_paths(
            "ui/en/gameplay/../background.png",
            "ui/en/gameplay/quick-slot/occupied.png",
            "ui/en/gameplay/quick-slot/empty.png",
            "ui/en/gameplay/quick-slot/cooldown.png",
        )
        .is_err()
    );
}

#[test]
fn source_skin_styles_match_inventory_assignments_and_serialized_pointers() {
    for (name, path_id) in [
        ("slotbox", QUICK_SLOT_OCCUPIED_STYLE_PATH_ID),
        ("slotboxempty", QUICK_SLOT_EMPTY_STYLE_PATH_ID),
    ] {
        let style = gui_style("FusionFallInvenSkin", name).unwrap();
        assert_eq!(style.states["normal"].background.path_id, path_id);
        assert_eq!(style.states["hover"].background.path_id, path_id);
        assert_eq!(style.states["active"].background.path_id, path_id);
        assert_eq!(style.border, GuiInsets::default());
        assert_eq!(style.padding, GuiInsets::default());
    }
}

#[test]
fn converted_source_texture_bytes_and_dimensions_remain_exact() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for contract in QUICK_SLOT_SOURCE_TEXTURES {
        let converted = fs::read(root.join(contract.converted_path)).unwrap();
        let semantic = fs::read(root.join(contract.runtime_path)).unwrap();
        assert_eq!(semantic, converted);
        assert_eq!(format!("{:X}", Sha256::digest(&semantic)), contract.sha256);
        let image = image::load_from_memory(&semantic).unwrap();
        assert_eq!(
            (image.width(), image.height()),
            (contract.width, contract.height)
        );
    }
}

#[test]
fn plugin_owns_no_camera_and_missing_asset_files_fail_closed() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .add_plugins(QuickSlotUiPlugin);
    app.world_mut().spawn((
        Window {
            resolution: WindowResolution::new(1_264, 681),
            ..default()
        },
        PrimaryWindow,
    ));
    *app.world_mut().resource_mut::<QuickSlotUiModel>() = ready_model();
    *app.world_mut().resource_mut::<QuickSlotParityPreview>() =
        QuickSlotParityPreview::enabled();
    app.update();

    let world = app.world_mut();
    let mut roots = world.query_filtered::<(&Node, &Visibility), With<QuickSlotUiRoot>>();
    let (node, visibility) = roots.single(world).unwrap();
    assert_eq!(*visibility, Visibility::Hidden);
    assert_eq!((node.width, node.height), (px(1_264), px(681)));
    let mut elements = world.query::<&QuickSlotUiElement>();
    assert_eq!(elements.iter(world).count(), 1 + QUICK_SLOT_COUNT * 3);
    let mut cameras = world.query::<&Camera>();
    assert_eq!(cameras.iter(world).count(), 0);
}
