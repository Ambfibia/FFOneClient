#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

#[path = "../src/app/gameplay_ui_actions/gm_commands.rs"]
mod gm_commands;
#[path = "../src/app/gameplay_ui_actions/gm_extended.rs"]
mod gm_extended;
#[path = "../src/app/gameplay_ui_actions/gm_state.rs"]
#[allow(dead_code)]
mod gm_state;

#[test]
fn item_queue_waits_for_matching_reply_and_retains_lock_after_timeout() {
    let mut state = gm_state::GmRuntime::default();
    state
        .items
        .extend(["/item 1 101 1".to_owned(), "/item 1 102 1".to_owned()]);
    assert_eq!(state.next_item().as_deref(), Some("/item 1 101 1"));
    let request = PcGiveItemRequest0104::decode(
        gm_commands::request("/item 1 101 1", 0, Some(81), |_, _| Some(0))
            .unwrap()
            .payload(),
    )
    .unwrap();
    state.start_item(request.clone());
    assert_eq!(state.next_item(), None);
    state.stalled = true;
    assert_eq!(state.next_item(), None);
    let frame = |id, payload| ffone_protocol::DecodedFrame {
        packet_type: id,
        payload,
        flags: 0,
        checksum: 0,
    };
    state.accept_item_reply(&frame(0x31000062, vec![0; 3]));
    assert!(state.pending.is_some());
    let mut reply = PcGiveItemSuccess0104 {
        e_il: 1,
        slot_num: 1,
        item: request.item.clone(),
    };
    reply.item.id = 102;
    state.accept_item_reply(&frame(0x31000061, reply.encode()));
    assert!(state.pending.is_some());
    reply.item.id = 101;
    state.accept_item_reply(&frame(0x31000061, reply.encode()));
    assert!(state.pending.is_none());
    assert!(!state.stalled);
    assert_eq!(state.next_item().as_deref(), Some("/item 1 102 1"));
    state.start_item(request);
    state.accept_item_reply(&frame(0x31000062, 1i32.to_le_bytes().to_vec()));
    assert!(state.pending.is_none());
    assert_eq!(state.notice.unwrap().0, "ui.chat.command.gm.item_rejected");
}

use ffone_protocol::{WirePayload, packet, wire_0104::*};
use gm_commands::{Error, request};

#[test]
fn values_have_exact_server_types_and_aliases() {
    for (name, kind) in [
        ("/taro", 5),
        ("/taros", 5),
        ("/fm", 4),
        ("/fusionmatter", 4),
        ("/health", 1),
        ("/batteryW", 2),
        ("/batteryN", 3),
        ("/jump", 7),
    ] {
        let r = request(&format!("{name} 1234"), 50, Some(81), |_, _| None).unwrap();
        assert_eq!(r.packet_type(), packet::P_CL2FE_GM_REQ_PC_SET_VALUE);
        let value = ffone_protocol::GmSetValueRequest0104::decode(r.payload()).unwrap();
        assert_eq!(
            (value.pc_id, value.value_type, value.value),
            (81, kind, 1234)
        );
    }
}

#[test]
fn warp_uses_tile_centres_and_goto_uses_world_units() {
    for (text, expected) in [
        ("/warp 1 -2", [76800, -76800, 10000]),
        ("/goto 1 -2 3", [100, -200, 300]),
        ("/goto 1 2", [100, 200, 10000]),
    ] {
        let r = request(text, 50, Some(81), |_, _| None).unwrap();
        assert_eq!(r.packet_type(), packet::P_CL2FE_REQ_PC_GOTO);
        let p = PcGotoRequest0104::decode(r.payload()).unwrap();
        assert_eq!([p.to_x, p.to_y, p.to_z], expected);
    }
}

#[test]
fn item_preserves_slot_type_count_and_expiration() {
    for name in ["/item", "/itemN"] {
        let r = request(&format!("{name} 10 123 2 60"), 50, Some(81), |quest, _| {
            assert!(!quest);
            Some(17)
        })
        .unwrap();
        assert_eq!(r.packet_type(), packet::P_CL2FE_REQ_PC_GIVE_ITEM);
        let p = PcGiveItemRequest0104::decode(r.payload()).unwrap();
        assert_eq!(
            (
                p.e_il,
                p.slot_num,
                p.item.type_,
                p.item.id,
                p.item.opt,
                p.time_left
            ),
            (1, 17, 10, 123, 2, 60)
        );
    }
}

#[test]
fn quest_item_uses_quest_inventory_and_passes_identity_to_slot_selection() {
    let r = request("/itemQ 123 2", 50, Some(81), |quest, id| {
        assert!(quest);
        assert_eq!(id, 123);
        Some(3)
    })
    .unwrap();
    let p = PcGiveItemRequest0104::decode(r.payload()).unwrap();
    assert_eq!(
        (p.e_il, p.slot_num, p.item.type_, p.item.id, p.item.opt),
        (2, 3, 8, 123, 2)
    );
}

#[test]
fn invalid_or_unauthorized_commands_never_produce_packets() {
    assert_eq!(
        request("/fm 1", 51, Some(81), |_, _| None),
        Err(Error::Denied)
    );
    assert_eq!(
        request("/fm 1", 50, None, |_, _| None),
        Err(Error::MissingPlayer)
    );
    assert_eq!(
        request("/item 1 1 1", 50, Some(81), |_, _| None),
        Err(Error::Full)
    );
    for text in [
        "/fm",
        "/fm 1 2",
        "/fm x",
        "/warp 2147483647 0",
        "/goto 1 2 2147483647",
        "/item 11 1 1",
        "/item 1 32768 1",
        "/item 1 1 0",
    ] {
        assert_eq!(
            request(text, 50, Some(81), |_, _| Some(0)),
            Err(Error::Usage),
            "{text}"
        );
    }
}

#[test]
fn production_gm_feedback_is_localized() {
    let en: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/game/localization/en.json")).unwrap();
    let ru: serde_json::Value =
        serde_json::from_str(include_str!("../../../assets/game/localization/ru.json")).unwrap();
    let en = en["entries"].as_object().unwrap();
    let ru = ru["entries"].as_object().unwrap();
    assert_eq!(en.keys().collect::<Vec<_>>(), ru.keys().collect::<Vec<_>>());
    let location_key = "ui.chat.command.gm.view_location";
    assert_eq!(
        en[location_key],
        "map={map} XYZ={x},{y},{z} iAngle={angle}° (player)"
    );
    assert_eq!(
        ru[location_key],
        "Карта={map} XYZ={x},{y},{z} iAngle={angle}° (персонаж)"
    );
    for key in en
        .keys()
        .filter(|key| key.starts_with("ui.chat.command.gm."))
    {
        let placeholders = |s: &str| {
            s.split('{')
                .skip(1)
                .map(|part| part.split('}').next().unwrap().to_owned())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            placeholders(en[key].as_str().unwrap()),
            placeholders(ru[key].as_str().unwrap())
        );
    }
}

#[test]
fn all_extended_commands_have_valid_examples_and_access_guards() {
    let examples = [
        "/rule 0",
        "/motd 1 -Hello world",
        "/announce area 1 5 -Hello world",
        "/bcast global 1 5 -Hi",
        "/nano_equip 1 0",
        "/nano_unequip 0",
        "/nano_active -1",
        "/warptopc 2",
        "/nanoArr 36",
        "/summon 1 2",
        "/groupsummon 1",
        "/summonshiny 1",
        "/unsummon",
        "/nanoskill 1 2",
        "/mission 1",
        "/task 1",
        "/unstick_n Test Player",
        "/unstick_i 2",
        "/unstick_ui 9223372036854775807",
        "/unstick",
        "/locate_i 2",
        "/locate_ui 9223372036854775807",
        "/locate_n Test Player",
        "/teleport2me_n Test Player",
        "/teleport2me_i 2",
        "/teleport2me_ui 3",
        "/teleportXYZ_i 2 -1 2 3",
        "/teleportXYZ_ui 3 -1 2 3",
        "/teleportXYZ_n -1 2 3 Test Player",
        "/teleportMapXYZ_i 2 1 -1 2 3",
        "/teleportMapXYZ_n 1 -1 2 3 Test Player",
        "/teleportMapXYZ_ui 3 1 -1 2 3",
        "/teleport_i_i 2 3",
        "/teleport_ui_ui 2 3",
        "/teleport_i_n 2 Other Player",
        "/teleport_n_n Test Player;Other Player",
        "/kick_i 2",
        "/kick_ui 3",
        "/kick_n Test Player",
        "/invisible",
        "/invulnerable",
        "/gmmarker",
        "/equipitem 0",
        "/viewloc",
        "/viweloc",
        "/viewnetinfo",
        "/mute_i_on 2",
        "/mute_i_off 2",
        "/mute_ui_on 3",
        "/mute_ui_off 3",
        "/mute_n_on Test Player",
        "/mute_n_off Test Player",
        "/hideui",
        "/viewcol",
        "/qinven",
        "/chnum",
        "/chinfo",
        "/chwarp 1",
        "/shwarp 1",
        "/viewid",
        "/Store",
        "/rateT",
        "/rateF 4 200",
        "/tasklog",
        "/shardwarp 1",
    ];
    let names = examples
        .iter()
        .map(|s| s.split_whitespace().next().unwrap())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(names, gm_extended::NAMES.iter().copied().collect());
    for example in examples {
        let result = gm_extended::parse(example, 0, Some(81), [1000, 2000, 3000], Some(-2));
        assert!(result.is_ok(), "{example}: {result:?}");
        assert!(
            matches!(
                gm_extended::parse(example, 100, Some(81), [0; 3], Some(-2)),
                Err(gm_extended::Error::Denied(_))
            ),
            "{example}"
        );
        assert_eq!(
            gm_extended::parse(example, 0, None, [0; 3], None),
            Err(gm_extended::Error::PlayerMissing)
        );
    }
}

#[test]
fn extended_target_coordinates_and_announcement_are_exact() {
    let packet = |s| match gm_extended::parse(s, 0, Some(81), [1000, 2000, 3000], Some(-2)).unwrap()
    {
        gm_extended::Action::Packet(p) => p,
        _ => panic!(),
    };
    let p = GmTargetPcTeleportRequest0104::decode(
        packet("/teleportMapXYZ_ui 9223372036854775807 9 -1 2 3").payload(),
    )
    .unwrap();
    assert_eq!(
        (
            p.teleport_type,
            p.target_pc_search_by,
            p.target_pc_uid,
            p.to_map,
            p.to_x,
            p.to_y,
            p.to_z
        ),
        (1, 2, i64::MAX, 9, -100, 200, 300)
    );
    let p = GmTargetPcTeleportRequest0104::decode(
        packet("/teleport_i_n 2 Other Player Name").payload(),
    )
    .unwrap();
    assert_eq!(p.goal_pc_last_name.to_string_lossy(), "Player Name");
    assert_eq!(p.target_pc_id, 2);
    let p =
        GmPcAnnounceRequest0104::decode(packet("/announce global 1 5 -Hello - world").payload())
            .unwrap();
    assert_eq!(p.announce_msg.to_string_lossy(), "Hello - world");
    assert_eq!((p.area_type, p.during_time), (3, 5));
    assert!(
        gm_extended::parse("/teleportXYZ_i 2 2147483647 0 0", 0, Some(81), [0; 3], None).is_err()
    );
    assert!(gm_extended::parse("/nano_equip 1 3", 0, Some(81), [0; 3], None).is_err());
    assert!(gm_extended::parse("/summon 1 101", 0, Some(81), [0; 3], None).is_err());
    assert!(gm_extended::parse("/locate_i -1", 0, Some(81), [0; 3], None).is_err());
    assert!(gm_extended::parse("/unsummon", 0, Some(81), [0; 3], None).is_err());
}
