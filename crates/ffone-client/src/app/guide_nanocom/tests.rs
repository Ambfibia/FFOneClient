use super::*;

fn content() -> TutorialMissionContent {
    TutorialMissionContent::open(
        &AssetLocator::open(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
        )
        .unwrap(),
    )
    .unwrap()
}

fn guide(raw_mentor: i16) -> GuideRuntime {
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
    load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&raw_mentor.to_le_bytes());
    let mut guide = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    guide.load_pc_state(&load);
    guide
}

#[test]
fn every_guide_level_up_uses_production_localized_copy_and_voice_owner() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content = TutorialMissionContent::open(&AssetLocator::open(&root).unwrap()).unwrap();
    for locale in ["en", "ru"] {
        let (localization, language) = Localization::open(&root, locale).unwrap();
        for raw_mentor in 1..=5 {
            let definition = content.gameplay_guide_nanocom(raw_mentor).unwrap();
            let mut messages = NanocomMessageUiModel::default();
            assert!(enqueue_level_up(
                &content,
                &guide(raw_mentor),
                4,
                5,
                &mut messages
            ));
            let request = &messages.active().unwrap().request;
            let body = request.compact_body_localized(10.0);
            assert_eq!(
                body.key,
                format!(
                    "content.tabledata.guide.guide_string.{}.sz_string",
                    definition.level_up_string_id
                )
            );
            assert_eq!(body.fallback, definition.level_up_text);
            let text = localization.text(&language, &body);
            assert!(!text.is_empty());
            if locale == "ru" {
                assert_ne!(
                    text, definition.level_up_text,
                    "Guide {raw_mentor} needs RU copy"
                );
            }
            let npc = content.gameplay_npc(definition.npc_type).unwrap();
            assert_eq!(
                request.compact_title_localized().key,
                format!("content.npc.{}.name", npc.npc_type)
            );
            assert!(
                request
                    .voice_true_name
                    .as_ref()
                    .unwrap()
                    .starts_with(&format!("{}_CommOut0", npc.move_voice_owner))
            );
        }
    }
}

#[test]
fn level_up_ignores_duplicate_decreasing_and_unowned_progression() {
    let content = content();
    let mut messages = NanocomMessageUiModel::default();
    let guide = guide(5);
    assert!(!enqueue_level_up(&content, &guide, 5, 5, &mut messages));
    assert!(!enqueue_level_up(&content, &guide, 5, 4, &mut messages));
    assert!(!enqueue_level_up(
        &content,
        &GuideRuntime::new(GuideServerProfile::OpenFusion0104),
        4,
        5,
        &mut messages
    ));
    assert!(messages.active().is_none());
}

#[test]
fn server_level_change_requires_a_complete_packet_before_notifying() {
    let content = content();
    let guide = guide(5);
    let mut messages = NanocomMessageUiModel::default();
    let mut runtime = RuntimeStatus::default();
    runtime.player_id = Some(77);
    runtime.player_level = 4;
    let mut payload = Vec::from(5_i32.to_le_bytes());
    payload.extend_from_slice(&750_i32.to_le_bytes());
    for size in [4, 7, 8, 8] {
        let previous_level = runtime.player_level;
        apply_runtime_frame(
            &DecodedFrame {
                packet_type: packet::P_FE2CL_REP_PC_CHANGE_LEVEL_SUCC,
                flags: 0,
                checksum: 0,
                payload: payload[..size].to_vec(),
            },
            &mut runtime,
        );
        let should_notify = size == 8 && previous_level == 4;
        assert_eq!(
            enqueue_level_up(
                &content,
                &guide,
                previous_level,
                runtime.player_level,
                &mut messages
            ),
            should_notify
        );
    }
    assert_eq!(messages.len(), 1);
    assert_eq!(runtime.player_level, 5);
    assert_eq!(runtime.fusion_matter, 750);
}
