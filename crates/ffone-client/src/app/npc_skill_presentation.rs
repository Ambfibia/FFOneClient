//! Presentation of validated server NPC cast phases.
use super::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use ffone_client::entity_lifecycle::NetworkNpcSkillEffectEvents0104;
use ffone_client::native_terrain::NativeHeightmapCollider;
use ffone_client::world::{
    AuthoredColliderWorldBounds, AuthoredTriMeshCollider, collider_ground_height_with_bounds,
};
use ffone_protocol::NpcSkillSignalKind0104;

#[derive(SystemParam)]
pub(super) struct GroundProjection<'w, 's> {
    terrain: Query<'w, 's, (&'static NativeHeightmapCollider, &'static GlobalTransform)>,
    colliders: Query<
        'w,
        's,
        (
            &'static AuthoredTriMeshCollider,
            &'static GlobalTransform,
            &'static AuthoredColliderWorldBounds,
        ),
    >,
}

#[derive(Component)]
pub(super) struct EruptionMark {
    npc_id: i32,
    remaining: f32,
}

pub(super) fn present_npc_skills(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<State<ClientState>>,
    content: Res<TutorialMissionContent>,
    mut events: ResMut<NetworkNpcSkillEffectEvents0104>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104, &GlobalTransform)>,
    names: Query<&Name>,
    children: Query<&Children>,
    ground: GroundProjection,
    mut marks: Query<(Entity, &mut EruptionMark)>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut audio: ResMut<GameplayAudioRuntime>,
) {
    if marks.is_empty() && events.0.is_empty() {
        *material = None;
    }
    let mut removed = std::collections::HashSet::new();
    for (entity, mut mark) in &mut marks {
        mark.remaining -= time.delta_secs();
        if mark.remaining <= 0.0 || *state.get() != ClientState::World {
            removed.insert(entity);
            commands.entity(entity).despawn();
        }
    }
    if *state.get() != ClientState::World {
        events.0.clear();
        return;
    }
    let mut spawned = std::collections::HashMap::new();
    for event in events.0.drain(..) {
        // Finish/cancel clears the ground warning even if its caster has left
        // the streamed slice. READY also replaces a previous cast's warning.
        if matches!(event.signal.kind, NpcSkillSignalKind0104::Ready
            | NpcSkillSignalKind0104::Hit | NpcSkillSignalKind0104::Cancel) {
            for (entity, mark) in &mut marks {
                if mark.npc_id == event.signal.npc_id && removed.insert(entity) {
                    commands.entity(entity).despawn();
                }
            }
            if let Some(entity) = spawned.remove(&event.signal.npc_id) {
                commands.entity(entity).despawn();
            }
        }
        let Some((owner, npc, pose)) = npcs
            .iter()
            .find(|(_, npc, _)| npc.0.npc_id == event.signal.npc_id)
        else {
            continue;
        };
        let effect_name = format!("npc-special-{}", event.signal.npc_id);
        if matches!(
            event.signal.kind,
            NpcSkillSignalKind0104::Cancel
                | NpcSkillSignalKind0104::CorruptionHit
                | NpcSkillSignalKind0104::Hit
        ) {
            effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
                name: effect_name.clone(),
                source_line: 0,
            });
        }
        if event.signal.kind == NpcSkillSignalKind0104::CorruptionReady {
            let Some(style @ 0..=2) = event.style else {
                continue;
            };
            let mut center = Vec::new();
            tutorial_descendants_named(owner, "center", &names, &children, &mut center);
            let placement = if center.len() == 1 {
                let rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
                TutorialEffectPlacement::ExactEntityBone {
                    root_entity: owner,
                    node_name: "center".into(),
                    spawn_world_rotation: rotation,
                    local_translation_after_parenting: Vec3::ZERO,
                    local_rotation_after_parenting: rotation,
                }
            } else {
                TutorialEffectPlacement::ExactEntityWorld {
                    root_entity: owner,
                    position: pose.translation(),
                    rotation: pose.rotation(),
                }
            };
            let scale = content
                .gameplay_npc(npc.0.npc_type)
                .map_or(1.0, |npc| npc.radius() * 2.0);
            let fusion = content
                .gameplay_npc_minimap(npc.0.npc_type)
                .is_some_and(|npc| npc.sound == 2);
            for id in [Some(530 + i32::from(style)), fusion.then_some(592)]
                .into_iter()
                .flatten()
            {
                effects.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id: id,
                    placement: placement.clone(),
                    scale,
                    tracked: false,
                    name: Some(effect_name.clone()),
                    destroy_after_seconds: None,
                    source_line: 0,
                });
            }
            audio.queue_legacy_world_sound(
                owner,
                [
                    "CorruptionAtypeCasting",
                    "CorruptionBtypeCasting",
                    "CorruptionCtypeCasting",
                ][style as usize],
            );
        } else if event.mega && event.signal.kind == NpcSkillSignalKind0104::Ready {
            let Some(skill) = event
                .signal
                .skill_id
                .and_then(|id| content.gameplay_skill(id))
            else {
                continue;
            };
            let Some(position) = event.position else {
                continue;
            };
            let position = ProtocolPosition::new(position).to_native();
            let scale = skill.area as f32 * 0.02 * 1.414 * 1.3;
            if scale <= 0.0 {
                continue;
            }
            let material = material
                .get_or_insert_with(|| {
                    materials.add(StandardMaterial {
                        base_color_texture: Some(
                            asset_server.load("effects/npc-skills/eruption-mark.png"),
                        ),
                        unlit: true,
                        alpha_mode: AlphaMode::Blend,
                        cull_mode: None,
                        ..default()
                    })
                })
                .clone();
            let colliders = ground
                .colliders
                .iter()
                .filter(|(collider, _, bounds)| {
                    let (min, max) = bounds.debug_bounds();
                    !collider.is_trigger()
                        && min.x <= position.x + scale * 0.5
                        && max.x >= position.x - scale * 0.5
                        && min.z <= position.z + scale * 0.5
                        && max.z >= position.z - scale * 0.5
                        && min.y <= position.y + 1.0
                        && max.y >= position.y - 8.0
                })
                .collect::<Vec<_>>();
            let mut vertices = Vec::new();
            let mut uv = Vec::new();
            let mut indices = Vec::new();
            const N: u32 = 16;
            for z in 0..=N {
                for x in 0..=N {
                    let u = x as f32 / N as f32;
                    let v = z as f32 / N as f32;
                    let px = position.x + (u - 0.5) * scale;
                    let pz = position.z + (v - 0.5) * scale;
                    let terrain_y = ground
                        .terrain
                        .iter()
                        .filter_map(|(terrain, global)| {
                            terrain.ground_height(
                                global,
                                px,
                                pz,
                                position.y - 8.0,
                                position.y + 1.0,
                            )
                        })
                        .max_by(f32::total_cmp);
                    let mesh_y = colliders
                        .iter()
                        .filter_map(|(collider, global, bounds)| {
                            collider_ground_height_with_bounds(
                                collider,
                                global.to_matrix(),
                                bounds,
                                px,
                                pz,
                                position.y - 8.0,
                                position.y + 1.0,
                            )
                        })
                        .max_by(f32::total_cmp);
                    let y = terrain_y
                        .into_iter()
                        .chain(mesh_y)
                        .max_by(f32::total_cmp)
                        .unwrap_or(position.y);
                    vertices.push([px, y + if mesh_y == Some(y) { 0.07 } else { 0.05 }, pz]);
                    uv.push([1.0 - u, v]);
                    if x < N && z < N {
                        let a = z * (N + 1) + x;
                        indices.extend_from_slice(&[
                            a,
                            a + N + 1,
                            a + 1,
                            a + 1,
                            a + N + 1,
                            a + N + 2,
                        ]);
                    }
                }
            }
            let mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                vec![[0.0, 1.0, 0.0]; ((N + 1) * (N + 1)) as usize],
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
            .with_inserted_indices(Indices::U32(indices));
            let entity = commands.spawn((
                Name::new("NPC eruption warning"),
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material),
                Transform::default(),
                EruptionMark { npc_id: event.signal.npc_id, remaining: 3.0 },
            )).id();
            spawned.insert(event.signal.npc_id, entity);
            audio.queue_legacy_world_sound(owner, "MegaAttackSpell");
        } else if event.mega && event.signal.kind == NpcSkillSignalKind0104::Hit {
            if let Some(position) = event.position {
                effects.enqueue(TutorialEffectRuntimeCommand::Add {
                    effect_id: 766,
                    placement: TutorialEffectPlacement::World {
                        position: ProtocolPosition::new(position).to_native(),
                        rotation: Quat::IDENTITY,
                    },
                    scale: 0.6,
                    tracked: false,
                    name: None,
                    destroy_after_seconds: None,
                    source_line: 0,
                });
                audio.queue_legacy_world_sound(owner, "MegaAttackMissile");
            }
        }
    }
}
