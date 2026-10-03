//! Capture a production HNPC appearance using only native runtime assets.

use std::{fs, path::PathBuf, time::Duration};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::TimeUpdateStrategy,
    window::{PresentMode, WindowResolution},
};
use ffone_client::{assets::AssetLocator, hnpc_runtime::HnpcRuntimeCatalog};
use ffone_client::{
    legacy_model_material::{
        LegacyMaterialMetadataError, LegacyMaterialPassCompanion, LegacyMaterialRendererOrder,
        LegacyMaterialSortOrderApplied, LegacyModelMaterialPlugin, PendingLegacyModelMaterial,
    },
    player_preview::{
        NativePlayerPreviewModel, NativePlayerPreviewPlugin, NativePlayerPreviewStage,
        NativePlayerPreviewStatus,
    },
    player_shared_rig::{
        NativePlayerRigCatalog, NativePlayerRigPartScene, NativePlayerRigPartsBound,
        NativePlayerRigStatus, NativePlayerSharedRigPlugin,
    },
};

const FRAME_STEP: Duration = Duration::from_nanos(16_666_667);
const READY_WARMUP_FRAMES: u32 = 90;
const TIMEOUT_FRAMES: u32 = 900;

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    capture_issued: bool,
    capture_saved: bool,
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let asset_root = fs::canonicalize(args.next().expect("ASSET_ROOT")).expect("asset root");
    let ordinal: usize = args
        .next()
        .expect("ORDINAL")
        .to_string_lossy()
        .parse()
        .expect("ordinal");
    let output = PathBuf::from(args.next().expect("OUTPUT.png"));
    let yaw_degrees: f32 = args
        .next()
        .map(|value| value.to_string_lossy().parse().expect("YAW_DEGREES"))
        .unwrap_or(0.0);
    assert!(
        args.next().is_none(),
        "usage: hnpc_player_gpu_preview ASSET_ROOT ORDINAL OUTPUT.png [YAW_DEGREES]"
    );
    let locator = AssetLocator::open(&asset_root).expect("asset locator");
    let base = NativePlayerRigCatalog::open(&asset_root).expect("base rig");
    let appearances = HnpcRuntimeCatalog::open(&locator, &base).expect("production HNPC catalog");
    let look = appearances
        .appearance(ordinal)
        .and_then(|entry| entry.look.clone())
        .expect("renderable appearance");
    let catalog = appearances.rig_catalog().clone();
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
        .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME_STEP))
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewState::default())
        .insert_resource(catalog)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("FFOne HNPC Player audit #{ordinal}"),
                        resolution: WindowResolution::new(800, 720),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        visible: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            NativePlayerSharedRigPlugin,
            NativePlayerPreviewPlugin,
        ))
        .add_systems(
            Startup,
            move |mut model: ResMut<NativePlayerPreviewModel>| {
                model.set_look(look.clone()).expect("set HNPC look");
                model.visible = true;
                model.stage = NativePlayerPreviewStage::Creation;
                model.camera_distance = 3.0;
                model.yaw_degrees = yaw_degrees;
            },
        )
        .add_systems(Update, drive_capture)
        .run();
}

fn drive_capture(
    mut commands: Commands,
    mut state: ResMut<PreviewState>,
    model: Res<NativePlayerPreviewModel>,
    material_errors: Query<&LegacyMaterialMetadataError>,
    rig_statuses: Query<&NativePlayerRigStatus>,
    bound_parts: Query<&NativePlayerRigPartsBound>,
    ordered_materials: Query<(
        &Name,
        &LegacyMaterialRendererOrder,
        Option<&LegacyMaterialSortOrderApplied>,
        Option<&LegacyMaterialPassCompanion>,
        Option<&PendingLegacyModelMaterial>,
    )>,
    part_scenes: Query<(&NativePlayerRigPartScene, Option<&Children>)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames += 1;
    if state.frames % 60 == 0 {
        eprintln!("HNPC preview frame {}: {:?}", state.frames, model.status);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
        return;
    }
    if let NativePlayerPreviewStatus::Blocked(error) = &model.status {
        panic!("HNPC Player preview blocked: {error}");
    }
    if let Some(error) = material_errors.iter().next() {
        panic!("HNPC Player material blocked: {}", error.0);
    }
    if matches!(
        model.status,
        NativePlayerPreviewStatus::ReadyAnimated { .. }
    ) {
        let frame = state.frames;
        state.ready_frame.get_or_insert(frame);
    }
    if state.capture_issued
        || state
            .ready_frame
            .is_none_or(|ready| state.frames < ready + READY_WARMUP_FRAMES)
    {
        if state.frames > TIMEOUT_FRAMES {
            let part_details = part_scenes
                .iter()
                .map(|(part, children)| {
                    format!(
                        "{} => {} direct children ({})",
                        part.exact_route,
                        children.map_or(0, Children::len),
                        part.glb
                    )
                })
                .collect::<Vec<_>>();
            let ordered_details = ordered_materials
                .iter()
                .map(|(name, order, applied, companion, pending)| {
                    format!(
                        "{} order={} applied={applied:?} expected={} companion={}",
                        name.as_str(),
                        order.renderer_index,
                        pending.map_or(0, |pending| pending.params.render_plan().passes.len()),
                        companion.is_some()
                    )
                })
                .collect::<Vec<_>>();
            panic!(
                "HNPC Player preview timed out: {:?}; detail={:?}; bound={:?}; parts={part_details:#?}; ordered={ordered_details:#?}",
                model.status,
                (
                    model.loading_detail.as_deref(),
                    rig_statuses.iter().collect::<Vec<_>>()
                ),
                bound_parts.iter().collect::<Vec<_>>()
            );
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    state.capture_issued = true;
    eprintln!("HNPC preview capture requested at frame {}", state.frames);
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_screenshot);
}

fn save_screenshot(
    capture: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
    mut state: ResMut<PreviewState>,
) {
    if let Some(parent) = output.0.parent() {
        fs::create_dir_all(parent).expect("create screenshot parent");
    }
    capture
        .image
        .clone()
        .try_into_dynamic()
        .expect("convert screenshot")
        .save(&output.0)
        .unwrap_or_else(|error| panic!("save {}: {error}", output.0.display()));
    state.capture_saved = true;
}
