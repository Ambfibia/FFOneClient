use super::*;

#[test]
fn character_creation_delayed_name_reply_restarts_loading_and_retains_server_identity() {
    let mut app = creation_loading_test_app(CharacterCreationScreen::Name);
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .finish();
    app.world_mut()
        .resource_mut::<CharacterCreationUiModel>()
        .asset_status = CharacterCreationAssetStatus::Ready;
    let saved = CharacterNameSaveSuccess0104 {
        pc_uid: 42,
        slot: 2,
        gender: 1,
        first_name: ffone_protocol::FixedUtf16::from_str("Test").unwrap(),
        last_name: ffone_protocol::FixedUtf16::from_str("Creator").unwrap(),
    };
    let mut session = CharacterCreationSession::default();
    session.pending_name_check =
        Some(CharacterNameCheckRequest0104::new("Test", "Creator", 0, 0, 0).unwrap());
    let mut params = bevy::ecs::system::SystemState::<(
        ResMut<CharacterCreationUiModel>,
        ResMut<GameplayLoadingState>,
    )>::new(app.world_mut());
    let (mut model, mut loading) = params.get_mut(app.world_mut()).unwrap();
    character_flow::accept_reserved_character_name(
        saved.clone(),
        &mut session,
        &mut model,
        &mut loading,
        &mut RuntimeStatus::default(),
    );
    assert!(session.pending_name_check.is_none());
    assert_eq!(session.saved_name.as_ref().unwrap().pc_uid, 42);
    assert_eq!(model.pc_uid, Some(42));
    assert_eq!(model.screen, CharacterCreationScreen::Appearance);
    assert!(model.randomize_on_appearance_open);
    assert_eq!(loading.scope, Some(ResourceLoadingScope::CharacterCreation));
    assert!(loading.visible);
    drop((model, loading));
    app.update();
    assert!(
        app.world().resource::<GameplayLoadingState>().visible,
        "the name reply must not bypass avatar loading"
    );
}
