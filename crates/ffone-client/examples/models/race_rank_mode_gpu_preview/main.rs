//! Deterministic settled 1264x681 GPU acceptance frame for clean-Retrobution
//! `cnRaceRankMode`.

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

use race_ui::rank::{
    RACE_RANK_RETROBUTION_URL, RaceRankCatalog, RaceRankModel, RaceRankOpenContext, RaceRankPhase,
    RaceRankPresentationAssetStatus, RaceRankPresentationLeft, RaceRankPresentationLocationRow,
    RaceRankPresentationRight, RaceRankPresentationRoot, RaceRankPresentationScoreRow,
    RaceRankUiPlugin,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const DEFAULT_OUTPUT: &str = "target/ui-parity/race-rank-mode-1264x681.png";
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 180_000;

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
        eprintln!("usage: race_rank_mode_gpu_preview [OUTPUT.png]");
        std::process::exit(2);
    });
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let catalog = RaceRankCatalog::embedded().expect("clean rank catalog must validate");
    let model = build_preview_model(&catalog);

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(catalog)
        .insert_resource(model)
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
                        title: "FFOne Retrobution RaceRankMode acceptance".to_owned(),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(RaceRankUiPlugin)
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
        return Err("RaceRankMode output must use the .png extension".to_owned());
    }
    Ok(output)
}

fn build_preview_model(catalog: &RaceRankCatalog) -> RaceRankModel {
    let mut model = RaceRankModel::default();
    model
        .open(
            catalog,
            RaceRankOpenContext {
                pcuid: 42,
                current_ep_id: 15,
                npc_name: "Dexter".to_owned(),
                npc_target_instance_id: 1_234,
                rank_url: RACE_RANK_RETROBUTION_URL.to_owned(),
            },
        )
        .expect("EP 15 must exist in clean catalog");
    model.clear_outputs();
    model
        .complete_fetch(1, &preview_rank_response())
        .expect("preview rank response must parse");
    model.advance_slide(1.0);
    model
}

fn preview_rank_response() -> String {
    fn score(pcuid: i32, value: i32, rank: i32, first: &str, last: &str) -> String {
        format!(
            "<score PCUID=\"{pcuid}\" Score=\"{value}\" Rank=\"{rank}\" FirstName=\"{first}\" LastName=\"{last}\"/>"
        )
    }
    let mut body = "SUCCESS".to_owned();
    for (top, personal) in [
        ("day", "myday"),
        ("week", "myweek"),
        ("month", "mymonth"),
        ("alltime", "myalltime"),
    ] {
        body.push_str(&format!(
            "<{personal}>{}</{personal}>",
            score(42, 45_678, 2, "Amb", "Fibia")
        ));
        body.push_str(&format!("<{top}>"));
        for index in 0..10 {
            let (pcuid, first, last) = if index == 1 {
                (42, "Amb", "Fibia")
            } else {
                (100 + index, "Player", "One")
            };
            body.push_str(&score(
                pcuid,
                50_000 - index * 1_337,
                index + 1,
                first,
                last,
            ));
        }
        body.push_str(&format!("</{top}>"));
    }
    body
}

fn setup_preview(mut commands: Commands) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    status: Res<RaceRankPresentationAssetStatus>,
    model: Res<RaceRankModel>,
    mut state: ResMut<PreviewState>,
    roots: Query<&ComputedNode, With<RaceRankPresentationRoot>>,
    lefts: Query<
        &ComputedNode,
        (
            With<RaceRankPresentationLeft>,
            Without<RaceRankPresentationRoot>,
            Without<RaceRankPresentationRight>,
        ),
    >,
    rights: Query<
        &ComputedNode,
        (
            With<RaceRankPresentationRight>,
            Without<RaceRankPresentationRoot>,
            Without<RaceRankPresentationLeft>,
        ),
    >,
    rows: Query<&RaceRankPresentationLocationRow>,
    score_rows: Query<&RaceRankPresentationScoreRow>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let RaceRankPresentationAssetStatus::Failed { ref asset_path } = *status {
        eprintln!("RaceRankMode source asset failed to load: {asset_path}");
        exit.write(AppExit::error());
        return;
    }
    let layout_ready = roots
        .single()
        .is_ok_and(|node| node.size() == Vec2::new(1_264.0, 681.0))
        && lefts
            .single()
            .is_ok_and(|node| node.size() == Vec2::new(490.0, 632.0))
        && rights
            .single()
            .is_ok_and(|node| node.size() == Vec2::new(472.0, 632.0));
    let ready = matches!(*status, RaceRankPresentationAssetStatus::Ready)
        && model.phase() == RaceRankPhase::Browsing
        && model.window_scroll() == 1.0
        && layout_ready
        && rows.iter().count() == 5
        && score_rows.iter().count() == 10;
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
            eprintln!("RaceRankMode capture timed out");
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
            "rejecting RaceRankMode capture at {}x{}",
            image.width(),
            image.height()
        );
        state.capture_failed = true;
        return;
    }
    let visible = image
        .to_rgba8()
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 35
        })
        .count();
    if visible < MIN_VISIBLE_PIXELS {
        eprintln!("rejecting empty RaceRankMode capture: {visible} visible pixels");
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
