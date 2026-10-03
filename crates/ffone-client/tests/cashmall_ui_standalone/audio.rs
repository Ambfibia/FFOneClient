use super::*;

#[test]
fn close_gate_restores_cursor_stops_audio_and_preserves_legacy_state() {
    let (mut state, mut outbox) = visible_state(true);
    state
        .select_tab(Default::default(), CashmallTab0104::Etc, &mut outbox)
        .unwrap();
    outbox.clear();
    state.note_scroll_areas(true, false);
    state.apply_scroll_axis(Default::default(), -1.0).unwrap();
    assert_eq!(state.inventory_scroll_y(), 190.0);

    let accepted = CashmallCloseGate0104 {
        mode_accepts_escape: true,
        exit_arbitration_clear: true,
    };
    state
        .request_close(
            CashmallCloseSource0104::ConfigurableKey4,
            Default::default(),
            accepted,
            &mut outbox,
        )
        .unwrap();
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Hidden);
    assert!(state.cursor_locked());
    assert!(!state.ui_input_active());
    assert_eq!(state.tab(), CashmallTab0104::Etc);
    assert_eq!(state.inventory_scroll_y(), 190.0);
    assert_eq!(
        outbox.pop_front(),
        Some(CashmallLocalEffect0104::Close(CashmallCloseBoundary0104 {
            ui_input_event: [11, 0],
            ui_input_exit_value: 10,
            restore_cursor_locked: true,
            notify_game_mode_exit: [2, 1],
            stop_ui_mode_sound: true,
            request_asset_gc: true,
            loaded_textures_actually_cleared: false,
        }))
    );

    state.open_from(
        CashmallOpenSource0104::HiddenChatCommand,
        false,
        &mut outbox,
    );
    assert_eq!(state.tab(), CashmallTab0104::Etc);
    assert_eq!(state.inventory_scroll_y(), 190.0);
}
