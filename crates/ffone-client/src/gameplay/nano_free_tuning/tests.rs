use crate::nano_free_tuning_runtime::*;
use crate::network::NanoTunePending0104;
use ffone_net::NanoTuneGameplayFrame0104;
use ffone_protocol::{DecodedFrame, ItemBase0104, NanoTuneSuccess0104, WirePayload, packet};

#[test]
fn every_production_nano_tune_has_unique_wire_identity_and_localized_power() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = crate::assets::AssetLocator::open(root.clone()).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let read = |path: &str| -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(root.join(path)).unwrap()).unwrap()
    };
    let tables = locator.read_table_set().unwrap();
    let table = tables["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|table| table["value"].get("m_pNanoTable"))
        .unwrap();
    let tunes = table["m_pNanoTuneData"].as_array().unwrap();
    let mut wire_ids = std::collections::BTreeSet::new();
    for (index, tune) in tunes.iter().enumerate() {
        assert_eq!(tune["m_iTuneNumber"].as_u64(), Some(index as u64));
        assert!(
            wire_ids.insert(index),
            "duplicate tuning overwrites server lookup"
        );
    }
    let en = read("localization/en.json");
    let ru = read("localization/ru.json");
    let mut count = 0;
    for nano in content.gameplay_nanos() {
        let projected = project_nano_free_tuning_content(&content, nano.nano_id)
            .unwrap_or_else(|error| panic!("Nano {}: {error}", nano.nano_id));
        for power in projected.powers {
            count += 1;
            let tune = &tunes[power.tune_id as usize];
            assert_eq!(tune["m_iSkillID"].as_i64(), Some(i64::from(power.skill_id)));
            let string_id = tune["m_iTuneName"].as_u64().unwrap();
            for (field, source_field, copy) in [
                ("name", "str_name", power.name),
                ("type_label", "str_comment1", power.power_type),
                ("description", "str_comment", power.description),
            ] {
                let key = format!("content.nano_tune.{}.{field}", power.tune_id);
                let source_key = format!(
                    "content.tabledata.nano.nano_tune_string.{string_id}.{source_field}"
                );
                assert_eq!(en["entries"][&key].as_str(), Some(copy.as_str()), "{key}");
                for locale in [&en, &ru] {
                    assert!(locale["entries"][&key].is_string(), "{key}");
                    assert_eq!(
                        locale["entries"][&key], locale["entries"][&source_key],
                        "{key}"
                    );
                }
            }
        }
    }
    assert!(count >= 183);
}

#[test]
fn bank_scan_and_authoritative_mutations_follow_clean_index_contract() {
    let mut bank = NanoFreeTuningBank0104::default();
    bank.bank[9] = Nano0104 {
        id: 9,
        skill_id: 0,
        stamina: 150,
    };
    bank.bank[3] = Nano0104 {
        id: 3,
        skill_id: 0,
        stamina: 120,
    };
    assert_eq!(bank.first_untuned_index(), Some(3));
    bank.apply_tune_success(3, 144).unwrap();
    assert_eq!(bank.first_untuned_index(), Some(9));
    bank.apply_create_success(PcNanoCreateSuccess0104 {
        fusion_matter: 0,
        quest_item_slot: 0,
        quest_item: ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        nano: Nano0104 {
            id: 2,
            skill_id: 0,
            stamina: 150,
        },
        player_level: 2,
    })
    .unwrap();
    assert_eq!(bank.first_untuned_index(), Some(2));
}

#[test]
fn bank_mutations_fail_closed_on_bad_identity() {
    let mut bank = NanoFreeTuningBank0104::default();
    let before = bank.clone();
    assert!(bank.apply_tune_success(37, 1).is_err());
    assert_eq!(bank, before);
    assert!(bank.apply_tune_success(1, 1).is_err());
    assert_eq!(bank, before);
}

#[test]
fn nano_book_extends_login_prefix_and_restores_nano_50() {
    use ffone_protocol::wire_0104::{Nano0104 as WireNano, NanoBookSubsetReply0104};
    let mut bank = NanoFreeTuningBank0104::default();
    bank.bank[1] = Nano0104 {
        id: 1,
        skill_id: 1,
        stamina: 150,
    };
    let mut page = NanoBookSubsetReply0104 {
        pcuid: 77,
        book_size: 73,
        element_offset: 0,
        element: std::array::from_fn(|_| WireNano {
            id: 0,
            skill_id: 0,
            stamina: 0,
        }),
    };
    bank.apply_book_subset(&page, 77).unwrap();
    assert_eq!(bank.entries().len(), 73);
    assert_eq!(bank.entries()[1].id, 1);
    page.element_offset = 50;
    page.element[0] = WireNano {
        id: 50,
        skill_id: 0,
        stamina: 150,
    };
    bank.apply_book_subset(&page, 77).unwrap();
    assert_eq!(bank.first_untuned_index(), Some(50));
    bank.apply_tune_success(50, 234).unwrap();
    assert_eq!(bank.entries()[50].skill_id, 234);
    assert_eq!(bank.first_untuned_index(), None);
    let before = bank.clone();
    page.element[1].id = 12;
    assert!(bank.apply_book_subset(&page, 77).is_err());
    assert_eq!(bank, before);
    page.element[1].id = 0;
    assert!(bank.apply_book_subset(&page, 78).is_err());
    page.book_size = i32::MAX;
    assert!(bank.apply_book_subset(&page, 77).is_err());
    assert_eq!(bank, before);
}

#[test]
fn bootstrap_pages_are_applied_before_nano_50_grants_and_resume_scan() {
    use ffone_net::{LoadingComplete, WorldBootstrapPacket};
    use ffone_protocol::{
        PcLoadingCompleteSuccess,
        wire_0104::{Nano0104 as WireNano, NanoBookSubsetReply0104},
    };
    let load = PcLoadData0104::decode(&vec![0; PcLoadData0104::SIZE]).unwrap();
    let mut page = NanoBookSubsetReply0104 {
        pcuid: 77,
        book_size: 67,
        element_offset: 0,
        element: std::array::from_fn(|_| WireNano {
            id: 0,
            skill_id: 0,
            stamina: 0,
        }),
    };
    let frame = |page: &NanoBookSubsetReply0104| DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NANO_BOOK_SUBSET,
        flags: 0,
        checksum: 0,
        payload: page.encode(),
    };
    let mut bootstrap = LoadingComplete {
        response: PcLoadingCompleteSuccess { pc_id: 77 },
        prelude: vec![frame(&page)],
    }
    .into_world_bootstrap();
    let create = PcNanoCreateSuccess0104 {
        fusion_matter: 0,
        quest_item_slot: -1,
        quest_item: ItemBase0104 {
            item_type: 0,
            item_id: 0,
            option: 0,
            time_limit: 0,
        },
        nano: Nano0104 {
            id: 50,
            skill_id: 0,
            stamina: 150,
        },
        player_level: 1,
    };
    let mut bank = NanoFreeTuningBank0104::default();
    bank.seed(&load);
    assert!(
        bank.apply_create_success(create).is_err(),
        "old login-only path reproduces the rejection"
    );
    bank.seed_with_bootstrap(&load, 77, &bootstrap).unwrap();
    assert_eq!(bank.entries().len(), 67);
    bank.apply_create_success(create).unwrap();
    assert_eq!(bank.first_untuned_index(), Some(50));

    page.element_offset = 50;
    page.element[0] = WireNano {
        id: 50,
        skill_id: 0,
        stamina: 150,
    };
    bootstrap
        .packets
        .push(WorldBootstrapPacket::Passthrough(frame(&page)));
    bank.seed_with_bootstrap(&load, 77, &bootstrap).unwrap();
    assert_eq!(
        bank.first_untuned_index(),
        Some(50),
        "relogin resumes untuned Nano 50"
    );
    page.element[0].skill_id = 234;
    bootstrap.packets[1] = WorldBootstrapPacket::Passthrough(frame(&page));
    bank.seed_with_bootstrap(&load, 77, &bootstrap).unwrap();
    assert_eq!(bank.entries()[50].skill_id, 234);
    assert_eq!(bank.first_untuned_index(), None);

    let before = bank.clone();
    page.pcuid = 78;
    bootstrap
        .packets
        .push(WorldBootstrapPacket::Passthrough(frame(&page)));
    assert!(bank.seed_with_bootstrap(&load, 77, &bootstrap).is_err());
    assert_eq!(bank, before, "invalid later page must not partially commit");
    bootstrap
        .packets
        .last_mut()
        .unwrap()
        .clone_from(&WorldBootstrapPacket::Passthrough(DecodedFrame {
            payload: vec![0],
            ..frame(&page)
        }));
    assert!(bank.seed_with_bootstrap(&load, 77, &bootstrap).is_err());
    assert_eq!(bank, before);
    bootstrap.packets.clear();
    bank.seed_with_bootstrap(&load, 77, &bootstrap).unwrap();
    assert_eq!(
        bank.entries().len(),
        PcLoadData0104::NANO_BANK_COUNT,
        "legacy servers retain the fixed prefix"
    );
}

#[test]
fn correlated_success_projection_retains_every_authoritative_field() {
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 42,
        option: 3,
        time_limit: 99,
    };
    let success = NanoTuneSuccess0104 {
        nano_id: 2,
        skill_id: 144,
        fusion_matter: 12_345,
        item_slots: [0, 1, 2, 3, 4, 5, 6, 7, 8, -1],
        items: [item; NANO_TUNE_ITEM_SLOT_COUNT_0104],
    };
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_NANO_TUNE_SUCC,
        flags: 0,
        checksum: 0,
        payload: success.encode(),
    };
    let correlated = NanoTunePending0104 {
        request_token: 19,
        player_id: 77,
        nano_id: 2,
        skill_id: 144,
    }
    .correlate(NanoTuneGameplayFrame0104::decode(frame))
    .unwrap()
    .unwrap();
    let envelope = project_correlated_nano_tune_reply(correlated);
    assert_eq!(envelope.request_token, 19);
    assert_eq!(envelope.payload_size, NanoTuneSuccess0104::SIZE);
    let NanoFreeTuningReplyBody::Success(projected) = envelope.body else {
        panic!("expected success");
    };
    assert_eq!(projected.nano_id, 2);
    assert_eq!(projected.skill_id, 144);
    assert_eq!(projected.fusion_matter, 12_345);
    assert_eq!(projected.item_slots[9], -1);
    assert_eq!(projected.items[0].item_id, 42);
    assert_eq!(projected.items[0].time_limit, 99);
}
