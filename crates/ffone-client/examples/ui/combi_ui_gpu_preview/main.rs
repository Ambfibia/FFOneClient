//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `eGameMode.Combi`.
//!
//! This harness injects an immutable authoritative inventory snapshot and a
//! Style/Stats selection overlay. It exercises no network owner and performs
//! no item or Taros mutation. Both legacy NPC camera boundaries intentionally
//! remain transparent.

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
use ffone_client::combi_ui::{
    COMBI_UI_DEFAULT_IMAGE_PATHS, CombiAuthoritativeSnapshot0104, CombiItemCatalog0104,
    CombiItemMetadata0104, CombiModeProjection0104, CombiPhase0104, CombiRecipeTable0104,
    CombiSelectionOverlay0104, CombiSelectionSlot0104, CombiSourceLocation0104, CombiUiElement0104,
    CombiUiPlugin, CombiUiRoot0104, CombiUiSet0104, CombiUiState0104, project_combi_mode_0104,
};
use ffone_client::inventory_runtime::{EQUIPMENT_SLOT_COUNT_0104, INVENTORY_SLOT_COUNT_0104};
use ffone_client::localization::{Localization, LocalizationPlugin};
use ffone_protocol::ItemBase0104;

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const DEFAULT_OUTPUT: &str = "target/ui-parity/combi-ready-1264x681.png";
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 90_000;
const ICON_STYLE: &str = "icons/items/weapons/wpnicon_01.png";
const ICON_STATS: &str = "icons/items/weapons/wpnicon_02.png";

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    fonts: Vec<Handle<Font>>,
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
        return Err("usage: combi_ui_gpu_preview [OUTPUT.png]");
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
    let (localization, language) =
        Localization::open(&asset_root, "en").expect("open production localization bundles");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(localization)
        .insert_resource(language)
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
                        title: "FFOne Retrobution Combi acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, CombiUiPlugin))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture.after(CombiUiSet0104::Bind))
        .run();
}

#[derive(Default)]
struct PreviewCatalog;

impl CombiItemCatalog0104 for PreviewCatalog {
    fn resolve(&self, item_type: i16, item_id: i16) -> Option<CombiItemMetadata0104> {
        let (name, level, rarity, price, point, group, defense, icon) = match (item_type, item_id) {
            (0, 100) => ("Retro Blaster", 4, 0, 90, 45, 32, 8, ICON_STYLE),
            (0, 101) => ("Croc Pot Blaster", 7, 1, 100, 55, 42, 12, ICON_STYLE),
            (0, 200) => ("Fusion Cannon", 5, 2, 200, 72, 61, 18, ICON_STATS),
            (0, 300) => ("Equipped Blaster", 3, 1, 70, 40, 30, 7, ICON_STATS),
            _ => return None,
        };
        Some(CombiItemMetadata0104 {
            name: name.to_owned(),
            description: "A clean-Retrobution Style/Stats preview item.".to_owned(),
            minimum_level: level,
            required_gender: 1,
            rarity,
            rarity_label: match rarity {
                0 => "Common",
                1 => "Uncommon",
                _ => "Rare",
            }
            .to_owned(),
            mentor: 0,
            cashable: 0,
            item_price: price,
            point_rating: point,
            group_rating: group,
            defense_rating: defense,
            delay_time: 10,
            equip_type: 4,
            target_mode: 5,
            type_label: "Weapon".to_owned(),
            trade_label: "Tradable".to_owned(),
            icon_path: Some(icon.to_owned()),
        })
    }

    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }

    fn enable_equip_combi(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }
}

fn preview_projection() -> CombiModeProjection0104 {
    let mut inventory = [empty_item(); INVENTORY_SLOT_COUNT_0104];
    inventory[0] = item(0, 100, 101 << 16, 0);
    inventory[1] = item(0, 200, 0, 0);
    inventory[2] = item(0, 300, 0, 0);
    inventory[5] = item(0, 100, 0, 0);
    inventory[8] = item(0, 200, 0, 0);
    let mut equipment = [empty_item(); EQUIPMENT_SLOT_COUNT_0104];
    equipment[0] = item(0, 300, 0, 0);
    let snapshot = CombiAuthoritativeSnapshot0104 {
        owner_pc_id: 4_242,
        gender: 1,
        level: 9,
        guide: 2,
        taros: 8_150,
        equipment,
        inventory,
    };
    let mut selection = CombiSelectionOverlay0104::default();
    selection
        .attach(
            &snapshot,
            CombiSourceLocation0104::Inventory,
            0,
            CombiSelectionSlot0104::Style,
        )
        .expect("preview Style attach");
    selection
        .attach(
            &snapshot,
            CombiSourceLocation0104::Inventory,
            1,
            CombiSelectionSlot0104::Stats,
        )
        .expect("preview Stats attach");
    let table_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game/data/tables/xdt.json");
    let table_bytes = fs::read(&table_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", table_path.display()));
    let recipes =
        CombiRecipeTable0104::from_table_set_bytes(&table_bytes).expect("clean Combi table");
    project_combi_mode_0104(&snapshot, selection, &PreviewCatalog, &recipes)
        .expect("preview projection")
}

const fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

const fn empty_item() -> ItemBase0104 {
    item(0, 0, 0, 0)
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<CombiUiState0104>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.insert_resource(preview_projection());
    state.phase = CombiPhase0104::Ready;
    state.primary_npc_camera_bound = false;
    state.waiting_npc_camera_bound = false;
    commands.insert_resource(PreviewAssets {
        images: COMBI_UI_DEFAULT_IMAGE_PATHS
            .iter()
            .copied()
            .chain([ICON_STYLE, ICON_STATS])
            .map(|path| asset_server.load(path))
            .collect(),
        fonts: vec![
            asset_server.load("fonts/jeffe.otf"),
            asset_server.load("fonts/chaletbook-regular.ttf"),
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
    roots: Query<(&ComputedNode, &Visibility), With<CombiUiRoot0104>>,
    elements: Query<(&CombiUiElement0104, &ComputedNode, Option<&BackgroundColor>)>,
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
        eprintln!("Combi acceptance asset failed to load");
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
    let panel_exact = elements.iter().any(|(element, node, _)| {
        *element == CombiUiElement0104::MainGroup && node.size() == Vec2::new(585.0, 653.0)
    });
    let look_exact = elements.iter().any(|(element, node, _)| {
        *element == CombiUiElement0104::LookDrop && node.size() == Vec2::new(305.0, 133.0)
    });
    let stats_exact = elements.iter().any(|(element, node, _)| {
        *element == CombiUiElement0104::StatDrop && node.size() == Vec2::new(305.0, 230.0)
    });
    let camera_boundary_transparent = elements.iter().any(|(element, node, background)| {
        *element == CombiUiElement0104::PrimaryNpcBoundary
            && node.size() == Vec2::new(143.0, 128.0)
            && background.is_some_and(|background| background.0 == Color::NONE)
    });
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && panel_exact
        && look_exact
        && stats_exact
        && camera_boundary_transparent;
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
        eprintln!("Combi acceptance capture timed out");
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
            "rejecting Combi capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let visible_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 35
        })
        .count();
    if visible_pixels < MIN_VISIBLE_PIXELS {
        eprintln!(
            "rejecting incomplete Combi capture: visible={visible_pixels} \
             (min {MIN_VISIBLE_PIXELS})"
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
