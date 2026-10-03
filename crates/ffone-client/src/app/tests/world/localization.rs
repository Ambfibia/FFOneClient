use super::*;

#[test]
fn skybox_uses_dynamic_default_ambience_tint_and_strips_camera_translation() {
    let source_fog = [0.6, 0.7, 0.8, 0.85];
    let source_light = [0.4, 0.75, 1.0, 1.0];
    let tint = legacy_sky_shader_tint(source_fog, source_light).to_srgba();
    assert!((tint.red - 1.2).abs() < 0.000_01);
    assert!((tint.green - 1.4).abs() < 0.000_01);
    assert!((tint.blue - 1.6).abs() < 0.000_01);
    assert_eq!(tint.alpha, 1.0);

    let world_camera = Transform::from_xyz(125.0, -48.0, 911.0).with_rotation(Quat::from_euler(
        EulerRot::YXZ,
        0.7,
        -0.2,
        0.0,
    ));
    let background = legacy_skybox_camera_transform(&world_camera);
    assert_eq!(background.translation, Vec3::ZERO);
    assert_eq!(background.rotation, world_camera.rotation);
    assert_eq!(background.scale, Vec3::ONE);
}

#[test]
fn mission_delete_confirmation_uses_correlated_id_and_exact_localized_body() {
    let content = runtime_test_mission_content();
    let definition = content.mission(500).expect("primary task 500");
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let (localization, mut language) = Localization::open(&asset_root, "en").unwrap();
    let body = localized_mission_delete_confirmation(
        &content,
        &localization,
        &language,
        definition.provenance.task_id,
    )
    .unwrap();
    let kind = match definition.mission_type {
        TutorialMissionType::Guide => "Guide",
        TutorialMissionType::Nano => "Nano",
        TutorialMissionType::World => "Quest",
    };
    assert_eq!(
        body,
        format!(
            "Delete\n {kind} : {}\nThis mission will no longer appear in your mission journal,\nbut you can get it again later by visiting the mission giver.",
            definition.title
        )
    );

    let mut messages = SystemMessageUiModel::default();
    let mut runtime = MissionDeleteConfirmationRuntime::default();
    let request_id = runtime
        .queue(&mut messages, definition.provenance.task_id, body.clone())
        .unwrap();
    assert_eq!(request_id, MISSION_DELETE_SYSTEM_MESSAGE_ID_BASE);
    assert_ne!(request_id, definition.provenance.task_id as u64);
    assert_ne!(request_id, 7);
    assert_ne!(request_id, u64::MAX);
    assert!(
        runtime
            .queue(&mut messages, definition.provenance.task_id, body)
            .is_err(),
        "one journal cannot own duplicate confirmations"
    );
    let request = messages.current().unwrap();
    assert_eq!(request.request_id, request_id);
    assert_eq!(request.button_type, SystemMessageButtonType::DeleteMission);
    runtime.reset();
    assert!(runtime.pending.is_empty());
    assert_eq!(
        runtime.next_request_id,
        MISSION_DELETE_SYSTEM_MESSAGE_ID_BASE
    );

    localization.select(&mut language, "ru");
    let russian = localized_mission_delete_confirmation(
        &content,
        &localization,
        &language,
        definition.provenance.task_id,
    )
    .unwrap();
    assert!(russian.starts_with("Удалить\n "));
    assert!(!russian.contains("This mission will no longer appear"));
}
