//! Per-avatar transitions; never mutate a shared appearance material.
use super::*;
use crate::{
    entity_lifecycle::RemotePcVisibility0104,
    legacy_model_material::{LegacyBlendMode, LegacyRenderMode},
};

#[derive(Component)]
pub(super) struct FadeSurface {
    mode: LegacyRenderMode,
    alpha: f32,
}
#[derive(Component)]
pub(super) struct FadeOutline {
    mode: LegacyRenderMode,
    alpha: f32,
}

pub(super) fn animate_pc_visibility(
    mut commands: Commands,
    time: Option<Res<Time>>,
    mut roots: Query<(Entity, &NetworkPcVisual0104, &mut RemotePcVisibility0104)>,
    rigs: Query<&NetworkPcRigAppearanceStatus0104>,
    children: Query<&Children>,
    materials: Option<ResMut<Assets<LegacyModelMaterial>>>,
    outlines: Option<ResMut<Assets<LegacyOutlineMaterial>>>,
    mut surfaces: Query<(
        Entity,
        &mut MeshMaterial3d<LegacyModelMaterial>,
        Option<&FadeSurface>,
    )>,
    mut outline_surfaces: Query<(
        Entity,
        &mut MeshMaterial3d<LegacyOutlineMaterial>,
        Option<&FadeOutline>,
    )>,
) {
    let (Some(time), Some(mut materials), Some(mut outlines)) = (time, materials, outlines) else {
        return;
    };
    for (root, visual, mut fade) in &mut roots {
        if !fade.retiring && fade.alpha >= 1.0 {
            continue;
        }
        if !fade.retiring
            && !rigs
                .get(visual.rig_root)
                .is_ok_and(|s| *s == NetworkPcRigAppearanceStatus0104::Ready)
        {
            continue;
        }
        fade.alpha = pc_fade_step(fade.alpha, fade.retiring, time.delta_secs());
        if fade.retiring && fade.alpha <= 0.0 {
            commands.entity(root).despawn();
            continue;
        }
        let mut stack = vec![visual.rig_root];
        while let Some(entity) = stack.pop() {
            if let Ok(descendants) = children.get(entity) {
                stack.extend(descendants.iter());
            }
            if let Ok((entity, mut handle, bound)) = surfaces.get_mut(entity) {
                let original = if let Some(bound) = bound {
                    (bound.mode, bound.alpha)
                } else {
                    let Some(material) = materials.get(&handle.0).cloned() else {
                        continue;
                    };
                    let original = (material.render_mode, material.uniform.base_color.alpha);
                    handle.0 = materials.add(material);
                    commands.entity(entity).insert(FadeSurface {
                        mode: original.0,
                        alpha: original.1,
                    });
                    original
                };
                if let Some(mut material) = materials.get_mut(&handle.0) {
                    material.render_mode = fade_mode(original.0, fade.alpha);
                    material.uniform.base_color.alpha = original.1 * fade.alpha;
                }
            }
            if let Ok((entity, mut handle, bound)) = outline_surfaces.get_mut(entity) {
                let original = if let Some(bound) = bound {
                    (bound.mode, bound.alpha)
                } else {
                    let Some(material) = outlines.get(&handle.0).cloned() else {
                        continue;
                    };
                    let original = (material.render_mode, material.uniform.color.alpha);
                    handle.0 = outlines.add(material);
                    commands.entity(entity).insert(FadeOutline {
                        mode: original.0,
                        alpha: original.1,
                    });
                    original
                };
                if let Some(mut material) = outlines.get_mut(&handle.0) {
                    material.render_mode = fade_mode(original.0, fade.alpha);
                    material.uniform.color.alpha = original.1 * fade.alpha;
                }
            }
        }
    }
}

fn pc_fade_step(alpha: f32, retiring: bool, delta: f32) -> f32 {
    // 0.5 seconds in either direction, bounded after a stalled frame.
    (alpha
        + if retiring {
            -delta.max(0.0) * 2.0
        } else {
            delta.max(0.0) * 2.0
        })
    .clamp(0.0, 1.0)
}
fn fade_mode(mut mode: LegacyRenderMode, alpha: f32) -> LegacyRenderMode {
    if alpha < 1.0 {
        mode.blend = LegacyBlendMode::SrcAlphaOneMinusSrcAlpha;
        mode.depth_write = false;
    }
    mode
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn transition_is_bounded_and_retires_from_current_alpha() {
        assert_eq!(pc_fade_step(0.0, false, 0.1), 0.2);
        assert_eq!(pc_fade_step(0.2, true, 0.1), 0.0);
        assert_eq!(pc_fade_step(0.0, false, 10.0), 1.0);
        assert_eq!(pc_fade_step(1.0, true, 10.0), 0.0);
    }
    #[test]
    fn transition_detaches_shared_material_restores_pass_and_despawns_children() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default())
            .init_resource::<Assets<LegacyModelMaterial>>()
            .init_resource::<Assets<LegacyOutlineMaterial>>()
            .add_systems(Update, animate_pc_visibility);
        let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToon);
        let material = params
            .material_for_pass(
                params.render_plan().passes[0],
                &LegacyModelTextures::default(),
            )
            .unwrap();
        let outline = params.outline_material().unwrap();
        let outline_mode = outline.render_mode;
        let outline_alpha = outline.uniform.color.alpha;
        let shared_outline = app
            .world_mut()
            .resource_mut::<Assets<LegacyOutlineMaterial>>()
            .add(outline);
        let original_mode = material.render_mode;
        let original_alpha = material.uniform.base_color.alpha;
        let shared = app
            .world_mut()
            .resource_mut::<Assets<LegacyModelMaterial>>()
            .add(material);
        let rig = app
            .world_mut()
            .spawn(NetworkPcRigAppearanceStatus0104::Ready)
            .id();
        let surface = app
            .world_mut()
            .spawn((ChildOf(rig), MeshMaterial3d(shared.clone())))
            .id();
        let outline_entity = app
            .world_mut()
            .spawn((ChildOf(rig), MeshMaterial3d(shared_outline.clone())))
            .id();
        let pc =
            ffone_protocol::PcAppearance0104::decode(&[0; ffone_protocol::PcAppearance0104::SIZE])
                .unwrap();
        let root = app
            .world_mut()
            .spawn((
                RemotePcVisibility0104::default(),
                NetworkPcVisual0104 {
                    pc_id: 17,
                    rig_root: rig,
                    generation: 1,
                    request: PendingPcVisual0104::from(&pc),
                },
            ))
            .id();
        app.world_mut().entity_mut(rig).insert(ChildOf(root));
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(100));
        app.update();
        let private = app
            .world()
            .get::<MeshMaterial3d<LegacyModelMaterial>>(surface)
            .unwrap()
            .0
            .clone();
        assert_ne!(shared, private);
        let private_outline = app
            .world()
            .get::<MeshMaterial3d<LegacyOutlineMaterial>>(outline_entity)
            .unwrap()
            .0
            .clone();
        assert_ne!(shared_outline, private_outline);
        let outlines = app.world().resource::<Assets<LegacyOutlineMaterial>>();
        assert_eq!(
            outlines.get(&shared_outline).unwrap().uniform.color.alpha,
            outline_alpha
        );
        assert_eq!(
            outlines.get(&private_outline).unwrap().uniform.color.alpha,
            outline_alpha * 0.2
        );
        let materials = app.world().resource::<Assets<LegacyModelMaterial>>();
        assert_eq!(
            materials.get(&shared).unwrap().uniform.base_color.alpha,
            original_alpha
        );
        assert_eq!(
            materials.get(&private).unwrap().uniform.base_color.alpha,
            original_alpha * 0.2
        );
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs(1));
        app.update();
        assert_eq!(
            app.world()
                .resource::<Assets<LegacyModelMaterial>>()
                .get(&private)
                .unwrap()
                .render_mode,
            original_mode
        );
        assert_eq!(
            app.world()
                .resource::<Assets<LegacyOutlineMaterial>>()
                .get(&private_outline)
                .unwrap()
                .render_mode,
            outline_mode
        );
        app.world_mut()
            .get_mut::<RemotePcVisibility0104>(root)
            .unwrap()
            .retiring = true;
        app.update();
        assert!(app.world().get_entity(root).is_err());
        assert!(app.world().get_entity(surface).is_err());
    }
}
