//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `UserEquip` Item/Nano mode.
//!
//! This harness supplies an already-authoritative 9+50 projection, populated
//! player/status values, the shared native player rig in the source-sized
//! inventory render target, and a final one-second sine-slide state. Packet
//! writes remain outside this capture boundary.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    gltf::Gltf,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    assets::AssetLocator,
    character_creation_data::CharacterCreationData,
    character_creation_ui::CharacterAppearance,
    gameplay_nano_portraits::{GameplayNanoPortraitCatalog, GameplayNanoPortraitPlugin},
    gameplay_ui::{
        GAMEPLAY_UI_CAMERA_ORDER, GameplayNanoPortraitImages, GameplayUiModel, NanoSlotUi,
    },
    inventory_runtime::InventoryRuntime0104,
    legacy_model_material::LegacyModelMaterialPlugin,
    localization::{Localization, LocalizationPlugin},
    player_preview::{
        NativePlayerInventoryPreviewImage, NativePlayerPreviewModel, NativePlayerPreviewPlugin,
        NativePlayerPreviewStage, NativePlayerPreviewStatus,
    },
    player_shared_rig::{NativePlayerRigCatalog, NativePlayerSharedRigPlugin},
    tutorial_mission_content::TutorialMissionContent,
    user_equip_ui::{
        USER_EQUIP_DEFAULT_ASSET_PATHS, USER_EQUIP_GUIDE_DEXTER_PATH, USER_EQUIP_OPEN_SECONDS,
        UserEquipCatalogKind, UserEquipCatalogQuery, UserEquipIconRef, UserEquipItemCatalog,
        UserEquipItemModeProjection, UserEquipItemPopupState, UserEquipModalState,
        UserEquipNanoEquippedAuthority, UserEquipNanoModeProjection, UserEquipNanoViewerState,
        UserEquipPresentationContext, UserEquipPresentationIcon, UserEquipSlotEndpoint,
        UserEquipUiElement, UserEquipUiPlugin, UserEquipUiRoot, UserEquipUiSet, UserEquipUiState,
        user_equip_nano_scroll_max,
    },
};
use ffone_protocol::{ItemBase0104, Nano0104, PcLoadData0104};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const DEFAULT_OUTPUT: &str = "target/ui-parity/user-equip-item-1264x681.png";
const DEFAULT_NANO_OUTPUT: &str = "target/ui-parity/user-equip-nano-1264x681.png";
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 100_000;
const MIN_MISSING_CHECKER_PIXELS: usize = 600;

const ICON_GENERAL_00: &str = "icons/items/general/generalitemicon_00.png";
const ICON_GENERAL_01: &str = "icons/items/general/generalitemicon_01.png";
const ICON_GUMBALL_ADAPTIUM: &str = "icons/items/general/generalitemicon_13.png";
const ICON_DAMAGE_POWER: &str = "icons/items/general/generalitemicon_16.png";
const ICON_LIGHTNING_GUN: &str = "icons/items/weapons/wpnicon_385.png";
const ICON_WEAPON_01: &str = "icons/items/weapons/wpnicon_01.png";
const ICON_WEAPON_02: &str = "icons/items/weapons/wpnicon_02.png";
const ICON_COSMETIC_00: &str = "icons/items/cosmetics/cosicon_00.png";
const ICON_COSMETIC_01: &str = "icons/items/cosmetics/cosicon_01.png";
const ICON_COSMETIC_02: &str = "icons/items/cosmetics/cosicon_02.png";
const ICON_COSMETIC_03: &str = "icons/items/cosmetics/cosicon_03.png";
const ICON_COSMETIC_04: &str = "icons/items/cosmetics/cosicon_04.png";
const ICON_COSMETIC_05: &str = "icons/items/cosmetics/cosicon_05.png";
const ICON_VEHICLE_00: &str = "icons/items/vehicles/vehicle_00.png";
const PREVIEW_ICON_PATHS: [&str; 14] = [
    ICON_GENERAL_00,
    ICON_GENERAL_01,
    ICON_GUMBALL_ADAPTIUM,
    ICON_DAMAGE_POWER,
    ICON_LIGHTNING_GUN,
    ICON_WEAPON_01,
    ICON_WEAPON_02,
    ICON_COSMETIC_00,
    ICON_COSMETIC_01,
    ICON_COSMETIC_02,
    ICON_COSMETIC_03,
    ICON_COSMETIC_04,
    ICON_COSMETIC_05,
    ICON_VEHICLE_00,
];

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewNanoMode(bool);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    nano_models: Vec<Handle<Gltf>>,
    font: Handle<Font>,
}

#[derive(Resource)]
struct PreviewCharacterData(CharacterCreationData);

#[derive(Resource)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    ready_at: Option<Instant>,
    capture_issued: bool,
    capture_saved: bool,
    capture_failed: bool,
    started_at: Instant,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            frames: 0,
            ready_frame: None,
            ready_at: None,
            capture_issued: false,
            capture_saved: false,
            capture_failed: false,
            started_at: Instant::now(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    output: PathBuf,
    nano: bool,
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut arguments = arguments.into_iter().peekable();
    let nano = arguments
        .peek()
        .is_some_and(|argument| argument.to_str() == Some("--nano"));
    if nano {
        arguments.next();
    }
    let output = arguments.next().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if nano {
            DEFAULT_NANO_OUTPUT
        } else {
            DEFAULT_OUTPUT
        })
    });
    if arguments.next().is_some()
        || output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return Err("usage: user_equip_ui_gpu_preview [--nano] [OUTPUT.png]");
    }
    Ok(PreviewCli { output, nano })
}

fn main() {
    let cli = match parse_cli(std::env::args_os().skip(1)) {
        Ok(cli) => cli,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let locale = env::var("FFONE_USER_EQUIP_LOCALE").unwrap_or_else(|_| "en".to_owned());
    let (localization, language) =
        Localization::open(&asset_root, &locale).expect("open production localization bundles");
    let character_data =
        CharacterCreationData::open(&asset_root).expect("open native character data");
    let rig_catalog =
        NativePlayerRigCatalog::open(&asset_root).expect("open native shared player-rig catalog");
    let assets = AssetLocator::open(&asset_root).expect("open preview asset catalog");
    let nano_catalog = GameplayNanoPortraitCatalog::open(&assets)
        .expect("open source-backed Nano portrait catalog");
    let mut gameplay = GameplayUiModel {
        visible: true,
        ..default()
    };
    for (slot, (nano_id, stamina_fraction, active)) in
        [(1_i16, 0.5, true), (5, 110.0 / 150.0, false)]
            .into_iter()
            .enumerate()
    {
        gameplay.nanos[slot] = NanoSlotUi {
            nano_id: Some(nano_id),
            model_path: nano_catalog.model_path(nano_id).map(str::to_owned),
            stamina_fraction,
            active,
            ..default()
        };
    }

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(PreviewNanoMode(cli.nano))
        .insert_resource(PreviewState::default())
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(PreviewCharacterData(character_data))
        .insert_resource(rig_catalog)
        .insert_resource(gameplay)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution UserEquip Item acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            NativePlayerPreviewPlugin,
            NativePlayerSharedRigPlugin,
            GameplayNanoPortraitPlugin,
            UserEquipUiPlugin,
            ffone_client::shared_input_ui::SharedInputUiPlugin,
            LocalizationPlugin,
        ))
        .add_systems(Startup, setup_preview)
        .add_systems(
            PreUpdate,
            exercise_card_action
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(Update, drive_capture.after(UserEquipUiSet::Bind))
        .add_systems(
            PreUpdate,
            exercise_redeem_label
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .add_systems(
            PreUpdate,
            exercise_nano_station_label
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .run();
}

fn exercise_nano_station_label(
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    mut windows: Query<&mut Window>,
    labels: Query<(
        &ffone_client::localization::LocalizedText,
        &UiGlobalTransform,
        &ComputedNode,
    )>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut outbox: ResMut<ffone_client::user_equip_ui::UserEquipUiOutbox>,
    projection: Res<UserEquipNanoModeProjection>,
) {
    if env::var_os("FFONE_NANO_STATION_CLICK").is_none() || preview.ready_frame.is_none() {
        return;
    }
    *frame += 1;
    if *frame <= 10 {
        let mut targets: Vec<_> = labels
            .iter()
            .filter(|(t, _, n)| t.key == "ui.nano_station.slot" && n.size().min_element() > 0.0)
            .collect();
        targets.sort_by(|a, b| a.1.translation.x.total_cmp(&b.1.translation.x));
        assert_eq!(targets.len(), 3);
        let mut window = windows.single_mut().unwrap();
        window.focused = true;
        window.set_physical_cursor_position(Some(targets[1].1.translation.as_dvec2()));
    }
    if *frame == 10 {
        mouse.press(MouseButton::Left);
    }
    if *frame == 15 {
        mouse.release(MouseButton::Left);
    }
    if *frame == 20 {
        use ffone_client::user_equip_ui::{UserEquipNanoStationAction, UserEquipUiAction};
        let nano_id: i16 = env::var("FFONE_NANO_STATION_ID").unwrap().parse().unwrap();
        assert_eq!(
            outbox.drain().collect::<Vec<_>>(),
            vec![UserEquipUiAction::NanoStation(
                UserEquipNanoStationAction::Equip { nano_id, slot: 1 }
            )]
        );
        assert_eq!(
            projection.status[1].nano_id,
            Some(5),
            "slot changes only after server acknowledgement"
        );
        println!(
            "PASS Nano {nano_id}: actual station label click replaces occupied slot 1 with exact native ID"
        );
    }
}

fn exercise_redeem_label(
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    mut windows: Query<&mut Window>,
    labels: Query<(
        &ffone_client::localization::LocalizedText,
        &UiGlobalTransform,
        &ComputedNode,
    )>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    redeem: Res<ffone_client::shared_input_ui::SharedRedeemCode>,
    modal: Res<UserEquipModalState>,
) {
    if env::var_os("FFONE_USER_EQUIP_REDEEM").is_none() || preview.ready_frame.is_none() {
        return;
    }
    *frame += 1;
    if *frame <= 10 {
        let (_, transform, _) = labels
            .iter()
            .find(|(text, _, node)| {
                text.key == "ui.inventory.redeem_code" && node.size().min_element() > 0.0
            })
            .expect("visible inventory redeem label");
        let mut window = windows.single_mut().unwrap();
        window.focused = true;
        window.set_physical_cursor_position(Some(transform.translation.as_dvec2()));
    }
    if *frame == 10 {
        mouse.press(MouseButton::Left);
    }
    if *frame == 15 {
        mouse.release(MouseButton::Left);
    }
    if *frame == 25 {
        assert_eq!(
            redeem.source,
            Some(ffone_client::shared_input_ui::RedeemSource::Inventory { pc: 4242 })
        );
        assert!(modal.redeem_code_view);
        println!("PASS inventory redeem label pointer");
    }
}

struct PreviewCatalog;

impl UserEquipItemCatalog for PreviewCatalog {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        let path = match (query.kind, query.base_item_id) {
            (UserEquipCatalogKind::Equipment { .. }, 328) => ICON_LIGHTNING_GUN,
            (UserEquipCatalogKind::Equipment { .. }, 100) => ICON_WEAPON_01,
            (UserEquipCatalogKind::Equipment { .. }, 108) => ICON_WEAPON_02,
            (UserEquipCatalogKind::Equipment { .. }, 101) => ICON_COSMETIC_00,
            (UserEquipCatalogKind::Equipment { .. }, 102) => ICON_COSMETIC_01,
            (UserEquipCatalogKind::Equipment { .. }, 103) => ICON_COSMETIC_02,
            (UserEquipCatalogKind::Equipment { .. }, 104) => ICON_COSMETIC_03,
            (UserEquipCatalogKind::Equipment { .. }, 105) => ICON_COSMETIC_04,
            (UserEquipCatalogKind::Equipment { .. }, 106) => ICON_COSMETIC_05,
            (UserEquipCatalogKind::Equipment { .. }, 107) => ICON_VEHICLE_00,
            (UserEquipCatalogKind::General, 7) => ICON_DAMAGE_POWER,
            (UserEquipCatalogKind::General, 119) => ICON_GUMBALL_ADAPTIUM,
            (UserEquipCatalogKind::General, 201) => ICON_GENERAL_01,
            (UserEquipCatalogKind::Chest, 77) => ICON_GENERAL_01,
            _ => return None,
        };
        UserEquipIconRef::new(path).ok()
    }
}

fn preview_projection() -> UserEquipItemModeProjection {
    let mut load = PcLoadData0104::zeroed();
    let equipment = [
        (0, item(0, 100, 0, 0)),
        (1, item(1, 101, 0, 0)),
        (3, item(3, 103, 0, 0)),
        (4, item(4, 104, 0x0012_0000, 0)),
        (5, item(5, 105, 0, 0)),
        (6, item(6, 106, 0, 0)),
        (7, item(0, 108, 0, 0)),
        (8, item(10, 107, 0, 0)),
    ];
    let inventory = [
        // The first three rows intentionally reproduce the user's primary
        // Lightning Gun, Damage Power Item and 16Lv SPECIAL crate references.
        (0, item(0, 328, 0x0011_0002, 0)),
        (1, item(7, 7, 1, 0)),
        (2, item(9, 77, 0, 0)),
        (4, item(1, 101, 0, 0)),
        (5, item(2, 102, 0, 0)),
        (6, item(3, 103, 0, 0)),
        (7, item(4, 104, 0, 0)),
        (8, item(5, 105, 0, 0)),
        (9, item(6, 106, 0, 0)),
        (10, item(10, 107, 0, 0)),
        (11, item(7, 119, 3, 0)),
        (12, item(8, 401, 0, 0)),
    ];
    for (slot, value) in equipment {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::EQUIPMENT_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    for (slot, value) in inventory {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + slot * ItemBase0104::SIZE,
            value,
        );
    }
    if let Ok(mode) = env::var("FFONE_USER_EQUIP_CARD") {
        let value = match mode.as_str() {
            "rental" => item(10, 1, 0, 1800000000),
            "expires" => item(1, 100, 0, 1800000000),
            "combined" => item(1, 100, 107 << 16, 0),
            _ => item(1, 92, 0, 0),
        };
        write_item(load.as_bytes_mut(), PcLoadData0104::INVENTORY_OFFSET, value);
    }
    let runtime = InventoryRuntime0104::from_pc_load(4_242, &load);
    UserEquipItemModeProjection::from_authoritative(&runtime, &PreviewCatalog)
}

fn preview_presentation_context() -> UserEquipPresentationContext {
    UserEquipPresentationContext {
        player_name: "Dexter".into(),
        level: 36,
        gender: 1,
        guide: 2,
        hp: 1_782,
        max_hp: 2_000,
        fusion_matter: 8_750,
        max_fusion_matter: 10_000,
        taros: 125_430,
        weapon_battery: 64,
        nano_battery: 29,
        guide_name: "Dexter".into(),
        guide_name_key: Some("ui.guide.mentor.dexter".into()),
        guide_icon_path: Some(USER_EQUIP_GUIDE_DEXTER_PATH.into()),
    }
}

const fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

fn write_item(bytes: &mut [u8], offset: usize, value: ItemBase0104) {
    bytes[offset..offset + 2].copy_from_slice(&value.item_type.to_le_bytes());
    bytes[offset + 2..offset + 4].copy_from_slice(&value.item_id.to_le_bytes());
    bytes[offset + 4..offset + 8].copy_from_slice(&value.option.to_le_bytes());
    bytes[offset + 8..offset + 12].copy_from_slice(&value.time_limit.to_le_bytes());
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<UserEquipUiState>,
    mut modal: ResMut<UserEquipModalState>,
    mut item_popup: ResMut<UserEquipItemPopupState>,
    mut nano_viewer: ResMut<UserEquipNanoViewerState>,
    mut presentation: ResMut<UserEquipPresentationContext>,
    character_data: Res<PreviewCharacterData>,
    mut player_preview: ResMut<NativePlayerPreviewModel>,
    gameplay: Res<GameplayUiModel>,
    nano_mode: Res<PreviewNanoMode>,
) {
    commands.spawn((
        Camera2d,
        Camera {
            order: GAMEPLAY_UI_CAMERA_ORDER,
            ..default()
        },
        IsDefaultUiCamera,
    ));
    let content_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let content_assets = AssetLocator::open(&content_root).expect("open preview content catalog");
    commands.insert_resource(
        TutorialMissionContent::from_project_assets(&content_assets)
            .expect("open preview item and Nano content"),
    );
    commands.insert_resource(preview_projection());
    state.open_item_mode();
    if env::var_os("FFONE_NANO_STATION").is_some() {
        state.open_nano_station(1);
    }
    state.tick(USER_EQUIP_OPEN_SECONDS);
    let nano_projection = preview_nano_projection();
    if nano_mode.0 {
        state.select_nano_tab();
        if env::var_os("FFONE_USER_EQUIP_SCROLL_BOTTOM").is_some() {
            state.set_scroll_y(user_equip_nano_scroll_max());
        }
        if env::var_os("FFONE_USER_EQUIP_NANO_VIEWER").is_some() {
            let visual_index = env::var("FFONE_USER_EQUIP_NANO_VIEWER_INDEX")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            nano_viewer.open(visual_index);
        }
    }
    *modal = UserEquipModalState::default();
    if !nano_mode.0 && env::var_os("FFONE_USER_EQUIP_ITEM_POPUP").is_some() {
        let slot_index = env::var("FFONE_USER_EQUIP_POPUP_SLOT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        item_popup.open(UserEquipSlotEndpoint::Inventory { slot_index });
        modal.item_popup_active = true;
    }
    *presentation = preview_presentation_context();
    let resolved = character_data
        .0
        .resolve_creator(4_242, 0, "GPU", "Fixture", &CharacterAppearance::default())
        .expect("resolve source-backed inventory preview look");
    player_preview
        .set_look(resolved.look)
        .expect("valid inventory preview look");
    player_preview.stage = NativePlayerPreviewStage::Inventory;
    player_preview.visible = true;

    let (font_path, image_paths) = USER_EQUIP_DEFAULT_ASSET_PATHS
        .split_last()
        .expect("UserEquip asset contract must end with its font");

    let dynamic_nano_paths = nano_projection
        .gallery
        .iter()
        .map(|entry| &entry.icon)
        .chain(
            nano_projection
                .status
                .iter()
                .flat_map(|slot| [&slot.nano_icon, &slot.skill_icon]),
        )
        .filter_map(|icon| match icon {
            UserEquipPresentationIcon::Resolved(path) => Some(path.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut preview_images = image_paths
        .iter()
        .copied()
        .chain(PREVIEW_ICON_PATHS)
        .chain([USER_EQUIP_GUIDE_DEXTER_PATH])
        .map(|path| asset_server.load(path))
        .collect::<Vec<_>>();
    preview_images.extend(
        dynamic_nano_paths
            .into_iter()
            .map(|path| asset_server.load(path)),
    );
    commands.insert_resource(PreviewAssets {
        images: preview_images,
        nano_models: gameplay
            .nanos
            .iter()
            .filter_map(|nano| nano.model_path.as_ref())
            .map(|path| asset_server.load(path.clone()))
            .collect(),
        font: asset_server.load(*font_path),
    });
    commands.insert_resource(nano_projection);
}

fn preview_nano_projection() -> UserEquipNanoModeProjection {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    let assets = AssetLocator::open(&root).expect("open preview asset catalog");
    let content = TutorialMissionContent::from_project_assets(&assets)
        .expect("open source-backed Nano preview content");
    let empty = Nano0104 {
        id: 0,
        skill_id: 0,
        stamina: 0,
    };
    let mut bank = [empty; PcLoadData0104::NANO_BANK_COUNT];
    // Bank position is not a Nano ID. Keep the fixture deliberately shuffled
    // so the preview catches index/identity regressions.
    bank[4] = Nano0104 {
        id: 1,
        skill_id: 1,
        stamina: 75,
    };
    bank[9] = Nano0104 {
        id: 5,
        skill_id: 8,
        stamina: 110,
    };
    if env::var_os("FFONE_NANO_STATION").is_some() {
        let id = env::var("FFONE_NANO_STATION_ID")
            .ok()
            .and_then(|value| value.parse::<i16>().ok())
            .unwrap_or(2);
        bank[10] = Nano0104 {
            id,
            skill_id: content
                .journal_nano(i32::from(id))
                .expect("Nano journal")
                .skills[0]
                .skill_id as i16,
            stamina: 150,
        };
    }
    UserEquipNanoModeProjection::from_authoritative(
        &bank,
        [
            UserEquipNanoEquippedAuthority {
                nano_id: Some(1),
                skill_id: 1,
                stamina: 75,
                active: true,
            },
            UserEquipNanoEquippedAuthority {
                nano_id: Some(5),
                skill_id: 8,
                stamina: 110,
                active: false,
            },
            UserEquipNanoEquippedAuthority::default(),
        ],
        &content,
    )
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    output: Res<PreviewOutput>,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    nano_portraits: Res<GameplayNanoPortraitImages>,
    inventory_preview: Res<NativePlayerInventoryPreviewImage>,
    player_preview: Res<NativePlayerPreviewModel>,
    nano_mode: Res<PreviewNanoMode>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&ComputedNode, &Visibility), With<UserEquipUiRoot>>,
    elements: Query<(&UserEquipUiElement, &ComputedNode)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Failed(_)
        )
        || preview_assets
            .nano_models
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("UserEquip acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Loaded
        )
        && preview_assets
            .nano_models
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && (!nano_mode.0 || nano_portraits.0[..2].iter().all(Option::is_some));
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.font).is_some();
    let root_exact = roots.iter().any(|(node, visibility)| {
        *visibility == Visibility::Visible
            && node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let viewport_exact = elements.iter().any(|(element, node)| {
        *element
            == (if nano_mode.0 {
                UserEquipUiElement::NanoViewport
            } else {
                UserEquipUiElement::InventoryViewport
            })
            && node.size() == Vec2::new(366.0, 500.0)
    });
    let panel_geometry_exact = elements.iter().any(|(element, node)| {
        *element == UserEquipUiElement::PcStuffPanel && node.size() == Vec2::new(380.0, 632.0)
    }) && elements.iter().any(|(element, node)| {
        *element == UserEquipUiElement::EquipmentPanel && node.size() == Vec2::new(66.0, 639.0)
    });
    let status_geometry_exact = elements.iter().any(|(element, node)| {
        *element == UserEquipUiElement::StatusPanel && node.size() == Vec2::new(498.0, 115.0)
    });
    let avatar_geometry_exact = elements.iter().any(|(element, node)| {
        *element == UserEquipUiElement::AvatarPreview && node.size() == Vec2::new(500.0, 564.0)
    }) && elements.iter().any(|(element, node)| {
        *element == UserEquipUiElement::TurnLeftPositioned && node.size() == Vec2::new(43.0, 78.0)
    }) && elements.iter().any(|(element, node)| {
        *element == UserEquipUiElement::TurnRightPositioned && node.size() == Vec2::new(43.0, 78.0)
    });
    let counters_geometry_exact = [
        (UserEquipUiElement::BoostSlot, Vec2::new(64.0, 30.0)),
        (UserEquipUiElement::PotionSlot, Vec2::new(64.0, 30.0)),
        (UserEquipUiElement::TarosBack, Vec2::new(149.0, 32.0)),
    ]
    .into_iter()
    .all(|(expected_element, expected_size)| {
        elements
            .iter()
            .any(|(element, node)| *element == expected_element && node.size() == expected_size)
    });
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && viewport_exact
        && panel_geometry_exact
        && status_geometry_exact
        && counters_geometry_exact
        && avatar_geometry_exact
        && images.get(&inventory_preview.0).is_some()
        && matches!(
            player_preview.status,
            NativePlayerPreviewStatus::ReadyAnimated { .. }
        );
    if ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }

    let warmed = state
        .ready_frame
        .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES_AFTER_LOAD)
        && state
            .ready_at
            .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if ready && warmed && !state.capture_issued {
        state.capture_issued = true;
        if nano_mode.0 {
            for (slot, portrait) in nano_portraits.0.iter().enumerate() {
                let Some(portrait) = portrait else {
                    continue;
                };
                let stem = output
                    .0
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("user-equip-nano");
                let portrait_output = output
                    .0
                    .with_file_name(format!("{stem}-portrait-{}.png", slot + 1));
                commands
                    .spawn(Screenshot::image(portrait.clone()))
                    .observe(save_to_disk(portrait_output));
            }
        }
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!("UserEquip acceptance capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
    nano_mode: Res<PreviewNanoMode>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    let rgba = image.to_rgba8();
    if rgba.dimensions() != (CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT) {
        eprintln!(
            "rejecting UserEquip capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let mut visible_pixels = 0;
    let mut checker_pixels = 0;
    for pixel in rgba.pixels() {
        let [red, green, blue, _alpha] = pixel.0;
        if u16::from(red) + u16::from(green) + u16::from(blue) > 35 {
            visible_pixels += 1;
        }
        if red > 180 && blue > 180 && green < 80 {
            checker_pixels += 1;
        }
    }
    let minimum_checker_pixels = if nano_mode.0
        || env::var_os("FFONE_USER_EQUIP_ITEM_POPUP").is_some()
        || env::var_os("FFONE_USER_EQUIP_REDEEM").is_some()
    {
        0
    } else {
        MIN_MISSING_CHECKER_PIXELS
    };
    if visible_pixels < MIN_VISIBLE_PIXELS || checker_pixels < minimum_checker_pixels {
        eprintln!(
            "rejecting incomplete UserEquip capture: visible={visible_pixels} \
             (min {MIN_VISIBLE_PIXELS}), checker={checker_pixels} \
             (min {minimum_checker_pixels})"
        );
        state.capture_failed = true;
        return;
    }

    if let Some(parent) = output
        .0
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&output.0)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.0.display()));
    println!("{}", absolute_display(&output.0));
    state.capture_saved = true;
}

fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

#[cfg(test)]
mod tests;

fn exercise_card_action(
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    labels: Query<(
        &ffone_client::localization::LocalizedText,
        &UiGlobalTransform,
        &ComputedNode,
    )>,
    mut windows: Query<&mut Window>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    popup: Res<UserEquipItemPopupState>,
    outbox: Res<ffone_client::user_equip_ui::UserEquipUiOutbox>,
) {
    if env::var_os("FFONE_USER_EQUIP_CARD_ACTION").is_none() || preview.ready_frame.is_none() {
        return;
    }
    *frame += 1;
    if *frame == 15 {
        let (_, transform, _) = labels
            .iter()
            .find(|(t, _, n)| t.key == "ui.inventory.action.equip" && n.size().x > 0.)
            .expect("visible EQUIP label");
        let mut window = windows.single_mut().unwrap();
        let cursor = transform.translation / window.scale_factor();
        window.set_cursor_position(Some(cursor));
    }
    if *frame == 16 {
        mouse.press(MouseButton::Left);
    }
    if *frame == 17 {
        mouse.release(MouseButton::Left);
    }
    if *frame == 19 {
        assert!(
            popup.selected().is_none(),
            "EQUIP label must pass focus to its button"
        );
        assert_eq!(
            outbox.len(),
            1,
            "exactly one typed action; no local inventory mutation"
        );
        println!("Inventory real-pointer EQUIP label passed");
    }
}
