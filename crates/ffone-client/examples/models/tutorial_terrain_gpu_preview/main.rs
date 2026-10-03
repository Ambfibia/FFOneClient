//! GPU diagnostic for one native tutorial world tile.
//!
//! The full view loads the static world geometry that covers the heightfield
//! at the minimap player warp. Terrain-only diagnostic views keep those models
//! isolated so compositor changes remain directly comparable.

use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    movement::LegacyOrbitCamera,
    native_terrain::{
        NativeHeightmapTerrain, NativeTerrain, NativeTerrainMaterial,
        spawn_pending_native_heightmap,
    },
    world::{
        NativeWorldCatalog, NativeWorldPlugin, NativeWorldScope, load_native_world_scenes,
        spawn_native_world_scene,
    },
};

const READY_WARMUP_FRAMES: u32 = 120;
const TIMEOUT_FRAMES: u32 = 600;

#[derive(Debug, Clone, Copy)]
enum TilePreset {
    Tutorial,
    PastSectorVPool,
    FuturePokeyOaksPool,
    FuturePokeyOaksNorthPools,
}

impl TilePreset {
    fn parse(value: Option<&str>) -> Option<Self> {
        match value.unwrap_or("tutorial") {
            "tutorial" => Some(Self::Tutorial),
            "sector-v-pool" => Some(Self::PastSectorVPool),
            "pool" => Some(Self::FuturePokeyOaksPool),
            "pool-north" => Some(Self::FuturePokeyOaksNorthPools),
            _ => None,
        }
    }

    const fn terrain(self) -> (&'static str, &'static str) {
        match self {
            Self::Tutorial => (
                "map/tiles/map_01_01/tile.json",
                "map/tiles/map_01_01/terrain/terrain.json",
            ),
            Self::PastSectorVPool => (
                "map/tiles/map_07_08/tile.json",
                "map/tiles/map_07_08/terrain/terrain.json",
            ),
            Self::FuturePokeyOaksPool => (
                "map/tiles/map_12_01/tile.json",
                "map/tiles/map_12_01/terrain/terrain.json",
            ),
            Self::FuturePokeyOaksNorthPools => (
                "map/tiles/map_12_02/tile.json",
                "map/tiles/map_12_02/terrain/terrain.json",
            ),
        }
    }
}

fn authoritative_terrain_hash(asset_root: &Path, tile: TilePreset) -> String {
    let (manifest_path, expected_terrain_path) = tile.terrain();
    let bytes = fs::read(asset_root.join(manifest_path))
        .unwrap_or_else(|error| panic!("cannot read {manifest_path}: {error}"));
    let manifest: serde_json::Value = serde_json::from_slice(&bytes)
        .unwrap_or_else(|error| panic!("cannot parse {manifest_path}: {error}"));
    let terrain = manifest
        .get("terrain")
        .unwrap_or_else(|| panic!("{manifest_path} has no terrain artifact"));
    assert_eq!(
        terrain.get("path").and_then(serde_json::Value::as_str),
        Some(expected_terrain_path),
        "{manifest_path} terrain path drifted"
    );
    terrain
        .get("blake3")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("{manifest_path} terrain artifact has no hash"))
        .to_owned()
}

#[derive(Debug, Clone, Copy)]
enum DebugView {
    Full,
    Terrain,
    Surface,
    TotalWeights,
    SplatOnly,
    WeightMap(u8),
    LayerWeight(u8),
}

impl DebugView {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "full" => Some(Self::Full),
            "terrain" => Some(Self::Terrain),
            "surface" => Some(Self::Surface),
            "weights" => Some(Self::TotalWeights),
            "splat" => Some(Self::SplatOnly),
            "weight0" => Some(Self::WeightMap(0)),
            "weight1" => Some(Self::WeightMap(1)),
            "weight2" => Some(Self::WeightMap(2)),
            "weight3" => Some(Self::WeightMap(3)),
            "weight4" => Some(Self::WeightMap(4)),
            _ => value
                .strip_prefix("layer")
                .and_then(|index| index.parse::<u8>().ok())
                .filter(|index| *index < 17)
                .map(Self::LayerWeight),
        }
    }

    const fn shader_value(self) -> f32 {
        match self {
            Self::Full => 0.0,
            Self::Terrain => 0.0,
            Self::Surface => 1.0,
            Self::TotalWeights => 2.0,
            Self::SplatOnly => 3.0,
            Self::WeightMap(index) => 10.0 + index as f32,
            Self::LayerWeight(index) => 20.0 + index as f32,
        }
    }
}

#[derive(Resource)]
struct PreviewConfig {
    output: PathBuf,
    debug_view: DebugView,
    camera: CameraPreset,
    tile: TilePreset,
}

#[derive(Clone, Copy, Debug)]
enum CameraPreset {
    Overview,
    Near,
    TechSquare,
    Pool,
}

impl CameraPreset {
    fn parse(value: Option<&str>) -> Option<Self> {
        match value.unwrap_or("overview") {
            "overview" => Some(Self::Overview),
            "near" => Some(Self::Near),
            "tech-square" => Some(Self::TechSquare),
            "pool" => Some(Self::Pool),
            _ => None,
        }
    }
}

#[derive(Resource, Default)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
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
        .unwrap_or_else(|| PathBuf::from("target/tutorial-terrain-preview.png"));
    let debug_view = args
        .next()
        .and_then(|value| value.into_string().ok())
        .as_deref()
        .map(DebugView::parse)
        .unwrap_or(Some(DebugView::Full))
        .unwrap_or_else(|| {
            eprintln!(
                "view must be full, terrain, surface, weights, splat, weight0..weight4 or layer0..layer16"
            );
            std::process::exit(2);
        });
    let camera = CameraPreset::parse(
        args.next()
            .and_then(|value| value.into_string().ok())
            .as_deref(),
    )
    .unwrap_or_else(|| {
        eprintln!("camera must be overview, near, tech-square or pool");
        std::process::exit(2);
    });
    let tile = TilePreset::parse(
        args.next()
            .and_then(|value| value.into_string().ok())
            .as_deref(),
    )
    .unwrap_or_else(|| {
        eprintln!("tile must be tutorial, sector-v-pool, pool or pool-north");
        std::process::exit(2);
    });
    if args.next().is_some() {
        eprintln!(
            "usage: tutorial_terrain_gpu_preview [ASSET_ROOT] [OUTPUT.png] [VIEW] [overview|near|tech-square|pool] [tutorial|sector-v-pool|pool|pool-north]"
        );
        std::process::exit(2);
    }
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        eprintln!("output must end in .png");
        std::process::exit(2);
    }
    let asset_root = fs::canonicalize(&asset_root).unwrap_or_else(|error| {
        panic!(
            "cannot resolve asset root {}: {error}",
            asset_root.display()
        )
    });
    let catalog = load_native_world_scenes(&asset_root)
        .unwrap_or_else(|error| panic!("cannot open native world catalog: {error}"));
    let (_, terrain_path) = tile.terrain();
    let terrain_blake3 = authoritative_terrain_hash(&asset_root, tile);
    let terrain = Arc::new(
        NativeTerrain::open(&asset_root, terrain_path, &terrain_blake3)
            .unwrap_or_else(|error| panic!("cannot open isolated terrain: {error}")),
    );
    if output.exists() {
        fs::remove_file(&output)
            .unwrap_or_else(|error| panic!("cannot replace {}: {error}", output.display()));
    }

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.04, 0.055)))
        .insert_resource(PreviewConfig {
            output,
            debug_view,
            camera,
            tile,
        })
        .insert_resource(PreviewState::default())
        .insert_resource::<NativeWorldCatalog>(catalog)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("FFOne terrain {tile:?} {debug_view:?}"),
                        resolution: WindowResolution::new(1024, 768),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(NativeWorldPlugin)
        .add_systems(
            Startup,
            move |mut commands: Commands,
                  asset_server: Res<AssetServer>,
                  catalog: Res<NativeWorldCatalog>| {
                setup(
                    &mut commands,
                    &asset_server,
                    &catalog,
                    terrain.clone(),
                    debug_view,
                    camera,
                    tile,
                )
            },
        )
        .add_systems(Update, drive_capture)
        .run();
}

fn setup(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeWorldCatalog,
    terrain: Arc<NativeTerrain>,
    debug_view: DebugView,
    camera: CameraPreset,
    tile: TilePreset,
) {
    if matches!(debug_view, DebugView::Full) {
        let (scope, player_position) = match tile {
            TilePreset::Tutorial => (NativeWorldScope::Tutorial, Vec3::new(-566.0, -101.0, 665.0)),
            TilePreset::PastSectorVPool => (
                NativeWorldScope::WorldMap,
                Vec3::new(-3898.34, -56.23, 4475.21),
            ),
            TilePreset::FuturePokeyOaksPool => (
                NativeWorldScope::WorldMap,
                Vec3::new(-6374.318_8, -56.474_6, 668.203_9),
            ),
            TilePreset::FuturePokeyOaksNorthPools => (
                NativeWorldScope::WorldMap,
                Vec3::new(-6400.0, -50.0, 1280.0),
            ),
        };
        let scene = catalog
            .select_in_scope(scope, player_position)
            .expect("selected terrain preview tile must cover its camera target");
        spawn_native_world_scene(commands, asset_server, catalog, scene)
            .expect("complete tutorial tile_01_01 must spawn");
    } else {
        let (name, root_translation, terrain_name) = match tile {
            TilePreset::Tutorial => (
                "isolated tile_01_01 root",
                Vec3::new(-512.0, 0.0, 512.0),
                "isolated TerrainData_01_01",
            ),
            TilePreset::PastSectorVPool => (
                "isolated map_07_08 root",
                Vec3::new(-3584.0, 0.0, 4096.0),
                "isolated TerrainData_07_08",
            ),
            TilePreset::FuturePokeyOaksPool => (
                "isolated map_12_01 root",
                Vec3::new(-6144.0, 0.0, 512.0),
                "isolated TerrainData_12_01",
            ),
            TilePreset::FuturePokeyOaksNorthPools => (
                "isolated map_12_02 root",
                Vec3::new(-6144.0, 0.0, 1024.0),
                "isolated TerrainData_12_02",
            ),
        };
        let root = commands
            .spawn((
                Name::new(name),
                Transform::from_translation(root_translation),
                Visibility::Inherited,
            ))
            .id();
        spawn_pending_native_heightmap(
            commands,
            root,
            terrain_name,
            Transform::from_xyz(0.0, -300.0, 0.0),
            terrain,
        );
    }

    let (camera_position, target, orbit_position) = match camera {
        CameraPreset::Overview => (
            Vec3::new(-768.0, 360.0, 310.0),
            Vec3::new(-768.0, 0.0, 768.0),
            Vec3::new(-768.0, 0.0, 768.0),
        ),
        CameraPreset::Near => {
            // The minimap chapter warps the player to protocol
            // [56_600, 66_500, -10_100], native [-566, -101, 665].
            // Reproduce the default Retrobution orbit pose while facing the
            // Numbuh Two waypoint instead of placing the diagnostic camera at
            // the NPC, which frames a different authored part of tile_01_01.
            let player = Vec3::new(-566.0, -101.0, 665.0);
            (
                Vec3::new(-557.88, -97.41, 657.94),
                player + Vec3::Y * 1.4,
                player,
            )
        }
        CameraPreset::TechSquare => {
            // Numbuh Two's exact tutorial position is native
            // [-651, -82, 739]. A pulled-back orbit frames the authored road
            // and plaza cells visible in the Retrobution reference without
            // loading the complete tutorial progression.
            let player = Vec3::new(-651.0, -82.0, 739.0);
            (
                Vec3::new(-634.0, -69.0, 722.0),
                player + Vec3::Y * 1.4,
                player,
            )
        }
        CameraPreset::Pool => {
            match tile {
                TilePreset::PastSectorVPool => {
                    // Exact Past Sector V pool/diving-board neighborhood from
                    // primary Map_07_08. This crop exposes the vertical rim
                    // generated by TerrainData m_Heightmap.m_Shifts.
                    let pool = Vec3::new(-3898.34, -56.23, 4475.21);
                    (Vec3::new(-3865.0, -25.0, 4415.0), pool, pool)
                }
                TilePreset::FuturePokeyOaksNorthPools => {
                    // High diagnostic overview of the user-reported Pokey Oaks
                    // North terrain. The four RGBA-authored pool surfaces live
                    // in the center of primary TerrainData_12_02.
                    let pools = Vec3::new(-6400.0, -55.0, 1280.0);
                    (Vec3::new(-6330.0, 210.0, 1170.0), pools, pools)
                }
                _ => {
                    // Exact Future Pokey Oaks South pool anchor BuildPlayer-
                    // Map_12_01#7635. The pulled-back crop covers the alpha-authored
                    // Surface_01D_1 deck around the diving board.
                    let pool = Vec3::new(-6374.318_8, -56.474_6, 668.203_9);
                    (Vec3::new(-6348.0, -37.0, 640.0), pool, pool)
                }
            }
        }
    };
    let orbit_target = commands
        .spawn((
            Name::new("isolated terrain camera target"),
            Transform::from_translation(orbit_position),
            Visibility::Hidden,
        ))
        .id();
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 45_f32.to_radians(),
            far: 2_000.0,
            ..default()
        }),
        LegacyOrbitCamera::new(orbit_target),
        Transform::from_translation(camera_position).looking_at(target, Vec3::Y),
    ));
}

fn drive_capture(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    catalog: Res<NativeWorldCatalog>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    terrains: Query<&MeshMaterial3d<NativeTerrainMaterial>, With<NativeHeightmapTerrain>>,
    mut materials: ResMut<Assets<NativeTerrainMaterial>>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    if terrains.is_empty() {
        if state.frames >= TIMEOUT_FRAMES {
            eprintln!("isolated terrain did not materialize");
            exit.write(AppExit::error());
        }
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    let ambience = cameras.single().ok().map(|camera| {
        let tutorial = matches!(config.tile, TilePreset::Tutorial);
        let sample = catalog.sample_ambience(camera.translation(), tutorial);
        let applied = catalog.applied_ambience(camera.translation(), tutorial);
        (
            Vec4::from_array(applied.light_color),
            Vec4::new(
                sample.fog_color[0],
                sample.fog_color[1],
                sample.fog_color[2],
                if applied.fog_enabled {
                    applied.fog_density
                } else {
                    0.0
                },
            ),
        )
    });
    let mut ready_materials = 0_usize;
    for handle in &terrains {
        let Some(mut material) = materials.get_mut(handle) else {
            continue;
        };
        if let Some((ambience_light, ambience_fog)) = ambience {
            material.uniform.ambience_light = ambience_light;
            material.uniform.ambience_fog = ambience_fog;
        }
        material.uniform.metadata.w = config.debug_view.shader_value();
        ready_materials += 1;
    }
    if ready_materials != terrains.iter().count() {
        std::thread::sleep(Duration::from_millis(16));
        return;
    }
    let current_frame = state.frames;
    let ready = *state.ready_frame.get_or_insert(current_frame);
    if !state.capture_issued && state.frames.saturating_sub(ready) >= READY_WARMUP_FRAMES {
        println!(
            "terrain ready: view={:?}, camera={:?}, materialDebugValue={}",
            config.debug_view,
            config.camera,
            config.debug_view.shader_value()
        );
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.frames >= TIMEOUT_FRAMES {
        eprintln!("isolated terrain capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(16));
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    if let Some(parent) = config
        .output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&config.output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", config.output.display()));
    println!("{}", absolute_display(&config.output));
    state.capture_saved = true;
}

fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}
