//! GPU acceptance frame for the central native player and four animated
//! character-selection portraits.
//!
//! Success requires every occupied slot to reach the fail-closed
//! `ReadyAnimated` state before any portrait camera is allowed to render.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    character_creation_data::CharacterCreationData,
    character_creation_ui::CharacterAppearance,
    character_selection_portraits::{
        CharacterSelectionPortraitStatus, CharacterSelectionPortraitsModel,
        NativeCharacterSelectionPortraitsPlugin,
    },
    character_selection_ui::{
        CharacterLocationBackground, CharacterPreviewStatus, CharacterSelectionCapability,
        CharacterSelectionUiModel, CharacterSlotUi, NativeCharacterSelectionUiPlugin,
        OccupiedCharacterSlotUi,
    },
    legacy_model_material::LegacyModelMaterialPlugin,
    localization::Localization,
    network::CharacterSummary,
    player_preview::{
        NativePlayerLook, NativePlayerPreviewModel, NativePlayerPreviewPlugin,
        NativePlayerPreviewStage, NativePlayerPreviewStatus, prewarm_native_player_look,
    },
    player_shared_rig::{
        NativePlayerRigAssetCache, NativePlayerRigCatalog, NativePlayerSharedRigPlugin,
    },
};
use ffone_protocol::{
    CHARACTER_EQUIP_SLOT_COUNT_0104, CharacterEquipSlot0104, CharacterStyle0104, EquippedItem0104,
};

const READY_WARMUP_FRAMES: u32 = 90;
const TIMEOUT_FRAMES: u32 = 1_200;

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
    looks: [NativePlayerLook; 4],
}

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    capture_issued: bool,
    capture_saved: bool,
}

fn main() {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/ffone-character-selection-portraits.png"));
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("output must end in .png");
        std::process::exit(2);
    }
    let asset_root = env::var_os("FFONE_PREVIEW_ASSET_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join("assets/game")
        })
        .canonicalize()
        .expect("preview asset root must exist");
    let data =
        CharacterCreationData::open(&asset_root).expect("open native character-creation data");
    let catalog =
        NativePlayerRigCatalog::open(&asset_root).expect("open native shared player-rig catalog");
    let (localization, language) =
        Localization::open(&asset_root, "en").expect("open native localization");
    let mut looks = build_four_exact_looks(&data);
    looks[0] = build_slot_one_regression_look(&data);
    if output.exists() {
        fs::remove_file(&output)
            .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
    }

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewConfig { output, looks })
        .insert_resource(PreviewState::default())
        .insert_resource(catalog)
        .insert_resource(localization)
        .insert_resource(language)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne selection portrait acceptance".into(),
                        resolution: WindowResolution::new(1264, 681),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            NativeCharacterSelectionUiPlugin,
            NativePlayerPreviewPlugin,
            NativePlayerSharedRigPlugin,
            NativeCharacterSelectionPortraitsPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, drive_capture)
        .run();
}

fn build_slot_one_regression_look(data: &CharacterCreationData) -> NativePlayerLook {
    // Account-style combination that exposed the prewarm-first sampler race:
    // the exact values are intentionally retained as a deterministic GPU
    // regression without depending on a live OpenFusion database.
    let mut equipment = [EquippedItem0104::default(); CHARACTER_EQUIP_SLOT_COUNT_0104];
    for (slot, item_type, item_id) in [
        (CharacterEquipSlot0104::UpperBody, 1, 100),
        (CharacterEquipSlot0104::LowerBody, 2, 357),
        (CharacterEquipSlot0104::Foot, 3, 400),
        (CharacterEquipSlot0104::Head, 4, 46),
        // Dark Cloak exercises the Back equipType=1 shared-skin branch that
        // previously floated above the actor when treated as a rigid socket.
        (CharacterEquipSlot0104::Back, 6, 8),
        // The pistol proves selection portraits consume the same weapon
        // animation profile as the central preview and gameplay rigs.
        (CharacterEquipSlot0104::Hand, 0, 197),
    ] {
        equipment[slot as usize] = EquippedItem0104 {
            item_type,
            item_id,
            option: 1,
            time_limit: 0,
        };
    }
    data.resolve_character_summary(&CharacterSummary {
        slot: 1,
        level: 1,
        pc_uid: 1,
        first_name: "Slot".to_owned(),
        last_name: "One".to_owned(),
        position: [0; 3],
        style: CharacterStyle0104 {
            name_check: 1,
            gender: 1,
            face_style: 4,
            hair_style: 4,
            hair_color: 14,
            skin_color: 9,
            eye_color: 2,
            height: 2,
            body: 0,
            class: 0,
            appearance_flag: 1,
            tutorial_flag: 0,
            payzone_flag: 0,
        },
        equipment,
    })
    .expect("resolve slot-one regression look")
}

fn build_four_exact_looks(data: &CharacterCreationData) -> [NativePlayerLook; 4] {
    let mut appearances = std::array::from_fn::<_, 4, _>(|_| CharacterAppearance::default());
    appearances[1].skin_color = 6;
    appearances[1].hair_color = 8;
    appearances[2].skin_color = 10;
    appearances[2].hair_color = 15;
    appearances[3].skin_color = 3;
    appearances[3].hair_color = 12;

    let names = [
        ("Test", "Ser"),
        ("Gaia", "Roundbreath"),
        ("Dex", "Steel"),
        ("Zora", "Moon"),
    ];
    std::array::from_fn(|slot| {
        data.resolve_creator(
            (slot + 1) as i64,
            1,
            names[slot].0,
            names[slot].1,
            &appearances[slot],
        )
        .unwrap_or_else(|error| panic!("resolve exact slot {} look: {error}", slot + 1))
        .look
    })
}

fn setup(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    assets: Res<AssetServer>,
    catalog: Res<NativePlayerRigCatalog>,
    mut asset_cache: ResMut<NativePlayerRigAssetCache>,
    mut selection: ResMut<CharacterSelectionUiModel>,
    mut player_preview: ResMut<NativePlayerPreviewModel>,
    mut portraits: ResMut<CharacterSelectionPortraitsModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    selection.visible = true;
    selection.music_enabled = false;
    selection.create = CharacterSelectionCapability::Enabled;
    selection.delete = CharacterSelectionCapability::Enabled;
    selection.preview = CharacterPreviewStatus::PlayerAssemblyPending;
    selection.slots = [
        occupied(1, "Test Ser", 1),
        occupied(2, "Gaia Roundbreath", 12),
        occupied(3, "Dex Steel", 24),
        occupied(4, "Zora Moon", 36),
    ];
    selection.selected_slot = Some(0);

    player_preview
        .set_look(config.looks[0].clone())
        .expect("set central selection preview look");
    player_preview.visible = true;
    player_preview.stage = NativePlayerPreviewStage::Selection;

    portraits.visible = true;
    for (slot, look) in config.looks.iter().cloned().enumerate() {
        prewarm_native_player_look(&assets, &mut asset_cache, &catalog, &look)
            .unwrap_or_else(|error| panic!("prewarm slot {} portrait: {error}", slot + 1));
        portraits
            .set_look(slot, look)
            .unwrap_or_else(|error| panic!("set slot {} portrait: {error}", slot + 1));
    }
}

fn occupied(pc_uid: i64, name: &str, level: i16) -> CharacterSlotUi {
    CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
        pc_uid,
        display_name: name.to_owned(),
        level,
        district: "TECH SQUARE".to_owned(),
        zone: "THE FUTURE".to_owned(),
        background: CharacterLocationBackground::Future,
    })
}

fn drive_capture(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    player_preview: Res<NativePlayerPreviewModel>,
    portraits: Res<CharacterSelectionPortraitsModel>,
    mut state: ResMut<PreviewState>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let NativePlayerPreviewStatus::Blocked(error) = &player_preview.status {
        eprintln!("central selection preview blocked: {error}");
        exit.write(AppExit::error());
        return;
    }
    for (slot, portrait) in portraits.slots.iter().enumerate() {
        if let CharacterSelectionPortraitStatus::Blocked(error) = &portrait.status {
            eprintln!("selection portrait slot {} blocked: {error}", slot + 1);
            exit.write(AppExit::error());
            return;
        }
    }
    let all_ready = matches!(
        player_preview.status,
        NativePlayerPreviewStatus::ReadyAnimated { .. }
    ) && portraits.slots.iter().all(|portrait| {
        matches!(
            portrait.status,
            CharacterSelectionPortraitStatus::ReadyAnimated { .. }
        )
    });
    if all_ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
    }
    if !state.capture_issued
        && state
            .ready_frame
            .is_some_and(|ready| state.frames.saturating_sub(ready) >= READY_WARMUP_FRAMES)
    {
        state.capture_issued = true;
        println!(
            "central selection preview and all four portraits reached ReadyAnimated: central={:?}, portraits={:?}",
            player_preview.status,
            portraits
                .slots
                .iter()
                .map(|slot| &slot.status)
                .collect::<Vec<_>>()
        );
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!(
            "selection portrait GPU proof timed out: {:?}",
            portraits
                .slots
                .iter()
                .map(|slot| &slot.status)
                .collect::<Vec<_>>()
        );
        exit.write(AppExit::error());
    }
    // Real wall time is required so the real stand1 clip advances under CI.
    std::thread::sleep(Duration::from_millis(16));
    let _ = &config.output;
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    assert_eq!(image.width(), 1264);
    assert_eq!(image.height(), 681);
    if let Some(parent) = config
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&config.output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", config.output.display()));
    println!("saved {}", config.output.display());
    state.capture_saved = true;
}
