use super::*;
use crate::legacy_model_material::{
    LegacyModelMaterial, LegacyModelMaterialParams, LegacyModelTextures, LegacyShaderKind,
    LegacyStaticMaterialCache, ModelMaterialSharing, PendingLegacyStaticWorldMaterial,
};
use bevy::camera::visibility::SetViewVisibility;

fn player(renderer: Entity, value: f64) -> WorldAnimationPlayer {
    WorldAnimationPlayer {
        play_automatically: true,
        animate_physics: false,
        animate_only_if_visible: false,
        wrap_mode: 2,
        clip_count: 1,
        model_entities: vec![renderer],
        default_clip: None,
        clip_refs: vec![],
        default_clip_id: None,
        clip_ids: vec![],
        targets: HashMap::from([(
            "target".into(),
            WorldAnimationTargetBinding {
                entity: renderer,
                model_entities: vec![renderer],
            },
        )]),
        elapsed_seconds: 0.0,
        transform_repeats: false,
        active_clip: Some(Arc::new(WorldAnimationClip {
            id: "test".into(),
            name: "test".into(),
            duration: 1.0,
            sample_rate: 30.0,
            looped: true,
            channels: vec![],
            events: vec![],
            float_curves: vec![WorldAnimationFloatCurve {
                target_path: "target".into(),
                class_id: 21,
                attribute: "_Color.r".into(),
                pre_infinity: 0,
                post_infinity: 0,
                times: vec![0.0],
                values: vec![value],
                in_tangents: vec![0.0],
                out_tangents: vec![0.0],
            }],
        })),
    }
}

fn fixture(
    shared: bool,
    overlap: bool,
) -> (App, Vec<Entity>, Vec<Entity>, Handle<LegacyModelMaterial>) {
    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<Assets<LegacyModelMaterial>>()
        .insert_resource(ModelMaterialSharing(shared))
        .add_systems(
            Update,
            (
                prepare_world_animation_material_bindings,
                update_world_animation_materials,
            )
                .chain(),
        );
    let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::OpaqueNormal);
    let mut cache = LegacyStaticMaterialCache::default();
    let material = cache.admit_immutable(
        params
            .material_for_pass(
                params.render_plan().passes[0],
                &LegacyModelTextures::default(),
            )
            .unwrap(),
        &mut app
            .world_mut()
            .resource_mut::<Assets<LegacyModelMaterial>>(),
    );
    app.insert_resource(cache);
    let root = app
        .world_mut()
        .spawn(crate::world::NativeWorldPresentationStatus::Ready)
        .id();
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/objects/collections/dt_highway_02/models/dt_highway_01_10_default/visual.glb");
    let bytes = std::fs::read(path).unwrap();
    let length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let doc: serde_json::Value = serde_json::from_slice(&bytes[20..20 + length]).unwrap();
    let pending = PendingLegacyStaticWorldMaterial::from_gltf_extras(
        doc["materials"][0]["name"].as_str(),
        &doc["materials"][0]["extras"].to_string(),
    )
    .unwrap();
    let mut renderers = Vec::new();
    for _ in 0..if overlap { 1 } else { 2 } {
        renderers.push(
            app.world_mut()
                .spawn((
                    Mesh3d::default(),
                    MeshMaterial3d(material.clone()),
                    ViewVisibility::HIDDEN,
                    pending.clone(),
                    crate::world::SpawnedNativeWorldVisual {
                        model_path: "test.glb".into(),
                        source_model_path: "test.glb".into(),
                        scene: 0,
                    },
                    crate::world::NativeWorldVisualSceneReady,
                ))
                .id(),
        );
    }
    let mut players = Vec::new();
    for (index, value) in [1.0, 0.75].into_iter().enumerate() {
        players.push(
            app.world_mut()
                .spawn((
                    ChildOf(root),
                    player(renderers[index % renderers.len()], value),
                    WorldAnimationMaterialBindings::default(),
                    WorldAnimationVisibilityBindings::default(),
                ))
                .id(),
        );
    }
    (app, players, renderers, material)
}

fn bound(app: &App, entity: Entity) -> Handle<LegacyModelMaterial> {
    app.world()
        .get::<MeshMaterial3d<LegacyModelMaterial>>(entity)
        .unwrap()
        .0
        .clone()
}

#[test]
fn world_static_animation_defers_copy_until_a_visible_value_changes() {
    let (mut app, players, renderers, shared) = fixture(true, false);
    app.update();
    assert!(renderers.iter().all(|e| bound(&app, *e) == shared));
    for entity in &renderers {
        app.world_mut()
            .get_mut::<ViewVisibility>(*entity)
            .unwrap()
            .set_visible();
    }
    app.update();
    assert_eq!(
        bound(&app, renderers[0]),
        shared,
        "an unchanged sampled value keeps sharing"
    );
    let second = bound(&app, renderers[1]);
    assert_ne!(second, shared);
    assert_eq!(
        app.world()
            .resource::<Assets<LegacyModelMaterial>>()
            .get(&shared)
            .unwrap()
            .uniform
            .base_color
            .red,
        1.0
    );
    app.world_mut()
        .entity_mut(players[0])
        .insert(player(renderers[0], 0.25));
    app.update();
    let first = bound(&app, renderers[0]);
    assert_ne!(first, shared);
    assert_ne!(first, second);
    app.update();
    assert_eq!(bound(&app, renderers[0]), first);
    assert_eq!(bound(&app, renderers[1]), second);
    assert_eq!(
        app.world().resource::<Assets<LegacyModelMaterial>>().len(),
        3
    );
}

#[test]
fn overlapping_world_animation_owners_preserve_the_original_rendered_binding() {
    let mut results = Vec::new();
    for enabled in [false, true] {
        let (mut app, players, renderers, _) = fixture(enabled, true);
        app.world_mut()
            .entity_mut(players[0])
            .insert(player(renderers[0], 0.25));
        app.update();
        if enabled {
            assert_eq!(
                app.world()
                    .get::<WorldAnimationMaterialOwner>(renderers[0])
                    .unwrap()
                    .player,
                players[1]
            );
        }
        app.world_mut()
            .get_mut::<ViewVisibility>(renderers[0])
            .unwrap()
            .set_visible();
        app.update();
        let handle = bound(&app, renderers[0]);
        results.push(
            app.world()
                .resource::<Assets<LegacyModelMaterial>>()
                .get(&handle)
                .unwrap()
                .uniform
                .base_color
                .red,
        );
        app.update();
        assert_eq!(bound(&app, renderers[0]), handle);
    }
    assert_eq!(results, vec![0.75, 0.75]);
}
