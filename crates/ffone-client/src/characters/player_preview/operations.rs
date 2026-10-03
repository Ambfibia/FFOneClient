use super::*;

pub fn prewarm_native_player_look(
    asset_server: &AssetServer,
    asset_cache: &mut NativePlayerRigAssetCache,
    catalog: &NativePlayerRigCatalog,
    look: &NativePlayerLook,
) -> Result<(), String> {
    look.validate()?;
    asset_cache.preload_rig_parts(
        asset_server,
        catalog,
        look.gender,
        look.parts
            .iter()
            .filter(|part| part.uses_shared_skin())
            .map(|part| part.exact_route.as_str()),
    )?;
    for texture in look.skin_texture.iter().chain(
        look.parts
            .iter()
            .flat_map(|part| {
                [
                    part.primary_texture.as_ref(),
                    part.secondary_texture.as_ref(),
                ]
            })
            .flatten(),
    ) {
        let handle = load_character_runtime_texture_with_contract(
            asset_server,
            texture.path.clone(),
            &texture.contract,
        )?;
        asset_cache.retain_image(texture.path.clone(), handle);
    }
    Ok(())
}

pub(super) fn setup_native_player_preview_stage(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut inventory_target = Image::new_target_texture(
        NATIVE_PLAYER_INVENTORY_PREVIEW_WIDTH,
        NATIVE_PLAYER_INVENTORY_PREVIEW_HEIGHT,
        TextureFormat::Rgba8UnormSrgb,
        None,
    );
    inventory_target.sampler = bevy::image::ImageSampler::linear();
    commands.insert_resource(NativePlayerInventoryPreviewImage(images.add(inventory_target)));
    commands.insert_resource(NativePlayerBarberPreviewImage(images.add(
        Image::new_target_texture(307, 498, TextureFormat::Rgba8UnormSrgb, None),
    )));
    commands.insert_resource(NativePlayerTryOnPreviewImage(images.add(
        Image::new_target_texture(210, 375, TextureFormat::Rgba8UnormSrgb, None),
    )));
    commands.spawn((
        Name::new("Native player preview light"),
        NativePlayerPreviewLight,
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -3.0).looking_at(Vec3::new(0.0, 0.8, 0.0), Vec3::Y),
        RenderLayers::layer(NATIVE_PLAYER_PREVIEW_RENDER_LAYER),
    ));
}

pub(super) fn rebuild_native_player_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut asset_cache: ResMut<NativePlayerRigAssetCache>,
    catalog: Res<NativePlayerRigCatalog>,
    mut model: ResMut<NativePlayerPreviewModel>,
    mut runtime: ResMut<NativePlayerPreviewRuntime>,
    parents: Query<&ChildOf>,
    parts: Query<(Entity, &NativePlayerPreviewPart)>,
    bound_materials: Query<Entity, With<NativePlayerPreviewMaterialBound>>,
) {
    if runtime.built_revision == model.revision {
        return;
    }
    let Some(look) = model.look.as_ref() else {
        let pending = runtime.root.take();
        let displayed = runtime.displayed_root.take();
        if let Some(root) = pending {
            commands.entity(root).despawn();
        }
        if let Some(root) = displayed.filter(|root| Some(*root) != pending) {
            commands.entity(root).despawn();
        }
        runtime.textures.clear();
        runtime.geometry = None;
        runtime.ready_generation = None;
        runtime.built_revision = model.revision;
        model.status = NativePlayerPreviewStatus::Empty;
        return;
    };
    if let Err(error) = look.validate() {
        model.status = NativePlayerPreviewStatus::Blocked(error);
        return;
    }
    let geometry = NativePlayerPreviewGeometry::from(look);
    if let Some(root) = runtime.root
        && runtime.geometry.as_ref() == Some(&geometry)
    {
        // Palette/color and texture choices change much more often than the
        // underlying modular geometry. Rebind in place, but treat the new look
        // as a new generation so the old clothing cannot be shown while its
        // replacement textures and materials are still loading.
        commands.entity(root).insert(NativePlayerBodyShape {
            height: look.height_selector,
            body: look.body_selector,
        });
        for (entity, part) in &parts {
            if is_descendant_of(entity, root, &parents) {
                let mut current = part.clone();
                current.generation = model.revision;
                commands.entity(entity).insert(current);
            }
        }
        for entity in &bound_materials {
            if entity == root || is_descendant_of(entity, root, &parents) {
                commands
                    .entity(entity)
                    .remove::<NativePlayerPreviewMaterialBound>();
            }
        }
        runtime.textures.clear();
        runtime.binding = NativePlayerBindingDiagnostics::default();
        runtime.root_generation = model.revision;
        runtime.ready_generation = None;
        runtime.built_revision = model.revision;
        model.status = NativePlayerPreviewStatus::Loading;
        return;
    }
    if let Some(root) = superseded_pending_root(runtime.root.take(), runtime.displayed_root) {
        commands.entity(root).despawn();
    }
    runtime.textures.clear();
    runtime.geometry = None;
    runtime.ready_generation = None;
    runtime.root_generation = model.revision;
    runtime.built_revision = model.revision;
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
        look.identity.clone(),
        model.revision,
        look.gender,
        RenderLayers::layer(NATIVE_PLAYER_PREVIEW_RENDER_LAYER),
    )
    .use_part_routes(&catalog, routes)
    {
        Ok(request) => request,
        Err(error) => {
            model.status = NativePlayerPreviewStatus::Blocked(error);
            return;
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
    // The root is revealed only after the exact equipped idle rig and every dynamic
    // material are ready. This removes the one-frame bind-pose/unstyled flash
    // that appeared as face flicker during scene loading.
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
            model.status = NativePlayerPreviewStatus::Blocked(error);
            return;
        }
    };
    commands
        .entity(spawned.root)
        .insert(NativePlayerPreviewRoot);
    for (part_root, part) in spawned.part_scenes.into_iter().zip(&skinned_parts) {
        commands.entity(part_root).insert(NativePlayerPreviewPart {
            kind: part.kind,
            exact_route: part.exact_route.clone(),
            generation: model.revision,
        });
    }
    for part in look.parts.iter().filter(|part| !part.uses_shared_skin()) {
        let slot = match part.kind {
            NativePlayerPartKind::Hat => LegacyPlayerAttachmentSlot::Hat,
            NativePlayerPartKind::Glasses => LegacyPlayerAttachmentSlot::Glasses,
            NativePlayerPartKind::Back => LegacyPlayerAttachmentSlot::Back,
            NativePlayerPartKind::Weapon => LegacyPlayerAttachmentSlot::RightPistol,
            NativePlayerPartKind::Vehicle => LegacyPlayerAttachmentSlot::Vehicle,
            NativePlayerPartKind::Face
            | NativePlayerPartKind::Hair
            | NativePlayerPartKind::Shirt
            | NativePlayerPartKind::Pants
            | NativePlayerPartKind::Shoes => continue,
        };
        if part.glb.trim().is_empty() {
            continue;
        }
        let scene = asset_cache.scene(&asset_server, part.glb.clone());
        let placement = standard_player_attachment_placement();
        commands.spawn((
            Name::new(format!("Native player attachment: {}", part.exact_route)),
            NativePlayerPreviewPart {
                kind: part.kind,
                exact_route: part.exact_route.clone(),
                generation: model.revision,
            },
            PendingNativePlayerPreviewAttachment {
                rig_root: spawned.root,
                socket_full_path: player_attachment_socket_full_path(look.gender, slot),
                socket_local_scale_override: placement.socket_local_scale_override,
            },
            ChildOf(spawned.root),
            WorldAssetRoot(scene),
            placement.item_local,
            Visibility::Inherited,
            RenderLayers::layer(NATIVE_PLAYER_PREVIEW_RENDER_LAYER),
        ));
    }
    runtime.root = Some(spawned.root);
    runtime.geometry = Some(geometry);
    model.status = NativePlayerPreviewStatus::Loading;
}

pub(super) fn bind_native_player_preview_attachments(
    mut commands: Commands,
    mut model: ResMut<NativePlayerPreviewModel>,
    bones: Query<&NativePlayerRigBones>,
    pending: Query<(Entity, &PendingNativePlayerPreviewAttachment)>,
    mut transforms: Query<&mut Transform>,
) {
    for (attachment, pending) in &pending {
        let Ok(bones) = bones.get(pending.rig_root) else {
            continue;
        };
        let Some(socket) = bones.by_full_path(&pending.socket_full_path) else {
            model.status = NativePlayerPreviewStatus::Blocked(format!(
                "native player preview attachment socket {:?} is absent from the ready rig",
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
            .remove::<PendingNativePlayerPreviewAttachment>();
    }
}

pub(super) fn bind_native_player_preview_materials(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<NativePlayerPreviewModel>,
    mut runtime: ResMut<NativePlayerPreviewRuntime>,
    parents: Query<&ChildOf>,
    parts: Query<&NativePlayerPreviewPart>,
    metadata: Query<&PendingLegacyModelMaterial>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut surfaces: Query<
        (
            Entity,
            &mut MeshMaterial3d<LegacyModelMaterial>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialPassCompanion>,
            Option<&NativePlayerPreviewMaterialBaseline>,
        ),
        Without<NativePlayerPreviewMaterialBound>,
    >,
) {
    let Some(look) = model.look.clone() else {
        return;
    };
    runtime.binding = NativePlayerBindingDiagnostics::default();
    for (entity, mut handle, own_metadata, companion, baseline) in &mut surfaces {
        runtime.binding.seen += 1;
        let Some(part) = ancestor_part(entity, &parents, &parts) else {
            runtime.binding.no_part += 1;
            continue;
        };
        if part.generation != runtime.root_generation {
            runtime.binding.wrong_generation += 1;
            continue;
        }
        let Some(part_look) = look.parts.iter().find(|look| look.kind == part.kind) else {
            runtime.binding.no_look += 1;
            continue;
        };
        let Some(pending) = own_metadata
            .or_else(|| companion.and_then(|pass| metadata.get(pass.source_mesh_entity).ok()))
        else {
            runtime.binding.no_metadata += 1;
            continue;
        };
        let Some(current_material) = materials.get(&handle.0) else {
            runtime.binding.no_material += 1;
            continue;
        };
        let mut material = current_material.clone();
        if let Some(baseline) = baseline {
            baseline.restore(&mut material);
        } else {
            commands
                .entity(entity)
                .insert(NativePlayerPreviewMaterialBaseline::capture(&material));
        }
        let binding = match bind_native_player_look_material(
            &asset_server,
            pending,
            &mut material,
            &look,
            part_look,
            companion.is_some(),
        ) {
            Ok(binding) => binding,
            Err(error) => {
                model.status = NativePlayerPreviewStatus::Blocked(error);
                return;
            }
        };
        let bound = NativePlayerPreviewMaterialBound {
            kind: part.kind,
            exact_route: part.exact_route.clone(),
            material_true_name: pending.true_name.clone(),
            binding: binding.clone(),
            base_texture_assigned: material.base_texture.is_some(),
            source_main_texture: pending
                .texture_bindings
                .iter()
                .find(|texture| texture.slot == "_MainTex")
                .and_then(|texture| texture.source_name.clone().or_else(|| texture.uri.clone())),
        };
        if binding.hide_surface {
            commands.entity(entity).insert((Visibility::Hidden, bound));
            runtime.binding.bound += 1;
            continue;
        }
        if let Some(texture) = &binding.texture
            && !runtime
                .textures
                .iter()
                .any(|(_, existing)| existing == &texture.handle)
        {
            runtime
                .textures
                .push((texture.path.clone(), texture.handle.clone()));
        }
        if let Some(texture) = &binding.texture {
            match asset_server.load_state(texture.handle.id()) {
                LoadState::Loaded => {}
                LoadState::Failed(error) => {
                    model.status = NativePlayerPreviewStatus::Blocked(format!(
                        "native player texture {} failed to load: {error}",
                        texture.path
                    ));
                    return;
                }
                // Keep the displayed material intact while its replacement
                // loads. Retaining the handle above also keeps retries alive.
                _ => continue,
            }
        }
        crate::legacy_model_material::make_legacy_material_unique(&mut handle.0, &mut materials);
        *materials
            .get_mut(&handle.0)
            .expect("material was just read") = material;
        commands.entity(entity).insert(bound);
        runtime.binding.bound += 1;
    }
}

pub(super) fn superseded_pending_root(requested: Option<Entity>, displayed: Option<Entity>) -> Option<Entity> {
    requested.filter(|requested| Some(*requested) != displayed)
}

pub(super) fn native_player_preview_camera_distance_and_height(
    model: &NativePlayerPreviewModel,
    head_height: Option<f32>,
) -> (f32, f32) {
    match model.stage {
        NativePlayerPreviewStage::Selection => (
            NATIVE_PLAYER_SELECTION_CAMERA_DISTANCE,
            NATIVE_PLAYER_SELECTION_CAMERA_HEIGHT,
        ),
        NativePlayerPreviewStage::Creation | NativePlayerPreviewStage::Barber => {
            let distance = if model.camera_distance.is_finite() {
                model.camera_distance.clamp(
                    NATIVE_PLAYER_CREATION_CAMERA_MIN_DISTANCE,
                    NATIVE_PLAYER_CREATION_CAMERA_MAX_DISTANCE,
                )
            } else {
                NATIVE_PLAYER_CREATION_CAMERA_DISTANCE
            };
            let close_progress = ((NATIVE_PLAYER_CREATION_CAMERA_DISTANCE - distance)
                / (NATIVE_PLAYER_CREATION_CAMERA_DISTANCE
                    - NATIVE_PLAYER_CREATION_CAMERA_MIN_DISTANCE))
                .clamp(0.0, 1.0);
            let head_height = head_height
                .filter(|height| height.is_finite())
                .unwrap_or(NATIVE_PLAYER_CREATION_CAMERA_HEIGHT);
            (
                distance,
                NATIVE_PLAYER_CREATION_CAMERA_HEIGHT
                    + (head_height - NATIVE_PLAYER_CREATION_CAMERA_HEIGHT) * close_progress,
            )
        }
        NativePlayerPreviewStage::TryOn => (2.2, 0.7),
        NativePlayerPreviewStage::Inventory => (
            NATIVE_PLAYER_INVENTORY_CAMERA_DISTANCE,
            NATIVE_PLAYER_INVENTORY_CAMERA_HEIGHT,
        ),
    }
}

pub(super) fn native_player_preview_viewport(
    model: &NativePlayerPreviewModel,
    creation_ui: Option<&CharacterCreationUiModel>,
    selection_ui: Option<&CharacterSelectionUiModel>,
    window: &Window,
) -> Option<Viewport> {
    if matches!(
        model.stage,
        NativePlayerPreviewStage::Inventory | NativePlayerPreviewStage::TryOn | NativePlayerPreviewStage::Barber
    ) {
        return None;
    }
    let logical_size = Vec2::new(window.width(), window.height());
    let (x, y, width, height) = match model.stage {
        NativePlayerPreviewStage::Creation => {
            let ui = creation_ui?;
            let layout = CharacterCreationLayout::for_viewport(logical_size, ui.ui_scale);
            let scale = layout.appearance.width / CHARACTER_CREATION_APPEARANCE_WIDTH;
            (
                layout.appearance.x + 77.0 * scale,
                layout.appearance.y + 44.0 * scale,
                307.0 * scale,
                498.0 * scale,
            )
        }
        NativePlayerPreviewStage::Selection => {
            let ui = selection_ui?;
            let layout = CharacterSelectionLayout::for_viewport(logical_size, ui.ui_scale);
            let scale = layout.avatar_group.width / 600.0;
            (
                layout.avatar_group.x,
                layout.avatar_group.y,
                CHARACTER_SELECTION_PREVIEW_WIDTH * scale,
                CHARACTER_SELECTION_PREVIEW_HEIGHT * scale,
            )
        }
        NativePlayerPreviewStage::Inventory | NativePlayerPreviewStage::TryOn | NativePlayerPreviewStage::Barber => {
            unreachable!("inventory uses its full off-screen target")
        }
    };
    let scale_factor = window.resolution.scale_factor();
    let physical_size = UVec2::new(
        window.resolution.physical_width(),
        window.resolution.physical_height(),
    );
    let mut viewport = Viewport {
        physical_position: UVec2::new(
            (x * scale_factor).round().max(0.0) as u32,
            (y * scale_factor).round().max(0.0) as u32,
        ),
        physical_size: UVec2::new(
            (width * scale_factor).round().max(1.0) as u32,
            (height * scale_factor).round().max(1.0) as u32,
        ),
        ..default()
    };
    viewport.clamp_to_size(physical_size);
    Some(viewport)
}

pub(super) fn ancestor_part<'a>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    parts: &'a Query<&NativePlayerPreviewPart>,
) -> Option<&'a NativePlayerPreviewPart> {
    loop {
        if let Ok(part) = parts.get(entity) {
            return Some(part);
        }
        entity = parents.get(entity).ok()?.parent();
    }
}
