use super::*;

#[test]
fn creator_gender_switch_resolves_a_distinct_female_preview_rig() {
    let data = CharacterCreationData::open(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    let session = CharacterCreationSession {
        saved_name: Some(CharacterNameSaveSuccess0104 {
            pc_uid: 77,
            slot: 1,
            gender: 1,
            first_name: FixedUtf16::from_str("Preview").unwrap(),
            last_name: FixedUtf16::from_str("Switch").unwrap(),
        }),
        ..default()
    };
    let counts = data.option_counts().unwrap();
    let mut appearance = CharacterAppearance::default();
    let male = resolve_current_creator(&data, &session, &appearance).unwrap();
    appearance.set_gender(CharacterGender::Girl, counts);
    let female = resolve_current_creator(&data, &session, &appearance).unwrap();

    assert_eq!(
        male.look.gender,
        ffone_runtime_contracts::PlayerRigGender::Male
    );
    assert_eq!(
        female.look.gender,
        ffone_runtime_contracts::PlayerRigGender::Female
    );
    assert_ne!(male.look.parts, female.look.parts);
}

#[test]
fn character_table_routes_validate_root_and_animation_without_migration_hashes() {
    let temp = tempfile::tempdir().unwrap();
    let model = temp.path().join(DEFAULT_CHARACTER_MODEL);
    fs::create_dir_all(model.parent().unwrap()).unwrap();
    let glb = b"native logical GLB fixture";
    fs::write(&model, glb).unwrap();
    fs::create_dir_all(temp.path().join("data/tables")).unwrap();
    fs::write(
        temp.path().join("data/tables/xdt.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema": "ffone.table-set.v1",
            "tables": [{"name":"native_asset_routes","value":{"m_pCharacterModelData": [{
                "id": "npc/npc_dexter",
                "glb": DEFAULT_CHARACTER_MODEL,
                "logicalName": DEFAULT_CHARACTER_ROOT,
                "glbBlake3": blake3::hash(glb).to_hex().to_string(),
                "category": "npc",
                "animations": [DEFAULT_CHARACTER_ANIMATION]
            }]}}]
        }))
        .unwrap(),
    )
    .unwrap();

    let config = ClientConfig::from_args([
        "--character-asset-root".to_owned(),
        temp.path().display().to_string(),
    ])
    .unwrap()
    .unwrap();
    assert_eq!(config.validate_character_asset(), Ok(()));

    fs::write(&model, b"tampered native GLB").unwrap();
    assert_eq!(config.validate_character_asset(), Ok(()));
    let mut invalid = config.clone();
    invalid.character_animation = "missing_animation".to_owned();
    assert!(invalid.validate_character_asset().is_err());
}
