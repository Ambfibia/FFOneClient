use super::*;

#[test]
fn one_second_sine_opening_and_input_gates_are_preserved() {
    let mut state = CashmallUiState0104::default();
    let mut outbox = CashmallUiOutbox0104::default();
    state.open_from(CashmallOpenSource0104::HiddenChatCommand, true, &mut outbox);
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Opening);
    assert_eq!(state.cursor_locked(), false);
    assert!(state.ui_input_active());
    assert_eq!(
        outbox.pop_front(),
        Some(CashmallLocalEffect0104::EnterMode {
            ui_input_event: [11, 0],
            ui_input_enter_value: 1,
            force_inventory_tab: 0,
            force_cursor_unlocked: true,
        })
    );
    assert_eq!(
        cashmall_mode_layout_0104(1_264, 681, 0.0, 0.0)
            .cashmall_panel
            .left,
        -498.0
    );
    state.tick(0.5);
    assert!(
        (cashmall_opening_eased_fraction_0104(0.5) - std::f32::consts::FRAC_1_SQRT_2).abs()
            < 0.000_01
    );
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Opening);
    assert!(!state.input_capabilities(Default::default()).tabs);
    state.tick(0.5);
    assert_eq!(state.phase(), CashmallLifecyclePhase0104::Visible);
    assert!(state.input_capabilities(Default::default()).tabs);
}

#[test]
fn selected_tab_click_is_discarded_and_other_tabs_reset_only_cashmall_scroll() {
    let (mut state, mut outbox) = visible_state(false);
    assert_eq!(
        state
            .select_tab(Default::default(), CashmallTab0104::New, &mut outbox)
            .unwrap(),
        CashmallTabActivation0104::SelectedButtonReturnDiscarded
    );
    assert!(outbox.is_empty());
    state.note_scroll_areas(true, false);
    state.apply_scroll_axis(Default::default(), -1.0).unwrap();
    assert_eq!(state.inventory_scroll_y(), 190.0);
    assert_eq!(
        state
            .select_tab(Default::default(), CashmallTab0104::Potion, &mut outbox)
            .unwrap(),
        CashmallTabActivation0104::Changed
    );
    assert_eq!(state.tab(), CashmallTab0104::Potion);
    assert_eq!(state.cashmall_scroll_y(), 0.0);
    assert_eq!(state.inventory_scroll_y(), 190.0);
    assert_eq!(
        outbox.pop_front(),
        Some(CashmallLocalEffect0104::PlayAudio(
            CashmallAudioCue0104::TabClick01
        ))
    );
}

#[test]
fn primary_row_click_delegates_to_vendor_popup_without_a_purchase_request() {
    let (state, mut outbox) = visible_state(false);
    let projection = one_row_projection(7);
    state
        .activate_row(
            Default::default(),
            &projection,
            0,
            CashmallPointerButton0104::Primary,
            &mut outbox,
        )
        .unwrap();
    assert_eq!(
        outbox.pop_front(),
        Some(CashmallLocalEffect0104::PlayAudio(
            CashmallAudioCue0104::ButtonSound
        ))
    );
    assert_eq!(
        outbox.pop_front(),
        Some(CashmallLocalEffect0104::VendorClickItem(
            CashmallVendorClickBoundary0104 {
                slot_type: 9,
                slot_id: 0,
                item: item(7, 77, 5),
                popup_action: 2,
                popup_rect: [-1, -1, 0, 0],
            }
        ))
    );
    assert!(outbox.is_empty());
}

#[test]
fn retained_right_click_event_is_local_and_slot12_branch_stays_unreachable() {
    let (state, mut outbox) = visible_state(false);
    let projection = one_row_projection(0);
    state
        .activate_row(
            Default::default(),
            &projection,
            0,
            CashmallPointerButton0104::Secondary,
            &mut outbox,
        )
        .unwrap();
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            CashmallLocalEffect0104::PlayAudio(CashmallAudioCue0104::ButtonSound),
            CashmallLocalEffect0104::RetainedRightClickEquipmentEvent {
                event: [2, 3, 1],
                item_id: 77,
                item_type: 0,
                option: 0,
            },
        ]
    );
    assert!(!CASHMALL_RECENT_BUY_SLOT12_BRANCH_REACHABLE);
}

#[test]
fn sticky_scroll_target_keeps_cashmall_scroll_dead_and_pc_stuff_clamped() {
    let (mut state, mut outbox) = visible_state(false);
    state.note_scroll_areas(false, true);
    assert_eq!(state.scroll_target(), CashmallScrollTarget0104::Cashmall);
    state.apply_scroll_axis(Default::default(), -1.0).unwrap();
    assert_eq!(state.cashmall_scroll_y(), 0.0);
    state.note_scroll_areas(false, false);
    assert_eq!(state.scroll_target(), CashmallScrollTarget0104::Cashmall);
    state
        .request_close(
            CashmallCloseSource0104::PcStuffCloseButton,
            Default::default(),
            CashmallCloseGate0104 {
                mode_accepts_escape: false,
                exit_arbitration_clear: true,
            },
            &mut outbox,
        )
        .unwrap();
    outbox.clear();
    state.open_from(
        CashmallOpenSource0104::HiddenChatCommand,
        false,
        &mut outbox,
    );
    state.tick(CASHMALL_OPEN_SECONDS);
    outbox.clear();
    assert_eq!(state.scroll_target(), CashmallScrollTarget0104::Cashmall);
    state.note_scroll_areas(true, true);
    assert_eq!(state.scroll_target(), CashmallScrollTarget0104::PcStuff);
    state.apply_scroll_axis(Default::default(), -1.0).unwrap();
    assert_eq!(state.inventory_scroll_y(), 190.0);
    assert!(outbox.is_empty());
}

#[test]
fn source_audit_keeps_every_text_key_first_and_dead_scrollbar_nodes_absent() {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cashmall_ui.rs"))
            .unwrap();
    assert!(!source.contains("CashmallUiElement0104::ScrollTrack"));
    assert!(!source.contains("CashmallUiElement0104::ScrollThumb"));
    assert!(!source.contains("CashmallStaticAssetRole0104::ScrollTrack"));
    let direct_text_spawns = source
        .match_indices("Text::new(")
        .filter(|(offset, _)| !source[..*offset].ends_with("Localized"))
        .count();
    assert!(source.matches("LocalizedText::new(").count() >= direct_text_spawns);
    assert!(!source.contains("audio/voice/en/"));
    assert!(!source.contains("audio/voice/ru/"));
}
