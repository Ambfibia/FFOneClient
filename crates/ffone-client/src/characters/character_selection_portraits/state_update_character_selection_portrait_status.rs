use super::*;

pub const CHARACTER_SELECTION_PORTRAIT_COUNT: usize = 4;

pub const CHARACTER_SELECTION_PORTRAIT_CAMERA_ORDER_BASE: isize =
    NATIVE_PLAYER_PREVIEW_CAMERA_ORDER + 1;

pub const CHARACTER_SELECTION_PORTRAIT_X: f32 = 26.0;

pub const CHARACTER_SELECTION_PORTRAIT_WIDTH: f32 = 63.0;

pub const CHARACTER_SELECTION_PORTRAIT_HEIGHT: f32 = 85.0;

pub const CHARACTER_SELECTION_PORTRAIT_Y: [f32; 4] = [31.0, 128.0, 225.0, 321.0];

/// Serialized override on sharedassets0 Camera0..Camera3
/// `cnSimpleCharRenderCamera` components, not the class defaults used by the
/// central 600x600 preview.
pub const CHARACTER_SELECTION_PORTRAIT_CAMERA_DISTANCE: f32 = 0.5;

pub const CHARACTER_SELECTION_PORTRAIT_CAMERA_HEIGHT: f32 = 0.12;

pub const CHARACTER_SELECTION_PORTRAIT_CAMERA_FOV_DEGREES: f32 = 45.0;

pub const CHARACTER_SELECTION_PORTRAIT_CAMERA_NEAR: f32 = 0.01;

pub const CHARACTER_SELECTION_PORTRAIT_CAMERA_FAR: f32 = 1_000.0;

/// Per-slot fail-closed state. `ReadyAnimated` is reached only after the real
/// equipped-idle rig, the unique head target, every dynamic material, and every
/// referenced PNG are ready.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CharacterSelectionPortraitStatus {
    #[default]
    Empty,
    Loading,
    ReadyAnimated {
        actor_bones: usize,
        parts: usize,
    },
    Blocked(String),
}

#[derive(Clone, Debug)]
pub struct CharacterSelectionPortraitSlot {
    pub(super) look: Option<NativePlayerLook>,
    pub(super) revision: u64,
    pub status: CharacterSelectionPortraitStatus,
}

impl Default for CharacterSelectionPortraitSlot {
    fn default() -> Self {
        Self {
            look: None,
            revision: 0,
            status: CharacterSelectionPortraitStatus::Empty,
        }
    }
}

impl CharacterSelectionPortraitSlot {
    #[must_use]
    pub fn look(&self) -> Option<&NativePlayerLook> {
        self.look.as_ref()
    }

    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

#[derive(Component, Clone)]
pub(super) struct CharacterSelectionPortraitPart {
    pub(super) slot: usize,
    pub(super) kind: NativePlayerPartKind,
    pub(super) exact_route: String,
    pub(super) generation: u64,
}

#[derive(Component)]
pub(super) struct PendingCharacterSelectionPortraitAttachment {
    pub(super) rig_root: Entity,
    pub(super) socket_full_path: String,
    pub(super) socket_local_scale_override: Option<Vec3>,
}

#[derive(Component)]
pub(super) struct CharacterSelectionPortraitCamera(pub(super) usize);

#[derive(Component)]
pub struct CharacterSelectionPortraitLight;

#[derive(Default)]
pub(super) struct CharacterSelectionPortraitRuntimeSlot {
    pub(super) built_revision: u64,
    pub(super) root: Option<Entity>,
    pub(super) camera: Option<Entity>,
    pub(super) textures: Vec<(String, Handle<Image>)>,
    pub(super) ready_once: bool,
}

#[derive(Resource)]
pub(super) struct CharacterSelectionPortraitsRuntime {
    pub(super) slots: [CharacterSelectionPortraitRuntimeSlot; CHARACTER_SELECTION_PORTRAIT_COUNT],
}

impl Default for CharacterSelectionPortraitsRuntime {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| CharacterSelectionPortraitRuntimeSlot::default()),
        }
    }
}

pub struct NativeCharacterSelectionPortraitsPlugin;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum CharacterSelectionPortraitsSet {
    Rebuild,
    Materials,
    Status,
    Camera,
}

impl Plugin for NativeCharacterSelectionPortraitsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterSelectionPortraitsModel>()
            .init_resource::<GameplayPlayerPortraitModel>()
            .init_resource::<CharacterSelectionPortraitsRuntime>()
            .init_resource::<GameplayPortraitImage>()
            .add_systems(
                Startup,
                (
                    setup_character_selection_portrait_cameras,
                    setup_gameplay_player_portrait_camera,
                ),
            )
            .configure_sets(
                Update,
                (
                    CharacterSelectionPortraitsSet::Rebuild,
                    CharacterSelectionPortraitsSet::Materials,
                    CharacterSelectionPortraitsSet::Status,
                    CharacterSelectionPortraitsSet::Camera,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    rebuild_character_selection_portraits
                        .in_set(CharacterSelectionPortraitsSet::Rebuild),
                    sync_character_selection_portrait_render_layers
                        .in_set(CharacterSelectionPortraitsSet::Rebuild)
                        .after(rebuild_character_selection_portraits),
                    bind_character_selection_portrait_attachments
                        .in_set(CharacterSelectionPortraitsSet::Rebuild)
                        .after(rebuild_character_selection_portraits),
                    bind_character_selection_portrait_materials
                        .in_set(CharacterSelectionPortraitsSet::Materials),
                    update_character_selection_portrait_status
                        .in_set(CharacterSelectionPortraitsSet::Status),
                    report_character_selection_portrait_failures
                        .in_set(CharacterSelectionPortraitsSet::Status)
                        .after(update_character_selection_portrait_status),
                    sync_character_selection_portrait_cameras
                        .in_set(CharacterSelectionPortraitsSet::Camera),
                ),
            );
    }
}

pub(super) fn report_character_selection_portrait_failures(
    model: Res<CharacterSelectionPortraitsModel>,
    mut previous: Local<[Option<String>; CHARACTER_SELECTION_PORTRAIT_COUNT]>,
) {
    for (slot, state) in model.slots.iter().enumerate() {
        let current = match &state.status {
            CharacterSelectionPortraitStatus::Blocked(error) => Some(error.clone()),
            _ => None,
        };
        if current != previous[slot] {
            if let Some(error) = &current {
                warn!(
                    "character-selection portrait slot {} blocked: {error}",
                    slot + 1
                );
            }
            previous[slot] = current;
        }
    }
}

pub(super) fn setup_character_selection_portrait_cameras(
    mut commands: Commands,
    mut runtime: ResMut<CharacterSelectionPortraitsRuntime>,
) {
    for slot in 0..CHARACTER_SELECTION_PORTRAIT_COUNT {
        let layers = RenderLayers::layer(CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS[slot]);
        let camera = commands
            .spawn((
                Name::new(format!("Character selection portrait camera {}", slot + 1)),
                CharacterSelectionPortraitCamera(slot),
                Camera3d::default(),
                Camera {
                    is_active: false,
                    order: CHARACTER_SELECTION_PORTRAIT_CAMERA_ORDER_BASE + slot as isize,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                Projection::Perspective(PerspectiveProjection {
                    fov: CHARACTER_SELECTION_PORTRAIT_CAMERA_FOV_DEGREES.to_radians(),
                    near: CHARACTER_SELECTION_PORTRAIT_CAMERA_NEAR,
                    far: CHARACTER_SELECTION_PORTRAIT_CAMERA_FAR,
                    ..default()
                }),
                layers.clone(),
                Transform::default(),
            ))
            .id();
        runtime.slots[slot].camera = Some(camera);
        commands.spawn((
            Name::new(format!("Character selection portrait light {}", slot + 1)),
            CharacterSelectionPortraitLight,
            DirectionalLight {
                illuminance: 8_000.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(-2.0, 4.0, -3.0).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
            layers,
        ));
    }
}

pub(super) fn rebuild_character_selection_portraits(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_cache: ResMut<NativePlayerRigAssetCache>,
    catalog: Res<NativePlayerRigCatalog>,
    mut model: ResMut<CharacterSelectionPortraitsModel>,
    mut runtime: ResMut<CharacterSelectionPortraitsRuntime>,
) {
    for slot in 0..CHARACTER_SELECTION_PORTRAIT_COUNT {
        let revision = model.slots[slot].revision;
        if runtime.slots[slot].built_revision == revision {
            continue;
        }
        if let Some(root) = runtime.slots[slot].root.take() {
            commands.entity(root).despawn();
        }
        runtime.slots[slot].textures.clear();
        runtime.slots[slot].ready_once = false;
        runtime.slots[slot].built_revision = revision;
        let Some(look) = model.slots[slot].look.clone() else {
            continue;
        };
        if let Err(error) = look.validate() {
            model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(error);
            continue;
        }
        let skinned_parts = look
            .parts
            .iter()
            .filter(|part| part.uses_shared_skin())
            .cloned()
            .collect::<Vec<_>>();
        let routes = skinned_parts
            .iter()
            .map(|part| part.exact_route.clone())
            .collect::<Vec<_>>();
        let mut request = match NativePlayerRigSpawnRequest::new(
            format!("selection-slot-{}-{}", slot + 1, look.identity),
            revision,
            look.gender,
            RenderLayers::layer(CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS[slot]),
        )
        .use_part_routes(&catalog, routes)
        {
            Ok(request) => request,
            Err(error) => {
                model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(error);
                continue;
            }
        };
        request.transform = native_scene_container_transform(NativeSceneRole::CharacterGameplay);
        request.body_shape = Some(NativePlayerBodyShape {
            height: look.height_selector,
            body: look.body_selector,
        });
        if let Some(profile) = look.weapon_animation_profile {
            request.animation_name = profile.stand().name().to_owned();
        }
        // Never expose a bind-pose or unstyled face while scenes are loading.
        request.visibility = Visibility::Hidden;
        let spawned = match spawn_native_player_rig(
            &mut commands,
            &asset_server,
            &mut asset_cache,
            &catalog,
            request,
        ) {
            Ok(spawned) => spawned,
            Err(error) => {
                model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(error);
                continue;
            }
        };
        commands
            .entity(spawned.root)
            .insert(CharacterSelectionPortraitRig {
                slot,
                generation: revision,
            });
        for (part_root, part) in spawned.part_scenes.into_iter().zip(&skinned_parts) {
            commands
                .entity(part_root)
                .insert(CharacterSelectionPortraitPart {
                    slot,
                    kind: part.kind,
                    exact_route: part.exact_route.clone(),
                    generation: revision,
                });
        }
        for part in look.parts.iter().filter(|part| !part.uses_shared_skin()) {
            let attachment_slot = match part.kind {
                NativePlayerPartKind::Hat => LegacyPlayerAttachmentSlot::Hat,
                NativePlayerPartKind::Glasses => LegacyPlayerAttachmentSlot::Glasses,
                NativePlayerPartKind::Back => LegacyPlayerAttachmentSlot::Back,
                NativePlayerPartKind::Weapon => LegacyPlayerAttachmentSlot::RightPistol,
                NativePlayerPartKind::Face
                | NativePlayerPartKind::Hair
                | NativePlayerPartKind::Shirt
                | NativePlayerPartKind::Pants
                | NativePlayerPartKind::Shoes
                | NativePlayerPartKind::Vehicle => continue,
            };
            if part.glb.trim().is_empty() {
                model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                    "slot {} rigid attachment {:?} has no native GLB",
                    slot + 1,
                    part.exact_route
                ));
                continue;
            }
            let scene = asset_cache.scene(&asset_server, part.glb.clone());
            let placement = standard_player_attachment_placement();
            commands.spawn((
                Name::new(format!(
                    "Character selection portrait attachment: {}",
                    part.exact_route
                )),
                CharacterSelectionPortraitPart {
                    slot,
                    kind: part.kind,
                    exact_route: part.exact_route.clone(),
                    generation: revision,
                },
                PendingCharacterSelectionPortraitAttachment {
                    rig_root: spawned.root,
                    socket_full_path: player_attachment_socket_full_path(
                        look.gender,
                        attachment_slot,
                    ),
                    socket_local_scale_override: placement.socket_local_scale_override,
                },
                ChildOf(spawned.root),
                WorldAssetRoot(scene),
                placement.item_local,
                Visibility::Inherited,
                RenderLayers::layer(CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS[slot]),
            ));
        }
        runtime.slots[slot].root = Some(spawned.root);
        model.slots[slot].status = CharacterSelectionPortraitStatus::Loading;
    }
}

pub(super) fn bind_character_selection_portrait_attachments(
    mut commands: Commands,
    mut model: ResMut<CharacterSelectionPortraitsModel>,
    bones: Query<&NativePlayerRigBones>,
    native_statuses: Query<&NativePlayerRigStatus>,
    pending: Query<(
        Entity,
        &CharacterSelectionPortraitPart,
        &PendingCharacterSelectionPortraitAttachment,
    )>,
    mut transforms: Query<&mut Transform>,
) {
    for (attachment, part, pending) in &pending {
        if matches!(
            model.slots[part.slot].status,
            CharacterSelectionPortraitStatus::Blocked(_)
        ) {
            continue;
        }
        let Ok(bones) = bones.get(pending.rig_root) else {
            if native_statuses
                .get(pending.rig_root)
                .is_ok_and(NativePlayerRigStatus::is_ready)
            {
                model.slots[part.slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                    "slot {} ready portrait rig has no resolved exact bone map",
                    part.slot + 1
                ));
            }
            continue;
        };
        let Some(socket) = bones.by_full_path(&pending.socket_full_path) else {
            model.slots[part.slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                "slot {} portrait attachment {:?} has no exact socket {:?}",
                part.slot + 1,
                part.exact_route,
                pending.socket_full_path
            ));
            continue;
        };
        if let Some(scale) = pending.socket_local_scale_override
            && let Ok(mut transform) = transforms.get_mut(socket)
        {
            transform.scale = scale;
        }
        commands
            .entity(attachment)
            .insert(ChildOf(socket))
            .remove::<PendingCharacterSelectionPortraitAttachment>();
    }
}

pub(super) fn bind_character_selection_portrait_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<CharacterSelectionPortraitsModel>,
    mut runtime: ResMut<CharacterSelectionPortraitsRuntime>,
    children: Query<&Children>,
    parts: Query<(Entity, &CharacterSelectionPortraitPart)>,
    metadata: Query<&PendingLegacyModelMaterial>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut surfaces: Query<
        (
            &mut MeshMaterial3d<LegacyModelMaterial>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialPassCompanion>,
        ),
        Without<CharacterSelectionPortraitMaterialBound>,
    >,
    mut hierarchy_scratch: Local<Vec<Entity>>,
) {
    for (part_entity, part) in &parts {
        let Some(slot_model) = model.slots.get(part.slot) else {
            continue;
        };
        if part.generation != slot_model.revision {
            continue;
        }
        let Some(look) = slot_model.look.clone() else {
            continue;
        };
        let Some(part_look) = look.parts.iter().find(|look| look.kind == part.kind) else {
            model.slots[part.slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                "slot {} has no {:?} look for a loaded portrait surface",
                part.slot + 1,
                part.kind
            ));
            continue;
        };
        visit_portrait_hierarchy(
            part_entity,
            &children,
            hierarchy_scratch.as_mut(),
            |entity| {
                let Ok((mut handle, own_metadata, companion)) = surfaces.get_mut(entity) else {
                    return;
                };
                let Some(pending) = own_metadata.or_else(|| {
                    companion.and_then(|pass| metadata.get(pass.source_mesh_entity).ok())
                }) else {
                    return;
                };
                crate::legacy_model_material::make_legacy_material_unique(
                    &mut handle.0,
                    &mut materials,
                );
                let Some(mut material) = materials.get_mut(&handle.0) else {
                    return;
                };
                let material_name = pending.true_name.as_str();
                if companion.is_some()
                    && actor_skin_texture_role(material_name) == ActorSkinTextureRole::SecondarySkin
                    && matches!(
                        part.kind,
                        NativePlayerPartKind::Shirt
                            | NativePlayerPartKind::Pants
                            | NativePlayerPartKind::Shoes
                    )
                {
                    commands
                        .entity(entity)
                        .insert((Visibility::Hidden, CharacterSelectionPortraitMaterialBound));
                    return;
                }
                match actor_skin_texture_role(material_name) {
                    ActorSkinTextureRole::SecondarySkin => {
                        let (texture, skin_tint) = if uses_global_skin_secondary(part.kind) {
                            (look.skin_texture.as_ref(), true)
                        } else {
                            (part_look.secondary_texture.as_ref(), false)
                        };
                        if let Some(texture) = texture {
                            match load_legacy_main_texture_replacement_with_contract(
                                &asset_server,
                                pending,
                                texture.path.clone(),
                                &texture.contract,
                            ) {
                                Ok(handle) => remember_texture(
                                    &mut runtime.slots[part.slot].textures,
                                    texture.path.clone(),
                                    handle.clone(),
                                    || material.base_texture = Some(handle),
                                ),
                                Err(error) => {
                                    model.slots[part.slot].status =
                                        CharacterSelectionPortraitStatus::Blocked(error);
                                    return;
                                }
                            }
                        }
                        if skin_tint {
                            material.uniform.base_color = half_tint(look.skin_color);
                            material.uniform.emission =
                                multiply_rgb(material.uniform.emission, look.skin_color);
                        }
                    }
                    ActorSkinTextureRole::PreserveOverride => {}
                    ActorSkinTextureRole::Primary => {
                        if let Some(texture) = &part_look.primary_texture {
                            match load_legacy_main_texture_replacement_with_contract(
                                &asset_server,
                                pending,
                                texture.path.clone(),
                                &texture.contract,
                            ) {
                                Ok(handle) => remember_texture(
                                    &mut runtime.slots[part.slot].textures,
                                    texture.path.clone(),
                                    handle.clone(),
                                    || material.base_texture = Some(handle),
                                ),
                                Err(error) => {
                                    model.slots[part.slot].status =
                                        CharacterSelectionPortraitStatus::Blocked(error);
                                    return;
                                }
                            }
                        }
                    }
                }
                if material_name.contains("hair") {
                    material.uniform.base_color = half_tint(look.hair_color);
                    material.uniform.emission =
                        multiply_rgb(material.uniform.emission, look.hair_color);
                }
                commands
                    .entity(entity)
                    .insert(CharacterSelectionPortraitMaterialBound);
            },
        );
    }
}

pub(super) fn update_character_selection_portrait_status(
    asset_server: Res<AssetServer>,
    mut model: ResMut<CharacterSelectionPortraitsModel>,
    mut runtime: ResMut<CharacterSelectionPortraitsRuntime>,
    rigs: Query<(&NativePlayerRigStatus, Option<&NativePlayerRigBones>)>,
    children: Query<&Children>,
    part_roots: Query<(
        Entity,
        &CharacterSelectionPortraitPart,
        Option<&WorldAssetRoot>,
        Option<&PendingCharacterSelectionPortraitAttachment>,
    )>,
    errors: Query<&LegacyMaterialMetadataError>,
    material_surfaces: Query<
        Option<&CharacterSelectionPortraitMaterialBound>,
        With<MeshMaterial3d<LegacyModelMaterial>>,
    >,
    mut hierarchy_scratch: Local<Vec<Entity>>,
) {
    for slot in 0..CHARACTER_SELECTION_PORTRAIT_COUNT {
        let Some(root) = runtime.slots[slot].root else {
            continue;
        };
        if matches!(
            model.slots[slot].status,
            CharacterSelectionPortraitStatus::Blocked(_)
        ) {
            continue;
        }
        let Some(look) = model.slots[slot].look().cloned() else {
            model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                "slot {} portrait lost its requested look",
                slot + 1
            ));
            continue;
        };
        let mut parts_ready = true;
        for expected in &look.parts {
            let mut matches = part_roots.iter().filter(|(_, part, _, _)| {
                part.slot == slot
                    && part.generation == model.slots[slot].revision
                    && part.kind == expected.kind
                    && part.exact_route == expected.exact_route
            });
            let Some((part_entity, _, scene, pending_attachment)) = matches.next() else {
                parts_ready = false;
                continue;
            };
            if matches.next().is_some() {
                model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                    "slot {} repeats exact portrait part route {:?}",
                    slot + 1,
                    expected.exact_route
                ));
                parts_ready = false;
                break;
            }
            if pending_attachment.is_some() {
                parts_ready = false;
            }
            let Some(scene) = scene else {
                parts_ready = false;
                continue;
            };
            match asset_server.load_state(scene.0.id()) {
                LoadState::Failed(error) => {
                    model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                        "slot {} portrait part {:?} failed to load: {error}",
                        slot + 1,
                        expected.exact_route
                    ));
                    parts_ready = false;
                    break;
                }
                _ if matches!(
                    asset_server.get_recursive_dependency_load_state(scene.0.id()),
                    Some(RecursiveDependencyLoadState::Failed(_))
                ) =>
                {
                    let error = asset_server
                        .get_recursive_dependency_load_state(scene.0.id())
                        .and_then(|state| match state {
                            RecursiveDependencyLoadState::Failed(error) => Some(error),
                            _ => None,
                        })
                        .expect("recursive portrait dependency failure matched above");
                    model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                        "slot {} portrait part {:?} dependency failed to load: {error}",
                        slot + 1,
                        expected.exact_route
                    ));
                    parts_ready = false;
                    break;
                }
                _ if !asset_server.is_loaded_with_dependencies(scene.0.id()) => {
                    parts_ready = false;
                }
                _ => {}
            }
            let mut part_surface_count = 0_usize;
            let mut part_surfaces_bound = true;
            visit_portrait_hierarchy(
                part_entity,
                &children,
                hierarchy_scratch.as_mut(),
                |entity| {
                    if let Ok(bound) = material_surfaces.get(entity) {
                        part_surface_count += 1;
                        part_surfaces_bound &= bound.is_some();
                    }
                },
            );
            if part_surface_count == 0 || !part_surfaces_bound {
                parts_ready = false;
            }
        }
        if matches!(
            model.slots[slot].status,
            CharacterSelectionPortraitStatus::Blocked(_)
        ) {
            continue;
        }
        let mut metadata_error = None;
        let mut surface_count = 0_usize;
        let mut all_surfaces_bound = true;
        visit_portrait_hierarchy(root, &children, hierarchy_scratch.as_mut(), |entity| {
            if metadata_error.is_none()
                && let Ok(error) = errors.get(entity)
            {
                metadata_error = Some(error.0.clone());
            }
            if let Ok(bound) = material_surfaces.get(entity) {
                surface_count += 1;
                all_surfaces_bound &= bound.is_some();
            }
        });
        if let Some(error) = metadata_error {
            model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(error);
            continue;
        }
        let (actor_bones, part_count) = match rigs.get(root) {
            Ok((NativePlayerRigStatus::ReadyAnimated { actor_bones, .. }, Some(bones))) => {
                if bones
                    .unique_by_true_name(CHARACTER_SELECTION_PORTRAIT_HEAD_BONE)
                    .is_none()
                {
                    model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                        "slot {} shared rig does not resolve one exact {CHARACTER_SELECTION_PORTRAIT_HEAD_BONE:?}",
                        slot + 1
                    ));
                    continue;
                }
                (
                    *actor_bones,
                    model.slots[slot].look().map_or(0, |look| look.parts.len()),
                )
            }
            Ok((NativePlayerRigStatus::ReadyAnimated { .. }, None)) => {
                model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                    "slot {} reached ReadyAnimated without its instance-local bone map",
                    slot + 1
                ));
                continue;
            }
            Ok((NativePlayerRigStatus::Blocked(error), _)) => {
                model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(error.clone());
                continue;
            }
            Ok((NativePlayerRigStatus::Loading(_), _)) | Err(_) => {
                if !runtime.slots[slot].ready_once {
                    model.slots[slot].status = CharacterSelectionPortraitStatus::Loading;
                }
                continue;
            }
        };
        let mut textures_ready = true;
        for (path, texture) in &runtime.slots[slot].textures {
            match asset_server.load_state(texture.id()) {
                LoadState::Failed(error) => {
                    model.slots[slot].status = CharacterSelectionPortraitStatus::Blocked(format!(
                        "slot {} native player texture {path} failed to load: {error}",
                        slot + 1
                    ));
                    textures_ready = false;
                    break;
                }
                LoadState::Loaded => {}
                _ => textures_ready = false,
            }
        }
        if matches!(
            model.slots[slot].status,
            CharacterSelectionPortraitStatus::Blocked(_)
        ) {
            continue;
        }
        if parts_ready && textures_ready && surface_count > 0 && all_surfaces_bound {
            runtime.slots[slot].ready_once = true;
            model.slots[slot].status = CharacterSelectionPortraitStatus::ReadyAnimated {
                actor_bones,
                parts: part_count,
            };
        } else if !runtime.slots[slot].ready_once {
            model.slots[slot].status = CharacterSelectionPortraitStatus::Loading;
        }
    }
}
