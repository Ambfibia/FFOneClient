use super::*;

#[test]
fn cargo_run_uses_project_assets_by_default() {
    let config = ClientConfig::from_args(Vec::<String>::new())
        .unwrap()
        .unwrap();
    assert_eq!(config.asset_root, PathBuf::from("assets/game"));
    assert!(!config.asset_root_explicit);
    assert_eq!(config.login_address, "127.0.0.1:23000");
    assert_eq!(config.password_env, "FFONE_PASSWORD");
    assert_eq!(
        config.character_asset_root,
        PathBuf::from(DEFAULT_CHARACTER_ASSET_ROOT)
    );
    assert!(!config.character_asset_root_explicit);
    assert_eq!(config.character_model, DEFAULT_CHARACTER_MODEL);
    assert_eq!(config.character_root, DEFAULT_CHARACTER_ROOT);
    assert_eq!(config.character_animation, DEFAULT_CHARACTER_ANIMATION);
    assert!(!config.validate_assets);
    assert!(!config.network_smoke);
}
