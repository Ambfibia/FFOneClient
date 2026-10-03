use super::*;

#[test]
fn player_folder_page_and_read_requests_remain_exactly_correlated() {
    let (mut harness, _) = open_harness(None, EmailItemFeaturePolicy0104::default(), false);
    let switched = harness
        .production
        .switch_folder(
            EmailFolder::Player,
            &mut harness.model,
            &mut harness.transport,
            &mut harness.actions,
            &mut harness.audio,
        )
        .unwrap()
        .unwrap();
    assert_eq!(switched.audio, vec![EmailUiAudioCue::TabClick]);
    let outbound = harness
        .production
        .dispatch_next_request(
            &harness.model,
            &mut harness.transport,
            &mut harness.network,
            &TestCatalog,
        )
        .unwrap()
        .request
        .unwrap();
    assert_eq!(outbound.packet_type(), EMAIL_REQ_PAGE_LIST_ID);

    let page_frame = page_success_frame(1, 44);
    let EmailFrameDisposition0104::Applied { frame, output } = harness.production.route_frame(
        page_frame.clone(),
        &mut harness.model,
        &mut harness.actions,
        &mut harness.transport,
        &mut harness.audio,
        &mut harness.network,
    ) else {
        panic!("page frame must apply")
    };
    assert_eq!(frame, page_frame);
    assert!(output.commit.is_none());
    assert_eq!(harness.model.player_messages[0].email_index, 44);
    assert!(matches!(
        harness.transport.0.front(),
        Some(EmailRequest::Read { email_index: 44 })
    ));

    harness
        .production
        .dispatch_next_request(
            &harness.model,
            &mut harness.transport,
            &mut harness.network,
            &TestCatalog,
        )
        .unwrap();
    let read_frame = read_success_frame(44, None);
    let EmailFrameDisposition0104::Applied { output, .. } = harness.production.route_frame(
        read_frame,
        &mut harness.model,
        &mut harness.actions,
        &mut harness.transport,
        &mut harness.audio,
        &mut harness.network,
    ) else {
        panic!("read frame must apply")
    };
    assert_eq!(harness.model.read_message.as_ref().unwrap().email_index, 44);
    assert_eq!(output.audio, vec![EmailUiAudioCue::EmailArrived]);
    assert!(harness.production.pending_request().is_none());
}
