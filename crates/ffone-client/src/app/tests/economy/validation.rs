use super::*;

#[test]
fn email_update_check_poll_matches_clean_nanocom_timer_and_openfusion_abi() {
    let mut poll = EmailUpdateCheckPoll0104::default();
    assert!(!poll.advance(59.9));
    assert!(poll.advance(0.2));
    assert!(!poll.advance(599.0));
    assert!(poll.advance(1.5));
    let request = email_update_check_request_0104().unwrap();
    assert_eq!(request.packet_type(), 0x1300_007B);
    assert_eq!(request.payload(), &[0]);
}

#[test]
fn bank_error_two_queues_exact_popup_and_waits_for_its_ack_owner() {
    let inventory =
        InventoryRuntime0104::from_pc_load(77, &ffone_protocol::PcLoadData0104::zeroed());
    let request = ffone_protocol::PcBankOpenRequest0104 {
        pc_id: 77,
        npc_id: 9_001,
    };
    let source = BankOpenIdentity0104::from(request);
    let mut production = BankProductionRuntime0104::default();
    production.begin_open(request, &inventory).unwrap();
    let event = production
        .apply_bank_reply(
            ffone_protocol::PcBankReply0104::OpenFailure(ffone_protocol::PcBankFailure0104 {
                error_code: 2,
            }),
            &inventory,
        )
        .unwrap();
    assert!(matches!(
        event,
        BankProductionEvent0104::OpenFailed {
            show_access_required_popup: true,
            close_immediately: false,
            ..
        }
    ));
    assert_eq!(production.opening(), Some(source));

    let mut owner = BankSystemMessageRuntime::default();
    let mut messages = SystemMessageUiModel::default();
    let request_id = owner.queue_access_required(source, &mut messages);
    let popup = messages.current().unwrap();
    assert_eq!(popup.request_id, request_id);
    assert_eq!(popup.text, BANK_ACCESS_REQUIRED_MESSAGE);
    assert_eq!(popup.button_type, SystemMessageButtonType::Ok);
    assert_eq!(
        owner.pending_access_required.get(&request_id),
        Some(&source)
    );
    assert!(matches!(
        production.close_locally(),
        Ok(BankProductionEvent0104::ClosedLocally { source: closed }) if closed == source
    ));
}

#[test]
fn email_npc_letters_do_not_require_a_selected_mentor() {
    let content = runtime_test_mission_content();
    let catalog = runtime_test_email_catalog_0104();
    let mut guide = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    for mentor in [0_i16, -1, 6] {
        let mut load = ffone_protocol::PcLoadData0104::zeroed();
        let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
        load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&mentor.to_le_bytes());
        guide.load_pc_state(&load);
        let messages = email_guide_messages_0104(&catalog, &content, &guide, [198], [255]).unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].mission_task_id, 198);
    }
}
