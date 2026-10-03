//! Native regression for the tutorial Fusion Portal's deferred material reveal.
use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
};
use ffone_client::{
    assets::AssetLocator,
    character_scene::{LegacyCharacterSceneDeferredReveal, LegacyCharacterSceneStatus},
    legacy_model_material::{
        LegacyMaterialApplied, LegacyMaterialMetadataError, LegacyModelMaterial,
        LegacyModelMaterialPlugin,
    },
    network_world_runtime::{
        NetworkNpcMaterialSurface0104, NetworkNpcTextureVariantBound0104, NetworkNpcVisual0104,
        NetworkNpcVisualCatalog0104, NetworkNpcVisualCatalogState0104,
        bind_network_npc_texture_variants_0104, finalize_network_npc_material_visibility_0104,
        spawn_network_npc_visual_0104,
    },
    tutorial_actors::{
        TutorialActor, TutorialActorCommandQueue, TutorialActorPlugin,
        apply_tutorial_actor_animation_playback, spawn_tutorial_actor_visuals,
    },
    tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn},
    tutorial_mission_content::TutorialMissionContent,
    world::{
        NativeWorldCatalog, NativeWorldPlugin, NativeWorldPresentationStatus, NativeWorldScope,
        load_native_world_scenes, spawn_native_world_scene,
    },
};
use std::{fs, path::PathBuf, time::Instant};

#[derive(Resource)]
struct Probe {
    start: Instant,
    output: PathBuf,
    ready_frames: u32,
    captured: bool,
    world: bool,
    actor_spawned: bool,
    recreated: bool,
}

fn main() {
    let root = fs::canonicalize("assets/game").unwrap();
    let output = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "target/performance/fusiongate/runtime.png".into()),
    );
    fs::create_dir_all(output.parent().unwrap()).unwrap();
    let locator = AssetLocator::open(root.clone()).unwrap();
    let catalog = NetworkNpcVisualCatalog0104::open(&locator).unwrap();
    let definition = catalog.get(2695).unwrap().clone();
    let world_mode = std::env::args().nth(2).as_deref() == Some("--world");
    let world_catalog = world_mode.then(|| load_native_world_scenes(&root).unwrap());
    let content = TutorialMissionContent::open(&locator).unwrap();
    let mut visual_catalog = NetworkNpcVisualCatalogState0104::default();
    visual_catalog.attempted = true;
    visual_catalog.catalog = Some(catalog);
    let mut app = App::new();
    app.insert_resource(Probe {
        start: Instant::now(),
        output,
        ready_frames: 0,
        captured: false,
        world: world_mode,
        actor_spawned: false,
        recreated: false,
    })
    .insert_resource(content)
    .insert_resource(visual_catalog)
    .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.055)))
    .add_plugins(DefaultPlugins.set(AssetPlugin {
        file_path: root.to_string_lossy().into_owned(),
        ..default()
    }))
    .add_plugins(LegacyModelMaterialPlugin)
    .add_plugins(TutorialActorPlugin)
    .add_systems(
        Startup,
        move |mut commands: Commands,
              server: Res<AssetServer>,
              world: Option<Res<NativeWorldCatalog>>| {
            if let Some(catalog) = world.as_deref() {
                let scene = catalog
                    .select_in_scope(
                        NativeWorldScope::Tutorial,
                        Vec3::new(-564.31, -90.0, 724.16),
                    )
                    .unwrap();
                spawn_native_world_scene(&mut commands, &server, catalog, scene).unwrap();
            } else {
                let root = commands
                    .spawn((Transform::default(), Visibility::Inherited))
                    .id();
                let visual = spawn_network_npc_visual_0104(
                    &mut commands,
                    &server,
                    root,
                    &definition,
                    "tutorial Fusion Portal",
                );
                commands.entity(root).insert(visual);
            }
            let offset = if world_mode {
                Vec3::new(-564.31, -91.0, 724.16)
            } else {
                Vec3::ZERO
            };
            let camera_offset = if world_mode {
                Vec3::new(-13.0, 5.0, 21.0)
            } else {
                Vec3::new(0.0, 4.0, -18.0)
            };
            let target = commands.spawn(Transform::from_translation(offset)).id();
            commands.spawn((
                Camera3d::default(),
                ffone_client::movement::LegacyOrbitCamera::new(target),
                Transform::from_translation(offset + camera_offset)
                    .looking_at(offset + Vec3::new(0.0, 3.0, 0.0), Vec3::Y),
            ));
        },
    )
    .add_systems(
        Update,
        (
            spawn_tutorial_actor_visuals,
            apply_tutorial_actor_animation_playback,
            bind_network_npc_texture_variants_0104,
            finalize_network_npc_material_visibility_0104,
            inspect,
        )
            .chain(),
    );
    if world_mode {
        app.insert_resource(world_catalog.unwrap());
        app.add_plugins(NativeWorldPlugin);
        let effects =
            ffone_client::tutorial_effects_runtime::TutorialEffectLibrary::load(&root).unwrap();
        app.insert_resource(effects.clone());
        app.insert_resource(
            ffone_client::tutorial_effects_runtime::TutorialEffectRuntime::with_library(effects),
        );
        app.add_plugins(ffone_client::tutorial_effects_runtime::TutorialEffectsRuntimePlugin);
        app.insert_resource(ffone_client::world_behaviour::NativeWorldBehaviourRoot(
            root.clone(),
        ));
        ffone_client::world_behaviour::add_world_preview_behaviour_systems(&mut app);
    }
    app.run();
}

fn inspect(
    mut commands: Commands,
    mut probe: ResMut<Probe>,
    mut actors: ResMut<TutorialActorCommandQueue>,
    world_scenes: Query<&NativeWorldPresentationStatus>,
    actor_positions: Query<(&TutorialActor, &Transform)>,
    roots: Query<&NetworkNpcVisual0104>,
    scenes: Query<(
        &LegacyCharacterSceneStatus,
        &Visibility,
        Option<&LegacyCharacterSceneDeferredReveal>,
    )>,
    errors: Query<&LegacyMaterialMetadataError>,
    surfaces: Query<
        (
            Option<&LegacyMaterialApplied>,
            Option<&NetworkNpcTextureVariantBound0104>,
        ),
        (
            With<MeshMaterial3d<LegacyModelMaterial>>,
            With<NetworkNpcMaterialSurface0104>,
        ),
    >,
) {
    if probe.captured {
        return;
    }
    if probe.world
        && !probe.actor_spawned
        && !world_scenes.is_empty()
        && world_scenes
            .iter()
            .all(|status| *status == NativeWorldPresentationStatus::Ready)
    {
        actors.spawn(TutorialNpcSpawn::new(
            1012,
            2695,
            LegacySpawnPosition::centiunits(56_431, 72_416, -9_100),
            None,
        ));
        probe.actor_spawned = true;
    }
    let ready = !roots.is_empty()
        && roots.iter().all(|visual| {
            scenes
                .get(visual.scene)
                .is_ok_and(|(status, visibility, deferred)| {
                    matches!(status, LegacyCharacterSceneStatus::Ready { .. })
                        && *visibility != Visibility::Hidden
                        && deferred.is_none()
                })
        });
    if ready && !roots.is_empty() {
        probe.ready_frames += 1;
    }
    if probe.world && !probe.recreated && probe.ready_frames == 90 {
        // InfectionA deletes its flythrough portal; the ordinary warp later
        // creates a new actor with the same semantic ID and native model.
        actors.delete(1012);
        probe.actor_spawned = false;
        probe.recreated = true;
        probe.ready_frames = 0;
        eprintln!("recreating portal after flythrough deletion");
        return;
    }
    let timeout = probe.start.elapsed().as_secs() >= if probe.world { 120 } else { 25 };
    if probe.ready_frames < if probe.world { 600 } else { 90 } && !timeout {
        return;
    }
    for visual in &roots {
        eprintln!("portal scene: {:?}", scenes.get(visual.scene));
    }
    eprintln!(
        "world presentation: {:?}",
        world_scenes.iter().collect::<Vec<_>>()
    );
    for (actor, transform) in &actor_positions {
        eprintln!("actor {} position {:?}", actor.id, transform.translation);
    }
    for error in &errors {
        eprintln!("material error: {error:?}");
    }
    for (applied, bound) in &surfaces {
        eprintln!(
            "portal material: applied={} bound={}",
            applied.is_some(),
            bound.is_some()
        );
    }
    eprintln!("portal revealed={ready}");
    probe.captured = true;
    let output = probe.output.clone();
    commands.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            event
                .image
                .clone()
                .try_into_dynamic()
                .unwrap()
                .save(&output)
                .unwrap();
            exit.write(if ready {
                AppExit::Success
            } else {
                AppExit::error()
            });
        },
    );
}
