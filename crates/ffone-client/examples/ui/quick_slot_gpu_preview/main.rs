//! Deterministic 1264x681 GPU acceptance frame for the dormant clean
//! Retrobution QuickSlot HUD.

use std::{
    array, env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::quick_slot_ui::{
    LegacyQuickSlotEntry, QUICK_SLOT_BACKGROUND_PATH, QUICK_SLOT_COOLDOWN_PATH, QUICK_SLOT_COUNT,
    QUICK_SLOT_EMPTY_STYLE_PATH, QUICK_SLOT_OCCUPIED_STYLE_PATH, QuickSlotParityPreview,
    QuickSlotUiModel, QuickSlotUiPlugin, QuickSlotUiRoot,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_QUICK_SLOT_VISIBLE_PIXELS: usize = 200;
const PREVIEW_ICON_PATHS: [&str; QUICK_SLOT_COUNT] = [
    "icons/items/general/generalitemicon_00.png",
    "icons/items/general/generalitemicon_01.png",
    "icons/items/general/generalitemicon_02.png",
    "icons/items/general/generalitemicon_03.png",
    "icons/items/general/generalitemicon_04.png",
    "icons/items/general/generalitemicon_05.png",
    "icons/items/general/generalitemicon_06.png",
    "icons/items/general/generalitemicon_07.png",
];
const PREVIEW_COOLDOWN_FRACTIONS: [f32; QUICK_SLOT_COUNT] =
    [1.0, 0.85, 0.65, 0.5, 0.3, 0.15, 0.0, 0.0];

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets(Vec<Handle<Image>>);

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

fn main() {
    let mut args = std::env::args_os().skip(1);
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/ui-parity/quick-slot.png"));
    if args.next().is_some() || output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("usage: quick_slot_gpu_preview [OUTPUT.png]");
        std::process::exit(2);
    }

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewState::default())
        .insert_resource(QuickSlotParityPreview::enabled())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution QuickSlot acceptance".into(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(QuickSlotUiPlugin)
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
}

fn preview_model() -> QuickSlotUiModel {
    QuickSlotUiModel {
        ready_for_play: true,
        slots: array::from_fn(|slot| LegacyQuickSlotEntry {
            item_type: 7,
            item_id: 1_000 + slot as i32,
            icon_path: Some(PREVIEW_ICON_PATHS[slot].into()),
            inventory_empty: slot == QUICK_SLOT_COUNT - 1,
            cooldown_remaining_fraction: PREVIEW_COOLDOWN_FRACTIONS[slot],
        }),
        ..default()
    }
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<QuickSlotUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    *model = preview_model();
    commands.insert_resource(PreviewAssets(
        [
            QUICK_SLOT_BACKGROUND_PATH,
            QUICK_SLOT_OCCUPIED_STYLE_PATH,
            QUICK_SLOT_EMPTY_STYLE_PATH,
            QUICK_SLOT_COOLDOWN_PATH,
        ]
        .into_iter()
        .chain(PREVIEW_ICON_PATHS)
        .map(|path| asset_server.load(path))
        .collect(),
    ));
}

fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    preview_assets: Res<PreviewAssets>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&ComputedNode, &Visibility), With<QuickSlotUiRoot>>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);

    if preview_assets
        .0
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
    {
        eprintln!("QuickSlot acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .0
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded));
    let cpu_assets_present = preview_assets
        .0
        .iter()
        .all(|handle| images.get(handle).is_some());
    let exact_layout_ready = roots.iter().any(|(node, visibility)| {
        *visibility == Visibility::Visible
            && node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let ready = assets_loaded && cpu_assets_present && exact_layout_ready;
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
    if !state.capture_issued && ready && warmed {
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
        eprintln!("QuickSlot acceptance capture timed out");
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
            "rejecting QuickSlot capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let visible_pixels = (630..CLIENT_AREA_HEIGHT)
        .flat_map(|y| (440..750).map(move |x| (x, y)))
        .filter(|&(x, y)| {
            let [red, green, blue, _alpha] = rgba.get_pixel(x, y).0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 20
        })
        .count();
    if visible_pixels < MIN_QUICK_SLOT_VISIBLE_PIXELS {
        eprintln!(
            "rejecting empty QuickSlot capture: {visible_pixels} visible HUD pixels \
             < {MIN_QUICK_SLOT_VISIBLE_PIXELS}"
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
