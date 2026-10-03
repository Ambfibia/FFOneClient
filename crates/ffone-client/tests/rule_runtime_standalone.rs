#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{localization, ui_startup};

#[path = "../src/gameplay/rule/mod.rs"]
mod rule_runtime;
#[path = "../src/ui/rule/mod.rs"]
mod rule_ui;

use rule_runtime::*;
use rule_ui::{
    RulePageId, RuleUiAction, RuleUiAudioOutbox, RuleUiButtonKind, RuleUiDismissalSource,
    RuleUiModel, RuleUiOutbox, activate_rule_ui_button, request_rule_ui_escape_close,
};

fn drain(runtime: &mut RuleRuntime) -> Vec<RuleRuntimeCommand> {
    runtime.drain_commands().collect()
}

fn npc_request(service_number: i32) -> RuleOpenRequest {
    RuleOpenRequest::NpcService {
        runtime_npc_id: 9_001,
        table_npc_id: 1_337,
        service_number,
    }
}

#[test]
fn clean_npc_services_route_only_nine_and_ten_to_the_two_pages() {
    let vehicle = rule_npc_service_route(9).unwrap();
    assert_eq!(vehicle.page, RulePageId::Vehicle);
    assert_eq!(vehicle.table_index, 1);
    assert_eq!(vehicle.label_key, "WHAT ARE VEHICLES?");
    assert_eq!(vehicle.label_prefix, " ");

    let combining = rule_npc_service_route(10).unwrap();
    assert_eq!(combining.page, RulePageId::Combining);
    assert_eq!(combining.table_index, 2);
    assert_eq!(combining.label_key, "WHAT IS COMBINING?");
    assert_eq!(combining.label_prefix, " ");

    for unsupported in [i32::MIN, -1, 0, 1, 8, 11, i32::MAX] {
        assert_eq!(rule_npc_service_route(unsupported), None);
    }
}

#[test]
fn npc_entry_commits_one_exact_local_linked_mode_transaction() {
    let mut runtime = RuleRuntime::default();
    let mut model = RuleUiModel::default();
    let session = runtime.open(&mut model, npc_request(10)).unwrap();

    assert_eq!(session.page, RulePageId::Combining);
    assert_eq!(session.dispatch.event_group, 2);
    assert_eq!(session.dispatch.create_mode_function, 0);
    assert_eq!(session.dispatch.game_mode, 30);
    assert_eq!(session.dispatch.send_to_element_function, 3);
    assert_eq!(session.dispatch.mode_element_function, 0);
    assert_eq!(session.dispatch.table_index, 2);
    assert_eq!(runtime.phase(), RuleRuntimePhase::Active);
    assert_eq!(runtime.camera_sub_target_npc_id(), Some(9_001));
    assert!(model.visible);
    assert_eq!(model.current_page, Some(RulePageId::Combining));

    let commands = drain(&mut runtime);
    assert_eq!(
        commands[0],
        RuleRuntimeCommand::PlaySound(RuleRuntimeSound::RandomButtonSound)
    );
    assert_eq!(
        commands[1],
        RuleRuntimeCommand::PlaySound(RuleRuntimeSound::RandomButtonSound)
    );
    let RuleRuntimeCommand::EnterMode(enter) = commands[2] else {
        panic!("third command must enter Rule mode after both NPC click sounds")
    };
    assert_eq!(enter.session, session);
    assert_eq!(enter.linked_game_mode, 5);
    assert!(enter.hide_gameplay_hud);
    assert!(!enter.gameplay_input_enabled);
    assert!(!enter.cursor_locked);
    assert!(enter.preserve_camera_sub_target);
    assert!(!enter.send_npc_interaction_close_packet);
    assert!(!enter.send_special_state_packet);
    assert_eq!(
        commands[3],
        RuleRuntimeCommand::PlaySound(RuleRuntimeSound::OpenScreen)
    );
    assert_eq!(commands.len(), 4);
    assert!(RULE_RUNTIME_IS_LOCAL_ONLY);
    assert!(!RULE_RUNTIME_SENDS_PACKETS);
    assert!(!RULE_RUNTIME_CHANGES_SPECIAL_STATE);
}

#[test]
fn invalid_entry_is_transactional_and_gm_chat_keeps_the_clean_gate() {
    let mut runtime = RuleRuntime::default();
    let mut model = RuleUiModel::default();

    assert_eq!(
        runtime.open(&mut model, npc_request(8)),
        Err(RuleRuntimeError::UnsupportedNpcServiceNumber(8))
    );
    assert_eq!(runtime.phase(), RuleRuntimePhase::Hidden);
    assert_eq!(runtime.session(), None);
    assert!(!model.visible);
    assert!(runtime.commands_are_empty());

    assert_eq!(
        runtime.open(
            &mut model,
            RuleOpenRequest::GmChatCommand {
                user_level: 51,
                table_index: 1,
            },
        ),
        Err(RuleRuntimeError::GmChatNotAuthorized { user_level: 51 })
    );
    assert_eq!(
        runtime.open(
            &mut model,
            RuleOpenRequest::GmChatCommand {
                user_level: 50,
                table_index: 0,
            },
        ),
        Err(RuleRuntimeError::InvalidPageIndex(
            rule_ui::RulePageIndexError(0)
        ))
    );
    assert!(!model.visible);
    assert!(runtime.commands_are_empty());

    let session = runtime
        .open(
            &mut model,
            RuleOpenRequest::GmChatCommand {
                user_level: 50,
                table_index: 1,
            },
        )
        .unwrap();
    assert_eq!(session.page, RulePageId::Vehicle);
    assert_eq!(runtime.camera_sub_target_npc_id(), None);
    let gm_commands = drain(&mut runtime);
    assert!(matches!(gm_commands[0], RuleRuntimeCommand::EnterMode(_)));
    assert_eq!(
        gm_commands[1],
        RuleRuntimeCommand::PlaySound(RuleRuntimeSound::OpenScreen)
    );
    assert_eq!(gm_commands.len(), 2, "GM chat has no NPC button audio");
}

#[test]
fn help_disables_pointer_controls_but_not_escape_while_popup_blocks_both() {
    let mut runtime = RuleRuntime::default();
    let mut model = RuleUiModel::default();
    runtime.open(&mut model, npc_request(9)).unwrap();
    let _ = drain(&mut runtime);

    runtime.sync_external_gates(&mut model, false, true);
    let help = runtime.mode_lease(&model).unwrap();
    assert!(help.blocks_lower_ui);
    assert!(help.blocks_gameplay_input);
    assert!(help.requires_pointer);
    assert!(!help.mouse_controls_enabled);
    assert!(help.escape_close_gate_enabled);
    assert!(!help.gameplay_hud_visible);
    assert!(!help.gameplay_input_enabled);
    assert_eq!(
        help.cursor_policy,
        RuleCursorPolicy::ForceUnlockedDuringAndAfterExit
    );
    assert_eq!(
        help.camera_policy,
        RuleCameraPolicy::PreserveInheritedUntilExitThenEnd
    );
    assert_eq!(
        help.transport_policy,
        RuleTransportPolicy::LocalEventBusOnlyNoPackets
    );

    runtime.sync_external_gates(&mut model, true, true);
    let popup = runtime.mode_lease(&model).unwrap();
    assert!(!popup.mouse_controls_enabled);
    assert!(!popup.escape_close_gate_enabled);
}

#[test]
fn escape_gate_is_correlated_rejectable_and_has_no_button_audio() {
    let mut runtime = RuleRuntime::default();
    let mut model = RuleUiModel::default();
    let mut ui = RuleUiOutbox::default();
    let mut audio = RuleUiAudioOutbox::default();
    runtime.open(&mut model, npc_request(9)).unwrap();
    let _ = drain(&mut runtime);

    assert!(request_rule_ui_escape_close(&mut model, &mut ui));
    assert!(
        runtime
            .flush_ui_outboxes(&model, &mut ui, &mut audio)
            .is_empty()
    );
    let gate_commands = drain(&mut runtime);
    let [
        RuleRuntimeCommand::RequestComputressExitGate {
            token,
            event_group: 2,
            event_function: 24,
        },
    ] = gate_commands.as_slice()
    else {
        panic!("Escape must request only the local 2/24 gate")
    };
    let token = *token;
    assert_eq!(token.get(), 1);
    assert_eq!(
        runtime.phase(),
        RuleRuntimePhase::AwaitingEscapeGate { token }
    );

    let stale = RuleEscapeGateToken::from_test_value(token.get() + 1);
    assert_eq!(
        runtime.resolve_escape_gate(&mut model, stale, true),
        Err(RuleRuntimeError::StaleEscapeGate {
            expected: token,
            actual: stale,
        })
    );
    assert!(model.visible);
    assert!(model.escape_close_pending);

    runtime
        .resolve_escape_gate(&mut model, token, false)
        .unwrap();
    assert_eq!(runtime.phase(), RuleRuntimePhase::Active);
    assert!(model.visible);
    assert!(!model.escape_close_pending);
    assert!(runtime.commands_are_empty());

    assert!(request_rule_ui_escape_close(&mut model, &mut ui));
    assert!(
        runtime
            .flush_ui_outboxes(&model, &mut ui, &mut audio)
            .is_empty()
    );
    let Some(RuleRuntimeCommand::RequestComputressExitGate { token, .. }) = runtime.pop_command()
    else {
        panic!("second Escape gate missing")
    };
    runtime
        .resolve_escape_gate(&mut model, token, true)
        .unwrap();
    assert!(!model.visible);
    assert_eq!(runtime.phase(), RuleRuntimePhase::Hidden);
    assert_eq!(runtime.session(), None);

    let commands = drain(&mut runtime);
    let RuleRuntimeCommand::ExitMode(exit) = commands[0] else {
        panic!("accepted gate must exit")
    };
    assert_eq!(exit.source, RuleUiDismissalSource::EscapeCloseGate);
    assert_eq!(
        commands[1],
        RuleRuntimeCommand::PlaySound(RuleRuntimeSound::CloseScreen)
    );
    assert_eq!(commands.len(), 2);
}

#[test]
fn close_button_restores_main_shell_but_never_restores_locked_cursor_or_sends_packet() {
    let mut runtime = RuleRuntime::default();
    let mut model = RuleUiModel::default();
    let mut ui = RuleUiOutbox::default();
    let mut audio = RuleUiAudioOutbox::default();
    runtime.open(&mut model, npc_request(9)).unwrap();
    let _ = drain(&mut runtime);

    assert!(activate_rule_ui_button(
        RuleUiButtonKind::Close,
        &mut model,
        &mut ui,
        &mut audio,
    ));
    assert!(
        runtime
            .flush_ui_outboxes(&model, &mut ui, &mut audio)
            .is_empty()
    );
    let commands = drain(&mut runtime);
    let RuleRuntimeCommand::ExitMode(exit) = commands[0] else {
        panic!("close must exit")
    };
    assert_eq!(exit.source, RuleUiDismissalSource::CloseButton);
    assert_eq!((exit.event_group, exit.event_function), (2, 1));
    assert_eq!((exit.game_mode, exit.linked_game_mode), (30, 5));
    assert!(!exit.cursor_locked);
    assert!(exit.show_gameplay_hud);
    assert!(exit.gameplay_input_enabled);
    assert!(!exit.force_nanocom_open);
    assert!(exit.end_camera_sub_target);
    assert!(exit.clear_npc_interaction_locally);
    assert!(!exit.send_npc_interaction_close_packet);
    assert_eq!(
        commands[1],
        RuleRuntimeCommand::PlaySound(RuleRuntimeSound::CloseScreen)
    );
    assert_eq!(
        commands[2],
        RuleRuntimeCommand::PlaySound(RuleRuntimeSound::RandomButtonSound)
    );
    assert_eq!(commands.len(), 3);
    assert_eq!(RuleRuntimeSound::RandomButtonSound.legacy_name(), None);
    assert_eq!(
        RuleRuntimeSound::RandomButtonSound.random_candidates(),
        Some(&rule_ui::RULE_UI_BUTTON_SOUND_PATHS)
    );
    assert_eq!(RuleRuntimeSound::RandomButtonSound.gain(), 0.7);
}

#[test]
fn reset_is_a_silent_state_teardown_and_clears_stale_queues() {
    let mut runtime = RuleRuntime::default();
    let mut model = RuleUiModel::default();
    let mut ui = RuleUiOutbox::default();
    let mut audio = RuleUiAudioOutbox::default();
    runtime.open(&mut model, npc_request(9)).unwrap();
    assert!(request_rule_ui_escape_close(&mut model, &mut ui));
    runtime.reset_session(&mut model, &mut ui, &mut audio);

    assert_eq!(runtime.phase(), RuleRuntimePhase::Hidden);
    assert_eq!(runtime.session(), None);
    assert!(!model.visible);
    assert!(!model.system_popup_active);
    assert!(!model.help_active);
    assert!(!model.escape_close_pending);
    assert!(ui.is_empty());
    assert!(audio.is_empty());
    assert!(runtime.commands_are_empty());
}

#[test]
fn synthetic_wrong_routes_fail_closed_without_ending_the_session() {
    let mut runtime = RuleRuntime::default();
    let mut model = RuleUiModel::default();
    let mut ui = RuleUiOutbox::default();
    let mut audio = RuleUiAudioOutbox::default();
    runtime.open(&mut model, npc_request(9)).unwrap();
    let _ = drain(&mut runtime);

    let error = runtime.accept_ui_action_for_test(
        &model,
        RuleUiAction::ExitMode {
            source: RuleUiDismissalSource::BackButton,
            event_group: 2,
            event_function: 99,
            cursor_locked: false,
        },
    );
    assert_eq!(
        error,
        Err(RuleRuntimeError::UnexpectedUiEventRoute {
            event_group: 2,
            event_function: 99,
        })
    );
    assert!(
        runtime
            .flush_ui_outboxes(&model, &mut ui, &mut audio)
            .is_empty()
    );
    assert_eq!(runtime.phase(), RuleRuntimePhase::Active);
    assert!(runtime.session().is_some());
    assert!(runtime.commands_are_empty());
}
