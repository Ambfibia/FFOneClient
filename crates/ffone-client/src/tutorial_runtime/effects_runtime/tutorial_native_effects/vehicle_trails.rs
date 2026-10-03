//! Persistent vehicle ribbons, owned by the model's animated exhaust sockets.
use super::*;
use crate::{
    assets::AssetLocator, tutorial_player_rig_runtime::personal_vehicle::VehicleAttachment,
};
use bevy::camera::visibility::RenderLayers;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema: String,
    trails: Vec<Definition>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Definition {
    name: String,
    length: usize,
    height: f32,
    interpolation_step: f32,
    blend_mode: ParticleBlendMode,
    material_tint: [f32; 4],
    texture: Vec<String>,
    mesh: Option<String>,
}

struct Visual {
    length: usize,
    height: f32,
    step: f32,
    material: Handle<TutorialParticleMaterial>,
    mesh: Option<String>,
}

#[derive(Default, Resource)]
struct VehicleVisuals {
    loaded: bool,
    plans: BTreeMap<String, Visual>,
}

#[derive(Component)]
struct Bound;

#[derive(Component, Default)]
pub(super) struct Tip {
    pub(super) ready: bool,
}

#[derive(Component)]
struct Ribbon {
    owner: Entity,
    socket: Entity,
    points: Vec<Vec3>,
    height: f32,
    step: f32,
    elapsed: f32,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<VehicleVisuals>().add_systems(
        PostUpdate,
        (prepare, prepare_tips, update)
            .chain()
            .after(TransformSystems::Propagate),
    );
}

fn style(model: &str) -> &'static str {
    if model.split('/').any(|part| part == "vehicle_hoverboard") {
        "board"
    } else if model
        .split('/')
        .any(|part| matches!(part, "vehicle_cloud" | "vehicle_hoverboard2"))
    {
        "cloud"
    } else {
        "standard"
    }
}

fn read_catalog(locator: &AssetLocator) -> Result<Vec<(Definition, ExactTexture)>, String> {
    let bytes =
        std::fs::read(locator.path("effects/vehicles/catalog.json")?).map_err(|e| e.to_string())?;
    let catalog: Catalog = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if catalog.schema != "ffone.vehicle-trails.v1" {
        return Err("unknown vehicle trail schema".into());
    }
    let mut names = BTreeSet::new();
    let mut result = Vec::new();
    for d in catalog.trails {
        if !names.insert(d.name.clone())
            || !matches!(d.name.as_str(), "standard" | "board" | "cloud")
            || !(2..=1024).contains(&d.length)
            || !d.height.is_finite()
            || d.height <= 0.0
            || !d.interpolation_step.is_finite()
            || d.interpolation_step < 0.001
            || d.material_tint.iter().any(|v| !v.is_finite())
            || d.texture.is_empty()
            || d.texture.len() > 32
        {
            return Err("invalid vehicle trail definition".into());
        }
        if let Some(mesh) = &d.mesh {
            locator.path(mesh)?;
        }
        let mut pixels = Vec::new();
        let mut size = (0, 0);
        for (level, path) in d.texture.iter().enumerate() {
            let bytes = std::fs::read(locator.path(path)?).map_err(|e| e.to_string())?;
            let image = image::load_from_memory(&bytes)
                .map_err(|e| e.to_string())?
                .to_rgba8();
            if level == 0 {
                size = image.dimensions();
            }
            if image.dimensions() != ((size.0 >> level).max(1), (size.1 >> level).max(1)) {
                return Err("invalid vehicle trail mip dimensions".into());
            }
            pixels.extend_from_slice(image.as_raw());
        }
        let texture = ExactTexture {
            key: (
                d.texture[0].clone(),
                0,
                blake3::hash(&pixels).to_hex().to_string(),
            ),
            width: size.0,
            height: size.1,
            mip_count: d.texture.len() as u32,
            rgba_mips: pixels.into(),
        };
        result.push((d, texture));
    }
    if names.len() != 3 {
        return Err("incomplete vehicle trail catalog".into());
    }
    Ok(result)
}

fn prepare(
    mut commands: Commands,
    locator: Option<Res<AssetLocator>>,
    assets: Res<AssetServer>,
    vehicles: Query<(Entity, &VehicleAttachment, Option<&RenderLayers>)>,
    bound: Query<(), With<Bound>>,
    scene_ready: Query<&bevy::world_serialization::WorldInstance>,
    scene_spawner: Res<bevy::world_serialization::WorldInstanceSpawner>,
    children: Query<&Children>,
    names: Query<&Name>,
    globals: Query<&GlobalTransform>,
    mut visuals: ResMut<VehicleVisuals>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<TutorialParticleMaterial>>,
) {
    if vehicles.is_empty() {
        // Strong GPU handles and decoded staging are not retained between rides.
        if visuals.loaded {
            *visuals = VehicleVisuals::default();
        }
        return;
    }
    if !visuals.loaded {
        let Some(locator) = locator else {
            return;
        };
        visuals.loaded = true;
        match read_catalog(&locator) {
            Ok(definitions) => {
                let mut cache = NativeVisualAssets::default();
                for (d, texture) in definitions {
                    let plan = TrailPlan {
                        source_asset: d.name.clone(),
                        source_path_id: 0,
                        length: d.length,
                        height: d.height,
                        interpolation_step: d.interpolation_step,
                        blend_mode: d.blend_mode,
                        material_tint: Vec4::from_array(d.material_tint),
                        texture,
                    };
                    let material =
                        shared_trail_material(&plan, &mut images, &mut materials, &mut cache);
                    visuals.plans.insert(
                        d.name,
                        Visual {
                            length: d.length,
                            height: d.height,
                            step: d.interpolation_step,
                            material,
                            mesh: d.mesh,
                        },
                    );
                }
            }
            Err(error) => error!("Vehicle trails could not load: {error}"),
        }
    }
    for (owner, vehicle, layers) in &vehicles {
        if bound.contains(owner)
            || !scene_ready
                .get(owner)
                .is_ok_and(|scene| scene_spawner.instance_is_ready(**scene))
        {
            continue;
        }
        let Some(visual) = visuals.plans.get(style(vehicle.model_path())) else {
            continue;
        };
        for socket in children.iter_descendants(owner) {
            if !names
                .get(socket)
                .is_ok_and(|n| matches!(n.as_str(), "tag01" | "tag02" | "tag03" | "tag04"))
            {
                continue;
            }
            let Ok(global) = globals.get(socket) else {
                continue;
            };
            let points = vec![global.translation(); visual.length];
            let mesh = meshes.add(trail_mesh(&points, Vec3::X, 0.0, visual.height));
            commands.spawn((
                Name::new("Vehicle exhaust trail"),
                Ribbon {
                    owner,
                    socket,
                    points,
                    height: visual.height,
                    step: visual.step,
                    elapsed: 0.0,
                },
                Mesh3d(mesh),
                MeshMaterial3d(visual.material.clone()),
                Transform::IDENTITY,
                Visibility::Inherited,
                NoFrustumCulling,
                layers.cloned().unwrap_or_default(),
            ));
            if let Some(path) = &visual.mesh {
                // LoadVehicle's effect root starts at unit world scale and is
                // parented preserving that scale, independently of the rig.
                let placement = Transform::from_matrix(
                    global.to_matrix().inverse()
                        * Mat4::from_rotation_translation(global.rotation(), global.translation()),
                );
                commands.spawn((
                    Name::new("Vehicle exhaust tip"),
                    Tip::default(),
                    ChildOf(socket),
                    WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(path.clone()))),
                    NativeMeshEffectPlayback {
                        gltf: assets.load(path.clone()),
                        graph: None,
                        repeat: true,
                        resolved: false,
                        started: false,
                    },
                    placement,
                    Visibility::Inherited,
                    layers.cloned().unwrap_or_default(),
                ));
            }
        }
        commands.entity(owner).insert(Bound);
    }
}

fn prepare_tips(
    mut commands: Commands,
    mut tips: Query<(Entity, &mut Tip)>,
    children: Query<&Children>,
    mut surfaces: Query<
        (Entity, &mut MeshMaterial3d<LegacyModelMaterial>),
        Without<NativePreparedMeshEffectSurface>,
    >,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut cache: ResMut<crate::legacy_model_material::LegacyStaticMaterialCache>,
) {
    for (root, mut tip) in &mut tips {
        if tip.ready {
            continue;
        }
        let mut found = false;
        for entity in children.iter_descendants(root) {
            let Ok((entity, mut handle)) = surfaces.get_mut(entity) else {
                continue;
            };
            let Some(mut material) = materials.get(&handle.0).cloned() else {
                continue;
            };
            material.sort_bias =
                mesh_effect_render_queue_sort_bias(material.render_mode.source_queue);
            handle.0 = cache.admit_immutable(material, &mut materials);
            commands
                .entity(entity)
                .insert(NativePreparedMeshEffectSurface);
            found = true;
        }
        tip.ready = found;
    }
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    cameras: Query<(&GlobalTransform, Option<&LegacyOrbitCamera>), With<Camera3d>>,
    owners: Query<&InheritedVisibility, With<VehicleAttachment>>,
    globals: Query<&GlobalTransform>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ribbons: Query<(Entity, &Mesh3d, &mut Ribbon, &mut Visibility)>,
) {
    let right = cameras
        .iter()
        .find(|(_, orbit)| orbit.is_some())
        .or_else(|| cameras.iter().next())
        .map(|(c, _)| c.rotation() * Vec3::X)
        .unwrap_or(Vec3::X);
    for (entity, handle, mut ribbon, mut visibility) in &mut ribbons {
        let Ok(socket) = globals.get(ribbon.socket) else {
            commands.entity(entity).despawn();
            continue;
        };
        let Ok(owner_visibility) = owners.get(ribbon.owner) else {
            commands.entity(entity).despawn();
            continue;
        };
        let desired = if owner_visibility.get() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != desired {
            *visibility = desired;
        }
        let step = ribbon.step;
        push_interpolated_trail_points(&mut ribbon.points, socket.translation(), step);
        ribbon.elapsed += time.delta_secs();
        if let Some(mut mesh) = meshes.get_mut(&handle.0) {
            *mesh = trail_mesh(&ribbon.points, right, ribbon.elapsed, ribbon.height);
        }
    }
}

#[cfg(test)]
mod tests;
