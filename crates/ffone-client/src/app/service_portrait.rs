//! Shared live render ownership for service NPC panels.
use crate::app::*;
use bevy::{camera::visibility::VisibilitySystems, transform::TransformSystems};
use ffone_client::service_portrait::ServicePortraitSlot;

const FIRST_LAYER: usize = 32;

pub(super) struct ServicePortraitPlugin;

impl Plugin for ServicePortraitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServicePortraitRuntime>().add_systems(
            PostUpdate,
            sync_service_portraits
                .after(TransformSystems::Propagate)
                .before(VisibilitySystems::CheckVisibility),
        );
    }
}

#[derive(Default)]
struct Surface {
    camera: Option<Entity>,
    image: Option<Handle<Image>>,
}

#[derive(Default, Resource)]
struct ServicePortraitRuntime {
    surfaces: [Surface; 5],
}

#[derive(Component)]
struct ServiceCamera;

#[derive(Component)]
struct ServiceLayerLease {
    implicit: bool,
    layers: Vec<usize>,
}

fn camera_pose(slot: ServicePortraitSlot, avatar: &GlobalTransform) -> Transform {
    // NPC roots do not carry the player-avatar neck resolver. Preserve the
    // root fallback and LookAt-before-world-height order. Native forward is
    // -Z; conjugating the source yaw into that basis changes its sign.
    let origin = avatar.translation();
    let (distance, height, yaw) = match slot {
        ServicePortraitSlot::Vendor => (0.4, 1.1, 20_f32),
        ServicePortraitSlot::CombiWaiting => (2.2, 0.42, 0_f32),
        _ => (1.3, 0.55, 20_f32),
    };
    let direction = avatar.rotation() * Quat::from_rotation_y(yaw.to_radians()) * Vec3::NEG_Z;
    let mut pose =
        Transform::from_translation(origin + direction * distance).looking_at(origin, Vec3::Y);
    pose.translation.y += height;
    pose
}

#[derive(SystemParam)]
struct ServiceOwners<'w> {
    vendor: Option<Res<'w, VendorUiState>>,
    vendor_projection: Option<Res<'w, VendorModeProjection0104>>,
    combi: Res<'w, CombiProductionShell0104>,
    combi_ui: ResMut<'w, CombiUiState0104>,
    enchant: Res<'w, EnchantProductionRuntime0104>,
    enchant_ui: Res<'w, EnchantModeProjection0104>,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn sync_service_portraits(
    mut commands: Commands,
    mut owners: ServiceOwners,
    mut runtime: ResMut<ServicePortraitRuntime>,
    mut images: ResMut<Assets<Image>>,
    npcs: Query<(Entity, &NetworkNpcAppearance0104, &GlobalTransform), Without<ServiceCamera>>,
    hnpcs: Query<(), With<NetworkHnpcVisual0104>>,
    children: Query<&Children>,
    meshes: Query<(), With<Mesh3d>>,
    lights: Query<Entity, With<DirectionalLight>>,
    layers: Query<Option<&RenderLayers>>,
    leased: Query<(Entity, &ServiceLayerLease)>,
    mut cameras: Query<(&mut Transform, &mut GlobalTransform), With<ServiceCamera>>,
    mut slots: Query<(&ServicePortraitSlot, &mut ImageNode)>,
) {
    let mut admitted: BTreeMap<Entity, Vec<usize>> = BTreeMap::new();
    let mut bound = [false; 5];
    for slot in ServicePortraitSlot::ALL {
        let index = slot.index();
        let layer = FIRST_LAYER + index;
        let desired = match slot {
            ServicePortraitSlot::Vendor => owners
                .vendor
                .as_ref()
                .filter(|state| state.phase != VendorLifecyclePhase::Hidden)
                .and(owners.vendor_projection.as_ref())
                .and_then(|projection| {
                    npcs.iter().find(|(entity, npc, _)| {
                        npc.0.npc_id == projection.session.requested_npc_id
                            && hnpcs.contains(*entity)
                    })
                }),
            ServicePortraitSlot::CombiPrimary | ServicePortraitSlot::CombiWaiting => {
                let active = !matches!(owners.combi_ui.phase, CombiPhase0104::Hidden)
                    && (slot == ServicePortraitSlot::CombiPrimary
                        || matches!(owners.combi_ui.phase, CombiPhase0104::Waiting { .. }));
                owners
                    .combi
                    .camera_npc_table_id
                    .filter(|_| active)
                    .and_then(|id| npcs.iter().find(|(_, npc, _)| npc.0.npc_type == id))
            }
            ServicePortraitSlot::EnchantPrimary | ServicePortraitSlot::EnchantWaiting => {
                let active = owners.enchant_ui.visible
                    && (slot == ServicePortraitSlot::EnchantPrimary
                        || matches!(owners.enchant_ui.phase, EnchantPhase0104::Waiting { .. }));
                owners
                    .enchant
                    .session()
                    .filter(|_| active)
                    .and_then(|session| {
                        let id = session.context().source.runtime_npc_id;
                        npcs.iter().find(|(_, npc, _)| npc.0.npc_id == id)
                    })
            }
        };
        let desired = desired.filter(|(root, _, _)| {
            children
                .iter_descendants(*root)
                .any(|entity| meshes.contains(entity))
        });
        let surface = &mut runtime.surfaces[index];
        let Some((root, _, avatar)) = desired else {
            if let Some(camera) = surface.camera.take() {
                commands.entity(camera).try_despawn();
            }
            surface.image = None;
            for (kind, mut image) in &mut slots {
                if *kind == slot && image.image != Handle::default() {
                    image.image = Handle::default();
                    if *kind == ServicePortraitSlot::Vendor {
                        image.color = Color::NONE;
                    }
                }
            }
            continue;
        };
        for entity in children
            .iter_descendants(root)
            .filter(|entity| meshes.contains(*entity))
            .chain(lights.iter())
        {
            admitted.entry(entity).or_default().push(layer);
        }
        let size = slot.size();
        let target = surface
            .image
            .get_or_insert_with(|| {
                images.add(Image::new_target_texture(
                    size.x,
                    size.y,
                    TextureFormat::Rgba8UnormSrgb,
                    None,
                ))
            })
            .clone();
        let pose = camera_pose(slot, avatar);
        if let Some((mut local, mut global)) =
            surface.camera.and_then(|id| cameras.get_mut(id).ok())
        {
            local.set_if_neq(pose);
            global.set_if_neq(GlobalTransform::from(pose));
        } else {
            surface.camera = Some(
                commands
                    .spawn((
                        Name::new(format!("{slot:?} NPC portrait")),
                        ServiceCamera,
                        Camera3d::default(),
                        (
                            Camera {
                                order: -14 + index as isize,

                                clear_color: ClearColorConfig::Custom(Color::NONE),
                                ..default()
                            },
                            bevy::camera::RenderTarget::from(target.clone()),
                        ),
                        Projection::Perspective(PerspectiveProjection {
                            fov: if slot == ServicePortraitSlot::Vendor {
                                70_f32
                            } else {
                                45_f32
                            }
                            .to_radians(),
                            near: 0.2,
                            far: if slot == ServicePortraitSlot::Vendor {
                                2.0
                            } else {
                                5.0
                            },
                            aspect_ratio: size.x as f32 / size.y as f32,
                            ..default()
                        }),
                        RenderLayers::layer(layer),
                        pose,
                        GlobalTransform::from(pose),
                    ))
                    .id(),
            );
        }
        for (kind, mut image) in &mut slots {
            if *kind == slot && image.image != target {
                image.image = target.clone();
                if *kind == ServicePortraitSlot::Vendor {
                    image.color = Color::WHITE;
                }
            }
        }
        bound[index] = true;
    }
    if owners.combi_ui.primary_npc_camera_bound != bound[0] {
        owners.combi_ui.primary_npc_camera_bound = bound[0];
    }
    if owners.combi_ui.waiting_npc_camera_bound != bound[1] {
        owners.combi_ui.waiting_npc_camera_bound = bound[1];
    }
    // Remove only leased visibility bits; other writers retain their changes.
    for (entity, lease) in &leased {
        let requested = admitted.remove(&entity).unwrap_or_default();
        let current = layers
            .get(entity)
            .ok()
            .flatten()
            .cloned()
            .unwrap_or_default();
        let wanted: Vec<_> = requested
            .into_iter()
            .filter(|layer| {
                lease.layers.contains(layer) || !current.iter().any(|bit| bit == *layer)
            })
            .collect();
        // Late rig reparenting can restore the gameplay layer after admission.
        // A lease records ownership, not proof that another writer kept the bits.
        if wanted == lease.layers
            && wanted
                .iter()
                .all(|bit| current.iter().any(|layer| layer == *bit))
        {
            continue;
        }
        if let Ok(current) = layers.get(entity) {
            let mut next = current.cloned().unwrap_or_default();
            for layer in &lease.layers {
                next = next.without(*layer);
            }
            for layer in &wanted {
                next = next.with(*layer);
            }
            if wanted.is_empty() && lease.implicit && next == RenderLayers::default() {
                commands.entity(entity).remove::<RenderLayers>();
            } else {
                commands.entity(entity).insert(next);
            }
        }
        if wanted.is_empty() {
            commands.entity(entity).remove::<ServiceLayerLease>();
        } else {
            commands.entity(entity).insert(ServiceLayerLease {
                implicit: lease.implicit,
                layers: wanted,
            });
        }
    }
    for (entity, wanted) in admitted {
        if let Ok(current) = layers.get(entity) {
            let mut next = current.cloned().unwrap_or_default();
            // Never claim ownership of a layer that was already present.
            let owned: Vec<_> = wanted
                .into_iter()
                .filter(|layer| !next.iter().any(|bit| bit == *layer))
                .collect();
            for layer in &owned {
                next = next.with(*layer);
            }
            if !owned.is_empty() {
                commands.entity(entity).insert((
                    next,
                    ServiceLayerLease {
                        implicit: current.is_none(),
                        layers: owned,
                    },
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests;

pub(super) fn vendor_capture_diagnostics(world: &mut World) -> serde_json::Value {
    let runtime = world.resource::<ServicePortraitRuntime>();
    let surface = &runtime.surfaces[ServicePortraitSlot::Vendor.index()];
    let camera = surface.camera;
    let target = surface.image.clone();
    let owner = world
        .resource::<VendorModeProjection0104>()
        .session
        .requested_npc_id;
    let root = world
        .query::<(Entity, &NetworkNpcAppearance0104)>()
        .iter(world)
        .find(|(_, npc)| npc.0.npc_id == owner)
        .map(|(entity, _)| entity);
    let mut descendants = Vec::new();
    if let Some(root) = root {
        descendants = {
            let mut pending = vec![root];
            let mut all = Vec::new();
            while let Some(entity) = pending.pop() {
                if let Some(children) = world.get::<Children>(entity) {
                    pending.extend(children.iter());
                    all.extend(children.iter());
                }
            }
            all
        };
    }
    let meshes: Vec<_> = descendants
        .iter()
        .filter(|e| world.get::<Mesh3d>(**e).is_some())
        .map(|e| {
            serde_json::json!({
                "entity":format!("{e:?}"), "pose":format!("{:?}",world.get::<GlobalTransform>(*e)),
                "visibility":format!("{:?}",world.get::<InheritedVisibility>(*e)),
                "layers":format!("{:?}",world.get::<RenderLayers>(*e)),
            })
        })
        .collect();
    serde_json::json!({"owner":owner,"camera":format!("{camera:?}"),
        "pose":camera.map(|e|format!("{:?}",world.get::<Transform>(e))),
        "projection":camera.map(|e|format!("{:?}",world.get::<Projection>(e))),
        "target":format!("{target:?}"),"meshes":meshes})
}
