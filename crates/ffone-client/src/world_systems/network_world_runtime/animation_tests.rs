use super::*;
use crate::entity_lifecycle::NetworkNpcCombatClip0104 as Combat;
use bevy::{
    animation::AnimationPlugin, gltf::GltfPlugin, image::ImagePlugin, mesh::MeshPlugin,
    time::TimeUpdateStrategy, world_serialization::WorldSerializationPlugin,
};

struct Fixture {
    app: App,
    root: Entity,
    player: Entity,
}

impl Fixture {
    /// Load the production Bad Max rig through Bevy's real GLTF loader. The
    /// headless clock exercises its authored curves, durations and end events.
    fn bad_max() -> Self {
        let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let path = "characters/mobs/mob_ammonia/mob_ammonia.glb";
        let bytes = std::fs::read(asset_root.join(path)).unwrap();
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            TransformPlugin,
            AssetPlugin {
                file_path: asset_root.to_string_lossy().into_owned(),
                ..default()
            },
            WorldSerializationPlugin,
            ImagePlugin::default(),
            MeshPlugin,
            AnimationPlugin,
            GltfPlugin::default(),
            crate::native_gltf::NativeGltfPlugin,
        ))
        .init_asset::<StandardMaterial>()
        .register_type::<MeshMaterial3d<StandardMaterial>>()
        .init_resource::<NetworkNpcAnimationAssets0104>()
        .init_resource::<GameplayAudioRuntime>()
        .insert_resource(LegacyNanoStandRandomStream::with_seed(7))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_systems(
            Update,
            (
                advance_network_npc_motion_0104,
                play_network_npc_animation_0104,
                emit_network_npc_animation_sounds_0104,
            )
                .chain(),
        );
        app.finish();
        app.cleanup();
        let gltf: Handle<Gltf> = app.world().resource::<AssetServer>().load(path);
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        while app.world().resource::<Assets<Gltf>>().get(&gltf).is_none() {
            assert!(
                std::time::Instant::now() < deadline,
                "Bad Max GLTF failed to load"
            );
            app.update();
            std::thread::sleep(Duration::from_millis(2));
        }
        let root = app
            .world_mut()
            .spawn((
                NetworkNpc0104 {
                    npc_id: 1,
                    npc_type: 461,
                },
                NetworkNpcAppearance0104(ffone_protocol::NpcAppearance0104 {
                    npc_id: 1,
                    npc_type: 461,
                    hp: 1000,
                    condition_bit_flag: 0,
                    position: [0; 3],
                    angle: 0,
                    barker_type: 0,
                }),
                NetworkNpcAnimationLayers0104::default(),
                Transform::default(),
            ))
            .id();
        let scene = app
            .world()
            .resource::<Assets<Gltf>>()
            .get(&gltf)
            .unwrap()
            .scenes[0]
            .clone();
        app.world_mut()
            .spawn((WorldAssetRoot(scene), ChildOf(root)));
        app.world_mut()
            .entity_mut(root)
            .insert(NetworkNpcVisual0104 {
                npc_type: 461,
                logical_name: "mob_ammonia".into(),
                glb_path: path.into(),
                table_scale: 4.8,
                visual_container: root,
                scene: root,
                gltf,
                main_texture: None,
                sub_texture: None,
                walk_animation_speed: 1.0,
                run_animation_speed: 1.0,
                animation_effect_events: parse_network_npc_animation_effect_events(&bytes)
                    .unwrap()
                    .into(),
                animation_sound_events: parse_network_npc_animation_sound_events(&bytes)
                    .unwrap()
                    .into(),
                animation_ends: Arc::new(parse_network_npc_animation_end_events(&bytes).unwrap()),
                material_animation_clips: Arc::from([]),
            });
        let player = loop {
            app.update();
            if let Some(entity) = app
                .world_mut()
                .query_filtered::<Entity, With<NetworkNpcAnimationApplied0104>>()
                .iter(app.world())
                .next()
            {
                break entity;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "Bad Max scene failed to spawn"
            );
        };
        Self { app, root, player }
    }

    fn request(&mut self, clip: Combat) -> u64 {
        let request = self
            .app
            .world_mut()
            .get_mut::<NetworkNpcAnimationLayers0104>(self.root)
            .unwrap()
            .request(clip);
        self.app.world_mut().entity_mut(self.root).insert(request);
        request.revision
    }

    fn frames(&mut self, count: usize) {
        for _ in 0..count {
            self.app.update();
        }
    }

    fn high(&self, index: usize) -> &NetworkNpcAnimationApplied0104 {
        self.app
            .world()
            .get::<NetworkNpcHighAnimations0104>(self.player)
            .unwrap()
            .applied[index]
            .as_ref()
            .unwrap()
    }

    fn seek(&self, index: usize) -> f32 {
        self.app
            .world()
            .get::<AnimationPlayer>(self.player)
            .unwrap()
            .animation(self.high(index).node)
            .unwrap()
            .seek_time()
    }

    fn low(&self) -> &str {
        &self
            .app
            .world()
            .get::<NetworkNpcAnimationApplied0104>(self.player)
            .unwrap()
            .clip
    }
}

#[test]
fn npc_bad_max_idle_finishes_each_full_clip_before_selecting_another() {
    let mut f = Fixture::bad_max();
    let mut changes = 0;
    for _ in 0..720 {
        let applied = f.app.world().get::<NetworkNpcAnimationApplied0104>(f.player).unwrap().clone();
        let active = f.app.world().get::<AnimationPlayer>(f.player).unwrap().animation(applied.node).unwrap();
        let completed = active.completions();
        f.frames(1);
        if f.low() != applied.clip {
            assert!(completed > 0, "{} changed before its full authored cycle completed", applied.clip);
            changes += 1;
        }
    }
    assert!(changes > 0, "idle alternatives must still play after completed cycles");
}

#[test]
fn npc_bad_max_walk_and_run_preserve_time_across_motion_packets() {
    let mut f = Fixture::bad_max();
    for (style, clip) in [(0, "walk"), (1, "run")] {
        for _ in 0..12 {
            f.app.world_mut().entity_mut(f.root).insert(NetworkNpcMotion0104 {
                destination: Vec3::new(100.0, 0.0, 0.0), speed: 3.0, move_style: style,
            });
            f.frames(1);
            assert_eq!(f.low(), clip);
        }
        let applied = f.app.world().get::<NetworkNpcAnimationApplied0104>(f.player).unwrap();
        let active = f.app.world().get::<AnimationPlayer>(f.player).unwrap().animation(applied.node).unwrap();
        assert!(active.seek_time() > 0.15, "motion packets must not rewind {clip}");
    }
}

#[test]
fn npc_bad_max_repeated_damage_preserves_time_and_completes() {
    let mut f = Fixture::bad_max();
    f.request(Combat::Wound);
    f.frames(8);
    let first = f.seek(1);
    let playback = f.high(1).combat_revision;
    f.request(Combat::Wound);
    f.frames(8);
    assert!(
        f.seek(1) > first + 0.10,
        "a second hit must not rewind wound"
    );
    assert_eq!(
        f.high(1).combat_revision,
        playback,
        "sounds retain the same playback cursor"
    );
    for _ in 0..6 {
        f.request(Combat::Wound);
        f.frames(2);
    }
    assert_eq!(
        f.low(),
        "ready",
        "frequent damage must reach authored end at 0.416667 s"
    );
}

#[test]
fn npc_bad_max_attack_and_wound_keep_both_layers_and_latest_end_owner() {
    let mut f = Fixture::bad_max();
    f.request(Combat::Melee);
    f.frames(30);
    let attack_node = f.high(0).node;
    let attack_time = f.seek(0);
    let wound = f.request(Combat::Wound);
    f.frames(15);
    assert!(
        f.app
            .world()
            .get::<AnimationPlayer>(f.player)
            .unwrap()
            .animation(attack_node)
            .is_some()
    );
    assert!(f.seek(0) > attack_time);
    assert_eq!(
        f.app
            .world()
            .get::<NetworkNpcAnimationLayers0104>(f.root)
            .unwrap()
            .high_owner,
        Some(wound),
        "old melee end must not consume the newer wound owner"
    );
    f.frames(12);
    assert_eq!(f.low(), "ready");
    assert_eq!(
        f.app
            .world()
            .get::<NetworkNpcAnimationLayers0104>(f.root)
            .unwrap()
            .high_owner,
        None
    );

    f.frames(40);
    f.request(Combat::Wound);
    let attack = f.request(Combat::Melee);
    f.frames(25);
    assert_eq!(
        f.app
            .world()
            .get::<NetworkNpcAnimationLayers0104>(f.root)
            .unwrap()
            .high_owner,
        Some(attack),
        "wound end must not consume the newer melee owner, even in one packet batch"
    );
    f.frames(25);
    assert_eq!(f.low(), "ready");
}

#[test]
fn npc_bad_max_high_layer_preserves_skill_and_skill_end_returns_to_stand() {
    let mut f = Fixture::bad_max();
    f.request(Combat::Skill);
    f.frames(3);
    f.request(Combat::Wound);
    f.frames(3);
    assert_eq!(
        f.low(),
        "skill0",
        "damage must not replace the skill's base pose"
    );
    f.frames(27);
    assert_eq!(
        f.low(),
        "ready",
        "current high-layer end owns the continuation"
    );
    f.request(Combat::Skill);
    f.frames(55);
    assert!(
        f.low().starts_with("stand"),
        "low-layer skill end selects stand, not ready"
    );
}

#[test]
fn npc_bad_max_attack_holds_motion_and_later_request_is_not_stale() {
    let mut f = Fixture::bad_max();
    f.app
        .world_mut()
        .entity_mut(f.root)
        .insert(NetworkNpcMotion0104 {
            destination: Vec3::new(100.0, 0.0, 0.0),
            speed: 3.0,
            move_style: 1,
        });
    f.frames(3);
    let first_request = f.request(Combat::Melee);
    let before = f.app.world().get::<Transform>(f.root).unwrap().translation;
    f.frames(20);
    assert_eq!(
        f.app.world().get::<Transform>(f.root).unwrap().translation,
        before
    );
    assert!(
        f.app.world().get::<NetworkNpcMotion0104>(f.root).is_some(),
        "hold retains destination"
    );
    f.frames(50);
    assert!(
        f.app
            .world()
            .get::<Transform>(f.root)
            .unwrap()
            .translation
            .x
            > before.x
    );
    let second_request = f.request(Combat::Melee);
    assert!(second_request > first_request);
    f.frames(2);
    assert_eq!(f.high(0).combat_revision, Some(second_request));
    assert!(f.seek(0) < 0.1);
}

#[test]
fn npc_production_skill_selection_uses_bad_max_table_animation() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let table: Value =
        crate::xdt::from_slice(&std::fs::read(root.join(TABLE_SET_PATH)).unwrap()).unwrap();
    let registry =
        ffone_client_foundation::asset_tables::character_models_from_document(&table).unwrap();
    let textures: Value =
        serde_json::from_slice(&std::fs::read(root.join(NPC_TEXTURE_CATALOG_PATH)).unwrap())
            .unwrap();
    let catalog = NetworkNpcVisualCatalog0104::from_documents(
        &table,
        &registry,
        &textures,
        |_| Ok(()),
        |_| Ok(()),
    )
    .unwrap();
    assert_eq!(
        catalog.get(461).unwrap().glb,
        "characters/mobs/mob_ammonia/mob_ammonia.glb"
    );
    let skills = catalog.skill_animations(461).unwrap();
    assert_eq!(skills.hit(114), Some(Combat::Skill1));
    assert_eq!(skills.ready(114), Combat::SkillReady);
    assert_eq!(skills.hit(0), None);
    assert_eq!(
        skills.hit(32767),
        None,
        "unknown skill must not substitute skill0"
    );
    assert_eq!(Combat::Mega.name(), "skill0");
}

#[test]
fn npc_bad_max_death_cannot_be_reset_by_old_high_layer_end() {
    let mut f = Fixture::bad_max();
    f.request(Combat::Melee);
    f.frames(5);
    f.app
        .world_mut()
        .get_mut::<NetworkNpcAppearance0104>(f.root)
        .unwrap()
        .0
        .hp = 0;
    f.request(Combat::Wound);
    f.frames(210);
    assert_eq!(f.low(), "death");
    let low = f
        .app
        .world()
        .get::<NetworkNpcAnimationApplied0104>(f.player)
        .unwrap();
    let player = f.app.world().get::<AnimationPlayer>(f.player).unwrap();
    assert!(player.animation(low.node).unwrap().is_finished());
    assert!(
        f.app
            .world()
            .get::<NetworkNpcReadyAnimation0104>(f.root)
            .is_none()
    );
}

#[test]
fn npc_bad_max_reselecting_active_melee_keeps_its_sound_cursor() {
    let mut f = Fixture::bad_max();
    // Pin both draws to the sole available variant, so this tests Play's
    // same-name behavior independently of the random melee1/melee2 selection.
    let gltf_id = f
        .app
        .world()
        .get::<NetworkNpcVisual0104>(f.root)
        .unwrap()
        .gltf
        .id();
    f.app
        .world_mut()
        .resource_mut::<NetworkNpcAnimationAssets0104>()
        .graphs
        .get_mut(&gltf_id)
        .unwrap()
        .nodes
        .remove("melee2");
    f.request(Combat::Melee);
    f.frames(20);
    let count = |f: &Fixture| {
        f.app
            .world()
            .resource::<GameplayAudioRuntime>()
            .queued_animation_sound_names()
            .iter()
            .filter(|(name, _)| name.contains("Melee"))
            .count()
    };
    assert_eq!(count(&f), 1);
    f.request(Combat::Melee);
    f.frames(15);
    assert_eq!(
        count(&f),
        1,
        "same active melee must not replay its earlier sound"
    );
    f.frames(35);
    f.request(Combat::Melee);
    f.frames(20);
    assert_eq!(
        count(&f),
        2,
        "a new playback after completion owns a new sound"
    );
}

#[test]
fn npc_bad_max_reused_visual_drops_previous_spawn_high_layers() {
    let mut f = Fixture::bad_max();
    let old_revision = f.request(Combat::Melee);
    f.frames(3);
    let old_node = f.high(0).node;
    f.app
        .world_mut()
        .get_mut::<NetworkNpcAnimationLayers0104>(f.root)
        .unwrap()
        .reset();
    let new_revision = f.request(Combat::Wound);
    f.frames(2);
    assert!(new_revision > old_revision);
    assert!(
        f.app
            .world()
            .get::<AnimationPlayer>(f.player)
            .unwrap()
            .animation(old_node)
            .is_none()
    );
    assert_eq!(f.high(1).combat_revision, Some(new_revision));
}
