use super::*;
use crate::legacy_model_material::{
    LegacyModelMaterialParams, LegacyModelTextures, LegacyShaderKind, LegacyStaticMaterialCache,
};

fn plan(value: f32) -> MaterialAnimationPlan {
    MaterialAnimationPlan {
        duration: 1.0,
        curves: vec![MaterialCurvePlan {
            node_name: "animated".into(),
            property: MaterialAnimatedProperty::BaseAlpha,
            curve: AnimationCurve(vec![CurveKey {
                time: 0.0,
                value,
                in_slope: 0.0,
                out_slope: 0.0,
            }]),
        }],
    }
}

#[test]
fn immutable_effect_models_share_and_late_animations_detach_only_their_targets() {
    verify_material_animation_ownership(false);
}

#[test]
fn numeric_effect_animation_keeps_instance_ownership_with_uniform_uploads() {
    verify_material_animation_ownership(true);
}

fn verify_material_animation_ownership(uniform_uploads: bool) {
    let mut app = App::new();
    if uniform_uploads {
        app.insert_resource(crate::legacy_model_material::NativeMaterialUniformUpdates::enabled_for_test());
    }
    app.init_resource::<Time>()
        .init_resource::<Assets<LegacyModelMaterial>>()
        .init_resource::<LegacyStaticMaterialCache>()
        .add_systems(
            Update,
            (
                prepare_native_mesh_effect_materials,
                animate_native_mesh_effect_materials,
            )
                .chain(),
        );
    let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::OpaqueNormal);
    let mut roots = Vec::new();
    let mut surfaces = Vec::new();
    for _ in 0..2 {
        let root = app
            .world_mut()
            .spawn(NativeEffectRoot {
                stream_owner: None,
                age: 0.0,
                destroy_after: None,
                natural_destroy_after: None,
                material_animation: None,
                waiting_for_mesh_surface: true,
            })
            .id();
        roots.push(root);
        for name in ["animated.surface", "unchanged"] {
            let material = params
                .material_for_pass(
                    params.render_plan().passes[0],
                    &LegacyModelTextures::default(),
                )
                .unwrap();
            let handle = app
                .world_mut()
                .resource_mut::<Assets<LegacyModelMaterial>>()
                .add(material);
            surfaces.push(
                app.world_mut()
                    .spawn((ChildOf(root), Name::new(name), MeshMaterial3d(handle)))
                    .id(),
            );
        }
    }
    let handle = |app: &App, entity| {
        app.world()
            .get::<MeshMaterial3d<LegacyModelMaterial>>(entity)
            .unwrap()
            .0
            .clone()
    };
    app.update();
    let shared = handle(&app, surfaces[0]);
    assert!(
        surfaces
            .iter()
            .all(|entity| handle(&app, *entity) == shared)
    );
    app.world_mut()
        .get_mut::<NativeEffectRoot>(roots[0])
        .unwrap()
        .material_animation = Some(plan(0.25));
    app.update();
    let private = handle(&app, surfaces[0]);
    assert_ne!(private, shared);
    assert!(
        surfaces[1..]
            .iter()
            .all(|entity| handle(&app, *entity) == shared)
    );
    assert_eq!(
        app.world()
            .resource::<Assets<LegacyModelMaterial>>()
            .get(&private)
            .unwrap()
            .uniform
            .base_color
            .alpha,
        0.25
    );
    assert_eq!(
        app.world()
            .resource::<Assets<LegacyModelMaterial>>()
            .get(&shared)
            .unwrap()
            .uniform
            .base_color
            .alpha,
        1.0
    );
    app.update();
    assert_eq!(
        handle(&app, surfaces[0]),
        private,
        "ongoing curves reuse their private material"
    );
    app.world_mut()
        .entity_mut(roots[1])
        .insert(TutorialEffectMaterialAnimation {
            age: 0.0,
            plan: plan(0.75),
            paused: false,
        });
    app.update();
    let standalone = handle(&app, surfaces[2]);
    assert_ne!(standalone, shared);
    assert_ne!(standalone, private);
    assert_eq!(handle(&app, surfaces[3]), shared);
    assert_eq!(
        app.world()
            .resource::<Assets<LegacyModelMaterial>>()
            .get(&standalone)
            .unwrap()
            .uniform
            .base_color
            .alpha,
        0.75
    );
}
