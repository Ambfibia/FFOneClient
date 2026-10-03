use super::*;

pub(super) fn setup_gameplay_player_portrait_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut portrait: ResMut<GameplayPortraitImage>,
) {
    let image = images.add(Image::new_target_texture(
        GAMEPLAY_PLAYER_PORTRAIT_TEXTURE_WIDTH,
        GAMEPLAY_PLAYER_PORTRAIT_TEXTURE_HEIGHT,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));
    portrait.0 = Some(image.clone());
    commands.spawn((
        Name::new("Gameplay independent stand1 portrait camera"),
        GameplayPlayerPortraitCamera,
        Camera3d::default(),
        bevy::camera::RenderTarget::from(image.clone()),
        Camera {
            is_active: false,
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Projection::Perspective(PerspectiveProjection {
            fov: GAMEPLAY_PLAYER_PORTRAIT_CAMERA_FOV_DEGREES.to_radians(),
            near: GAMEPLAY_PLAYER_PORTRAIT_CAMERA_NEAR,
            far: GAMEPLAY_PLAYER_PORTRAIT_CAMERA_FAR,
            ..default()
        }),
        RenderLayers::layer(CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS[0]),
        Transform::default(),
    ));
}

/// Visit one portrait-owned subtree without touching unrelated resident-world
/// entities. Scene descendants can arrive over several frames, so this remains
/// retry-safe without a global `Without<...>` scan.
pub(super) fn visit_portrait_hierarchy(
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

pub(super) fn gameplay_player_portrait_camera_transform(
    portrait_rig_rotation: Quat,
    focus: Vec3,
) -> Transform {
    // `cnCharRenderCamera` offsets along
    // `Avatar.rotation * Euler(0,-20,0) * Vector3.forward`. This independent
    // rig entity is the visual container itself, so its existing imported-model
    // half-turn is the corresponding Avatar rotation for the rendered mesh.
    // Compensating that half-turn would move the camera behind the character.
    let unity_offset =
        Quat::from_rotation_y(GAMEPLAY_PLAYER_PORTRAIT_CAMERA_YAW_DEGREES.to_radians())
            * Vec3::Z
            * GAMEPLAY_PLAYER_PORTRAIT_CAMERA_DISTANCE;
    let base_position = focus + portrait_rig_rotation * unity_to_native_vector(unity_offset);
    let rotation = Transform::from_translation(base_position)
        .looking_at(focus, Vec3::Y)
        .rotation;
    Transform {
        translation: base_position + Vec3::Y * GAMEPLAY_PLAYER_PORTRAIT_CAMERA_HEIGHT,
        rotation,
        ..default()
    }
}

pub(super) fn uses_global_skin_secondary(kind: NativePlayerPartKind) -> bool {
    matches!(
        kind,
        NativePlayerPartKind::Face
            | NativePlayerPartKind::Hair
            | NativePlayerPartKind::Shirt
            | NativePlayerPartKind::Pants
            | NativePlayerPartKind::Shoes
    )
}

pub(super) fn half_tint(color: LinearRgba) -> LinearRgba {
    LinearRgba::new(
        color.red * 0.5,
        color.green * 0.5,
        color.blue * 0.5,
        color.alpha * 0.5,
    )
}

pub(super) fn multiply_rgb(left: LinearRgba, right: LinearRgba) -> LinearRgba {
    LinearRgba::new(
        left.red * right.red,
        left.green * right.green,
        left.blue * right.blue,
        left.alpha,
    )
}
