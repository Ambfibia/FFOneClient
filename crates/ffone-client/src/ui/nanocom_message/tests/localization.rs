use super::*;

#[test]
fn npc_constructor_keeps_semantic_text_and_localized_comm_out_voice() {
    let title = LocalizedText::new("content.npc.730.name", "Computress");
    let body = LocalizedText::new(
        "content.tabledata.guide.guide_string.19.sz_string",
        "Welcome back. Please check your email to learn about an important mission from me.",
    );
    let request = NanocomMessageRequest::type_9_localized(
        5,
        title.clone(),
        body.clone(),
        "ui/en/gameplay/guide/compu_icon.png",
        Some("Computress"),
    );
    assert_eq!(request.compact_title_localized(), title);
    assert_eq!(request.compact_body_localized(10.0), body);
    assert_eq!(
        request.voice_true_name.as_deref(),
        Some("Computress_CommOut03")
    );

    let mut model = NanocomMessageUiModel::default();
    model.enqueue(request);
    assert_eq!(model.pop_sound(), Some(NanocomMessageSound::SlideIn));
    assert_eq!(
        model.pop_sound(),
        Some(NanocomMessageSound::Voice(
            "Computress_CommOut03".to_owned()
        ))
    );
}

#[test]
fn nano_mission_constructor_keeps_type_10_geometry_localization_and_voice() {
    let body = LocalizedText::new(
        "content.tabledata.mission.mission_string.3376.str_name_string",
        "A new Nano mission is ready.",
    );
    let request = NanocomMessageRequest::nano_mission_localized(
        5,
        body.clone(),
        Some("icons/entities/nanos/nanoicon_buttercup.png".to_owned()),
        Some("Computress"),
    );
    assert_eq!(request.kind, NanocomMessageKind::Nano);
    assert_eq!(request.compact_frame_path, NANOCOM_NANO_FRAME_PATH);
    assert_eq!(
        request.compact_icon_path.as_deref(),
        Some("icons/entities/nanos/nanoicon_buttercup.png")
    );
    assert_eq!(request.compact_frame_rect(), NANOCOM_NANO_FRAME_RECT);
    assert_eq!(request.compact_icon_rect(), NANOCOM_NANO_ICON_RECT);
    assert_eq!(
        request.compact_title_localized().key,
        NANOCOM_NANO_MISSION_TITLE_LOCALIZATION_KEY
    );
    assert_eq!(request.compact_body_localized(10.0), body);
    assert_eq!(
        request.voice_true_name.as_deref(),
        Some("Computress_CommOut03")
    );

    let mut model = NanocomMessageUiModel::default();
    model.enqueue(request);
    assert!(model.compact_visible());
    assert_eq!(model.pop_sound(), Some(NanocomMessageSound::SlideIn));
    assert_eq!(
        model.pop_sound(),
        Some(NanocomMessageSound::Voice(
            "Computress_CommOut03".to_owned()
        ))
    );
    assert_eq!(
        model.pop_sound(),
        Some(NanocomMessageSound::NanoCreationComplete)
    );
    assert_eq!(model.pop_sound(), None);
}
