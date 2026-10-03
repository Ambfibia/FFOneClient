//! Live transport NPC portrait; reuses the world model and its current animation.
use crate::app::*;
use bevy::{camera::visibility::VisibilitySystems, transform::TransformSystems};
use ffone_client::transportation_ui::TransportationPresentationNpcCameraSlot;

const PORTRAIT_LAYER: usize = 26;
const PORTRAIT_SIZE: u32 = 90;
const PORTRAIT_DISTANCE: f32 = 0.5;
const PORTRAIT_HEIGHT: f32 = 1.2;

pub(super) struct TransportationPortraitPlugin;

impl Plugin for TransportationPortraitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PortraitRuntime>().add_systems(
            PostUpdate,
            sync_portrait
                .after(TransformSystems::Propagate)
                .before(VisibilitySystems::CheckVisibility),
        );
    }
}

#[derive(Resource, Default)]
struct PortraitRuntime {
    camera: Option<Entity>,
    image: Option<Handle<Image>>,
}

#[derive(Component)]
struct PortraitCamera;

#[derive(Component)]
struct PortraitLayerLease {
    implicit: bool,
}

fn portrait_transform(avatar: &GlobalTransform) -> Transform {
    // The transport NPC prefab has no player-avatar component/neck target.
    // Face the root from its forward side, then raise the camera without
    // changing the look rotation, matching the camera's late-update order.
    let origin = avatar.translation();
    let mut pose =
        Transform::from_translation(origin + avatar.rotation() * Vec3::NEG_Z * PORTRAIT_DISTANCE)
            .looking_at(origin, Vec3::Y);
    pose.translation.y += PORTRAIT_HEIGHT;
    pose
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_portrait(
    mut commands: Commands,
    model: Res<TransportationModel>,
    production: Res<TransportationProductionRuntime>,
    mut runtime: ResMut<PortraitRuntime>,
    mut images: ResMut<Assets<Image>>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104, &GlobalTransform), Without<PortraitCamera>>,
    children: Query<&Children>,
    meshes: Query<(), With<Mesh3d>>,
    lights: Query<Entity, With<DirectionalLight>>,
    layers: Query<Option<&RenderLayers>>,
    leased: Query<(Entity, &PortraitLayerLease)>,
    mut cameras: Query<(&mut Transform, &mut GlobalTransform), With<PortraitCamera>>,
    mut slots: Query<&mut ImageNode, With<TransportationPresentationNpcCameraSlot>>,
) {
    let desired = (model.phase() == TransportationPhase::Browsing
        && model.service() == TransportationService::Warp
        && production.camera_subtarget_active)
        .then(|| production.active_npc.as_ref())
        .flatten()
        .and_then(|session| {
            npcs.iter()
                .find(|(_, npc, _)| npc.0.npc_id == session.npc_id)
        });
    let mut admitted = std::collections::HashSet::new();
    if let Some((root, _, _)) = desired {
        for entity in children.iter_descendants(root) {
            if meshes.contains(entity) {
                admitted.insert(entity);
            }
        }
        // Preserve the existing world's light values and shadow settings.
        // Only the additional camera visibility bit is leased while open.
        if !admitted.is_empty() {
            admitted.extend(lights.iter());
        }
    }
    for (entity, lease) in &leased {
        if !admitted.contains(&entity) {
            if let Ok(Some(current)) = layers.get(entity) {
                let restored = current.clone().without(PORTRAIT_LAYER);
                if lease.implicit && restored == RenderLayers::default() {
                    commands.entity(entity).remove::<RenderLayers>();
                } else {
                    commands.entity(entity).insert(restored);
                }
            }
            commands.entity(entity).remove::<PortraitLayerLease>();
        }
    }
    for entity in &admitted {
        let Ok(current) = layers.get(*entity) else {
            continue;
        };
        let expanded = current.cloned().unwrap_or_default().with(PORTRAIT_LAYER);
        if current != Some(&expanded) {
            commands.entity(*entity).insert(expanded);
            if !leased.contains(*entity) {
                commands.entity(*entity).insert(PortraitLayerLease {
                    implicit: current.is_none(),
                });
            }
        }
    }

    let Some((_, _, avatar)) = desired.filter(|_| !admitted.is_empty()) else {
        if let Some(camera) = runtime.camera.take() {
            commands.entity(camera).try_despawn();
        }
        for mut slot in &mut slots {
            if slot.image != Handle::default() {
                slot.image = Handle::default();
            }
        }
        // No cache retains streamed render targets after the menu closes.
        runtime.image = None;
        return;
    };
    let target = runtime
        .image
        .get_or_insert_with(|| {
            images.add(Image::new_target_texture(
                PORTRAIT_SIZE,
                PORTRAIT_SIZE,
                TextureFormat::Rgba8UnormSrgb,
                None,
            ))
        })
        .clone();
    let pose = portrait_transform(avatar);
    if let Some(camera) = runtime
        .camera
        .and_then(|entity| cameras.get_mut(entity).ok())
    {
        let (mut local, mut global) = camera;
        local.set_if_neq(pose);
        global.set_if_neq(GlobalTransform::from(pose));
    } else {
        runtime.camera = Some(
            commands
                .spawn((
                    Name::new("Transport NPC portrait camera"),
                    PortraitCamera,
                    Camera3d::default(),
                    (
                        Camera {
                            order: -15,

                            clear_color: ClearColorConfig::Custom(Color::NONE),
                            ..default()
                        },
                        bevy::camera::RenderTarget::from(target.clone()),
                    ),
                    Projection::Perspective(PerspectiveProjection {
                        fov: 45_f32.to_radians(),
                        near: 0.001,
                        far: 5.0,
                        aspect_ratio: 1.0,
                        ..default()
                    }),
                    RenderLayers::layer(PORTRAIT_LAYER),
                    pose,
                    GlobalTransform::from(pose),
                ))
                .id(),
        );
    }
    for mut slot in &mut slots {
        if slot.image != target {
            slot.image = target.clone();
        }
    }
}

#[cfg(test)]
mod tests;
