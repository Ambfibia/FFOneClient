use super::*;

/// The inventory target must be rendered before the legacy UI
/// camera samples it through `ImageNode`, matching `DrawCamera`'s immediate
/// repaint rather than displaying the previous GUI frame.
pub const NATIVE_PLAYER_INVENTORY_PREVIEW_CAMERA_ORDER: isize = GAMEPLAY_UI_CAMERA_ORDER - 1;

pub const NATIVE_PLAYER_SELECTION_CAMERA_DISTANCE: f32 = 2.0;

pub const NATIVE_PLAYER_SELECTION_CAMERA_HEIGHT: f32 = 0.75;

/// Exact `InventoryManagerScript.AvatarClothes` values assigned to
/// `cmOwnAvatar`'s `cnSimpleCharRenderCamera` before `RenderCharactor`.
pub const NATIVE_PLAYER_INVENTORY_CAMERA_DISTANCE: f32 = 2.65;

pub const NATIVE_PLAYER_INVENTORY_CAMERA_HEIGHT: f32 = 1.0;

/// Logical `Panel_UserClothes.ownuser` dimensions; the target follows UI pixel density.
pub const NATIVE_PLAYER_INVENTORY_PREVIEW_WIDTH: u32 = 500;

pub const NATIVE_PLAYER_INVENTORY_PREVIEW_HEIGHT: u32 = 564;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum NativePlayerPreviewStatus {
    #[default]
    Empty,
    Loading,
    ReadyAnimated {
        parts: usize,
    },
    Blocked(String),
}

#[derive(Resource, Default)]
pub(super) struct NativePlayerPreviewRuntime {
    pub(super) built_revision: u64,
    /// Newest requested root. It can still be loading and hidden.
    pub(super) root: Option<Entity>,
    /// Last fully ready root retained while `root` is prepared off-screen.
    /// Selection hides it when its generation no longer matches the chosen look.
    pub(super) displayed_root: Option<Entity>,
    pub(super) root_generation: u64,
    pub(super) geometry: Option<NativePlayerPreviewGeometry>,
    pub(super) ready_generation: Option<u64>,
    pub(super) camera: Option<Entity>,
    pub(super) textures: Vec<(String, Handle<Image>)>,
    pub(super) binding: NativePlayerBindingDiagnostics,
}

/// Transparent off-screen image drawn by the UserEquip hierarchy at the exact
/// `ownuser` Rect. The target scales with the UI's physical pixel density while
/// preserving the camera aspect and `Panel_UserClothes`'s `y=-50` clipping.
#[derive(Clone, Resource)]
pub struct NativePlayerInventoryPreviewImage(pub Handle<Image>);

pub(super) fn update_native_player_preview_status(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut model: ResMut<NativePlayerPreviewModel>,
    mut runtime: ResMut<NativePlayerPreviewRuntime>,
    mut audit: ResMut<NativePlayerPreviewAudit>,
    rig_statuses: Query<&NativePlayerRigStatus>,
    parents: Query<&ChildOf>,
    part_roots: Query<(
        Entity,
        &NativePlayerPreviewPart,
        Option<&WorldAssetRoot>,
        Option<&PendingNativePlayerPreviewAttachment>,
    )>,
    part_markers: Query<&NativePlayerPreviewPart>,
    errors: Query<(Entity, &LegacyMaterialMetadataError)>,
    material_surfaces: Query<
        (Entity, Option<&NativePlayerPreviewMaterialBound>),
        With<MeshMaterial3d<LegacyModelMaterial>>,
    >,
    pending_materials: Query<
        (Entity, Option<&LegacyMaterialApplied>),
        With<PendingLegacyModelMaterial>,
    >,
    mut root_visibility: Query<&mut Visibility, With<NativePlayerPreviewRoot>>,
) {
    if matches!(model.status, NativePlayerPreviewStatus::Blocked(_)) {
        return;
    }
    let Some(root) = runtime.root else {
        return;
    };
    let Some(look) = model.look.clone() else {
        model.status = NativePlayerPreviewStatus::Blocked(
            "native player preview lost its requested look".to_owned(),
        );
        return;
    };
    for (entity, error) in &errors {
        if is_descendant_of(entity, root, &parents) {
            model.status = NativePlayerPreviewStatus::Blocked(error.0.clone());
            return;
        }
    }
    let mut loading = false;
    let mut pending_rig = true;
    let mut pending_textures = 0_usize;
    match rig_statuses.get(root) {
        Ok(NativePlayerRigStatus::ReadyAnimated { .. }) => pending_rig = false,
        Ok(NativePlayerRigStatus::Blocked(error)) => {
            model.status = NativePlayerPreviewStatus::Blocked(error.clone());
            return;
        }
        Ok(NativePlayerRigStatus::Loading(_)) | Err(_) => loading = true,
    }
    let mut missing_parts = 0_usize;
    let mut pending_part_scenes = 0_usize;
    let mut pending_attachments = 0_usize;
    for expected in &look.parts {
        let mut matches = part_roots.iter().filter(|(entity, part, _, _)| {
            part.generation == runtime.root_generation
                && part.kind == expected.kind
                && part.exact_route == expected.exact_route
                && is_descendant_of(*entity, root, &parents)
        });
        let Some((_, _, scene, pending_attachment)) = matches.next() else {
            loading = true;
            missing_parts += 1;
            continue;
        };
        if matches.next().is_some() {
            model.status = NativePlayerPreviewStatus::Blocked(format!(
                "native player preview repeats exact part route {:?}",
                expected.exact_route
            ));
            return;
        }
        if pending_attachment.is_some() {
            loading = true;
            pending_attachments += 1;
        }
        let Some(scene) = scene else {
            loading = true;
            pending_part_scenes += 1;
            continue;
        };
        match asset_server.load_state(scene.0.id()) {
            LoadState::Failed(error) => {
                model.status = NativePlayerPreviewStatus::Blocked(format!(
                    "native player part {:?} failed to load: {error}",
                    expected.exact_route
                ));
                return;
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
                    .expect("recursive failure was matched above");
                model.status = NativePlayerPreviewStatus::Blocked(format!(
                    "native player part {:?} dependency failed to load: {error}",
                    expected.exact_route
                ));
                return;
            }
            _ if !asset_server.is_loaded_with_dependencies(scene.0.id()) => {
                loading = true;
                pending_part_scenes += 1;
            }
            _ => {}
        }
    }
    for (path, texture) in &runtime.textures {
        match asset_server.load_state(texture.id()) {
            LoadState::Failed(error) => {
                model.status = NativePlayerPreviewStatus::Blocked(format!(
                    "native player texture {path} failed to load: {error}"
                ));
                return;
            }
            LoadState::Loaded => {}
            _ => {
                loading = true;
                pending_textures += 1;
            }
        }
    }
    let mut surface_count = 0;
    let mut unbound_surfaces = 0_usize;
    let mut unapplied_materials = 0_usize;
    let mut surface_routes = Vec::new();
    for (entity, applied) in &pending_materials {
        if is_descendant_of(entity, root, &parents) && applied.is_none() {
            loading = true;
            unapplied_materials += 1;
        }
    }
    for (entity, bound) in &material_surfaces {
        if is_descendant_of(entity, root, &parents) {
            surface_count += 1;
            if let Some(part) = ancestor_part(entity, &parents, &part_markers)
                && part.generation == runtime.root_generation
            {
                surface_routes.push((part.exact_route.clone(), bound.is_some()));
            }
            if bound.is_none() {
                loading = true;
                unbound_surfaces += 1;
            }
        }
    }
    let mut missing_part_surfaces = 0_usize;
    for expected in &look.parts {
        if !surface_routes
            .iter()
            .any(|(route, _)| route == &expected.exact_route)
        {
            loading = true;
            missing_part_surfaces += 1;
        }
    }
    let part_count = look.parts.len();
    model.status = if loading {
        model.loading_detail = Some(format!(
            "pending rig={pending_rig}, parts missing={missing_parts}, scenes={pending_part_scenes}, attachments={pending_attachments}, textures={pending_textures}, unapplied materials={unapplied_materials}, surfaces={surface_count}, missing part surfaces={missing_part_surfaces}, unbound={unbound_surfaces}; bind seen={}, bound={}, no_part={}, generation={}, no_look={}, metadata={}, material={}",
            runtime.binding.seen,
            runtime.binding.bound,
            runtime.binding.no_part,
            runtime.binding.wrong_generation,
            runtime.binding.no_look,
            runtime.binding.no_metadata,
            runtime.binding.no_material,
        ));
        NativePlayerPreviewStatus::Loading
    } else {
        model.loading_detail = None;
        if runtime.ready_generation != Some(runtime.root_generation) {
            let previous = runtime.displayed_root.replace(root);
            if let Ok(mut visibility) = root_visibility.get_mut(root) {
                *visibility = Visibility::Inherited;
            }
            if let Some(previous) = previous.filter(|previous| *previous != root) {
                if let Ok(mut visibility) = root_visibility.get_mut(previous) {
                    *visibility = Visibility::Hidden;
                }
                commands.entity(previous).despawn();
            }
        }
        audit.identity = Some(look.identity.clone());
        audit.gender = Some(look.gender);
        audit.surfaces = material_surfaces
            .iter()
            .filter(|(entity, _)| is_descendant_of(*entity, root, &parents))
            .filter_map(|(_, bound)| bound)
            .map(|bound| NativePlayerPreviewSurfaceAudit {
                kind: bound.kind,
                exact_route: bound.exact_route.clone(),
                material_true_name: bound.material_true_name.clone(),
                role: bound.binding.role,
                bound_texture_path: bound
                    .binding
                    .texture
                    .as_ref()
                    .map(|texture| texture.path.clone()),
                base_texture_assigned: bound.base_texture_assigned,
                source_main_texture: bound.source_main_texture.clone(),
                hidden: bound.binding.hide_surface,
            })
            .collect();
        runtime.ready_generation = Some(runtime.root_generation);
        NativePlayerPreviewStatus::ReadyAnimated { parts: part_count }
    };
}
