//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `cnRaceMode.OnEndGUI`.

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{localization, ui_startup};
use std::{
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};

#[allow(dead_code)]
#[path = "../../../src/ui/race/mod.rs"]
mod race_ui;

use race_ui::mode::{
    RaceEcomType, RaceEndSuccess, RaceModeModel, RaceModeOpenContext, RaceModePhase,
    RaceModePresentationAssetStatus, RaceModePresentationPanel, RaceModePresentationRoot,
    RaceModeReply, RaceModeUiPlugin, RaceNpcContext, RacePlayerState, RaceReplyEnvelope,
    RaceRewardItem, RaceTopRecord,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const DEFAULT_OUTPUT: &str = "target/ui-parity/race-mode-result-1264x681.png";
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(20);
const MIN_NON_BLACK_PIXELS: usize = 40_000;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

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
    let output = parse_preview_args(std::env::args_os().skip(1)).unwrap_or_else(|error| {
        eprintln!("{error}");
        eprintln!("usage: race_mode_gpu_preview [OUTPUT.png]");
        std::process::exit(2);
    });
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.03, 0.08, 0.12)))
        .insert_resource(build_preview_model())
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne Retrobution RaceMode result acceptance".to_owned(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(RaceModeUiPlugin)
        .add_systems(Startup, setup_preview)
        .add_systems(Update, drive_capture)
        .run();
}

fn parse_preview_args(args: impl IntoIterator<Item = OsString>) -> Result<PathBuf, String> {
    let mut args = args.into_iter();
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT));
    if args.next().is_some() {
        return Err("too many arguments".to_owned());
    }
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err("RaceMode output must use the .png extension".to_owned());
    }
    Ok(output)
}

fn build_preview_model() -> RaceModeModel {
    let mut model = RaceModeModel::default();
    model
        .open(RaceModeOpenContext {
            ecom_type: RaceEcomType::End,
            npc: Some(RaceNpcContext {
                instance_id: 8_123,
                has_race_start_voice: false,
            }),
            player: RacePlayerState {
                ring_race_active: true,
                current_ep_id: 15,
                top_record: RaceTopRecord {
                    rank: 2,
                    rings: 31,
                    score: 47_125,
                    time_seconds: 91,
                },
                fatigue: 100,
                fatigue_level: 1,
                fusion_matter: 8_000,
                cursor_was_locked: true,
                ..default()
            },
            current_ep_instance_exists: true,
        })
        .expect("preview EndEcom must open");
    model.clear_outputs();
    model
        .apply_reply(
            RaceReplyEnvelope {
                request_id: 1,
                reply: RaceModeReply::EndSuccess(RaceEndSuccess {
                    race_mode: 1,
                    race_time_seconds: 94,
                    ring_count: 28,
                    score: 45_678,
                    rank: 2,
                    reward_fusion_matter: 750,
                    top_score: 47_125,
                    top_rank: 2,
                    top_time_seconds: 91,
                    top_ring_count: 31,
                    fusion_matter: 8_750,
                    reward_item: RaceRewardItem {
                        // No item makes the always-present FM row occupy the
                        // first exact clean reward row.
                        e_il: 4,
                        ..default()
                    },
                    fatigue: 100,
                    fatigue_level: 1,
                }),
            },
            0.0,
        )
        .expect("preview result reply must correlate");
    model.clear_outputs();
    model
}

fn setup_preview(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.30, 0.42)),
        GlobalZIndex(-1_000),
        Pickable::IGNORE,
    ));
}

fn drive_capture(
    mut commands: Commands,
    status: Res<RaceModePresentationAssetStatus>,
    model: Res<RaceModeModel>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<RaceModePresentationRoot>>,
    panels: Query<
        &ComputedNode,
        (
            With<RaceModePresentationPanel>,
            Without<RaceModePresentationRoot>,
        ),
    >,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let RaceModePresentationAssetStatus::Failed { asset_path } = &*status {
        eprintln!("RaceMode source asset failed to load: {asset_path}");
        exit.write(AppExit::error());
        return;
    }
    let layout_ready = roots
        .single()
        .is_ok_and(|node| node.size() == Vec2::new(1_264.0, 681.0))
        && panels
            .single()
            .is_ok_and(|node| node.size() == Vec2::new(398.0, 434.0));
    let ready = matches!(*status, RaceModePresentationAssetStatus::Ready)
        && model.phase() == RaceModePhase::Result
        && layout_ready;
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
    } else if state.capture_failed || state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        if !state.capture_failed {
            eprintln!("RaceMode result capture timed out");
        }
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
    if image.width() != CLIENT_AREA_WIDTH || image.height() != CLIENT_AREA_HEIGHT {
        eprintln!(
            "rejecting RaceMode capture at {}x{}",
            image.width(),
            image.height()
        );
        state.capture_failed = true;
        return;
    }
    let non_black = image
        .to_rgba8()
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 48
        })
        .count();
    if non_black < MIN_NON_BLACK_PIXELS {
        eprintln!("rejecting empty RaceMode capture: {non_black} visible pixels");
        state.capture_failed = true;
        return;
    }
    if let Some(parent) = output.0.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!("cannot create {}: {error}", parent.display());
            state.capture_failed = true;
            return;
        }
    }
    match image.save(&output.0) {
        Ok(()) => {
            println!("saved {}", output.0.display());
            state.capture_saved = true;
        }
        Err(error) => {
            eprintln!("cannot save {}: {error}", output.0.display());
            state.capture_failed = true;
        }
    }
}
