use super::*;

#[test]
fn compact_fixture_builds_ui_entries_and_source_owned_warps() {
    let fixture = Fixture::new(compact_document());
    let content = TutorialMissionContent::from_project_assets(&fixture.assets).unwrap();

    assert_eq!(content.missions().len(), 7);
    assert_eq!(content.npcs().len(), 7);
    assert_eq!(content.rewards().len(), 1);
    assert_eq!(content.scene_events().len(), 1);
    assert_eq!(content.warps().len(), 3);
    assert_eq!(
        content.gameplay_warp(GUIDE_FIRST_WARP_ID),
        Some(&GameplayWarpDefinition {
            row_index: 3,
            warp_id: GUIDE_FIRST_WARP_ID,
            npc_type: GUIDE_FIRST_WARP_NPC_TYPE,
            target: GUIDE_FIRST_WARP_TARGET,
            warp_group_type: 0,
            limit_level: 0,
            limit_task_id: 0,
            limit_item_id: 0,
            limit_item_type: 0,
            limit_use_item_id: 0,
            limit_use_item_type: 0,
            mission_id: 0,
            is_instance: 0,
            cost: 0,
        })
    );
    assert_eq!(content.gameplay_warp(i32::MAX), None);
    assert_eq!(
        content.gameplay_nano(1),
        Some(&GameplayNanoUiDefinition {
            nano_id: 1,
            sort_number: 5,
            name: "Buttercup".to_owned(),
            attribute: "Blastons".to_owned(),
            icon_number: 20,
            icon_path: None,
            ready_icon_path: None,
            style: 1,
            max_stamina: 150,
        })
    );
    assert_eq!(
        content.gameplay_skill(1),
        Some(&GameplaySkillUiDefinition {
            skill_id: 1,
            skill_type: 1,
            values_a: [0; 4],
            icon_number: 11,
            active: true,
            effect_type: 1,
            target_effect: 0,
            target: 3,
            target_type: 1,
            range: 1000,
            area: 0,
            angle: 90,
            target_number: 2,
            cooldown: 80,
            cool_type: 22,
        })
    );
    let nano = content.journal_nano(1).unwrap();
    assert_eq!(
        nano,
        &TutorialNanoJournalUi {
            nano_id: 1,
            icon_number: 20,
            name: "Buttercup".to_owned(),
            attribute: "Blastons".to_owned(),
            description: "REQUIRED NANO: Create to reach LVL 1.".to_owned(),
            skills: [
                TutorialNanoJournalSkillUi {
                    tune_id: 1,
                    skill_id: 1,
                    icon_number: 11,
                    name: "MISS FIRE".to_owned(),
                    type_label: "STUN - CONE".to_owned(),
                    required_item_id: 0,
                    required_item_count: 0,
                    description: "Buttercup’s alter-ego Mange uses fire to stun enemies in the target area.".to_owned(),
                },
                TutorialNanoJournalSkillUi {
                    tune_id: 2,
                    skill_id: 2,
                    icon_number: 5,
                    name: "RALLYING CRY".to_owned(),
                    type_label: "HEALTH - GROUP".to_owned(),
                    required_item_id: 0,
                    required_item_count: 0,
                    description: "Buttercup’s warcry pumps up your group, healing their injuries.".to_owned(),
                },
                TutorialNanoJournalSkillUi {
                    tune_id: 3,
                    skill_id: 3,
                    icon_number: 27,
                    name: "BUTTERCUP BURST".to_owned(),
                    type_label: "SCAVENGE".to_owned(),
                    required_item_id: 0,
                    required_item_count: 0,
                    description: "Buttercup intimidates friend and foe alike to get you even more Fusion Matter!".to_owned(),
                },
            ],
        }
    );
    assert_eq!(
        content.scene_text(2, 1).unwrap(),
        "COMPUTRESS: Fixture line one."
    );
    assert_eq!(
        content
            .scene_text_definition(2, 2)
            .unwrap()
            .provenance
            .element_index,
        1
    );
    let entry = content.mission_entry(2250, 42, "Pokey Oaks North").unwrap();
    assert_eq!(entry.task_id, 2250);
    assert_eq!(entry.npc_id, Some(42));
    assert_eq!(entry.title, "A Fusion Matter");
    assert_eq!(entry.npc_name, "Buttercup");
    assert!(entry.is_first_mission_task);
    assert_eq!(entry.nano.as_ref(), Some(nano));
    assert_eq!(
        entry.active_description,
        "Talk to Buttercup.\n\nCome speak to me near the infected zone. Something's happened to Dexter, and we need to find him!"
    );
    assert_eq!(
        entry.mission_summary,
        "Rescue Dexter from the infected zone."
    );
    assert_eq!(
        entry.mission_complete_summary,
        "I went into the infected zone and found Dexter inside Fusion Buttercup's lair. Together, we defeated her! But I don't know if Dexter managed to escape."
    );
    assert!(entry.rewards.is_empty());

    assert_eq!(content.outgoing_task_id(2251).unwrap(), Some(2252));
    assert_eq!(content.outgoing_task_id(2252).unwrap(), Some(2253));
    assert_eq!(content.outgoing_task_id(2254).unwrap(), None);
    let journal = content
        .active_journal_entries(&[2253, 2251, 2252, 2252], "Tutorial")
        .unwrap();
    assert_eq!(
        journal
            .iter()
            .map(|mission| mission.task_id)
            .collect::<Vec<_>>(),
        vec![2251, 2252, 2253]
    );
    assert!(journal.iter().all(|mission| mission.npc_id.is_none()));

    let completed = content
        .completed_journal_entries(
            &[2248, 2248, 2249, 2251, 2252, 2253, 2254, 2254],
            "Tutorial",
        )
        .unwrap();
    assert_eq!(
        completed
            .iter()
            .map(|mission| mission.task_id)
            .collect::<Vec<_>>(),
        vec![2249, 2254]
    );
    assert!(completed.iter().all(|mission| mission.npc_id.is_none()));
    assert_eq!(
        completed[0].mission_summary,
        "Recover a transmitter from Fuse's monsters."
    );
    assert_eq!(
        completed[0].mission_complete_summary,
        "I defeated an Oil Ogre and recovered its transmitter for Numbuh Two."
    );

    assert!(
        content
            .completed_journal_entries(&[2248, 2251, 2252, 2253], "Tutorial")
            .unwrap()
            .is_empty(),
        "native intermediate completion history must not leak into the legacy journal"
    );

    let warp = content.warp(2694).unwrap();
    assert_eq!(warp.label, content.npc_name(2694).unwrap());
    assert_eq!(warp.provenance.warp_id, 253);
    assert_eq!(warp.required_task_id, Some(2251));
    assert_eq!(
        warp.target,
        TutorialWarpTarget {
            map_id: 0,
            x: 59_573,
            y: 74_545,
            z: -9_066
        }
    );
}

#[test]
fn malformed_optional_user_equip_catalogs_fail_closed() {
    let mut missing_icons = compact_document();
    missing_icons["tables"][0]["value"]["m_pBackItemTable"] = json!({
        "m_pItemData": [{ "m_iItemNumber": 0, "m_iIcon": 0 }]
    });
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(missing_icons).assets)
            .unwrap_err()
            .to_string()
            .contains("m_pBackItemTable has no required m_pItemIconData")
    );

    let mut missing_reference = compact_document();
    missing_reference["tables"][0]["value"]["m_pWeaponItemTable"] = json!({
        "m_pItemData": [{ "m_iItemNumber": 0, "m_iIcon": 9 }],
        "m_pItemIconData": [{ "m_iIconType": 0, "m_iIconNumber": 0 }]
    });
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(missing_reference).assets)
            .unwrap_err()
            .to_string()
            .contains("references missing m_pItemIconData row 9")
    );

    let mut malformed_unreferenced_icon = compact_document();
    malformed_unreferenced_icon["tables"][0]["value"]["m_pChestItemTable"] = json!({
        "m_pItemData": [{ "m_iItemNumber": 0, "m_iIcon": 0 }],
        "m_pItemIconData": [
            { "m_iIconType": 0, "m_iIconNumber": 0 },
            { "m_iIconType": "bad", "m_iIconNumber": 1 }
        ]
    });
    assert!(
        TutorialMissionContent::from_project_assets(
            &Fixture::new(malformed_unreferenced_icon).assets
        )
        .unwrap_err()
        .to_string()
        .contains("m_iIconType must be an integer"),
        "the whole optional icon subtable is validated, not only referenced rows"
    );
}

#[test]
fn contradictory_world_map_icons_fail_closed() {
    let mut document = compact_document();
    let row = npc_rows_mut(&mut document)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap();
    row["m_iMapIcon"] = json!(19);
    let mut duplicate = row.clone();
    duplicate["m_iMapIcon"] = json!(20);
    npc_rows_mut(&mut document).push(duplicate);

    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets)
            .unwrap_err()
            .to_string()
            .contains("contradictory m_iMapIcon rows for gameplay NPC type 664")
    );
}

#[test]
fn gameplay_npc_duplicates_and_malformed_rows_fail_closed() {
    let mut identical = compact_document();
    let duplicate = npc_rows_mut(&mut identical).last().unwrap().clone();
    npc_rows_mut(&mut identical).push(duplicate);
    let identical =
        TutorialMissionContent::from_project_assets(&Fixture::new(identical).assets).unwrap();
    assert_eq!(identical.gameplay_npc(664).unwrap().max_hp, 439);

    let mut contradictory = compact_document();
    let mut duplicate = npc_rows_mut(&mut contradictory).last().unwrap().clone();
    duplicate["m_iHP"] = json!(440);
    npc_rows_mut(&mut contradictory).push(duplicate);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(contradictory).assets)
            .unwrap_err()
            .to_string()
            .contains("contradictory duplicate gameplay NPC type 664")
    );

    let mut missing_name = compact_document();
    npc_rows_mut(&mut missing_name)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()["m_iNpcName"] = json!(999);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(missing_name).assets)
            .unwrap_err()
            .to_string()
            .contains("references missing string row 999")
    );

    let mut missing_hp = compact_document();
    npc_rows_mut(&mut missing_hp)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("m_iHP");
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(missing_hp).assets)
            .unwrap_err()
            .to_string()
            .contains("has no m_iHP")
    );

    let mut negative_hp = compact_document();
    npc_rows_mut(&mut negative_hp)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()["m_iHP"] = json!(-1);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(negative_hp).assets)
            .unwrap_err()
            .to_string()
            .contains("m_iHP must be zero or positive")
    );

    let mut missing_level = compact_document();
    npc_rows_mut(&mut missing_level)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("m_iNpcLevel");
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(missing_level).assets)
            .unwrap_err()
            .to_string()
            .contains("has no m_iNpcLevel")
    );

    let mut negative_level = compact_document();
    npc_rows_mut(&mut negative_level)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()["m_iNpcLevel"] = json!(-1);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(negative_level).assets)
            .unwrap_err()
            .to_string()
            .contains("m_iNpcLevel must be zero or positive")
    );

    let mut invalid_style = compact_document();
    npc_rows_mut(&mut invalid_style)
        .iter_mut()
        .find(|row| row["m_iNpcNumber"] == json!(664))
        .unwrap()["m_iNpcStyle"] = json!(3);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(invalid_style).assets)
            .unwrap_err()
            .to_string()
            .contains("m_iNpcStyle must be 0, 1, or 2")
    );
}

#[test]
fn guide_first_change_warp_is_required_and_source_validated() {
    let mut missing = compact_document();
    warp_rows_mut(&mut missing).retain(|row| {
        row["m_iWarpNumber"]
            .as_i64()
            .is_none_or(|warp_id| warp_id != i64::from(GUIDE_FIRST_WARP_ID))
    });
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(missing).assets)
            .unwrap_err()
            .to_string()
            .contains("missing clean Guide first-change warp ID 76")
    );

    let mut changed_cost = compact_document();
    warp_rows_mut(&mut changed_cost)
        .iter_mut()
        .find(|row| row["m_iWarpNumber"] == json!(GUIDE_FIRST_WARP_ID))
        .unwrap()["m_iCost"] = json!(1);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(changed_cost).assets)
            .unwrap_err()
            .to_string()
            .contains("Guide first-change warp contradicts source contract")
    );

    let mut duplicate = compact_document();
    let guide = warp_rows_mut(&mut duplicate).last().unwrap().clone();
    warp_rows_mut(&mut duplicate).push(guide);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(duplicate).assets)
            .unwrap_err()
            .to_string()
            .contains("duplicate gameplay warp ID 76")
    );
}

#[test]
fn nano_mission_requires_exactly_three_source_tunes() {
    let mut document = compact_document();
    document["tables"][0]["value"]["m_pNanoTable"]["m_pNanoData"][1]["m_iTune"] = json!([1, 2]);
    let error = TutorialMissionContent::from_project_assets(&Fixture::new(document).assets)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("must contain exactly three integers")
    );
}

#[test]
fn missing_duplicate_and_contradictory_missions_fail_closed() {
    let mut missing = compact_document();
    mission_rows_mut(&mut missing).pop();
    let missing = Fixture::new(missing);
    assert!(
        TutorialMissionContent::from_project_assets(&missing.assets)
            .unwrap_err()
            .to_string()
            .contains("missing tutorial mission task 2254")
    );

    let mut duplicate = compact_document();
    let row = mission_rows_mut(&mut duplicate)[0].clone();
    mission_rows_mut(&mut duplicate).push(row);
    let duplicate = Fixture::new(duplicate);
    assert!(
        TutorialMissionContent::from_project_assets(&duplicate.assets)
            .unwrap_err()
            .to_string()
            .contains("duplicate mission task 2248")
    );

    let mut contradiction = compact_document();
    mission_rows_mut(&mut contradiction)[1]["m_iHMissionName"] = json!(3);
    let contradiction = Fixture::new(contradiction);
    assert!(
        TutorialMissionContent::from_project_assets(&contradiction.assets)
            .unwrap_err()
            .to_string()
            .contains("contradictory title sources")
    );
}

#[test]
fn malformed_cut_scene_rows_fail_closed() {
    let mut negative_event = compact_document();
    negative_event["tables"][0]["value"]["m_pCutSceneTable"]["m_SceneData"][0]["m_iEvent"] =
        json!(-1);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(negative_event).assets)
            .unwrap_err()
            .to_string()
            .contains("m_iEvent is negative")
    );

    let mut negative_count = compact_document();
    negative_count["tables"][0]["value"]["m_pCutSceneTable"]["m_SceneData"][0]["m_iElementCnt"] =
        json!(-1);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(negative_count).assets)
            .unwrap_err()
            .to_string()
            .contains("m_iElementCnt is negative")
    );

    let mut negative_line = compact_document();
    negative_line["tables"][0]["value"]["m_pCutSceneTable"]["m_SceneData"][0]["m_TextElement"]
        [0]["m_iLine"] = json!(-1);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(negative_line).assets)
            .unwrap_err()
            .to_string()
            .contains("m_iLine is negative")
    );

    let mut count_mismatch = compact_document();
    count_mismatch["tables"][0]["value"]["m_pCutSceneTable"]["m_SceneData"][0]["m_iElementCnt"] =
        json!(3);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(count_mismatch).assets)
            .unwrap_err()
            .to_string()
            .contains("declares 3 elements")
    );

    let mut duplicate_line = compact_document();
    duplicate_line["tables"][0]["value"]["m_pCutSceneTable"]["m_SceneData"][0]["m_TextElement"]
        [1]["m_iLine"] = json!(1);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(duplicate_line).assets)
            .unwrap_err()
            .to_string()
            .contains("duplicate cut-scene event 2 line 1")
    );

    let mut non_contiguous = compact_document();
    non_contiguous["tables"][0]["value"]["m_pCutSceneTable"]["m_SceneData"][0]["m_TextElement"]
        [1]["m_iLine"] = json!(3);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(non_contiguous).assets)
            .unwrap_err()
            .to_string()
            .contains("not contiguous")
    );

    let mut duplicate_event = compact_document();
    let event =
        duplicate_event["tables"][0]["value"]["m_pCutSceneTable"]["m_SceneData"][0].clone();
    duplicate_event["tables"][0]["value"]["m_pCutSceneTable"]["m_SceneData"]
        .as_array_mut()
        .unwrap()
        .push(event);
    assert!(
        TutorialMissionContent::from_project_assets(&Fixture::new(duplicate_event).assets)
            .unwrap_err()
            .to_string()
            .contains("duplicate cut-scene event 2")
    );
}

#[test]
fn real_assets_game_table_set_retains_login_and_mission_nanocom_producers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    if !root.join(TABLE_SET_PATH).is_file() {
        return;
    }
    let assets = AssetLocator::open(&root).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    let computress = content
        .gameplay_guide_nanocom(5)
        .expect("primary GuideTable row 5 must be projected");
    assert_eq!(computress.npc_type, 730);
    assert_eq!(computress.login_mail_string_id, 19);
    assert_eq!(
        computress.login_mail_text,
        "Welcome back. Please check your email to learn about an important mission from me."
    );
    assert_eq!(computress.login_no_mail_string_id, 14);
    assert_eq!(computress.level_up_string_id, 24);
    let mission_nanocom = content
        .missions()
        .flat_map(|mission| {
            [
                mission.start_nanocom_message.as_ref(),
                mission.success_nanocom_message.as_ref(),
                mission.failure_nanocom_message.as_ref(),
            ]
            .into_iter()
            .flatten()
        })
        .collect::<Vec<_>>();
    assert!(
        !mission_nanocom.is_empty(),
        "published MissionData must retain passive NanoCom producers"
    );
    let missing_portraits = mission_nanocom
        .iter()
        .filter(|message| {
            content
                .gameplay_npc_portrait_icon_path(message.npc_type)
                .is_none()
        })
        .map(|message| message.npc_type)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        missing_portraits,
        BTreeSet::from([790]),
        "primary NPC 790 alone requests absent Icons/wpnicon_00"
    );
    for message in mission_nanocom {
        assert!(message.message_type & 2 != 0);
        assert!(message.string_id > 0);
        assert!(message.text.chars().count() > 1);
        assert!(
            content.gameplay_npc(message.npc_type).is_some(),
            "NanoCom message NPC {} must exist",
            message.npc_type
        );
    }
}
