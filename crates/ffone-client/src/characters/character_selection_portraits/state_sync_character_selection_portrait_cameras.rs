use super::*;

pub(super) fn sync_character_selection_portrait_cameras(
    model: Res<CharacterSelectionPortraitsModel>,
    gameplay: Res<GameplayPlayerPortraitModel>,
    selection_ui: Res<CharacterSelectionUiModel>,
    runtime: Res<CharacterSelectionPortraitsRuntime>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<
        (&CharacterSelectionPortraitRig, &mut Visibility),
        Without<CharacterSelectionPortraitCamera>,
    >,
    bones: Query<&NativePlayerRigBones>,
    globals: Query<&GlobalTransform>,
    mut cameras: Query<
        (
            &CharacterSelectionPortraitCamera,
            &mut Camera,
            &mut Transform,
            &mut Projection,
        ),
        (
            Without<CharacterSelectionPortraitRig>,
            Without<GameplayPlayerPortraitCamera>,
        ),
    >,
    mut gameplay_cameras: Query<
        (
            &mut Camera,
            &mut Transform,
            &mut Projection,
            &mut RenderLayers,
        ),
        (
            With<GameplayPlayerPortraitCamera>,
            Without<CharacterSelectionPortraitCamera>,
            Without<CharacterSelectionPortraitRig>,
        ),
    >,
) {
    for (marker, mut visibility) in &mut roots {
        let selection_ready = model.visible
            && selection_ui.visible
            && marker.generation == model.slots[marker.slot].revision
            && runtime.slots[marker.slot].ready_once
            && !matches!(
                model.slots[marker.slot].status,
                CharacterSelectionPortraitStatus::Blocked(_)
            );
        let gameplay_ready = gameplay.visible
            && gameplay.slot == Some(marker.slot)
            && marker.generation == model.slots[marker.slot].revision
            && runtime.slots[marker.slot].ready_once
            && !matches!(
                model.slots[marker.slot].status,
                CharacterSelectionPortraitStatus::Blocked(_)
            );
        let ready = selection_ready || gameplay_ready;
        *visibility = if ready {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let window = windows.single().ok();
    for (marker, mut camera, mut transform, mut projection) in &mut cameras {
        let slot = marker.0;
        let ready = model.visible
            && selection_ui.visible
            && runtime.slots[slot].ready_once
            && !matches!(
                model.slots[slot].status,
                CharacterSelectionPortraitStatus::Blocked(_)
            );
        let Some(root) = runtime.slots[slot].root.filter(|_| ready) else {
            camera.is_active = false;
            camera.viewport = None;
            continue;
        };
        let Ok(bones) = bones.get(root) else {
            camera.is_active = false;
            continue;
        };
        let Some(head) = bones.unique_by_true_name(CHARACTER_SELECTION_PORTRAIT_HEAD_BONE) else {
            camera.is_active = false;
            continue;
        };
        let Ok(head_global) = globals.get(head) else {
            camera.is_active = false;
            continue;
        };
        let Some(window) = window else {
            camera.is_active = false;
            continue;
        };
        camera.is_active = true;
        camera.viewport = character_selection_portrait_viewport(&selection_ui, window, slot);
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = CHARACTER_SELECTION_PORTRAIT_CAMERA_FOV_DEGREES.to_radians();
            perspective.near = CHARACTER_SELECTION_PORTRAIT_CAMERA_NEAR;
            perspective.far = CHARACTER_SELECTION_PORTRAIT_CAMERA_FAR;
        }
        // Exact serialized Camera0..Camera3 overrides for
        // `cnSimpleCharRenderCamera::Update`, targeting this instance's
        // `Bip01 Head`: vAngle=(0,0,0), Distance=0.5, then LookAt the head and
        // add fHeight=0.12 without recalculating rotation.
        let target = head_global.translation();
        let authored_offset = Vec3::Z * CHARACTER_SELECTION_PORTRAIT_CAMERA_DISTANCE;
        let native_offset = Quat::from_rotation_y(std::f32::consts::PI) * authored_offset;
        let base_position = target + native_offset;
        let rotation = Transform::from_translation(base_position)
            .looking_at(target, Vec3::Y)
            .rotation;
        transform.translation =
            base_position + Vec3::Y * CHARACTER_SELECTION_PORTRAIT_CAMERA_HEIGHT;
        transform.rotation = rotation;
    }

    for (mut camera, mut transform, mut projection, mut layers) in &mut gameplay_cameras {
        let Some(slot) = gameplay
            .slot
            .filter(|slot| gameplay.visible && *slot < CHARACTER_SELECTION_PORTRAIT_COUNT)
        else {
            camera.is_active = false;
            continue;
        };
        let ready = runtime.slots[slot].ready_once
            && !matches!(
                model.slots[slot].status,
                CharacterSelectionPortraitStatus::Blocked(_)
            );
        let Some(root) = runtime.slots[slot].root.filter(|_| ready) else {
            camera.is_active = false;
            continue;
        };
        let Ok(rig_bones) = bones.get(root) else {
            camera.is_active = false;
            continue;
        };
        let Some(neck) = rig_bones.unique_by_true_name(GAMEPLAY_PLAYER_PORTRAIT_NECK_BONE) else {
            camera.is_active = false;
            continue;
        };
        let (Ok(root_global), Ok(neck_global)) = (globals.get(root), globals.get(neck)) else {
            camera.is_active = false;
            continue;
        };

        camera.is_active = true;
        camera.viewport = None;
        *layers = RenderLayers::layer(CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS[slot]);
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = GAMEPLAY_PLAYER_PORTRAIT_CAMERA_FOV_DEGREES.to_radians();
            perspective.near = GAMEPLAY_PLAYER_PORTRAIT_CAMERA_NEAR;
            perspective.far = GAMEPLAY_PLAYER_PORTRAIT_CAMERA_FAR;
        }
        *transform = gameplay_player_portrait_camera_transform(
            root_global.rotation(),
            neck_global.translation(),
        );
    }
}

#[must_use]
pub fn character_selection_portrait_rect(
    layout: CharacterSelectionLayout,
    slot: usize,
) -> Option<LegacySelectionRect> {
    let y = *CHARACTER_SELECTION_PORTRAIT_Y.get(slot)?;
    let scale = layout.selection_group.width / 478.0;
    Some(LegacySelectionRect::new(
        layout.selection_group.x + CHARACTER_SELECTION_PORTRAIT_X * scale,
        layout.selection_group.y + y * scale,
        CHARACTER_SELECTION_PORTRAIT_WIDTH * scale,
        CHARACTER_SELECTION_PORTRAIT_HEIGHT * scale,
    ))
}

pub(super) fn character_selection_portrait_viewport(
    selection_ui: &CharacterSelectionUiModel,
    window: &Window,
    slot: usize,
) -> Option<Viewport> {
    let layout = CharacterSelectionLayout::for_viewport(
        Vec2::new(window.width(), window.height()),
        selection_ui.ui_scale,
    );
    let rect = character_selection_portrait_rect(layout, slot)?;
    let scale_factor = window.resolution.scale_factor();
    let physical_size = UVec2::new(
        window.resolution.physical_width(),
        window.resolution.physical_height(),
    );
    let mut viewport = Viewport {
        physical_position: UVec2::new(
            (rect.x * scale_factor).round().max(0.0) as u32,
            (rect.y * scale_factor).round().max(0.0) as u32,
        ),
        physical_size: UVec2::new(
            (rect.width * scale_factor).round().max(1.0) as u32,
            (rect.height * scale_factor).round().max(1.0) as u32,
        ),
        ..default()
    };
    viewport.clamp_to_size(physical_size);
    Some(viewport)
}
