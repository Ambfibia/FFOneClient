//! Loads every published non-empty HNPC appearance through the production
//! player-preview pipeline and exits once each one reaches `ReadyAnimated`.

use std::{env, path::PathBuf, process::Command, time::Duration};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    time::TimeUpdateStrategy,
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    assets::AssetLocator,
    hnpc_runtime::HnpcRuntimeCatalog,
    legacy_model_material::LegacyModelMaterialPlugin,
    player_preview::{
        NativePlayerLook, NativePlayerPreviewModel, NativePlayerPreviewPlugin,
        NativePlayerPreviewSet, NativePlayerPreviewStage, NativePlayerPreviewStatus,
    },
    player_shared_rig::{
        NativePlayerRigCatalog, NativePlayerRigInstance, NativePlayerRigStatus,
        NativePlayerSharedRigPlugin,
    },
};

const FRAME_STEP: Duration = Duration::from_nanos(16_666_667);
const TIMEOUT_FRAMES_PER_APPEARANCE: u32 = 900;

#[derive(Resource)]
struct ProbeAppearances {
    entries: Vec<(usize, NativePlayerLook)>,
    cursor: usize,
    frames_on_current: u32,
    clearing_previous: bool,
    clear_settle_frames: u8,
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let asset_root = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("assets/game"));
    let start_appearance = args
        .next()
        .map(|value| {
            value
                .to_string_lossy()
                .parse::<usize>()
                .expect("START_APPEARANCE integer")
        })
        .unwrap_or_default();
    let end_appearance = args
        .next()
        .map(|value| {
            value
                .to_string_lossy()
                .parse::<usize>()
                .expect("END_APPEARANCE integer")
        })
        .unwrap_or(usize::MAX);
    assert!(
        args.next().is_none(),
        "usage: hnpc_runtime_batch_probe [ASSET_ROOT] [START_APPEARANCE] [END_APPEARANCE]"
    );
    assert!(
        start_appearance <= end_appearance,
        "START_APPEARANCE must not exceed END_APPEARANCE"
    );
    let asset_root = std::fs::canonicalize(asset_root).expect("canonical asset root");
    let locator = AssetLocator::open(&asset_root).expect("open asset root");
    let base_rig = NativePlayerRigCatalog::open(&asset_root).expect("open shared player rig");
    let catalog = HnpcRuntimeCatalog::open(&locator, &base_rig).expect("open HNPC runtime catalog");
    let entries = (0..catalog.len())
        .filter_map(|index| {
            catalog
                .appearance(index)
                .and_then(|appearance| appearance.look.clone())
                .map(|look| (index, look))
        })
        .filter(|(index, _)| *index >= start_appearance && *index <= end_appearance)
        .collect::<Vec<_>>();
    assert!(
        !entries.is_empty(),
        "HNPC runtime catalog has no renderable looks"
    );
    if entries.len() > 1 {
        let executable = env::current_exe().expect("current HNPC probe executable");
        for (index, _) in &entries {
            let status = Command::new(&executable)
                .arg(&asset_root)
                .arg(index.to_string())
                .arg(index.to_string())
                .env("RUST_LOG", "error")
                .status()
                .unwrap_or_else(|error| panic!("launch isolated HNPC appearance {index}: {error}"));
            assert!(
                status.success(),
                "isolated HNPC appearance {index} failed with {status}"
            );
        }
        eprintln!("all {} renderable HNPC appearances ready", entries.len());
        return;
    }

    let rig_catalog = catalog.rig_catalog().clone();
    App::new()
        .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME_STEP))
        .insert_resource(rig_catalog)
        .insert_resource(ProbeAppearances {
            entries,
            cursor: 0,
            frames_on_current: 0,
            clearing_previous: false,
            clear_settle_frames: 0,
        })
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne production HNPC batch probe".to_owned(),
                        resolution: WindowResolution::new(320, 240),
                        present_mode: PresentMode::AutoNoVsync,
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
        .add_systems(Startup, begin_probe)
        .add_systems(Update, advance_probe.after(NativePlayerPreviewSet::Rebuild))
        .run();
}

fn begin_probe(probe: Res<ProbeAppearances>, mut model: ResMut<NativePlayerPreviewModel>) {
    model.visible = true;
    model.stage = NativePlayerPreviewStage::Creation;
    model
        .set_look(probe.entries[0].1.clone())
        .expect("set first HNPC look");
}

fn advance_probe(
    mut probe: ResMut<ProbeAppearances>,
    mut model: ResMut<NativePlayerPreviewModel>,
    rigs: Query<(&NativePlayerRigInstance, &NativePlayerRigStatus)>,
    mut exit: MessageWriter<AppExit>,
) {
    probe.frames_on_current += 1;
    let appearance = probe.entries[probe.cursor].0;
    match &model.status {
        NativePlayerPreviewStatus::ReadyAnimated { .. } => {
            eprintln!(
                "HNPC appearance {appearance:03} ready in {} frames",
                probe.frames_on_current
            );
            probe.cursor += 1;
            probe.frames_on_current = 0;
            if probe.cursor == probe.entries.len() {
                eprintln!(
                    "all {} renderable HNPC appearances ready",
                    probe.entries.len()
                );
                exit.write(AppExit::Success);
                return;
            }
            model.clear_look();
            probe.clearing_previous = true;
            probe.clear_settle_frames = 0;
        }
        NativePlayerPreviewStatus::Blocked(error) => {
            eprintln!("HNPC appearance {appearance:03} blocked: {error}");
            exit.write(AppExit::error());
        }
        NativePlayerPreviewStatus::Empty if probe.clearing_previous => {
            if probe.clear_settle_frames < 3 {
                probe.clear_settle_frames += 1;
            } else {
                model
                    .set_look(probe.entries[probe.cursor].1.clone())
                    .expect("set next HNPC look");
                probe.clearing_previous = false;
                probe.clear_settle_frames = 0;
                probe.frames_on_current = 0;
            }
        }
        NativePlayerPreviewStatus::Empty | NativePlayerPreviewStatus::Loading => {
            if probe.frames_on_current > TIMEOUT_FRAMES_PER_APPEARANCE {
                let rig_statuses = rigs
                    .iter()
                    .filter(|(instance, _)| {
                        model
                            .look()
                            .is_some_and(|look| instance.identity == look.identity)
                    })
                    .map(|(instance, status)| (instance.generation, status.clone()))
                    .collect::<Vec<_>>();
                eprintln!(
                    "HNPC appearance {appearance:03} timed out: {:?}; detail={:?}; rigs={rig_statuses:?}",
                    model.status, model.loading_detail,
                );
                exit.write(AppExit::error());
            }
        }
    }
    std::thread::sleep(Duration::from_millis(2));
}
