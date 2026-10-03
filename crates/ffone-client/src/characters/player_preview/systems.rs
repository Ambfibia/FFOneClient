use super::*;

pub(super) fn sync_native_player_preview_transform(
    mut commands: Commands,
    model: Res<NativePlayerPreviewModel>,
    mut runtime: ResMut<NativePlayerPreviewRuntime>,
    creation_ui: Option<Res<CharacterCreationUiModel>>,
    selection_ui: Option<Res<CharacterSelectionUiModel>>,
    inventory_image: Res<NativePlayerInventoryPreviewImage>,
    try_on_image: Res<NativePlayerTryOnPreviewImage>,
    barber_image: Res<NativePlayerBarberPreviewImage>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<
        (Entity, &mut Transform, &mut Visibility),
        (
            With<NativePlayerPreviewRoot>,
            Without<NativePlayerPreviewCamera>,
        ),
    >,
    mut cameras: Query<
        (
            &mut Camera,
            &mut RenderTarget,
            &mut Transform,
            &mut Projection,
        ),
        (
            With<NativePlayerPreviewCamera>,
            Without<NativePlayerPreviewRoot>,
        ),
    >,
    rig_bones: Query<&NativePlayerRigBones>,
    globals: Query<&GlobalTransform>,
) {
    for (entity, mut transform, mut visibility) in &mut roots {
        let facing = native_scene_container_transform(NativeSceneRole::CharacterGameplay).rotation;
        let yaw_degrees = if model.yaw_degrees.is_finite() {
            model.yaw_degrees
        } else {
            0.0
        };
        transform.translation = Vec3::ZERO;
        transform.rotation = Quat::from_rotation_y(yaw_degrees.to_radians()) * facing;
        transform.scale = Vec3::ONE;
        // A failure belongs to the newest hidden generation. Selection hides
        // the prior character; other preview stages can retain their last
        // ready root until an explicit clear removes the double-buffer entry.
        *visibility = if model.visible
            && runtime.displayed_root == Some(entity)
            && (model.stage != NativePlayerPreviewStage::Selection
                || runtime.ready_generation == Some(model.revision))
            && (model.stage != NativePlayerPreviewStage::TryOn
                || runtime.ready_generation == Some(model.revision))
        {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let camera = match runtime.camera {
        Some(camera) => camera,
        None => {
            let camera = commands
                .spawn((
                    Name::new("Native player preview camera"),
                    NativePlayerPreviewCamera,
                    Camera3d::default(),
                    Camera {
                        is_active: model.visible,
                        // The panorama is an opaque UI image. Rendering first
                        // hides the assembled player behind that UI pass.
                        order: NATIVE_PLAYER_PREVIEW_CAMERA_ORDER,
                        clear_color: ClearColorConfig::None,
                        ..default()
                    },
                    Projection::Perspective(PerspectiveProjection {
                        fov: 45.0_f32.to_radians(),
                        near: 0.3,
                        far: 1000.0,
                        ..default()
                    }),
                    RenderLayers::layer(NATIVE_PLAYER_PREVIEW_RENDER_LAYER),
                    Transform::default(),
                ))
                .id();
            runtime.camera = Some(camera);
            camera
        }
    };
    if let Ok((mut camera, mut target, mut transform, mut projection)) = cameras.get_mut(camera) {
        let inventory_stage = matches!(
            model.stage,
            NativePlayerPreviewStage::Inventory | NativePlayerPreviewStage::TryOn | NativePlayerPreviewStage::Barber
        );
        // The inventory stage clears its transparent target even while hidden
        // or while a newly authoritative rig is assembling. This prevents
        // pixels from another character/session leaking into the UserEquip UI.
        camera.is_active = inventory_stage
            || (model.visible
                && runtime.displayed_root.is_some()
                && (model.stage != NativePlayerPreviewStage::Selection
                    || runtime.ready_generation == Some(model.revision)));
        camera.order = if inventory_stage {
            NATIVE_PLAYER_INVENTORY_PREVIEW_CAMERA_ORDER
        } else {
            NATIVE_PLAYER_PREVIEW_CAMERA_ORDER
        };
        *target = if inventory_stage {
            if model.stage == NativePlayerPreviewStage::Barber {
                barber_image.0.clone().into()
            } else if model.stage == NativePlayerPreviewStage::TryOn {
                try_on_image.0.clone().into()
            } else {
                inventory_image.0.clone().into()
            }
        } else {
            RenderTarget::Window(WindowRef::Primary)
        };
        camera.clear_color = if inventory_stage {
            ClearColorConfig::Custom(Color::NONE)
        } else {
            ClearColorConfig::None
        };
        camera.viewport = if inventory_stage {
            None
        } else {
            windows.single().ok().and_then(|window| {
                native_player_preview_viewport(
                    &model,
                    creation_ui.as_deref(),
                    selection_ui.as_deref(),
                    window,
                )
            })
        };
        if let Projection::Perspective(perspective) = &mut *projection {
            perspective.fov = 45.0_f32.to_radians();
            perspective.near = if inventory_stage { 0.3 } else { 0.1 };
            perspective.far = 1000.0;
        }
        let head_height = runtime
            .displayed_root
            .and_then(|root| rig_bones.get(root).ok())
            .and_then(|bones| bones.unique_by_true_name("Bip01 Head"))
            .and_then(|head| globals.get(head).ok())
            .map(GlobalTransform::translation)
            .map(|translation| translation.y);
        let (distance, camera_height) =
            native_player_preview_camera_distance_and_height(&model, head_height);
        // Exact `cnSimpleCharRenderCamera`: start at
        // `Avatar + Euler(-10, 0, 0) * Vector3.forward * Distance`, look at
        // the avatar, then add the serialized `fHeight` without recomputing
        // rotation. Creation raises that height toward the actual instance
        // head only while zooming closer than its authored distance.
        // The character container's one 180-degree gameplay turn maps the
        // authored +Z camera side to native -Z.
        let authored_camera = Quat::from_rotation_x((-10.0_f32).to_radians()) * Vec3::Z * distance;
        let base_position = Quat::from_rotation_y(std::f32::consts::PI) * authored_camera;
        let rotation = Transform::from_translation(base_position)
            .looking_at(Vec3::ZERO, Vec3::Y)
            .rotation;
        transform.translation = base_position + Vec3::Y * camera_height;
        transform.rotation = rotation;
    }
}

#[cfg(test)]
mod selection_generation_tests {
    use super::*;

    #[test]
    fn reused_selection_rig_is_hidden_until_new_look_is_ready() {
        let mut app = App::new();
        let mut model = NativePlayerPreviewModel::default();
        model.stage = NativePlayerPreviewStage::Selection;
        model.visible = true;
        model.revision = 2;
        app.insert_resource(model);
        let root = app
            .world_mut()
            .spawn((NativePlayerPreviewRoot, Transform::default(), Visibility::Inherited))
            .id();
        let camera = app
            .world_mut()
            .spawn((
                NativePlayerPreviewCamera,
                Camera::default(),
                RenderTarget::Window(WindowRef::Primary),
                Transform::default(),
                Projection::Perspective(PerspectiveProjection::default()),
            ))
            .id();
        app.insert_resource(NativePlayerPreviewRuntime {
            root: Some(root),
            displayed_root: Some(root),
            ready_generation: Some(1),
            camera: Some(camera),
            ..default()
        });
        app.insert_resource(NativePlayerInventoryPreviewImage(Handle::default()));
        app.insert_resource(NativePlayerTryOnPreviewImage(Handle::default()));
        app.insert_resource(NativePlayerBarberPreviewImage(Handle::default()));
        app.add_systems(Update, sync_native_player_preview_transform);

        app.update();
        assert_eq!(app.world().get::<Visibility>(root), Some(&Visibility::Hidden));
        assert!(!app.world().get::<Camera>(camera).unwrap().is_active);

        app.world_mut()
            .resource_mut::<NativePlayerPreviewRuntime>()
            .ready_generation = Some(2);
        app.update();
        assert_eq!(app.world().get::<Visibility>(root), Some(&Visibility::Inherited));
        assert!(app.world().get::<Camera>(camera).unwrap().is_active);
    }
}
