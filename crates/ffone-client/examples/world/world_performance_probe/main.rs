//! Offline A/B probe of the production world renderer. No login/server required.
//! Usage: world_performance_probe [shared|baseline] [OUTPUT_DIR] [X Y Z] [--freeze]
//! Baseline disables only content sharing. Camera/objects/streaming stay identical.

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::TimeUpdateStrategy,
    window::{PresentMode, WindowResolution},
};
pub use ffone_client::ui_startup;
use ffone_client::{
    assets::AssetLocator,
    legacy_model_material::{
        LegacyModelMaterial, LegacyModelMaterialPlugin, StaticAssetSharing,
        static_asset_sharing_statistics,
    },
    movement::{LegacyMovementPlugin, LegacyOrbitCamera},
    native_terrain::{
        NATIVE_TERRAIN_ANISOTROPIC_TAPS, NATIVE_TERRAIN_ISOTROPIC_TAPS, NativeTerrainMaterial,
    },
    terrain_ambience::distance_haze_density,
    tutorial_effects_runtime::{
        TutorialEffectLibrary, TutorialEffectRuntime, TutorialEffectsRuntimePlugin,
    },
    tutorial_mission_content::TutorialMissionContent,
    world::{
        EXTENDED_WORLD_CAMERA_FAR_NATIVE, NativeWorldCatalog, NativeWorldPlugin,
        NativeWorldPresentationStatus, NativeWorldSceneRoot, NativeWorldStreamingStatus,
        SpawnedNativeWorldVisual, load_native_world_scenes,
    },
    world_behaviour::{NativeWorldBehaviourRoot, add_world_preview_behaviour_systems},
};
use std::{
    collections::HashSet,
    env, fs,
    path::PathBuf,
    time::{Duration, Instant},
};

#[path = "../../world_performance_probe/streaming.rs"]
mod streaming;

#[path = "../../world_performance_probe/material_audit.rs"]
mod material_audit;

#[derive(Resource)]
struct Probe {
    output: PathBuf,
    anchor: Vec3,
    enabled: bool,
    frozen: bool,
    started: Instant,
    ready: Option<Instant>,
    previous: Instant,
    samples: Vec<f64>,
    capture_issued: bool,
}

fn main() {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    let frozen = args.last().is_some_and(|arg| arg == "--freeze");
    if frozen {
        args.pop();
    }
    let enabled = match args.first().map(String::as_str).unwrap_or("shared") {
        "shared" => true,
        "baseline" => false,
        _ => panic!("expected shared or baseline"),
    };
    let output = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/performance/probe"));
    let anchor = if args.len() == 5 {
        Vec3::new(
            args[2].parse().unwrap(),
            args[3].parse().unwrap(),
            args[4].parse().unwrap(),
        )
    } else {
        assert!(
            args.len() <= 2,
            "usage: [shared|baseline] [OUTPUT_DIR] [X Y Z] [--freeze]"
        );
        // Published ordinary-world pool anchor also used by tutorial_terrain_gpu_preview.
        Vec3::new(-6374.3188, -56.4746, 668.2039)
    };
    assert!(anchor.is_finite());
    fs::create_dir_all(&output).unwrap();
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .canonicalize()
        .unwrap();
    let catalog = load_native_world_scenes(&asset_root).unwrap();
    assert!(
        catalog.select(anchor).is_some(),
        "no native world tile at probe anchor"
    );
    let effects = TutorialEffectLibrary::load(&asset_root).unwrap();
    let mission_content =
        TutorialMissionContent::open(&AssetLocator::open(asset_root.clone()).unwrap()).unwrap();
    let now = Instant::now();
    let mut app = App::new();
    app.insert_resource(Probe {
        output,
        anchor,
        enabled,
        frozen,
        started: now,
        ready: None,
        previous: now,
        samples: Vec::new(),
        capture_issued: false,
    })
    .insert_resource(catalog)
    .insert_resource(mission_content)
    .insert_resource(StaticAssetSharing { enabled })
    .insert_resource(NativeWorldBehaviourRoot(asset_root.clone()))
    .insert_resource(effects.clone())
    .insert_resource(TutorialEffectRuntime::with_library(effects))
    .add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: asset_root.to_string_lossy().into_owned(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "FFOne world performance probe".into(),
                    resolution: WindowResolution::new(1280, 720),
                    present_mode: PresentMode::AutoNoVsync,
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    )
    .add_plugins((
        LegacyMovementPlugin,
        LegacyModelMaterialPlugin,
        NativeWorldPlugin,
        TutorialEffectsRuntimePlugin,
    ))
    .add_systems(Startup, setup)
    .add_systems(Update, apply_probe_terrain_ambience)
    .add_systems(Last, measure);
    if frozen {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    }
    add_world_preview_behaviour_systems(&mut app);
    streaming::install(&mut app);
    app.run();
}

fn setup(
    mut commands: Commands,
    probe: Res<Probe>,
    catalog: Res<NativeWorldCatalog>,
    mut clear: ResMut<ClearColor>,
) {
    let target = commands
        .spawn((
            Transform::from_translation(probe.anchor),
            Visibility::Hidden,
        ))
        .id();
    let mut orbit = LegacyOrbitCamera::new(target).with_yaw(135.0);
    if let Ok(value) = env::var("FFONE_PROBE_DISTANCE") {
        orbit.distance = value.parse().expect("finite camera distance");
        assert!(orbit.distance.is_finite() && orbit.distance > 0.0);
    }
    if let Ok(value) = env::var("FFONE_PROBE_PITCH") {
        orbit.pitch_degrees = value.parse().expect("finite camera pitch");
        assert!(orbit.pitch_degrees.is_finite());
    }
    let sample = catalog.sample_ambience(probe.anchor, false);
    let applied = catalog.applied_ambience(probe.anchor, false);
    clear.0 = Color::srgba(
        sample.sky_color[0],
        sample.sky_color[1],
        sample.sky_color[2],
        sample.sky_color[3],
    );
    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        Projection::Perspective(PerspectiveProjection {
            fov: 45.0_f32.to_radians(),
            far: EXTENDED_WORLD_CAMERA_FAR_NATIVE,
            ..default()
        }),
        DistanceFog {
            color: Color::srgba(
                sample.fog_color[0],
                sample.fog_color[1],
                sample.fog_color[2],
                sample.fog_color[3],
            ),
            falloff: FogFalloff::Exponential {
                density: distance_haze_density(&applied, EXTENDED_WORLD_CAMERA_FAR_NATIVE),
            },
            ..default()
        },
        orbit,
        Transform::default(),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: false,
            color: Color::srgba(
                applied.light_color[0],
                applied.light_color[1],
                applied.light_color[2],
                applied.light_color[3],
            ),
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// The gameplay schedule pushes the sampled ambience into every resident
/// terrain material. Reproducing that here keeps the probe's capture a picture
/// of the production renderer instead of an unfogged terrain composite.
fn apply_probe_terrain_ambience(
    probe: Res<Probe>,
    catalog: Res<NativeWorldCatalog>,
    mut terrain_materials: ResMut<Assets<NativeTerrainMaterial>>,
) {
    let sample = catalog.sample_ambience(probe.anchor, false);
    let applied = catalog.applied_ambience(probe.anchor, false);
    let ambience_light = Vec4::from_array(applied.light_color);
    let ambience_fog = Vec4::new(
        sample.fog_color[0],
        sample.fog_color[1],
        sample.fog_color[2],
        distance_haze_density(&applied, EXTENDED_WORLD_CAMERA_FAR_NATIVE),
    );
    // `FFONE_PROBE_ANISOTROPIC=0` reproduces the disabled graphics option so a
    // capture pair isolates terrain filtering from every other change.
    let taps = if env::var("FFONE_PROBE_ANISOTROPIC").as_deref() == Ok("0") {
        NATIVE_TERRAIN_ISOTROPIC_TAPS
    } else {
        NATIVE_TERRAIN_ANISOTROPIC_TAPS
    };
    let render_quality = Vec4::new(taps, 0.0, 0.0, 0.0);
    let stale = terrain_materials
        .iter()
        .filter_map(|(id, material)| {
            (material.uniform.ambience_light != ambience_light
                || material.uniform.ambience_fog != ambience_fog
                || material.uniform.render_quality != render_quality)
                .then_some(id)
        })
        .collect::<Vec<_>>();
    for id in stale {
        if let Some(mut material) = terrain_materials.get_mut(id) {
            material.uniform.ambience_light = ambience_light;
            material.uniform.ambience_fog = ambience_fog;
            material.uniform.render_quality = render_quality;
        }
    }
}

fn measure(world: &mut World) {
    let mut probe = world.remove_resource::<Probe>().unwrap();
    let now = Instant::now();
    let frame_ms = now.duration_since(probe.previous).as_secs_f64() * 1000.0;
    probe.previous = now;
    let streaming = world.resource::<NativeWorldStreamingStatus>().clone();
    if let Some(blocker) = &streaming.blocker {
        panic!("world probe blocked: {blocker}");
    }
    assert!(
        probe.started.elapsed() < Duration::from_secs(240),
        "world probe timed out: {streaming:?}"
    );
    let ready = streaming.resident_tiles > 0
        && streaming.resident_tiles == streaming.target_tiles
        && streaming.loading_colliders == 0
        && !streaming.resident_budget_exceeded
        && world
            .query_filtered::<&NativeWorldPresentationStatus, With<NativeWorldSceneRoot>>()
            .iter(world)
            .all(|status| *status == NativeWorldPresentationStatus::Ready);
    if !ready {
        probe.ready = None;
        probe.samples.clear();
    }
    if ready && !probe.capture_issued {
        let ready_at = *probe.ready.get_or_insert(now);
        if now.duration_since(ready_at) >= Duration::from_secs(3) {
            probe.samples.push(frame_ms);
        }
        if probe.samples.len() == 360 {
            if env::var_os("FFONE_WORLD_TRANSFORM_AUDIT").is_some() {
                let placements: Vec<_> = world
                    .query::<(&Name, &SpawnedNativeWorldVisual, &GlobalTransform)>()
                    .iter(world)
                    .map(|(name, visual, transform)| {
                        serde_json::json!({
                            "name": name.as_str(),
                            "source": visual.source_model_path,
                            "model": visual.model_path,
                            "world": transform.to_matrix().to_cols_array(),
                        })
                    })
                    .collect();
                fs::write(
                    probe.output.join("placements.json"),
                    serde_json::to_vec_pretty(&placements).unwrap(),
                )
                .unwrap();
            }
            let mut sorted = probe.samples.clone();
            sorted.sort_by(f64::total_cmp);
            let all_meshes = world.query::<&Mesh3d>().iter(world).count();
            let visible_meshes = world
                .query::<(&Mesh3d, &ViewVisibility)>()
                .iter(world)
                .filter(|(_, v)| v.get())
                .count();
            let visual_placements = world
                .query::<&SpawnedNativeWorldVisual>()
                .iter(world)
                .count();
            let material_handles: HashSet<_> = world
                .query::<&MeshMaterial3d<LegacyModelMaterial>>()
                .iter(world)
                .map(|h| h.0.id())
                .collect();
            let mesh_handles: HashSet<_> = world
                .query::<&Mesh3d>()
                .iter(world)
                .map(|h| h.0.id())
                .collect();
            let images = world.resource::<Assets<Image>>();
            let image_bytes: usize = images
                .iter()
                .filter_map(|(_, i)| i.data.as_ref().map(Vec::len))
                .sum();
            let report = serde_json::json!({
                "schema": "ffone.world-performance.v1", "sharing": probe.enabled, "frozen": probe.frozen,
                "anchor": probe.anchor.to_array(), "viewport": [1280, 720], "far": EXTENDED_WORLD_CAMERA_FAR_NATIVE,
                "frames": sorted.len(), "meanMs": sorted.iter().sum::<f64>() / sorted.len() as f64,
                "p50Ms": sorted[sorted.len()/2], "p95Ms": sorted[sorted.len()*95/100], "p99Ms": sorted[sorted.len()*99/100],
                "readySeconds": ready_at.duration_since(probe.started).as_secs_f64(),
                "residentTiles": streaming.resident_tiles, "visualPlacements": visual_placements,
                "meshEntities": all_meshes, "visibleMeshEntities": visible_meshes,
                "boundMeshes": mesh_handles.len(), "boundNativeMaterials": material_handles.len(),
                "residentImages": images.len(), "residentImageCpuBytes": image_bytes,
                "sharingStatistics": static_asset_sharing_statistics(world), "frameMs": probe.samples,
            });
            if env::var_os("FFONE_PERF_ASSET_AUDIT").is_some() {
                material_audit::write(world, &probe.output);
            }
            fs::write(
                probe.output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!(
                "probe: mean={:.3}ms p95={:.3}ms meshes={} visible={} nativeMaterials={}",
                report["meanMs"].as_f64().unwrap(),
                report["p95Ms"].as_f64().unwrap(),
                all_meshes,
                visible_meshes,
                material_handles.len()
            );
            world.spawn(Screenshot::primary_window()).observe(save);
            probe.capture_issued = true;
        }
    }
    world.insert_resource(probe);
}

fn save(event: On<ScreenshotCaptured>, probe: Res<Probe>, mut exit: MessageWriter<AppExit>) {
    event
        .image
        .clone()
        .try_into_dynamic()
        .unwrap()
        .save(probe.output.join("frame.png"))
        .unwrap();
    println!("report and screenshot: {}", probe.output.display());
    exit.write(AppExit::Success);
}
