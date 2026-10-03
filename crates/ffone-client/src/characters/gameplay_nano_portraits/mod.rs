//! Retrobution-compatible animated Nano portraits for the gameplay HUD.
//!
//! `cnNanoWheel` does not normally draw the Nano icon texture. With animated
//! Nanos enabled it asks `InventoryManagerScript.NanoClothes` for three live
//! `NanoAnimation` actors and renders each through a `cnSimpleCharRenderCamera`.
//! This module reproduces that path with validated native GLBs and three
//! transparent render targets.

use std::collections::BTreeMap;

use bevy::{
    animation::RepeatAnimation,
    asset::LoadState,
    camera::visibility::RenderLayers,
    gltf::{Gltf, GltfAssetLabel},
    prelude::*,
    render::render_resource::TextureFormat,
};
use serde_json::Value;

use crate::{
    assets::AssetLocator,
    character_scene::{NativeSceneRole, native_scene_container_transform},
    gameplay_ui::{GameplayNanoPortraitImages, GameplayUiModel},
    legacy_model_material::{
        LegacyColorWriteMask, LegacyModelMaterial, PendingLegacyModelMaterial,
        load_legacy_main_texture_replacement,
    },
    tutorial_nano_presentation::{
        TUTORIAL_NANO_FACE_MATERIAL_NAME, TUTORIAL_NANO_FACE_TEXTURE_PATH,
    },
};

const CHARACTER_REGISTRY_SCHEMA: &str = "ffone.semantic-character-registry.v2";
#[cfg(test)]
use crate::assets::TABLE_SET_PATH;
const CONSOLIDATED_TABLE: &str = "npc_imports_consolidated";

pub const GAMEPLAY_NANO_PORTRAIT_COUNT: usize = 3;
pub const GAMEPLAY_NANO_PORTRAIT_RENDER_LAYERS: [usize; 3] = [22, 23, 24];
pub const GAMEPLAY_NANO_PORTRAIT_TEXTURE_SIZE: u32 = 144;
pub const GAMEPLAY_NANO_PORTRAIT_TEXTURE_HEIGHT: u32 = 216;
pub const JOURNAL_NANO_PORTRAIT_RENDER_LAYER: usize = 21;
pub const JOURNAL_NANO_PORTRAIT_TEXTURE_SIZE: u32 = 128;
pub const GAMEPLAY_NANO_CAMERA_FOV_DEGREES: f32 = 45.0;
pub const GAMEPLAY_NANO_CAMERA_NEAR: f32 = 0.3;
pub const GAMEPLAY_NANO_CAMERA_FAR: f32 = 1_000.0;
pub const GAMEPLAY_NANO_CAMERA_DISTANCE: f32 = 0.8;
pub const GAMEPLAY_NANO_CAMERA_HEIGHT: f32 = 0.3;
pub const GAMEPLAY_NANO_CAMERA_PITCH_DEGREES: f32 = -10.0;
pub const GAMEPLAY_NANO_DEPLETED_CAMERA_OFFSET: f32 = 0.48;

/// Exact native model routes selected by
/// `m_pNanoData.m_iMesh -> m_pNanoMeshData -> semantic character registry`.
#[derive(Clone, Debug, Default, Resource)]
pub struct GameplayNanoPortraitCatalog {
    model_paths: BTreeMap<i16, String>,
}

impl GameplayNanoPortraitCatalog {
    pub fn open(locator: &AssetLocator) -> Result<Self, String> {
        let table: Value = locator.read_table_set()?;
        let registry =
            ffone_client_foundation::asset_tables::character_models_from_document(&table)?;
        Self::from_documents(&table, &registry, |glb| {
            locator.require_file(glb).map(|_| ())
        })
    }

    pub fn from_project_assets(assets: &AssetLocator) -> Result<Self, String> {
        Self::open(assets)
    }

    fn from_documents(
        table_set: &Value,
        registry: &Value,
        mut require_model: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, String> {
        if registry.get("schema").and_then(Value::as_str) != Some(CHARACTER_REGISTRY_SCHEMA) {
            return Err(format!(
                "native character registry must use {CHARACTER_REGISTRY_SCHEMA:?}"
            ));
        }
        let registry_models = registry
            .get("models")
            .and_then(Value::as_array)
            .ok_or_else(|| "native character registry has no models array".to_owned())?;
        let mut native_nanos = BTreeMap::<String, String>::new();
        for (index, entry) in registry_models.iter().enumerate() {
            if entry.get("category").and_then(Value::as_str) != Some("nano") {
                continue;
            }
            let id = required_string(entry, "id", &format!("registry.models[{index}]"))?;
            let glb = required_string(entry, "glb", &format!("registry.models[{index}]"))?;
            if !entry
                .get("animations")
                .and_then(Value::as_array)
                .is_some_and(|animations| {
                    animations.iter().any(|animation| {
                        animation
                            .as_str()
                            .is_some_and(|name| name.eq_ignore_ascii_case("stand1"))
                    })
                })
            {
                return Err(format!(
                    "native Nano registry entry {id:?} has no stand1 animation"
                ));
            }
            require_model(glb)?;
            if let Some(previous) = native_nanos.insert(id.to_owned(), glb.to_owned())
                && previous != glb
            {
                return Err(format!(
                    "native character registry maps Nano {id:?} to contradictory GLBs"
                ));
            }
        }

        let tables = table_set
            .get("tables")
            .and_then(Value::as_array)
            .ok_or_else(|| "TableData table-set has no tables array".to_owned())?;
        let matches = tables
            .iter()
            .filter(|table| table.get("name").and_then(Value::as_str) == Some(CONSOLIDATED_TABLE))
            .collect::<Vec<_>>();
        let [table] = matches.as_slice() else {
            return Err(format!(
                "TableData must contain exactly one {CONSOLIDATED_TABLE:?} table"
            ));
        };
        let nano_table = table
            .pointer("/value/m_pNanoTable")
            .ok_or_else(|| "consolidated TableData has no m_pNanoTable".to_owned())?;
        let rows = nano_table
            .get("m_pNanoData")
            .and_then(Value::as_array)
            .ok_or_else(|| "m_pNanoTable has no m_pNanoData array".to_owned())?;
        let meshes = nano_table
            .get("m_pNanoMeshData")
            .and_then(Value::as_array)
            .ok_or_else(|| "m_pNanoTable has no m_pNanoMeshData array".to_owned())?;

        let mut model_paths = BTreeMap::new();
        for (row_index, row) in rows.iter().enumerate() {
            let context = format!("m_pNanoData[{row_index}]");
            let nano_number = required_i64(row, "m_iNanoNumber", &context)?;
            if nano_number == 0 {
                continue;
            }
            let nano_id = i16::try_from(nano_number)
                .ok()
                .filter(|id| *id > 0)
                .ok_or_else(|| format!("{context}.m_iNanoNumber must be a positive i16"))?;
            let mesh_index = usize::try_from(required_i64(row, "m_iMesh", &context)?)
                .map_err(|_| format!("{context}.m_iMesh must be non-negative"))?;
            let mesh = meshes.get(mesh_index).ok_or_else(|| {
                format!("{context}.m_iMesh references missing m_pNanoMeshData[{mesh_index}]")
            })?;
            let logical_name =
                required_string(mesh, "m_pstrMMeshModelString", "m_pNanoMeshData row")?;
            if logical_name.is_empty() || logical_name == "null" {
                return Err(format!(
                    "{context} resolves to an empty native Nano model name"
                ));
            }
            let registry_id = format!("nano/{logical_name}");
            // Unpublished models remain unresolved; a portrait never substitutes
            // another Nano's model or a 2D icon.
            let Some(model_path) = native_nanos.get(&registry_id) else {
                continue;
            };
            if let Some(previous) = model_paths.insert(nano_id, model_path.clone())
                && previous != *model_path
            {
                return Err(format!(
                    "TableData maps Nano ID {nano_id} to contradictory native models"
                ));
            }
        }
        if model_paths.is_empty() {
            return Err("Nano portrait catalog resolved no native models".to_owned());
        }
        Ok(Self { model_paths })
    }

    #[must_use]
    pub fn model_path(&self, nano_id: i16) -> Option<&str> {
        self.model_paths.get(&nano_id).map(String::as_str)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.model_paths.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.model_paths.is_empty()
    }
}

/// The mission journal owns a fourth `cnSimpleCharRenderCamera`-style target.
/// Keeping the request separate from the three Nano-wheel slots mirrors the
/// clean client's journal camera lifetime and prevents a 2D icon fallback.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct JournalNanoPortraitRequest {
    nano_id: Option<i16>,
    model_path: Option<String>,
}

impl JournalNanoPortraitRequest {
    pub fn set(&mut self, nano_id: i16, model_path: impl Into<String>) {
        self.nano_id = Some(nano_id);
        self.model_path = Some(model_path.into());
    }

    pub fn clear(&mut self) {
        self.nano_id = None;
        self.model_path = None;
    }

    #[must_use]
    pub fn desired(&self) -> Option<(i16, &str)> {
        Some((self.nano_id?, self.model_path.as_deref()?))
    }
}

/// Runtime-generated mission-journal Nano portrait. No PNG substitute is used.
#[derive(Default, Resource)]
pub struct JournalNanoPortraitImage(pub Option<Handle<Image>>);

#[derive(Component)]
struct GameplayNanoPortraitRoot {
    slot: usize,
    nano_id: i16,
    model_path: String,
    gltf: Handle<Gltf>,
}

#[derive(Component)]
struct GameplayNanoPortraitCamera {
    slot: usize,
}

#[derive(Component)]
struct GameplayNanoPortraitLayerBound;

#[derive(Component)]
struct GameplayNanoPortraitAnimationApplied;

#[derive(Component)]
struct GameplayNanoPortraitFaceBound;

#[derive(Component)]
struct GameplayNanoPortraitAlphaBound;

pub struct GameplayNanoPortraitPlugin;

impl Plugin for GameplayNanoPortraitPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameplayNanoPortraitImages>()
            .init_resource::<JournalNanoPortraitRequest>()
            .init_resource::<JournalNanoPortraitImage>()
            .add_systems(
                Update,
                (
                    sync_gameplay_nano_portrait_rigs,
                    bind_gameplay_nano_portrait_layers,
                    bind_gameplay_nano_portrait_faces,
                    bind_gameplay_nano_portrait_alpha,
                    play_gameplay_nano_portrait_stand,
                    update_gameplay_nano_portrait_cameras,
                )
                    .chain(),
            );
    }
}

fn sync_gameplay_nano_portrait_rigs(
    mut commands: Commands,
    model: Res<GameplayUiModel>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut portraits: ResMut<GameplayNanoPortraitImages>,
    mut journal_portrait: ResMut<JournalNanoPortraitImage>,
    journal_request: Res<JournalNanoPortraitRequest>,
    roots: Query<(Entity, &GameplayNanoPortraitRoot)>,
) {
    for slot in 0..=GAMEPLAY_NANO_PORTRAIT_COUNT {
        let desired = if slot < GAMEPLAY_NANO_PORTRAIT_COUNT {
            model
                .visible
                .then(|| {
                    let nano = &model.nanos[slot];
                    Some((nano.nano_id?, nano.model_path.as_deref()?))
                })
                .flatten()
        } else {
            journal_request.desired()
        };
        let existing = roots.iter().find(|(_, root)| root.slot == slot);
        if existing.is_some_and(|(_, root)| {
            desired
                .is_some_and(|(nano_id, path)| root.nano_id == nano_id && root.model_path == path)
        }) {
            continue;
        }
        if let Some((entity, _)) = existing {
            commands.entity(entity).despawn();
        }
        let old = if slot < GAMEPLAY_NANO_PORTRAIT_COUNT {
            portraits.0[slot].take()
        } else {
            journal_portrait.0.take()
        };
        if let Some(old) = old {
            images.remove(old.id());
        }
        let Some((nano_id, model_path)) = desired else {
            continue;
        };

        let target = images.add(Image::new_target_texture(
            portrait_texture_size(slot),
            portrait_texture_height(slot),
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        if slot < GAMEPLAY_NANO_PORTRAIT_COUNT {
            portraits.0[slot] = Some(target.clone());
        } else {
            journal_portrait.0 = Some(target.clone());
        }
        let gltf = asset_server.load(model_path.to_owned());
        let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_path.to_owned()));
        let layers = RenderLayers::layer(portrait_render_layer(slot));
        let (camera_translation, camera_rotation) = authored_nano_camera_transform(0.0);
        commands
            .spawn((
                Name::new(format!("Gameplay Nano portrait {} root", slot + 1)),
                GameplayNanoPortraitRoot {
                    slot,
                    nano_id,
                    model_path: model_path.to_owned(),
                    gltf,
                },
                Transform::IDENTITY,
                Visibility::Hidden,
            ))
            .with_children(|root| {
                root.spawn((
                    Name::new(format!("Gameplay Nano portrait {} Scene0", slot + 1)),
                    WorldAssetRoot(scene),
                    native_scene_container_transform(NativeSceneRole::CharacterGameplay),
                ));
                root.spawn((
                    Name::new(format!("Gameplay Nano portrait {} camera", slot + 1)),
                    GameplayNanoPortraitCamera { slot },
                    Camera3d::default(),
                    bevy::camera::RenderTarget::from(target.clone()),
                    Camera {
                        order: -10 + slot as isize,
                        clear_color: ClearColorConfig::Custom(Color::NONE),
                        sub_camera_view: portrait_sub_camera_view(slot),
                        ..default()
                    },
                    Projection::Perspective(PerspectiveProjection {
                        fov: GAMEPLAY_NANO_CAMERA_FOV_DEGREES.to_radians(),
                        near: GAMEPLAY_NANO_CAMERA_NEAR,
                        far: GAMEPLAY_NANO_CAMERA_FAR,
                        ..default()
                    }),
                    layers.clone(),
                    Transform {
                        translation: camera_translation,
                        rotation: camera_rotation,
                        ..default()
                    },
                ));
                root.spawn((
                    Name::new(format!("Gameplay Nano portrait {} light", slot + 1)),
                    DirectionalLight {
                        illuminance: 8_000.0,
                        shadow_maps_enabled: false,
                        ..default()
                    },
                    Transform::from_xyz(-2.0, 4.0, 3.0)
                        .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
                    layers,
                ));
            });
    }
}

fn bind_gameplay_nano_portrait_layers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    children: Query<&Children>,
    surfaces: Query<Entity, (With<Mesh3d>, Without<GameplayNanoPortraitLayerBound>)>,
    mut roots: Query<(Entity, &GameplayNanoPortraitRoot, &mut Visibility)>,
    mut hierarchy_scratch: Local<Vec<Entity>>,
) {
    for (root_entity, root, mut visibility) in &mut roots {
        if matches!(
            asset_server.load_state(root.gltf.id()),
            LoadState::Failed(_)
        ) {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }
        let mut surface_count = 0;
        visit_hierarchy(
            root_entity,
            &children,
            hierarchy_scratch.as_mut(),
            |entity| {
                if surfaces.get(entity).is_err() {
                    return;
                }
                surface_count += 1;
                commands.entity(entity).insert((
                    RenderLayers::layer(portrait_render_layer(root.slot)),
                    GameplayNanoPortraitLayerBound,
                ));
                if root.slot < GAMEPLAY_NANO_PORTRAIT_COUNT {
                    // Bevy's CPU frustum uses Projection alone, without the
                    // camera's off-axis SubCameraView. Do not discard separate
                    // head/hair meshes in the newly exposed part of these tiny
                    // layer-isolated HUD rigs.
                    commands.entity(entity).insert(bevy::camera::visibility::NoFrustumCulling);
                }
            },
        );
        if surface_count > 0 && *visibility != Visibility::Inherited {
            *visibility = Visibility::Inherited;
        }
    }
}

/// Walk only one portrait's asynchronously materialized scene hierarchy.
///
/// The old binders iterated every unbound mesh/material/player in the resident
/// world once for each portrait and then climbed `ChildOf` to reject nearly all
/// of them. A portrait owns a small, disjoint subtree, so walking down from its
/// root preserves ownership while making unrelated world complexity irrelevant.
fn visit_hierarchy(
    root: Entity,
    children: &Query<&Children>,
    scratch: &mut Vec<Entity>,
    mut visit: impl FnMut(Entity),
) {
    scratch.clear();
    scratch.push(root);
    while let Some(entity) = scratch.pop() {
        visit(entity);
        if let Ok(descendants) = children.get(entity) {
            scratch.extend(descendants.iter());
        }
    }
}

fn bind_gameplay_nano_portrait_faces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    children: Query<&Children>,
    roots: Query<(Entity, &GameplayNanoPortraitRoot)>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut surfaces: Query<
        (
            &mut MeshMaterial3d<LegacyModelMaterial>,
            &PendingLegacyModelMaterial,
        ),
        Without<GameplayNanoPortraitFaceBound>,
    >,
    mut hierarchy_scratch: Local<Vec<Entity>>,
) {
    for (root_entity, root) in &roots {
        if root.nano_id != 1 {
            continue;
        }
        visit_hierarchy(
            root_entity,
            &children,
            hierarchy_scratch.as_mut(),
            |entity| {
                let Ok((mut handle, metadata)) = surfaces.get_mut(entity) else {
                    return;
                };
                if metadata.true_name != TUTORIAL_NANO_FACE_MATERIAL_NAME {
                    return;
                }
                let Ok(face_texture) = load_legacy_main_texture_replacement(
                    &asset_server,
                    metadata,
                    TUTORIAL_NANO_FACE_TEXTURE_PATH,
                ) else {
                    return;
                };
                crate::legacy_model_material::make_legacy_material_unique(
                    &mut handle.0,
                    &mut materials,
                );
                let Some(mut material) = materials.get_mut(&handle.0) else {
                    return;
                };
                material.base_texture = Some(face_texture);
                commands
                    .entity(entity)
                    .insert(GameplayNanoPortraitFaceBound);
            },
        );
    }
}

/// RGB-only source passes work against the world framebuffer but leave a
/// transparent portrait target's alpha at zero. Give only portrait instances
/// an alpha-writing copy so the HUD/journal can composite their visible pixels.
/// Color blending, depth, culling, textures and pass order retain source values.
fn bind_gameplay_nano_portrait_alpha(
    mut commands: Commands,
    children: Query<&Children>,
    roots: Query<Entity, With<GameplayNanoPortraitRoot>>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut surfaces: Query<
        &mut MeshMaterial3d<LegacyModelMaterial>,
        Without<GameplayNanoPortraitAlphaBound>,
    >,
    mut hierarchy_scratch: Local<Vec<Entity>>,
) {
    for root in &roots {
        visit_hierarchy(root, &children, hierarchy_scratch.as_mut(), |entity| {
            let Ok(mut handle) = surfaces.get_mut(entity) else {
                return;
            };
            let Some(material) = materials.get(&handle.0) else {
                return;
            };
            if material.render_mode.color_write == LegacyColorWriteMask::Rgb {
                let mut portrait_material = material.clone();
                portrait_material.render_mode.color_write = LegacyColorWriteMask::Rgba;
                handle.0 = materials.add(portrait_material);
            }
            commands
                .entity(entity)
                .insert(GameplayNanoPortraitAlphaBound);
        });
    }
}

fn play_gameplay_nano_portrait_stand(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    children: Query<&Children>,
    roots: Query<(Entity, &GameplayNanoPortraitRoot)>,
    mut players: Query<&mut AnimationPlayer, Without<GameplayNanoPortraitAnimationApplied>>,
    mut hierarchy_scratch: Local<Vec<Entity>>,
) {
    for (root_entity, root) in &roots {
        let Some(gltf) = gltfs.get(&root.gltf) else {
            continue;
        };
        let Some(clip) = gltf
            .named_animations
            .get("stand1")
            .or_else(|| gltf.named_animations.get("Stand1"))
            .cloned()
        else {
            continue;
        };
        visit_hierarchy(
            root_entity,
            &children,
            hierarchy_scratch.as_mut(),
            |entity| {
                let Ok(mut player) = players.get_mut(entity) else {
                    return;
                };
                let (graph, node) = AnimationGraph::from_clip(clip.clone());
                player.stop_all();
                player.start(node).set_repeat(RepeatAnimation::Forever);
                commands.entity(entity).insert((
                    AnimationGraphHandle(graphs.add(graph)),
                    GameplayNanoPortraitAnimationApplied,
                ));
            },
        );
    }
}

fn update_gameplay_nano_portrait_cameras(
    model: Res<GameplayUiModel>,
    mut cameras: Query<(&GameplayNanoPortraitCamera, &mut Transform)>,
) {
    for (camera, mut transform) in &mut cameras {
        let (translation, rotation) = if camera.slot < GAMEPLAY_NANO_PORTRAIT_COUNT {
            let nano = &model.nanos[camera.slot];
            let stamina = nano.stamina_fraction.clamp(0.0, 1.0);
            let depleted_offset = if !nano.active && stamina < 0.2 {
                GAMEPLAY_NANO_DEPLETED_CAMERA_OFFSET * (1.0 - stamina / 0.2)
            } else {
                0.0
            };
            authored_nano_camera_transform(depleted_offset)
        } else {
            authored_journal_nano_camera_transform()
        };
        // Avoid propagating an unchanged portrait camera transform through
        // Bevy's transform hierarchy on every rendered frame.
        if transform.translation != translation || transform.rotation != rotation {
            transform.translation = translation;
            transform.rotation = rotation;
        }
    }
}

fn portrait_render_layer(slot: usize) -> usize {
    if slot < GAMEPLAY_NANO_PORTRAIT_COUNT {
        GAMEPLAY_NANO_PORTRAIT_RENDER_LAYERS[slot]
    } else {
        JOURNAL_NANO_PORTRAIT_RENDER_LAYER
    }
}

fn portrait_texture_size(slot: usize) -> u32 {
    if slot < GAMEPLAY_NANO_PORTRAIT_COUNT {
        GAMEPLAY_NANO_PORTRAIT_TEXTURE_SIZE
    } else {
        JOURNAL_NANO_PORTRAIT_TEXTURE_SIZE
    }
}

fn portrait_texture_height(slot: usize) -> u32 {
    if slot < GAMEPLAY_NANO_PORTRAIT_COUNT {
        GAMEPLAY_NANO_PORTRAIT_TEXTURE_HEIGHT
    } else {
        JOURNAL_NANO_PORTRAIT_TEXTURE_SIZE
    }
}

fn portrait_sub_camera_view(slot: usize) -> Option<bevy::camera::SubCameraView> {
    (slot < GAMEPLAY_NANO_PORTRAIT_COUNT).then(|| bevy::camera::SubCameraView {
        full_size: UVec2::splat(GAMEPLAY_NANO_PORTRAIT_TEXTURE_SIZE),
        // Extend only above the original view. Changing FOV or stretching the
        // square texture would move or enlarge the Nano inside the wheel.
        offset: Vec2::new(
            0.0,
            -((GAMEPLAY_NANO_PORTRAIT_TEXTURE_HEIGHT - GAMEPLAY_NANO_PORTRAIT_TEXTURE_SIZE) as f32),
        ),
        size: UVec2::new(
            GAMEPLAY_NANO_PORTRAIT_TEXTURE_SIZE,
            GAMEPLAY_NANO_PORTRAIT_TEXTURE_HEIGHT,
        ),
    })
}

fn authored_journal_nano_camera_transform() -> (Vec3, Quat) {
    // `cnMissionJournal.SetMissionInfo` calls `SetAvatar` and immediately
    // disables `cmNano`. Consequently this camera uses the actor's unpitched
    // forward vector, calls LookAt before adding fHeight, and never reaches
    // `cnSimpleCharRenderCamera.Update`'s `vAngle` path.
    let authored_position = Vec3::Z * GAMEPLAY_NANO_CAMERA_DISTANCE;
    let look_at_position = Quat::from_rotation_y(std::f32::consts::PI) * authored_position;
    let rotation = Transform::from_translation(look_at_position)
        .looking_at(Vec3::ZERO, Vec3::Y)
        .rotation;
    (
        look_at_position + Vec3::Y * GAMEPLAY_NANO_CAMERA_HEIGHT,
        rotation,
    )
}

fn authored_nano_camera_transform(depleted_offset: f32) -> (Vec3, Quat) {
    // Exact `cnSimpleCharRenderCamera.Update`: rotate Vector3.forward by
    // vAngle, multiply by Distance, LookAt the actor, and only then add
    // fHeight. `DrawNanoCamera` adds the depletion offset without recomputing
    // the rotation.
    let authored_position = Quat::from_rotation_x(GAMEPLAY_NANO_CAMERA_PITCH_DEGREES.to_radians())
        * Vec3::Z
        * GAMEPLAY_NANO_CAMERA_DISTANCE;
    // Native character scenes own the shared 180-degree gameplay-container
    // turn. Move the camera to the corresponding native side, just as the
    // full player-preview camera does, so one-sided toon surfaces face it.
    let look_at_position = Quat::from_rotation_y(std::f32::consts::PI) * authored_position;
    let rotation = Transform::from_translation(look_at_position)
        .looking_at(Vec3::ZERO, Vec3::Y)
        .rotation;
    (
        look_at_position + Vec3::Y * (GAMEPLAY_NANO_CAMERA_HEIGHT + depleted_offset),
        rotation,
    )
}

fn required_string<'a>(value: &'a Value, field: &str, context: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{context}.{field} must be a string"))
}

fn required_i64(value: &Value, field: &str, context: &str) -> Result<i64, String> {
    value
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("{context}.{field} must be an integer"))
}

#[cfg(test)]
mod tests;
