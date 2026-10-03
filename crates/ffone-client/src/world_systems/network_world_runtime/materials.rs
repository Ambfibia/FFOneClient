use super::*;

#[derive(Clone, Debug, Component)]
pub(super) struct NetworkPcAppearanceMaterialBound0104 {
    pub(super) rig_root: Entity,
    pub(super) exact_route: String,
    pub(super) binding: NativePlayerMaterialBinding,
}

#[derive(Clone, Debug, Component)]
pub(super) struct NetworkHnpcMaterialBound0104 {
    pub(super) rig_root: Entity,
    pub(super) exact_route: String,
    pub(super) binding: NativePlayerMaterialBinding,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
pub struct NetworkNpcAppearMaterialBound0104;

/// Proof that an ordinary NPC scene was revealed only after every source
/// renderer left Bevy's temporary glTF `StandardMaterial` fallback and
/// received its exact XDT texture variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
pub struct NetworkNpcVisualMaterialReady0104;

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub struct NetworkNpcMaterialVisibilityBlocked0104 {
    pub(super) detail: String,
}

/// Marks a material surface as belonging to one concrete server-owned NPC.
///
/// Static-world materials also carry `LegacyMaterialApplied`; retaining the
/// resolved root here prevents material-animation sampling from walking every
/// world surface for every animated NPC each frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
pub struct NetworkNpcMaterialSurface0104 {
    pub(super) root: Entity,
}

/// Releases the shared NPC scene only after its actual GPU material path is
/// complete. A glTF scene is structurally ready before asynchronous external
/// textures are resolved; exposing it at that point renders packed Fusion
/// masks (notably the red R channel of `spawn11_green`) through
/// `StandardMaterial`. The ordinary world usually hid that transient state
/// behind loading, while tutorial choreography and the editor did not.
/// AppearEffect and companion/outline passes are deliberately not reveal
/// prerequisites: they are optional presentation work owned by the already
/// converted source renderer and must never keep a gameplay NPC invisible.
pub fn finalize_network_npc_material_visibility_0104(
    mut commands: Commands,
    children: Query<&Children>,
    roots: Query<(
        Entity,
        &NetworkNpcVisual0104,
        Option<&NetworkNpcMaterialVisibilityBlocked0104>,
    )>,
    mut scenes: Query<
        (&LegacyCharacterSceneStatus, &mut Visibility),
        With<LegacyCharacterSceneDeferredReveal>,
    >,
    standard_surfaces: Query<(), With<MeshMaterial3d<StandardMaterial>>>,
    legacy_surfaces: Query<
        (
            Option<&LegacyMaterialApplied>,
            Option<&NetworkNpcTextureVariantBound0104>,
            Option<&LegacyMaterialPassCompanion>,
        ),
        With<MeshMaterial3d<LegacyModelMaterial>>,
    >,
    material_errors: Query<&LegacyMaterialMetadataError>,
) {
    for (root, visual, blocked) in &roots {
        let Ok((scene_status, mut visibility)) = scenes.get_mut(visual.scene) else {
            continue;
        };
        *visibility = Visibility::Hidden;
        if !matches!(scene_status, LegacyCharacterSceneStatus::Ready { .. }) || blocked.is_some() {
            continue;
        }

        let mut stack = vec![visual.scene];
        let mut legacy_source_count = 0usize;
        let mut pending_fallback = false;
        let mut pending_binding = false;
        let mut metadata_error = None;
        while let Some(entity) = stack.pop() {
            if let Ok(error) = material_errors.get(entity) {
                metadata_error = Some(error.0.clone());
            }
            pending_fallback |= standard_surfaces.get(entity).is_ok();
            if let Ok((applied, texture_bound, companion)) = legacy_surfaces.get(entity) {
                if companion.is_none() {
                    legacy_source_count += 1;
                    pending_binding |= applied.is_none() || texture_bound.is_none();
                }
            }
            if let Ok(descendants) = children.get(entity) {
                stack.extend(descendants.iter());
            }
        }

        if let Some(error) = metadata_error {
            let detail = format!("legacy material conversion failed before NPC reveal: {error}");
            commands.entity(root).insert((
                NetworkNpcVisualIssue0104 {
                    npc_type: visual.npc_type,
                    detail: detail.clone(),
                },
                NetworkNpcMaterialVisibilityBlocked0104 { detail },
            ));
            continue;
        }
        if pending_fallback || pending_binding || legacy_source_count == 0 {
            continue;
        }

        *visibility = Visibility::Inherited;
        commands
            .entity(visual.scene)
            .remove::<LegacyCharacterSceneDeferredReveal>();
        commands
            .entity(root)
            .insert(NetworkNpcVisualMaterialReady0104);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub(super) struct NetworkNpcMaterialAnimationBase0104 {
    pub(super) root: Entity,
    pub(super) base_color: LinearRgba,
    pub(super) uv_scale_offset: Vec4,
}

pub(super) fn apply_network_npc_material_animation_0104(
    mut commands: Commands,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    roots: Query<(Entity, &NetworkNpcVisual0104)>,
    players: Query<(&AnimationPlayer, &NetworkNpcAnimationApplied0104)>,
    surfaces: Query<
        (
            Entity,
            &MeshMaterial3d<LegacyModelMaterial>,
            &NetworkNpcMaterialSurface0104,
            Option<&NetworkNpcMaterialAnimationBase0104>,
        ),
        With<LegacyMaterialApplied>,
    >,
    materials: Option<ResMut<Assets<LegacyModelMaterial>>>,
) {
    let Some(mut materials) = materials else {
        return;
    };
    for (player, applied) in &players {
        let Some(active) = player.animation(applied.node) else {
            continue;
        };
        let Ok((root, visual)) = roots.get(applied.root) else {
            continue;
        };
        let Some(clip) = visual
            .material_animation_clips
            .iter()
            .find(|clip| clip.name == applied.clip)
        else {
            continue;
        };
        let sample_time = active.seek_time();
        for (entity, handle, surface, baseline) in &surfaces {
            if surface.root != root {
                continue;
            }
            let matching_curves = clip
                .float_curves
                .iter()
                .filter(|curve| {
                    surface_has_named_ancestor(
                        entity,
                        root,
                        curve
                            .target_path
                            .rsplit('/')
                            .next()
                            .unwrap_or(&curve.target_path),
                        &parents,
                        &names,
                    )
                })
                .collect::<Vec<_>>();
            if matching_curves.is_empty() && baseline.is_none() {
                continue;
            }
            let Some(source_material) = materials.get(&handle.0) else {
                continue;
            };
            let had_baseline = baseline.is_some_and(|baseline| baseline.root == root);
            let baseline = baseline
                .filter(|baseline| baseline.root == root)
                .copied()
                .unwrap_or(NetworkNpcMaterialAnimationBase0104 {
                    root,
                    base_color: source_material.uniform.base_color,
                    uv_scale_offset: source_material.uniform.uv_scale_offset,
                });
            let mut material = source_material.clone();
            material.uniform.base_color = baseline.base_color;
            material.uniform.uv_scale_offset = baseline.uv_scale_offset;
            for curve in matching_curves {
                if let Some(value) =
                    sample_legacy_float_curve(curve, sample_time, clip.duration, clip.looped)
                {
                    apply_legacy_material_float(&mut material, &curve.property, value);
                }
            }
            if !had_baseline {
                commands
                    .entity(entity)
                    .insert((MeshMaterial3d(materials.add(material)), baseline));
            } else if let Some(mut existing) = materials.get_mut(&handle.0) {
                *existing = material;
            }
        }
    }
}
