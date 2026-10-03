use super::*;

/// Opt-in full-body transitions for owners such as ambient HNPC animation.
#[derive(Component, Clone, Copy, Debug)]
pub struct NativePlayerRigAnimationBlend(pub std::time::Duration);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NativePlayerRigMaterialMetadataGrace {
    pub(super) generation: u64,
    pub(super) missing_sources: usize,
}

pub(super) fn propagate_native_player_rig_render_layers(
    mut commands: Commands,
    added_children: Query<Entity, Added<ChildOf>>,
    parents: Query<&ChildOf>,
    rigs: Query<&RenderLayers, With<NativePlayerRigInstance>>,
) {
    for entity in &added_children {
        let mut cursor = entity;
        loop {
            if let Ok(layers) = rigs.get(cursor) {
                commands.entity(entity).insert(layers.clone());
                break;
            }
            let Ok(parent) = parents.get(cursor) else {
                break;
            };
            cursor = parent.parent();
        }
    }
}

/// Rebinds ShaderLab pass companions that were instantiated after the modular
/// part itself had already been rebound to the shared actor skeleton.
///
/// `LegacyModelMaterialPlugin` expresses Unity multi-pass materials as child
/// mesh entities. Keep every player companion synchronized with its source:
/// scene/material scheduling may replace the source palette after the
/// companion was first marked, and a one-shot copy would leave its outline in
/// the glTF bind pose beside the animated surface.
pub(super) fn rebind_native_player_rig_material_companions(
    mut commands: Commands,
    companions: Query<(Entity, &LegacyMaterialPassCompanion), With<SkinnedMesh>>,
    source_bindings: Query<&NativePlayerRigSkinBound>,
    mut skins: ParamSet<(Query<&SkinnedMesh>, Query<&mut SkinnedMesh>)>,
) {
    for (companion_entity, companion) in &companions {
        let Some((source_skin, source_binding)) = ({
            let source_skins = skins.p0();
            source_skins
                .get(companion.source_mesh_entity)
                .ok()
                .zip(source_bindings.get(companion.source_mesh_entity).ok())
                .map(|(skin, binding)| (skin.clone(), *binding))
        }) else {
            // The source part has not reached the exact shared-palette bind
            // yet. It will be retried next frame.
            continue;
        };
        let mut companion_skins = skins.p1();
        let Ok(mut companion_skin) = companion_skins.get_mut(companion_entity) else {
            continue;
        };
        if companion_skin.joints != source_skin.joints
            || companion_skin.inverse_bindposes != source_skin.inverse_bindposes
        {
            *companion_skin = source_skin;
        }
        commands
            .entity(companion_entity)
            .queue_silenced(move |mut entity: EntityWorldMut| {
                entity.insert(source_binding);
            });
    }
}

/// Converts terminal material-metadata failures into a typed rig failure.
///
/// Scene instantiation, glTF-extra discovery, and shared-palette binding are
/// separate deferred systems. One full Update of grace prevents schedule order
/// from treating a just-instantiated source as malformed; persistence into the
/// next Update is a terminal native-asset contract violation.
pub(super) fn fail_closed_native_player_rig_material_metadata(
    mut commands: Commands,
    mut rigs: Query<(
        Entity,
        &NativePlayerRigInstance,
        &NativePlayerRigPartsBound,
        &mut NativePlayerRigStatus,
        Option<&NativePlayerRigMaterialMetadataGrace>,
    )>,
    children: Query<&Children>,
    mut descendants: Local<RigDescendantScratch>,
    mut source_index: Local<RigQueryIndex>,
    material_sources: Query<
        (
            Entity,
            Option<&Name>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialMetadataError>,
        ),
        With<LegacyMaterialRendererOrder>,
    >,
) {
    if rigs.is_empty() {
        return;
    }
    let source_rows = if source_index.enabled {
        material_sources.iter().collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    source_index.rebuild(source_rows.iter().map(|row| row.0));
    for (rig_entity, instance, parts, mut status, grace) in &mut rigs {
        if status.is_blocked() {
            continue;
        }
        descendants.collect_for_query(rig_entity, &children, source_index.enabled);
        let sources = if source_index.enabled {
            descendants
                .ordered_matches(&source_index)
                .iter()
                .map(|index| source_rows[*index])
                .collect::<Vec<_>>()
        } else {
            material_sources
                .iter()
                .filter(|(entity, _, _, _)| descendants.contains(*entity))
                .collect::<Vec<_>>()
        };
        if sources.len() != parts.skinned_surfaces {
            if grace.is_some() {
                commands
                    .entity(rig_entity)
                    .remove::<NativePlayerRigMaterialMetadataGrace>();
            }
            continue;
        }
        if let Some((entity, name, _, Some(error))) = sources
            .iter()
            .copied()
            .find(|(_, _, _, error)| error.is_some())
        {
            *status = NativePlayerRigStatus::Blocked(format!(
                "native player-rig material metadata failed on {} {entity:?}: {}",
                name.map_or("<unnamed>", Name::as_str),
                error.0.as_str()
            ));
            commands
                .entity(rig_entity)
                .remove::<NativePlayerRigMaterialMetadataGrace>();
            continue;
        }
        let missing = sources
            .iter()
            .copied()
            .filter(|(_, _, pending, _)| pending.is_none())
            .collect::<Vec<_>>();
        if missing.is_empty() {
            if grace.is_some() {
                commands
                    .entity(rig_entity)
                    .remove::<NativePlayerRigMaterialMetadataGrace>();
            }
            continue;
        }
        if grace.is_some_and(|grace| {
            grace.generation == instance.generation && grace.missing_sources == missing.len()
        }) {
            let identities = missing
                .iter()
                .map(|(entity, name, _, _)| {
                    format!("{} {entity:?}", name.map_or("<unnamed>", Name::as_str))
                })
                .collect::<Vec<_>>()
                .join(", ");
            *status = NativePlayerRigStatus::Blocked(format!(
                "native player-rig sources have no typed PendingLegacyModelMaterial after scene load: {identities}"
            ));
            commands
                .entity(rig_entity)
                .remove::<NativePlayerRigMaterialMetadataGrace>();
        } else {
            commands
                .entity(rig_entity)
                .insert(NativePlayerRigMaterialMetadataGrace {
                    generation: instance.generation,
                    missing_sources: missing.len(),
                });
        }
    }
}
