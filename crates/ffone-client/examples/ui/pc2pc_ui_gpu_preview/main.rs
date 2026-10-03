//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `eGameMode.Pc2pc`.
//!
//! The fixture uses correlated authoritative register/cash outcomes. It keeps
//! the portrait and chat backends unavailable so both production boundaries
//! are visibly fail closed rather than populated with guessed legacy data.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    inventory_runtime::InventoryRuntime0104,
    localization::LocalizationPlugin,
    pc2pc_ui::{
        PC2PC_CHALET_FONT_PATH, PC2PC_JEFFE_FONT_PATH, PC2PC_OPEN_SECONDS,
        PC2PC_UI_DEFAULT_IMAGE_PATHS, Pc2pcAuthoritativeSnapshot0104, Pc2pcBackendCapabilities,
        Pc2pcEquipEligibility, Pc2pcModeProjection0104, Pc2pcOfferDirection0104, Pc2pcPair0104,
        Pc2pcParticipantNames0104, Pc2pcServerOutcome0104, Pc2pcSessionIdentity0104,
        Pc2pcTradeItem0104, Pc2pcUiElement, Pc2pcUiModel0104, Pc2pcUiPlugin, Pc2pcUiRoot,
        Pc2pcUiSet,
    },
    user_equip_ui::{
        UserEquipCatalogKind, UserEquipCatalogQuery, UserEquipIconRef, UserEquipItemCatalog,
    },
};
use ffone_protocol::{ItemBase0104, PcLoadData0104};
use image::RgbaImage;

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const DEFAULT_OUTPUT: &str = "target/ui-parity/pc2pc-offer-1264x681.png";
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 100_000;
const MIN_MISSING_CHECKER_PIXELS: usize = 500;
const WHITE_PLACEHOLDER_GUARD_SIDE: usize = 40;
const WHITE_PLACEHOLDER_MIN_COVERAGE_PERCENT: u32 = 95;
const EXPECTED_ELEMENT_COUNT: usize = 309;

const LOCAL_ID: i32 = 4_242;
const REMOTE_ID: i32 = 8_181;
const ICON_GENERAL_00: &str = "icons/items/general/generalitemicon_00.png";
const ICON_GENERAL_01: &str = "icons/items/general/generalitemicon_01.png";
const ICON_WEAPON_01: &str = "icons/items/weapons/wpnicon_01.png";
const ICON_WEAPON_02: &str = "icons/items/weapons/wpnicon_02.png";
const ICON_COSMETIC_00: &str = "icons/items/cosmetics/cosicon_00.png";
const ICON_COSMETIC_01: &str = "icons/items/cosmetics/cosicon_01.png";
const ICON_COSMETIC_02: &str = "icons/items/cosmetics/cosicon_02.png";
const ICON_COSMETIC_03: &str = "icons/items/cosmetics/cosicon_03.png";
const ICON_COSMETIC_04: &str = "icons/items/cosmetics/cosicon_04.png";
const ICON_COSMETIC_05: &str = "icons/items/cosmetics/cosicon_05.png";
const ICON_VEHICLE_00: &str = "icons/items/vehicles/vehicle_00.png";
const PREVIEW_ICON_PATHS: [&str; 11] = [
    ICON_GENERAL_00,
    ICON_GENERAL_01,
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
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    fonts: [Handle<Font>; 2],
}

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
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut arguments = arguments.into_iter();
    let output = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT));
    if arguments.next().is_some()
        || output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return Err("usage: pc2pc_ui_gpu_preview [OUTPUT.png]");
    }
    Ok(PreviewCli { output })
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

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution Pc2pc acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, Pc2pcUiPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture.after(Pc2pcUiSet::Bind))
        .run();
}

struct PreviewCatalog;

impl UserEquipItemCatalog for PreviewCatalog {
    fn resolve_icon(&self, query: UserEquipCatalogQuery) -> Option<UserEquipIconRef> {
        let path = match (query.kind, query.base_item_id) {
            (UserEquipCatalogKind::Equipment { .. }, 100) => ICON_WEAPON_01,
            (UserEquipCatalogKind::Equipment { .. }, 108) => ICON_WEAPON_02,
            (UserEquipCatalogKind::Equipment { .. }, 101) => ICON_COSMETIC_00,
            (UserEquipCatalogKind::Equipment { .. }, 102) => ICON_COSMETIC_01,
            (UserEquipCatalogKind::Equipment { .. }, 103) => ICON_COSMETIC_02,
            (UserEquipCatalogKind::Equipment { .. }, 104) => ICON_COSMETIC_03,
            (UserEquipCatalogKind::Equipment { .. }, 105) => ICON_COSMETIC_04,
            (UserEquipCatalogKind::Equipment { .. }, 106) => ICON_COSMETIC_05,
            (UserEquipCatalogKind::Equipment { .. }, 107) => ICON_VEHICLE_00,
            (UserEquipCatalogKind::General, 200) => ICON_GENERAL_00,
            (UserEquipCatalogKind::General, 201) => ICON_GENERAL_01,
            // Deliberate miss: exact clean AvatarUtil checker fallback.
            (UserEquipCatalogKind::Chest, 300) => return None,
            _ => return None,
        };
        UserEquipIconRef::new(path).ok()
    }
}

struct PreviewEligibility;

impl Pc2pcEquipEligibility for PreviewEligibility {
    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }
}

fn preview_model() -> Pc2pcUiModel0104 {
    let mut load = PcLoadData0104::zeroed();
    let equipment = [
        (0, item(0, 100, 0)),
        (1, item(1, 101, 0)),
        (3, item(3, 103, 0)),
        (4, item(4, 104, 0x0012_0000)),
        (5, item(5, 105, 0)),
        (6, item(6, 106, 0)),
        (7, item(0, 108, 0)),
        (8, item(10, 107, 0)),
    ];
    let inventory = [
        (0, item(7, 200, 25)),
        (1, item(0, 100, 0x0011_0002)),
        (2, item(9, 300, 0)),
        (4, item(1, 101, 0)),
        (5, item(2, 102, 0)),
        (6, item(3, 103, 0)),
        (7, item(4, 104, 0)),
        (8, item(5, 105, 0)),
        (9, item(6, 106, 0)),
        (10, item(10, 107, 0)),
        (11, item(7, 201, 3)),
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
    let runtime = InventoryRuntime0104::from_pc_load(LOCAL_ID, &load);
    let pair = Pc2pcPair0104::new(LOCAL_ID, REMOTE_ID).unwrap();
    let identity =
        Pc2pcSessionIdentity0104::new(pair, LOCAL_ID, Pc2pcOfferDirection0104::Outgoing).unwrap();
    let snapshot = Pc2pcAuthoritativeSnapshot0104::from_accepted_trade(
        identity,
        Pc2pcParticipantNames0104::new("Dexter", "Mandark").unwrap(),
        98_765,
        &runtime,
    )
    .unwrap();
    let mut model = Pc2pcUiModel0104::default();
    model.begin_session(snapshot, &PreviewCatalog, &PreviewEligibility);
    model.state.tick(PC2PC_OPEN_SECONDS);

    let local = model
        .request_register_item(Default::default(), 0, 0, Some(9))
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope: local.envelope,
                trade_item: local.item,
                inventory_item: Pc2pcTradeItem0104 {
                    option: 16,
                    ..local.item
                },
            },
            Default::default(),
            &PreviewCatalog,
            &PreviewEligibility,
        )
        .unwrap();
    let local_equipment = model
        .request_register_item(Default::default(), 1, 1, None)
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope: local_equipment.envelope,
                trade_item: local_equipment.item,
                inventory_item: Pc2pcTradeItem0104 {
                    item_id: 0,
                    ..local_equipment.item
                },
            },
            Default::default(),
            &PreviewCatalog,
            &PreviewEligibility,
        )
        .unwrap();

    let local_cash = model
        .request_register_taros(
            Default::default(),
            Pc2pcBackendCapabilities {
                numeric_popup_backend: true,
                ..default()
            },
            12_500,
        )
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterCashSuccess {
                envelope: local_cash.envelope,
                taros: local_cash.taros,
            },
            Default::default(),
            &PreviewCatalog,
            &PreviewEligibility,
        )
        .unwrap();

    let remote_envelope = ffone_client::pc2pc_ui::Pc2pcEnvelope0104 {
        pair,
        requester_pc_id: REMOTE_ID,
    };
    let remote_general = Pc2pcTradeItem0104 {
        item_type: 7,
        item_id: 201,
        option: 3,
        inventory_slot: 4,
        offer_slot: 0,
    };
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope: remote_envelope,
                trade_item: remote_general,
                inventory_item: remote_general,
            },
            Default::default(),
            &PreviewCatalog,
            &PreviewEligibility,
        )
        .unwrap();
    let remote_missing = Pc2pcTradeItem0104 {
        item_type: 9,
        item_id: 300,
        option: 0,
        inventory_slot: 5,
        offer_slot: 2,
    };
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterItemSuccess {
                envelope: remote_envelope,
                trade_item: remote_missing,
                inventory_item: remote_missing,
            },
            Default::default(),
            &PreviewCatalog,
            &PreviewEligibility,
        )
        .unwrap();
    model
        .apply_server_outcome(
            Pc2pcServerOutcome0104::RegisterCashSuccess {
                envelope: remote_envelope,
                taros: 8_750,
            },
            Default::default(),
            &PreviewCatalog,
            &PreviewEligibility,
        )
        .unwrap();

    model.state.local_ready = true;
    model.state.remote_ready = true;
    model
}

const fn item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
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
    mut model: ResMut<Pc2pcUiModel0104>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    *model = preview_model();
    commands.insert_resource(PreviewAssets {
        images: PC2PC_UI_DEFAULT_IMAGE_PATHS
            .iter()
            .copied()
            .chain(PREVIEW_ICON_PATHS)
            .map(|path| asset_server.load(path))
            .collect(),
        fonts: [
            asset_server.load(PC2PC_JEFFE_FONT_PATH),
            asset_server.load(PC2PC_CHALET_FONT_PATH),
        ],
    });
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&ComputedNode, &Visibility), With<Pc2pcUiRoot>>,
    elements: Query<(&Pc2pcUiElement, &ComputedNode)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || preview_assets
            .fonts
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("Pc2pc acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && preview_assets
            .fonts
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded));
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && preview_assets
            .fonts
            .iter()
            .all(|handle| fonts.get(handle).is_some());
    let root_exact = roots.iter().any(|(node, visibility)| {
        *visibility == Visibility::Visible
            && node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let element_count_exact = elements.iter().count() == EXPECTED_ELEMENT_COUNT;
    let viewport_exact = elements.iter().any(|(element, node)| {
        *element == Pc2pcUiElement::InventoryViewport && node.size() == Vec2::new(366.0, 500.0)
    });
    let panel_geometry_exact = elements.iter().any(|(element, node)| {
        *element == Pc2pcUiElement::TradePanel && node.size() == Vec2::new(550.0, 700.0)
    }) && elements.iter().any(|(element, node)| {
        *element == Pc2pcUiElement::PcStuffPanel && node.size() == Vec2::new(380.0, 632.0)
    }) && elements.iter().any(|(element, node)| {
        *element == Pc2pcUiElement::EquipmentPanel && node.size() == Vec2::new(66.0, 639.0)
    });
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && element_count_exact
        && viewport_exact
        && panel_geometry_exact;
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
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!(
            "Pc2pc acceptance capture timed out: elements={} expected={EXPECTED_ELEMENT_COUNT}",
            elements.iter().count()
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
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
            "rejecting Pc2pc capture with unexpected size {:?}",
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
    if visible_pixels < MIN_VISIBLE_PIXELS || checker_pixels < MIN_MISSING_CHECKER_PIXELS {
        eprintln!(
            "rejecting incomplete Pc2pc capture: visible={visible_pixels} \
             (min {MIN_VISIBLE_PIXELS}), checker={checker_pixels} \
             (min {MIN_MISSING_CHECKER_PIXELS})"
        );
        state.capture_failed = true;
        return;
    }
    if contains_large_near_white_placeholder(&rgba) {
        eprintln!(
            "rejecting Pc2pc capture with a near-white placeholder region of at least \
             {WHITE_PLACEHOLDER_GUARD_SIDE}x{WHITE_PLACEHOLDER_GUARD_SIDE}"
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

fn contains_large_near_white_placeholder(rgba: &RgbaImage) -> bool {
    let width = rgba.width() as usize;
    let height = rgba.height() as usize;
    if width < WHITE_PLACEHOLDER_GUARD_SIDE || height < WHITE_PLACEHOLDER_GUARD_SIDE {
        return false;
    }

    let stride = width + 1;
    let mut integral = vec![0_u32; stride * (height + 1)];
    for y in 0..height {
        let mut row_sum = 0_u32;
        for x in 0..width {
            let [red, green, blue, alpha] = rgba.get_pixel(x as u32, y as u32).0;
            row_sum += u32::from(red >= 245 && green >= 245 && blue >= 245 && alpha >= 245);
            integral[(y + 1) * stride + x + 1] = integral[y * stride + x + 1] + row_sum;
        }
    }

    let side = WHITE_PLACEHOLDER_GUARD_SIDE;
    let minimum = (side * side) as u32 * WHITE_PLACEHOLDER_MIN_COVERAGE_PERCENT / 100;
    for top in 0..=height - side {
        for left in 0..=width - side {
            let bottom = top + side;
            let right = left + side;
            let covered = integral[bottom * stride + right] + integral[top * stride + left]
                - integral[top * stride + right]
                - integral[bottom * stride + left];
            if covered >= minimum {
                return true;
            }
        }
    }
    false
}

fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

#[cfg(test)]
mod tests;
