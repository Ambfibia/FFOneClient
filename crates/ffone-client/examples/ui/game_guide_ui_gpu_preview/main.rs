//! Deterministic 1264x681 GPU acceptance frame for clean-Retrobution
//! `cnHelpMode` / `cnGuiHelp` initial page `(1, 0)`.

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
    game_guide_ui::{
        GAME_GUIDE_BACKGROUND_PATH, GAME_GUIDE_CHALET_FONT_PATH, GAME_GUIDE_CLOSE_OVER_PATH,
        GAME_GUIDE_CLOSE_PATH, GAME_GUIDE_HELP_BUTTON_OVER_PATH, GAME_GUIDE_HELP_BUTTON_PATH,
        GAME_GUIDE_JEFFE_FONT_PATH, GAME_GUIDE_NAV_BUTTON_OVER_PATH, GAME_GUIDE_NAV_BUTTON_PATH,
        GAME_GUIDE_SCROLL_DOWN_PATH, GAME_GUIDE_SCROLL_THUMB_PATH, GAME_GUIDE_SCROLL_TRACK_PATH,
        GAME_GUIDE_SCROLL_UP_PATH, GAME_GUIDE_TITLE_BAR_PATH, GameGuideUiModel, GameGuideUiPlugin,
        GameGuideUiRoot,
    },
    localization::{Localization, LocalizationPlugin},
};

const WIDTH: u32 = 1_264;
const HEIGHT: u32 = 681;
const TIMEOUT: Duration = Duration::from_secs(30);
const ASSETS: [&str; 14] = [
    "ui/en/shared/panelback.png",
    GAME_GUIDE_BACKGROUND_PATH,
    GAME_GUIDE_CLOSE_PATH,
    GAME_GUIDE_CLOSE_OVER_PATH,
    GAME_GUIDE_HELP_BUTTON_PATH,
    GAME_GUIDE_HELP_BUTTON_OVER_PATH,
    GAME_GUIDE_TITLE_BAR_PATH,
    GAME_GUIDE_NAV_BUTTON_PATH,
    GAME_GUIDE_NAV_BUTTON_OVER_PATH,
    GAME_GUIDE_SCROLL_UP_PATH,
    GAME_GUIDE_SCROLL_DOWN_PATH,
    GAME_GUIDE_SCROLL_TRACK_PATH,
    GAME_GUIDE_SCROLL_THUMB_PATH,
    "ui/en/gameplay/game-guide/screenshots/login.png",
];

#[derive(Resource)]
struct Output(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    fonts: Vec<Handle<Font>>,
}

#[derive(Resource)]
struct CaptureState {
    started: Instant,
    ready_frames: u32,
    issued: bool,
    saved: bool,
}

fn main() {
    let locale = std::env::args().nth(1).unwrap_or_else(|| "en".to_owned());
    let output = std::env::args_os().nth(2).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/game-guide-initial-{locale}-1264x681.png"
        ))
    });
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) = Localization::open(&asset_root, &locale)
        .unwrap_or_else(|error| panic!("open production localization {locale}: {error}"));

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.15, 0.42, 0.62)))
        .insert_resource(Output(output))
        .insert_resource(CaptureState {
            started: Instant::now(),
            ready_frames: 0,
            issued: false,
            saved: false,
        })
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
                        title: "FFOne Retrobution Game Guide acceptance".into(),
                        resolution: WindowResolution::new(WIDTH, HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((GameGuideUiPlugin, LocalizationPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<GameGuideUiModel>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    model.open_from_nanocom();
    if let Ok(event) = env::var("FFONE_HELP_EVENT") {
        assert!(
            model.open_first_use(event.parse().expect("FirstUse event number")),
            "unresolved native help route"
        );
    }
    commands.insert_resource(PreviewAssets {
        images: ASSETS
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
        fonts: [GAME_GUIDE_JEFFE_FONT_PATH, GAME_GUIDE_CHALET_FONT_PATH]
            .into_iter()
            .map(|path| asset_server.load(path))
            .collect(),
    });
}

fn capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    preview_assets: Res<PreviewAssets>,
    roots: Query<&ComputedNode, With<GameGuideUiRoot>>,
    mut state: ResMut<CaptureState>,
    mut exit: MessageWriter<AppExit>,
) {
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || preview_assets
            .fonts
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("Game Guide acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }
    let loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && preview_assets
            .fonts
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded));
    let root_ready = roots
        .single()
        .is_ok_and(|node| node.size() == Vec2::new(WIDTH as f32, HEIGHT as f32));
    if loaded && root_ready {
        state.ready_frames += 1;
    } else {
        state.ready_frames = 0;
    }
    if state.ready_frames >= 30 && !state.issued {
        state.issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.saved {
        exit.write(AppExit::Success);
    } else if state.started.elapsed() >= TIMEOUT {
        eprintln!("Game Guide capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<Output>,
    mut state: ResMut<CaptureState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert");
    if let Some(parent) = output.0.parent() {
        fs::create_dir_all(parent).expect("create capture directory");
    }
    image.save(&output.0).expect("save Game Guide capture");
    println!("{}", output.0.display());
    state.saved = true;
}
