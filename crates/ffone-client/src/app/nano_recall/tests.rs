use super::*;
#[test]
fn recall_requires_current_instance_and_correlated_registration() {
    let mut state = NanoRecallState::default();
    assert!(state.restriction().is_some());
    state.instance = Some(7);
    assert!(state.restriction().is_some());
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_REGIST_RXCOM,
        flags: 0,
        checksum: 0,
        payload: RegistRxcomReply0104 {
            map_num: 7,
            x: 100,
            y: 200,
            z: 300,
        }
        .encode(),
    };
    assert!(state.apply(&frame).unwrap().is_none());
    state.pending_npc = Some(42);
    let mut truncated = frame.clone();
    truncated.payload.pop();
    assert!(state.apply(&truncated).is_err());
    assert_eq!(state.pending_npc, Some(42));
    assert!(state.apply(&frame).unwrap().is_some());
    assert!(state.restriction().is_none());
    assert!(state.registered_here(42));
    assert!(!state.registered_here(43));
    assert_eq!(state.point, Some((7, [100, 200, 300])));
    state.leave_instance();
    assert!(state.restriction().is_some());
    assert!(state.point.is_none());
    assert!(!state.registered_here(42));
}

#[test]
fn nano_recall_notices_have_identical_production_locale_keys_and_placeholders() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game/localization");
    let read = |locale: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(format!("{locale}.json"))).unwrap())
            .unwrap()
    };
    let en = read("en");
    let ru = read("ru");
    let entries = |document: &serde_json::Value| -> std::collections::BTreeMap<String, String> {
        document
            .as_object()
            .unwrap()
            .values()
            .filter_map(|value| value.as_object())
            .flat_map(|entries| entries.iter())
            .filter_map(|(key, value)| {
                value.as_str().map(|value| (key.clone(), value.to_owned()))
            })
            .collect()
    };
    let en = entries(&en);
    let ru = entries(&ru);
    assert_eq!(en.keys().collect::<Vec<_>>(), ru.keys().collect::<Vec<_>>());
    for suffix in ["title", "registered", "registration_failed", "no_nano"] {
        let key = format!("ui.nano.recall.{suffix}");
        for locale in [&en, &ru] {
            assert!(!locale[&key].is_empty());
            assert!(!locale[&key].contains(['{', '}']));
        }
    }
}

#[test]
fn nano_recall_instance_replacement_invalidates_point_but_refresh_does_not() {
    let mut state = NanoRecallState::default();
    let mut frame = DecodedFrame {
        packet_type: packet::P_FE2CL_INSTANCE_MAP_INFO,
        flags: 0,
        checksum: 0,
        payload: vec![0; RACE_INSTANCE_MAP_INFO_BASE_SIZE],
    };
    frame.payload[..4].copy_from_slice(&7i32.to_le_bytes());
    state.apply(&frame).unwrap();
    state.point = Some((7, [1, 2, 3]));
    state.apply(&frame).unwrap();
    assert!(state.restriction().is_none());
    frame.payload[..4].copy_from_slice(&8i32.to_le_bytes());
    state.apply(&frame).unwrap();
    assert!(state.restriction().is_some());
    assert!(state.point.is_none());
}
