use super::*;

#[test]
fn created_character_roster_refresh_is_claimed_once_for_tutorial_entry() {
    let character = entry_test_character(99, 1, 0);
    let mut creation_session = CharacterCreationSession::default();
    creation_session.await_created_character_roster(character.pc_uid);

    let claimed = creation_session
        .take_created_character_from_roster(std::slice::from_ref(&character))
        .unwrap()
        .expect("the refreshed roster must yield the newly created character");
    assert_eq!(claimed, character);
    assert_eq!(
        creation_session
            .take_created_character_from_roster(std::slice::from_ref(&character))
            .unwrap(),
        None,
        "the automatic post-creation entry must be one-shot"
    );

    let bridge = NetworkBridge::start();
    let mut runtime = RuntimeStatus::default();
    let mut loading = GameplayLoadingState::default();
    let mut creation_ui = CharacterCreationUiModel::default();
    let mut preview = NativePlayerPreviewModel::default();
    let mut tutorial = TutorialSession::default();
    let mut next_state = NextState::<ClientState>::default();
    request_character_entry(
        &claimed,
        &bridge,
        &mut runtime,
        &mut loading,
        &mut creation_session,
        &mut creation_ui,
        &mut preview,
        &mut tutorial,
        &mut next_state,
    );

    assert_eq!(
        tutorial.character().map(|character| character.pc_uid),
        Some(99)
    );
    assert!(matches!(
        next_state,
        NextState::Pending(ClientState::TutorialIntro)
    ));
    assert_eq!(runtime.roster.pending_character_entry_uid, None);
}
