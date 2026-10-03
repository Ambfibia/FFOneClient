use super::*;

#[test]
fn mission_request_codecs_match_registered_0104_abi() {
    let start = PendingMissionUiRequest::TaskStart {
        task_id: 2248,
        npc_id: 1005,
    }
    .encode_registered()
    .unwrap();
    assert_eq!(start.packet_type(), 0x1300_000b);
    assert_eq!(start.payload().len(), 12);
    assert_eq!(&start.payload()[0..4], &2248i32.to_le_bytes());

    let end = PendingMissionUiRequest::QuestEnd {
        task_id: 2248,
        npc_id: 1005,
        box1_choice: 1,
        box2_choice: 2,
    }
    .encode_registered()
    .unwrap();
    assert_eq!(end.packet_type(), 0x1300_000c);
    assert_eq!(end.payload().len(), 16);
    assert_eq!(&end.payload()[8..12], &[1, 2, 0, 0]);
    assert!(matches!(
        PendingMissionUiRequest::QuestEnd {
            task_id: 1,
            npc_id: 2,
            box1_choice: 128,
            box2_choice: 0,
        }
        .encode_registered(),
        Err(MissionRequestCodecError0104::ChoiceOutOfRange { .. })
    ));

    let stop = PendingMissionUiRequest::TaskStop { task_id: 2248 }
        .encode_registered()
        .unwrap();
    assert_eq!(stop.packet_type(), 0x1300_0012);
    assert_eq!(stop.payload(), &2248i32.to_le_bytes());
}

#[test]
fn escort_request_codecs_preserve_live_npc_identity_and_reward_choices() {
    use ffone_protocol::{PcTaskEndRequest0104, PcTaskStartRequest0104, WirePayload};
    let start = PendingMissionUiRequest::TaskStart {
        task_id: 576,
        npc_id: 73,
    }
    .encode_registered_with_escort(42)
    .unwrap();
    assert_eq!(
        PcTaskStartRequest0104::decode(start.payload())
            .unwrap()
            .escort_npc_id,
        42
    );
    let end = PendingMissionUiRequest::QuestEnd {
        task_id: 576,
        npc_id: 74,
        box1_choice: 1,
        box2_choice: 2,
    }
    .encode_registered_with_escort(42)
    .unwrap();
    let decoded = PcTaskEndRequest0104::decode(end.payload()).unwrap();
    assert_eq!(decoded.escort_npc_id, 42);
    assert_eq!(decoded.npc_id, 74);
    assert_eq!(&end.payload()[8..12], &[1, 2, 0, 0]);
}

#[test]
fn warp_and_npc_close_emit_original_tutorial_event_semantics() {
    let mut model = MissionUiModel::default();
    let mut outbox = GameplayUiOutbox::default();
    model.show_npc_interaction(NpcInteractionUi {
        npc_id: 88,
        name: "Tech Square Attendant".to_owned(),
        warp: Some(WarpUiEntry {
            npc_id: 88,
            npc_type: 2694,
            warp_id: 9,
            required_task_id: Some(2251),
            target: TutorialWarpTarget {
                map_id: 0,
                x: 59_573,
                y: 74_545,
                z: -9_066,
            },
            label: "Tech Square".to_owned(),
        }),
        ..default()
    });
    assert!(model.warp(&mut outbox));
    assert!(!model.warp(&mut outbox));
    assert!(!model.close_npc_interaction(&mut outbox));
    assert!(!model.confirm_warp(
        88,
        2694,
        10,
        Some(2251),
        TutorialWarpTarget {
            map_id: 0,
            x: 59_573,
            y: 74_545,
            z: -9_066,
        },
    ));
    assert!(model.confirm_warp(
        88,
        2694,
        9,
        Some(2251),
        TutorialWarpTarget {
            map_id: 0,
            x: 59_573,
            y: 74_545,
            z: -9_066,
        },
    ));

    assert!(
        !model.close_npc_interaction(&mut outbox),
        "successful warp must not also request farewell"
    );
    model.show_npc_interaction(npc_with_available());
    assert!(model.close_npc_interaction(&mut outbox));
    assert_eq!(
        outbox.drain().collect::<Vec<_>>(),
        vec![
            GameplayUiAction::NpcWarp {
                npc_id: 88,
                npc_type: 2694,
                warp_id: 9,
                required_task_id: Some(2251),
                target: TutorialWarpTarget {
                    map_id: 0,
                    x: 59_573,
                    y: 74_545,
                    z: -9_066,
                },
            },
            GameplayUiAction::NpcIconClose { npc_id: 100 },
        ]
    );
}
