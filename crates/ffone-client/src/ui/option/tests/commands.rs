use super::*;

#[test]
fn defaults_and_typed_remove_buddy_action_stay_transport_agnostic() {
    let mut model = OptionUiModel {
        visible: true,
        selected_tab: OptionTab::Social,
        ..default()
    };
    assert_eq!(model.draft_options.social, SocialRequestSettings::default());
    assert!(model.set_social_request(SocialRequestKind::Trade, false));
    assert!(!model.clean);
    assert!(model.restore_social_defaults());
    assert_eq!(model.draft_options.social, SocialRequestSettings::default());

    model.buddy_slots[9] = OptionBuddySlot {
        pc_uid: 9_001,
        blocked: true,
        ..default()
    };
    assert!(model.select_blocked_slot(9));
    let mut outbox = OptionUiOutbox::default();
    assert!(model.remove_selected_buddy(&mut outbox));
    assert_eq!(
        outbox.pop_front(),
        Some(OptionUiEvent::Action(OptionUiAction::RemoveBuddy {
            slot: 9,
            pc_uid: 9_001,
        }))
    );
}
