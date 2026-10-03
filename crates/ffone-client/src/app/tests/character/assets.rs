use super::*;

#[test]
fn default_asset_roots_use_packaged_tree_when_launch_cwd_has_no_assets() {
    let temp = tempfile::tempdir().unwrap();
    let launch_cwd = temp.path().join("arbitrary-cwd");
    let executable_dir = temp.path().join("release");
    std::fs::create_dir_all(&launch_cwd).unwrap();
    std::fs::create_dir_all(executable_dir.join("assets/game")).unwrap();
    let mut config = ClientConfig::from_args(Vec::<String>::new())
        .unwrap()
        .unwrap();

    config.make_asset_roots_absolute_from(&launch_cwd, Some(&executable_dir));

    let packaged_assets = executable_dir.join("assets/game");
    assert_eq!(config.asset_root, packaged_assets);
    assert_eq!(config.character_asset_root, packaged_assets);
}

#[test]
fn default_asset_roots_keep_repo_cwd_for_development_runs() {
    let temp = tempfile::tempdir().unwrap();
    let repo_cwd = temp.path().join("repo");
    let executable_dir = temp.path().join("target/debug");
    std::fs::create_dir_all(repo_cwd.join("assets/game")).unwrap();
    std::fs::create_dir_all(executable_dir.join("assets/game")).unwrap();
    let mut config = ClientConfig::from_args(Vec::<String>::new())
        .unwrap()
        .unwrap();

    config.make_asset_roots_absolute_from(&repo_cwd, Some(&executable_dir));

    let development_assets = repo_cwd.join("assets/game");
    assert_eq!(config.asset_root, development_assets);
    assert_eq!(config.character_asset_root, development_assets);
}

#[test]
fn explicit_relative_asset_roots_never_fall_back_beside_executable() {
    let temp = tempfile::tempdir().unwrap();
    let launch_cwd = temp.path().join("arbitrary-cwd");
    let executable_dir = temp.path().join("release");
    std::fs::create_dir_all(&launch_cwd).unwrap();
    std::fs::create_dir_all(executable_dir.join("assets/game")).unwrap();
    let mut config = ClientConfig::from_args([
        "--asset-root".to_owned(),
        "assets/game".to_owned(),
        "--character-asset-root".to_owned(),
        "assets/game".to_owned(),
    ])
    .unwrap()
    .unwrap();

    config.make_asset_roots_absolute_from(&launch_cwd, Some(&executable_dir));

    let explicit_assets = launch_cwd.join("assets/game");
    assert!(config.asset_root_explicit);
    assert!(config.character_asset_root_explicit);
    assert_eq!(config.asset_root, explicit_assets);
    assert_eq!(config.character_asset_root, explicit_assets);
}

#[test]
fn native_asset_and_auto_character_flags_parse() {
    let config = ClientConfig::from_args([
        "--asset-root".to_owned(),
        "native-assets".to_owned(),
        "--login-user".to_owned(),
        "test-user".to_owned(),
        "--character".to_owned(),
        "1234".to_owned(),
        "--character-asset-root".to_owned(),
        "native-characters".to_owned(),
        "--character-model".to_owned(),
        "characters/npcs/npc_dexter/npc_dexter.glb".to_owned(),
        "--character-root".to_owned(),
        "npc_dexter".to_owned(),
        "--character-animation".to_owned(),
        "stand2".to_owned(),
        "--validate-assets".to_owned(),
        "--network-smoke".to_owned(),
    ])
    .unwrap()
    .unwrap();
    assert_eq!(config.asset_root, PathBuf::from("native-assets"));
    assert_eq!(config.login_user.as_deref(), Some("test-user"));
    assert_eq!(config.character_uid, Some(1234));
    assert_eq!(
        config.character_asset_root,
        PathBuf::from("native-characters")
    );
    assert_eq!(
        config.character_model,
        "characters/npcs/npc_dexter/npc_dexter.glb"
    );
    assert_eq!(config.character_root, "npc_dexter");
    assert_eq!(config.character_animation, "stand2");
    assert!(config.validate_assets);
    assert!(config.network_smoke);
}

#[test]
fn character_creation_asset_lease_reports_partial_work_and_clears_on_exit() {
    let mut lease = CharacterCreationAssetLease {
        total: 32,
        completed: 8,
        ..default()
    };
    assert_eq!(lease.progress(), 0.25);
    assert!(!lease.is_ready());
    lease.completed = 32;
    assert!(lease.is_ready());
    lease.clear();
    assert_eq!(lease.progress(), 0.0);
    assert!(!lease.is_ready());
    assert!(lease.pending.is_empty());
}
