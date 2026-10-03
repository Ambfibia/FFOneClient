use super::*;

#[test]
fn enqueue_mirrors_clean_set_message_box_chat_copy_once_per_request() {
    let computress = || {
        (
            LocalizedText::new("content.npc.730.name", "Computress"),
            LocalizedText::new("content.test.body", "Welcome back."),
        )
    };
    let mut model = NanocomMessageUiModel::default();
    let (title, body) = computress();
    let npc = model.enqueue_type_9_localized(title, body, NANOCOM_NUMBUH_TWO_ICON_PATH, None);
    model.enqueue(buddy(7));
    model.enqueue(NanocomMessageRequest::group_invite(8, "Host"));
    // Interactive priority reorders presentation, not the chat receipt.
    assert_eq!(model.active().unwrap().request.request_id, 7);

    let echo = model.pop_chat_echo().unwrap();
    assert_eq!(echo.request_id, npc);
    assert!(!echo.interactive);
    assert_eq!(echo.title.key, "content.npc.730.name");
    assert_eq!(echo.body.key, "content.test.body");
    let echo = model.pop_chat_echo().unwrap();
    assert!(echo.interactive);
    assert_eq!(echo.title.key, NANOCOM_COMPACT_TITLE_LOCALIZATION_KEY);
    assert_eq!(echo.body.key, NANOCOM_INVITATION_LOCALIZATION_KEY);
    assert_eq!(echo.body.args.get("name").unwrap(), "Buddy 7");
    let echo = model.pop_chat_echo().unwrap();
    assert!(echo.interactive);
    assert_eq!(echo.title.key, NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY);
    assert_eq!(echo.body.key, NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY);
    assert_eq!(model.pop_chat_echo(), None);

    // Unkeyed passive copy keeps its exact source text.
    model.enqueue(passive(3));
    let echo = model.pop_chat_echo().unwrap();
    assert_eq!(echo.body.args.get("text").unwrap(), "Passive");

    let (title, body) = computress();
    let owned = model.enqueue_type_9_localized(title, body, NANOCOM_NUMBUH_TWO_ICON_PATH, None);
    model.enqueue(passive(4));
    model.discard_chat_echo(owned);
    assert_eq!(model.pop_chat_echo().unwrap().request_id, 4);
    assert!(!model.has_chat_echo());

    for request_id in 0..100 {
        model.enqueue(passive(request_id));
    }
    assert_eq!(
        model.pop_chat_echo().unwrap().request_id,
        100 - NANOCOM_CHAT_ECHO_LIMIT as u64
    );
    model.clear();
    assert!(!model.has_chat_echo());
}

#[test]
fn buddy_constructor_preserves_exact_copy_and_lifetime() {
    let request = NanocomMessageRequest::buddy_invite(7, "Dexter");
    assert_eq!(request.kind, NanocomMessageKind::BuddyInvite);
    assert_eq!(request.title, "Buddy request");
    assert_eq!(request.body, "Dexter has invited you to be buddies.");
    assert_eq!(request.lifetime_seconds, 20.0);
    assert_eq!(
        compact_buddy_body(&request.body, 19.6),
        "Dexter has invited you to be buddies.\n\n\
         Press Enter to accept or decline.\n20 seconds remaining. "
    );
    assert_eq!(
        expanded_buddy_expiration(19.4),
        "Buddy will expire :19 seconds"
    );
    assert_eq!(request.buddy_name.as_deref(), Some("Dexter"));

    let compact = request.compact_body_localized(19.6);
    assert_eq!(compact.key, NANOCOM_COMPACT_BODY_LOCALIZATION_KEY);
    assert_eq!(compact.args.get("name").map(String::as_str), Some("Dexter"));
    assert_eq!(compact.args.get("seconds").map(String::as_str), Some("20"));
    let expanded = request.expanded_body_localized();
    assert_eq!(expanded.key, NANOCOM_INVITATION_LOCALIZATION_KEY);
    assert_eq!(
        expanded.args.get("name").map(String::as_str),
        Some("Dexter")
    );
    let expiration = request.expanded_expiration_localized(19.4);
    assert_eq!(expiration.key, NANOCOM_EXPIRATION_LOCALIZATION_KEY);
    assert_eq!(
        expiration.args.get("seconds").map(String::as_str),
        Some("19")
    );
}

#[test]
fn reached_group_invite_is_keyed_and_strictly_parses_only_clean_table_copy() {
    let request = NanocomMessageRequest::group_invite(14, "Remote Player");
    assert_eq!(request.kind, NanocomMessageKind::GroupInvite);
    assert_eq!(request.title, NANOCOM_GROUP_COMPACT_TITLE);
    assert_eq!(
        request.body,
        "Remote Player has invited you to join a group."
    );
    assert_eq!(
        request.compact_icon_path.as_deref(),
        Some(NANOCOM_GROUP_ICON_PATH)
    );

    let title = request.compact_title_localized();
    assert_eq!(title.key, NANOCOM_GROUP_COMPACT_TITLE_LOCALIZATION_KEY);
    let compact = request.compact_body_localized(19.6);
    assert_eq!(compact.key, NANOCOM_GROUP_COMPACT_BODY_LOCALIZATION_KEY);
    assert_eq!(
        compact.args.get("name").map(String::as_str),
        Some("Remote Player")
    );
    assert_eq!(compact.args.get("seconds").map(String::as_str), Some("20"));
    assert_eq!(
        request.expanded_title_localized().key,
        NANOCOM_GROUP_EXPANDED_TITLE_LOCALIZATION_KEY
    );
    assert_eq!(
        request.expanded_body_localized().key,
        NANOCOM_GROUP_INVITATION_LOCALIZATION_KEY
    );
    assert_eq!(
        request.expanded_expiration_localized(19.4).key,
        NANOCOM_GROUP_EXPIRATION_LOCALIZATION_KEY
    );

    let mut malformed = request;
    malformed.body = "Server-authored group invitation".to_owned();
    assert_eq!(
        malformed.compact_body_localized(20.0).key,
        NANOCOM_CONTENT_LOCALIZATION_KEY
    );
    assert_eq!(
        malformed.expanded_body_localized().key,
        NANOCOM_CONTENT_LOCALIZATION_KEY
    );
}
