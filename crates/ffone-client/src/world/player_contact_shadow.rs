use super::collision_collider_capsule_ground_contact_with_bounds::collider_ground_contact_with_bounds;
use super::*;
use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, render::render_resource::Face};

/// A single local, soft contact mark. It uses the already indexed walkable
/// geometry and one shared mesh/material; no world light or shadow map is added.
#[derive(Component)]
pub(super) struct PlayerContactShadow;

const MAX_SHADOW_DROP: f32 = 8.0;
const SHADOW_LIFT: f32 = 0.025;

fn shadow_mesh() -> Mesh {
    const SIDES: usize = 32;
    let mut positions = Vec::with_capacity(SIDES * 9);
    let mut normals = Vec::with_capacity(SIDES * 9);
    let mut colors = Vec::with_capacity(SIDES * 9);
    for i in 0..SIDES {
        let a = std::f32::consts::TAU * i as f32 / SIDES as f32;
        let b = std::f32::consts::TAU * (i + 1) as f32 / SIDES as f32;
        let point = |radius: f32, angle: f32| [radius * angle.cos(), 0.0, radius * angle.sin()];
        let inner_a = point(0.26, a);
        let inner_b = point(0.26, b);
        let outer_a = point(0.58, a);
        let outer_b = point(0.58, b);
        positions.extend_from_slice(&[
            [0.0, 0.0, 0.0],
            inner_b,
            inner_a,
            inner_a,
            inner_b,
            outer_b,
        ]);
        positions.extend_from_slice(&[inner_a, outer_b, outer_a]);
        colors.extend_from_slice(&[
            [0.0, 0.0, 0.0, 0.30],
            [0.0, 0.0, 0.0, 0.21],
            [0.0, 0.0, 0.0, 0.21],
            [0.0, 0.0, 0.0, 0.21],
            [0.0, 0.0, 0.0, 0.21],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.21],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ]);
        normals.extend(std::iter::repeat_n([0.0, 1.0, 0.0], 9));
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
}

pub(super) fn update_player_contact_shadow(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shared: Local<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
    spatial_index: Res<AuthoredColliderSpatialIndex>,
    colliders: Query<(
        &GlobalTransform,
        &AuthoredTriMeshCollider,
        &AuthoredColliderWorldBounds,
    )>,
    heightmaps: Query<(&GlobalTransform, &NativeHeightmapCollider)>,
    players: Query<
        (
            Entity,
            &Transform,
            &LegacyPlayerController,
            Option<&NativeWorldGroundSupport>,
            &Children,
        ),
        (With<LegacyPlayerController>, Without<PlayerContactShadow>),
    >,
    mut shadows: Query<
        (&mut Transform, &mut Visibility),
        (With<PlayerContactShadow>, Without<LegacyPlayerController>),
    >,
    mut candidates: Local<Vec<Entity>>,
) {
    for (player, transform, controller, support, children) in &players {
        let x = transform.translation.x;
        let z = transform.translation.z;
        let minimum_y = transform.translation.y - MAX_SHADOW_DROP;
        let maximum_y = transform.translation.y + GROUND_EPSILON;
        let mut contact: Option<(f32, Vec3)> = None;
        for (global, terrain) in &heightmaps {
            if let Some(hit) = terrain.ground_contact(global, x, z, minimum_y, maximum_y) {
                let mut normal = hit.normal.normalize_or_zero();
                if normal.y < 0.0 {
                    normal = -normal;
                }
                if normal.y >= AUTHORED_WALKABLE_MIN_UP_DOT
                    && contact.is_none_or(|(height, _)| hit.point.y > height)
                {
                    contact = Some((hit.point.y, normal));
                }
            }
        }
        spatial_index.candidates(
            Vec3::new(x, minimum_y, z),
            Vec3::new(x, maximum_y, z),
            &mut candidates,
        );
        for entity in candidates.iter().copied() {
            let Ok((global, collider, bounds)) = colliders.get(entity) else {
                continue;
            };
            if collider.is_trigger {
                continue;
            }
            if let Some((height, normal)) = collider_ground_contact_with_bounds(
                collider,
                global.to_matrix(),
                bounds,
                x,
                z,
                minimum_y,
                maximum_y,
            ) && contact.is_none_or(|(current, _)| height > current)
            {
                contact = Some((height, normal));
            }
        }

        // At a finite ledge the controller can rest on its lower sphere while
        // the vertical root ray misses the triangle. Keep the same support
        // in that case instead of dropping the mark onto a lower floor.
        // Only the known supporting collider needs a capsule query at an
        // edge. Scanning every nearby mesh here costs several ms in dense
        // scenes, even when the player stands still on ordinary terrain.
        if controller.grounded
            && contact.is_none_or(|(height, _)| transform.translation.y - height > GROUNDED_STEP_UP)
            && let Some(support) = support
            && let Ok((global, collider, bounds)) = colliders.get(support.collider)
            && let Some(edge) = collider_capsule_ground_contact_with_bounds(
                collider,
                global.to_matrix(),
                bounds,
                x,
                z,
                transform.translation.y,
                minimum_y,
                maximum_y,
            )
            && contact.is_none_or(|(height, _)| edge.0 > height + GROUND_EPSILON)
        {
            contact = Some(edge);
        }

        let existing = children.iter().find(|child| shadows.contains(*child));
        let Some((ground_y, normal)) = contact else {
            if let Some(child) = existing
                && let Ok((_, mut visibility)) = shadows.get_mut(child)
            {
                *visibility = Visibility::Hidden;
            }
            continue;
        };
        let distance = (transform.translation.y - ground_y).max(0.0);
        let world_rotation = Quat::from_rotation_arc(Vec3::Y, normal);
        let local_rotation = transform.rotation.inverse() * world_rotation;
        let local_position = transform.rotation.inverse()
            * (Vec3::new(x, ground_y + SHADOW_LIFT, z) - transform.translation);
        let scale = 1.0 + 0.04 * distance;
        let pose = Transform {
            translation: local_position,
            rotation: local_rotation,
            scale: Vec3::splat(scale),
        };
        if let Some(child) = existing {
            if let Ok((mut shadow_transform, mut visibility)) = shadows.get_mut(child) {
                *shadow_transform = pose;
                *visibility = Visibility::Visible;
            }
        } else {
            let (mesh, material) = shared.get_or_insert_with(|| {
                (
                    meshes.add(shadow_mesh()),
                    materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        cull_mode: Some(Face::Back),
                        ..default()
                    }),
                )
            });
            commands.spawn((
                Name::new("Player contact shadow"),
                PlayerContactShadow,
                ChildOf(player),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                pose,
            ));
        }
    }
}
