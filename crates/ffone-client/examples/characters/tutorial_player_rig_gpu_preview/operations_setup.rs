use super::*;

pub(super) fn drive_vehicle_trail_probe(
    config: Option<Res<PreviewConfig>>,
    state: Res<PreviewState>,
    mut transforms: Query<&mut Transform>,
) {
    let Some(config) = config else {
        return;
    };
    if !matches!(
        config.case,
        PreviewCase::VehicleBoard | PreviewCase::VehicleScooter
    ) || state.capture_issued
    {
        return;
    }
    if let Ok(mut transform) = transforms.get_mut(config.controller_root) {
        // Keep matched orbit captures centered after exercising trail motion.
        if env::var_os("FFONE_PLAYER_PREVIEW_DISMOUNT").is_some()
            && state
                .ready_frame
                .is_some_and(|ready| state.frames > ready + 100)
        {
            transform.translation = Vec3::ZERO;
            return;
        }
        let phase = state.frames as f32 * 0.05;
        transform.translation.x = phase.sin() * 1.3;
        transform.translation.z = phase.cos() * 0.8;
    }
}

pub(super) fn setup(
    commands: &mut Commands,
    assets: &AssetServer,
    asset_cache: &mut NativePlayerRigAssetCache,
    catalog: &NativePlayerRigCatalog,
    weapon_animations: &PlayerWeaponAnimationCatalog,
    character_data: &CharacterCreationData,
    output: PathBuf,
    height: u8,
    body: u8,
    direction: u8,
    case: PreviewCase,
    selected_gender: CharacterGender,
) {
    let mut controller = LegacyPlayerController::from_baseline_table();
    assert!(controller.set_current_direction_key(direction));
    if matches!(case, PreviewCase::AttackFall | PreviewCase::StartupLanding) {
        controller.set_grounded(false);
    }
    if case == PreviewCase::AttackFall {
        controller.jumping = true;
    }
    let controller_root = commands
        .spawn((
            Name::new("Tutorial GPU proof controller"),
            Transform::default(),
            Visibility::Inherited,
            controller,
            LegacyAvatarActionContext {
                tutorial_event: true,
                ..default()
            },
            LegacyAvatarTargetFeed::default(),
            LegacyAvatarClipBindings::default(),
            LegacyAvatarActionState::default(),
        ))
        .id();
    if matches!(case, PreviewCase::Skyway | PreviewCase::Zipline) {
        commands
            .entity(controller_root)
            .insert(LegacyAvatarPresentationContext {
                traversal: if case == PreviewCase::Skyway {
                    LegacyAvatarTraversalPresentation::BroomStick
                } else {
                    LegacyAvatarTraversalPresentation::Zipline
                },
                ..default()
            });
        commands.insert_resource(TutorialSkywayPresentation {
            active: case == PreviewCase::Skyway,
        });
    }
    if matches!(
        case,
        PreviewCase::VehicleBoard | PreviewCase::VehicleScooter
    ) {
        let board = case == PreviewCase::VehicleBoard;
        let family = if board {
            ffone_client::avatar_action::LegacyVehiclePresentationFamily::Board
        } else {
            ffone_client::avatar_action::LegacyVehiclePresentationFamily::Scooter
        };
        let item_id = env::var("FFONE_VEHICLE_PREVIEW_ITEM")
            .ok()
            .map(|v| v.parse::<i16>().expect("vehicle item id"))
            .unwrap_or(if board { 1 } else { 4 });
        commands.insert_resource(
            ffone_client::tutorial_player_rig_runtime::PersonalVehiclePresentation {
                item_id: Some(item_id),
                family,
                ..default()
            },
        );
        commands
            .entity(controller_root)
            .insert(LegacyAvatarPresentationContext {
                mounted_vehicle: family,
                ..default()
            });
    }
    let mut appearance = CharacterAppearance::default();
    appearance.gender = selected_gender;
    appearance.height = height;
    appearance.body = body;
    let creator = character_data
        .resolve_creator(1, 1, "GPU", "Proof", &appearance)
        .expect("resolve the production default creator");
    let gender = tutorial_player_gender_from_protocol(creator.style.gender)
        .expect("GPU proof creator must use a supported player gender");
    let character = CharacterSummary {
        slot: 1,
        level: 1,
        pc_uid: 1,
        first_name: "GPU".to_owned(),
        last_name: "Proof".to_owned(),
        position: [0; 3],
        style: CharacterStyle0104 {
            name_check: creator.style.name_check,
            gender: creator.style.gender,
            face_style: creator.style.face_style,
            hair_style: creator.style.hair_style,
            hair_color: creator.style.hair_color,
            skin_color: creator.style.skin_color,
            eye_color: creator.style.eye_color,
            height: creator.style.height,
            body: creator.style.body,
            class: creator.style.class,
            appearance_flag: 1,
            tutorial_flag: 0,
            payzone_flag: 0,
        },
        equipment: [EquippedItem0104::default(); CHARACTER_EQUIP_SLOT_COUNT_0104],
    };
    let spawned = spawn_tutorial_selected_player_rig(
        commands,
        assets,
        asset_cache,
        catalog,
        weapon_animations,
        character_data,
        controller_root,
        &character,
        1,
        RenderLayers::default(),
        None,
        true,
    )
    .expect("spawn the complete tutorial player rig");

    let target = Vec3::new(
        0.0,
        if case == PreviewCase::Beach {
            0.25
        } else {
            0.8
        },
        0.0,
    );
    let default_camera_distance = if case == PreviewCase::Skyway {
        5.0
    } else {
        2.3
    };
    let camera_distance = env::var("FFONE_PLAYER_PREVIEW_DISTANCE")
        .ok()
        .map(|value| value.parse::<f32>().expect("positive camera distance"))
        .unwrap_or(default_camera_distance);
    assert!(camera_distance.is_finite() && camera_distance > 0.0);
    let camera_pitch: f32 = if case == PreviewCase::Beach {
        35.0
    } else {
        10.0
    };
    let camera_pitch = env::var("FFONE_PLAYER_PREVIEW_PITCH")
        .ok()
        .map(|value| value.parse::<f32>().expect("preview pitch must be degrees"))
        .unwrap_or(camera_pitch);
    let camera_y = target.y + camera_distance * camera_pitch.to_radians().tan();
    let camera_yaw = env::var("FFONE_PLAYER_PREVIEW_YAW")
        .ok()
        .map(|value| value.parse::<f32>().expect("preview yaw must be degrees"))
        .unwrap_or(0.0)
        .to_radians();
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 45_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(
            camera_distance * camera_yaw.sin(),
            camera_y,
            -camera_distance * camera_yaw.cos(),
        )
        .looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -3.0).looking_at(target, Vec3::Y),
    ));
    commands.spawn((
        PointLight {
            intensity: 900_000.0,
            range: 8.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(1.2, 1.8, -2.0),
    ));
    commands.insert_resource(PreviewConfig {
        output,
        rig_root: spawned.rig_root,
        controller_root,
        gender,
        expected_clip: match direction {
            _ if case == PreviewCase::AttackFall => TutorialPlayerClip::JumpStart,
            0 => TutorialPlayerClip::Stand1,
            4..=6 => TutorialPlayerClip::RunBack,
            _ => TutorialPlayerClip::Run,
        },
        case,
    });
}

pub(super) fn drive_attack_input(
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    if !config.case.captures_attack() && env::var_os("FFONE_EMOTE_ATTACK_PROBE").is_none() {
        return;
    }
    if state.attack_pending {
        keyboard.press(KeyCode::KeyZ);
        state.attack_pending = false;
        state.attack_key_down = true;
        state.attack_dispatched = true;
    } else if state.attack_key_down {
        keyboard.release(KeyCode::KeyZ);
        state.attack_key_down = false;
    }
}

/// Diagnostic A/B only; normal previews retain every authored outline pass.
pub(super) fn disable_preview_outlines(
    mut outlines: Query<
        &mut Visibility,
        With<ffone_client::legacy_model_material::LegacyOutlineCompanion>,
    >,
) {
    if env::var_os("FFONE_PLAYER_PREVIEW_NO_OUTLINE").is_none() {
        return;
    }
    for mut visibility in &mut outlines {
        *visibility = Visibility::Hidden;
    }
}
