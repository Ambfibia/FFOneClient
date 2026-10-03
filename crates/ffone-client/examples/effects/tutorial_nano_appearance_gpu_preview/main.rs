//! Deterministic GPU acceptance for the first Buttercup reveal in
//! `cntutorialscript.Infection_Event_A`.

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
    assets::AssetLocator,
    coordinates::native_model_forward_child_rotation,
    legacy_model_material::{LegacyModelMaterial, LegacyModelMaterialPlugin},
    tutorial_nano_presentation::{
        TutorialNanoAnimationPlayback, TutorialNanoPresentationCommandQueue,
        TutorialNanoPresentationPlugin, TutorialNanoPresentationSpawn,
        TutorialNanoPresentationState, TutorialNanoPresentationStatus,
    },
};

const CAPTURE_AFTER_APPLIED_FRAMES: u32 = 18;
const TIMEOUT_FRAMES: u32 = 900;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    applied_frame: Option<u32>,
    capture_issued: bool,
    capture_saved: bool,
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let asset_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets/game"));
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/tutorial-nano-appearance.png"));
    if args.next().is_some() || output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("usage: tutorial_nano_appearance_gpu_preview [ASSET_ROOT] [OUTPUT.png]");
        std::process::exit(2);
    }
    let asset_root = fs::canonicalize(&asset_root).unwrap_or_else(|error| {
        panic!(
            "cannot resolve asset root {}: {error}",
            asset_root.display()
        )
    });
    if output.exists() {
        fs::remove_file(&output)
            .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
    }
    let locator =
        AssetLocator::open(asset_root.clone()).expect("production assets/game must be valid");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(locator)
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
                        title: "FFOne tutorial Nano appearance acceptance".to_owned(),
                        resolution: WindowResolution::new(900, 640),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LegacyModelMaterialPlugin, TutorialNanoPresentationPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, drive_capture)
        .run();
}

fn setup(mut commands: Commands, mut nano_commands: ResMut<TutorialNanoPresentationCommandQueue>) {
    // Clean reveal: the visible model faces +Z and the camera is placed on
    // that same side by `newNano.rotation * Vector3.forward * 1.5f`.
    let root_rotation = native_model_forward_child_rotation();
    nano_commands.spawn(TutorialNanoPresentationSpawn::at(Transform::from_rotation(
        root_rotation,
    )));
    nano_commands.play_emote("call2");

    let target = Vec3::new(0.0, 0.25, 0.0);
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 38_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 0.25, 1.5).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-3.0, 5.0, 4.0).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 500_000.0,
            range: 15.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(1.5, 2.5, 3.0),
    ));
}

fn drive_capture(
    mut commands: Commands,
    nano: Res<TutorialNanoPresentationState>,
    playback: Query<&TutorialNanoAnimationPlayback>,
    materials: Res<Assets<LegacyModelMaterial>>,
    mut state: ResMut<PreviewState>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if let TutorialNanoPresentationStatus::Blocked(error) = nano.status() {
        eprintln!("tutorial Nano preview blocked: {error}");
        exit.write(AppExit::error());
        return;
    }
    let call2_applied =
        nano.animation_applied() && playback.iter().any(|animation| animation.clip == "call2");
    if call2_applied && state.applied_frame.is_none() {
        state.applied_frame = Some(state.frames);
    }
    if !state.capture_issued
        && !materials.is_empty()
        && state
            .applied_frame
            .is_some_and(|frame| state.frames.saturating_sub(frame) >= CAPTURE_AFTER_APPLIED_FRAMES)
    {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!(
            "tutorial Nano preview timed out: status={:?}, call2Applied={call2_applied}, materials={}",
            nano.status(),
            materials.len()
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(16));
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
