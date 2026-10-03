use super::*;

#[test]
fn outgoing_chat_audio_drains_once_without_dirtying_idle_runtime() {
    let mut app = App::new();
    app.init_resource::<RuntimeStatus>()
        .init_resource::<MissionUiModel>()
        .init_resource::<GameplayUiAudioOutbox>()
        .init_resource::<GameplayAudioRuntime>()
        .add_systems(Update, consume_gameplay_ui_audio_outbox);
    app.world_mut()
        .resource_mut::<RuntimeStatus>()
        .chat
        .outgoing_audio_pending = 2;
    app.world_mut().resource_mut::<RuntimeStatus>().chat.pending_social_sfx =
        vec!["Incoming_Tell", "Outgoing_Chat", "Buddy_Warp"];
    app.update();
    assert_eq!(
        app.world()
            .resource::<RuntimeStatus>()
            .chat
            .outgoing_audio_pending,
        0
    );
    assert!(app.world().resource::<RuntimeStatus>().chat.pending_social_sfx.is_empty());
    app.world_mut().clear_trackers();
    app.update();
    assert!(!app.world().resource_ref::<RuntimeStatus>().is_changed());
}

#[test]
fn outgoing_chat_audio_requires_an_accepted_local_echo() {
    let mut runtime = RuntimeStatus::default();
    runtime.player_id = Some(81);
    runtime.player_name = "Local".into();
    runtime.free_chat = true;
    for (pc_id, blocked, enabled, expected) in [
        (82, false, true, 0),
        (81, true, true, 0),
        (81, false, false, 0),
        (81, false, true, 1),
    ] {
        runtime.free_chat = enabled;
        apply_freechat_success(
            FreeChatSuccess0104 {
                pc_id,
                message: FixedUtf16::from_str("Hello").unwrap(),
                emote_code: 0,
            },
            &mut runtime,
            Some("Remote".into()),
            blocked,
        );
        assert_eq!(runtime.chat.outgoing_audio_pending, expected);
    }
    apply_all_group_freechat_success(
        AllGroupFreeChatSuccess0104 {
            sender_pc_id: 81,
            message: FixedUtf16::from_str("Group").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        None,
        ChatChannel::All,
        false,
    );
    assert_eq!(
        runtime.chat.outgoing_audio_pending, 1,
        "unknown group member is silent"
    );
    apply_all_group_freechat_success(
        AllGroupFreeChatSuccess0104 {
            sender_pc_id: 81,
            message: FixedUtf16::from_str("Group").unwrap(),
            emote_code: 0,
        },
        &mut runtime,
        Some("Local".into()),
        ChatChannel::All,
        false,
    );
    assert_eq!(runtime.chat.outgoing_audio_pending, 2);
}
