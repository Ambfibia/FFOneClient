use super::*;

pub(super) fn mission_row(
    task_id: i32,
    mission_id: i32,
    title_id: i32,
    objective_id: i32,
    journal_npc: i32,
    outgoing: i32,
) -> Value {
    json!({
        "m_iHTaskID": task_id,
        "m_iHMissionID": mission_id,
        "m_iHMissionName": title_id,
        "m_iHCurrentObjective": objective_id,
        "m_iHMissionType": if mission_id == 1 { 3 } else { 2 },
        "m_iSTNanoID": if mission_id == 2 { 1 } else { 0 },
        "m_iCTRReqLvMin": 0,
        "m_iCTRReqLvMax": 0,
        "m_iRepeatflag": 0,
        "m_iCSTReqMission": [0, 0],
        "m_iCSTRReqNano": [0, 0, 0, 0, 0],
        "m_iCSTReqGuide": 0,
        "m_iCSTItemID": [0, 0, 0],
        "m_iCSTItemNumNeeded": [0, 0, 0],
        "m_iCSTTrigger": 0,
        "m_iSTGrantTimer": 0,
        "m_iCSUCheckTimer": 0,
        "m_iCSUEnemyID": [0, 0, 0],
        "m_iCSUNumToKill": [0, 0, 0],
        "m_iCSUItemID": [0, 0, 0],
        "m_iCSUItemNumNeeded": [0, 0, 0],
        "m_iCSUDEFNPCID": 0,
        "m_iRequireInstanceID": 0,
        "m_iHDifficultyType": 1,
        "m_iHJournalNPCID": journal_npc,
        "m_iHNPCID": journal_npc,
        "m_iHTerminatorNPCID": journal_npc,
        "m_iSTGrantWayPoint": journal_npc,
        "m_iSTJournalIDAdd": task_id,
        "m_iHTaskType": 1,
        "m_iSUOutgoingTask": outgoing,
        "m_iFOutgoingTask": 0,
        "m_iSUReward": if task_id == 2249 { 531 } else { 0 }
    })
}

pub(super) fn journal_row(
    offer_description: i32,
    active_task_description: i32,
    mission_summary: i32,
    mission_complete_summary: i32,
    completion_description: i32,
) -> Value {
    json!({
        "m_iDetaileMissionDesc": offer_description,
        "m_iDetailedTaskDesc": active_task_description,
        "m_iMissionSummary": mission_summary,
        "m_iMissionCompleteSummary": mission_complete_summary,
        "m_iDetaileMissionCompleteSummary": completion_description
    })
}

pub(super) fn compact_document() -> Value {
    let mission_strings = [
        "Transmitter Critters",
        "Defeat the Oil Ogre.",
        "Deliver transmitter to Numbuh Two.",
        "A Fusion Matter",
        "Talk to Buttercup.",
        "Enter the infected zone.",
        "Enter the Fusion Portal.",
        "Talk to Dexter.",
        "Defeat Fusion Buttercup.",
        "This attack was no accident. An Oil Ogre is using a transmitter to broadcast our position. We need to find that monster and get that transmitter! You ready?",
        "You need to find the Oil Ogre that is broadcasting our position. Defeat him and get the transmitter. You can do it!",
        "You've really got some skills. Good work defeating that monster! Now bring that transmitter you recovered back to me.",
        "Recover a transmitter from Fuse's monsters.",
        "I defeated an Oil Ogre and recovered its transmitter for Numbuh Two.",
        "Thanks! But the battle isn't over. Dexter says that you're the kid from the time machine. He's in trouble. You need to go to the infected zone right away.",
        "This area is getting dangerous. I was supposed to meet Dexter here, but I can't find him. You look in the infected zone while I check outside.",
        "Come speak to me near the infected zone. Something's happened to Dexter, and we need to find him!",
        "Use a warp gate to enter the infected zone. Once you're inside, you can look for Dexter.",
        "Fusion Buttercup has trapped Dexter inside her lair! You need to find him. Look for the Fusion Portal that leads to that monster's hideout.",
        "You have found Fusion Buttercup's lair. Quick, go talk to Dexter.",
        "You have found Dexter inside Fusion Buttercup's lair. Now you must defeat Fusion Buttercup!",
        "Rescue Dexter from the infected zone.",
        "I went into the infected zone and found Dexter inside Fusion Buttercup's lair. Together, we defeated her! But I don't know if Dexter managed to escape.",
        "So much for Fusion Buttercup. I hope Dexter got out safely. This area is falling apart. Find Numbuh Two and get yourself on the next ride outta here!",
    ]
    .into_iter()
    .map(|value| json!({ "m_pstrNameString": value }))
    .collect::<Vec<_>>();
    let missions = vec![
        mission_row(2248, 1, 0, 1, 2671, 2249),
        mission_row(2249, 1, 0, 2, 2671, 0),
        mission_row(2250, 2, 3, 4, 2672, 2251),
        mission_row(2251, 2, 3, 5, 2672, 2252),
        mission_row(2252, 2, 3, 6, 2672, 2253),
        mission_row(2253, 2, 3, 7, 2672, 2254),
        mission_row(2254, 2, 3, 8, 2672, 0),
    ];
    let mut journals = vec![Value::Null; 2255];
    journals[2248] = journal_row(9, 10, 12, 13, 14);
    journals[2249] = journal_row(9, 11, 12, 13, 14);
    journals[2250] = journal_row(15, 16, 21, 22, 23);
    journals[2251] = journal_row(15, 17, 21, 22, 23);
    journals[2252] = journal_row(15, 18, 21, 22, 23);
    journals[2253] = journal_row(15, 19, 21, 22, 23);
    journals[2254] = journal_row(15, 20, 21, 22, 23);
    let npc_names = [
        "Numbuh Two",
        "Buttercup",
        "Dexter",
        "Infected zone",
        "Fusion Buttercup's Lair",
        "Tech Square",
        "Scamper",
        "Time Squad Officer James",
    ];
    let npc_max_hp = [361, 361, 361, 361, 361, 361, 386];
    let mut npcs = TUTORIAL_MISSION_NPC_TYPES
        .into_iter()
        .enumerate()
        .map(|(index, npc_type)| {
            json!({
                "m_iNpcNumber": npc_type,
                "m_iNpcType": if index < 3 { 3 } else { 5 },
                "m_iAiType": 0,
                "m_iNpcName": index,
                "m_iNpcLevel": 1,
                "m_iNpcStyle": 0,
                "m_iEffect": 0,
                "m_iSound": 1,
                "m_iHP": npc_max_hp[index],
                "m_iTeam": 1,
                "m_iRadius": 120,
                "m_iHeight": 190,
                "m_iSightRange": 700
            })
        })
        .collect::<Vec<_>>();
    npcs.push(json!({
        "m_iNpcNumber": 664,
        "m_iNpcType": 3,
        "m_iAiType": 0,
        "m_iNpcName": 7,
        "m_iNpcLevel": 1,
        "m_iNpcStyle": 0,
        "m_iEffect": 0,
        "m_iSound": 1,
        "m_iHP": 439,
        "m_iTeam": 1,
        "m_iRadius": 120,
        "m_iHeight": 190,
        "m_iSightRange": 700
    }));
    let npc_strings = npc_names
        .into_iter()
        .map(|name| {
            json!({
                "m_strName": name,
                "m_strComment": " ",
                "m_strComment1": " ",
                "m_strComment2": ""
            })
        })
        .collect::<Vec<_>>();
    let warps = EXPECTED_WARPS
        .into_iter()
        .map(|warp| {
            json!({
                "m_iNpcNumber": warp.npc_type,
                "m_iWarpNumber": warp.warp_id,
                "m_iWarpGroupType": 0,
                "m_iLimit_Level": 0,
                "m_iLimit_TaskID": warp.required_task_id,
                "m_iLimit_ItemID": 0,
                "m_iLimit_ItemType": 0,
                "m_iLimit_UseItemID": 0,
                "m_iLimit_UseItemType": 0,
                "m_iToMapNum": warp.target.map_id,
                "m_iToX": warp.target.x,
                "m_iToY": warp.target.y,
                "m_iToZ": warp.target.z,
                "m_iMissionID": 0,
                "m_iIsInstance": 0,
                "m_iCost": 0
            })
        })
        .chain(std::iter::once(json!({
            "m_iNpcNumber": GUIDE_FIRST_WARP_NPC_TYPE,
            "m_iWarpNumber": GUIDE_FIRST_WARP_ID,
            "m_iWarpGroupType": 0,
            "m_iLimit_Level": 0,
            "m_iLimit_TaskID": 0,
            "m_iLimit_ItemID": 0,
            "m_iLimit_ItemType": 0,
            "m_iLimit_UseItemID": 0,
            "m_iLimit_UseItemType": 0,
            "m_iToMapNum": GUIDE_FIRST_WARP_TARGET.map_id,
            "m_iToX": GUIDE_FIRST_WARP_TARGET.x,
            "m_iToY": GUIDE_FIRST_WARP_TARGET.y,
            "m_iToZ": GUIDE_FIRST_WARP_TARGET.z,
            "m_iMissionID": 0,
            "m_iIsInstance": 0,
            "m_iCost": 0
        })))
        .collect::<Vec<_>>();
    let mut system_messages = vec![Value::Null; 265];
    for (row_id, raw_button_type, exact_text) in [
        (12, 0, "Inventory is full."),
        (21, 10, "You cannot use this vendor currently."),
        (22, 10, "Could not retrieve item information."),
        (23, 1, "This item could not be purchased."),
        (24, 1, "This item could not be purchased."),
        (25, 1, "This item could not be sold."),
        (107, 10, "Unopened CRATEs cannot be sold to shopkeepers."),
        (108, 10, "You cannot sell quest items."),
        (109, 10, "This item cannot be sold."),
        (153, 12, "DELETE THIS ITEM?"),
        (
            257,
            0,
            "Alert! One or more of your vehicle rentals has expired.",
        ),
        (
            264,
            1,
            "CANNOT PURCHASE: MISSION NOT COMPLETED. \nYou have not yet completed the mission that rewards this guide item. Please complete the associated mission, then try purchasing again.",
        ),
    ] {
        system_messages[row_id] = json!({
            "m_iButtonType": raw_button_type,
            "m_szString": exact_text
        });
    }

    json!({
        "schema": TABLE_SET_SCHEMA,
        "tables": [{
            "key": "fixture",
            "name": CONSOLIDATED_TABLE,
            "value": {
                "m_pMessageTable": {
                    "m_pMessageData": system_messages
                },
                "m_pMissionTable": {
                    "m_pMissionData": missions,
                    "m_pJournalData": journals,
                    "m_pMissionStringData": mission_strings,
                    "m_pRewardData": [{
                        "m_iMissionRewardID": 531,
                        "m_iCash": 0,
                        "m_iFusionMatter": 75,
                        "m_iBox1Choice": 0,
                        "m_iBox2Choice": 0,
                        "m_iMissionRewarItemType": [0, 0, 0, 0],
                        "m_iMissionRewardItemID": [0, 0, 0, 0],
                        "m_iMissionRewardItemType2": [0, 0, 0, 0],
                        "m_iMissionRewardItemID2": [0, 0, 0, 0]
                    }]
                },
                "m_pNpcTable": {
                    "m_pNpcData": npcs,
                    "m_pNpcStringData": npc_strings
                },
                "m_pCutSceneTable": {
                    "m_SceneData": [{
                        "m_iEvent": 2,
                        "m_iElementCnt": 2,
                        "m_TextElement": [
                            {
                                "m_iLine": 1,
                                "m_strText": "COMPUTRESS: Fixture line one."
                            },
                            {
                                "m_iLine": 2,
                                "m_strText": "BUTTERCUP: Fixture line two."
                            }
                        ]
                    }]
                },
                "m_pInstanceTable": {
                    "m_pWarpData": warps
                },
                "m_pNanoTable": {
                    "m_pNanoData": [
                        {
                            "m_iNanoNumber": 0,
                            "m_iNanoSet": 0,
                            "m_iIcon1": 0,
                            "m_iStyle": 0,
                            "m_iNanoBattery1": 0
                        },
                        {
                            "m_iNanoNumber": 1,
                            "m_iNanoSet": 5,
                            "m_iIcon1": 1,
                            "m_iNanoName": 1,
                            "m_iTune": [1, 2, 3],
                            "m_iStyle": 1,
                            "m_iNanoBattery1": 150
                        }
                    ],
                    "m_pNanoIconData": [
                        { "m_iIconNumber": 0, "m_iIconType": 0 },
                        { "m_iIconNumber": 20, "m_iIconType": 1 }
                    ],
                    "m_pNanoStringData": [
                        {
                            "m_strName": "",
                            "m_strComment1": "",
                            "m_strComment": ""
                        },
                        {
                            "m_strName": "Buttercup",
                            "m_strComment1": "Blastons",
                            "m_strComment": "REQUIRED NANO: Create to reach LVL 1."
                        }
                    ],
                    "m_pNanoTuneData": [
                        {
                            "m_iTuneNumber": 0,
                            "m_iTuneName": 0,
                            "m_iSkillID": 0
                        },
                        {
                            "m_iTuneNumber": 1,
                            "m_iTuneName": 1,
                            "m_iSkillID": 1
                        },
                        {
                            "m_iTuneNumber": 2,
                            "m_iTuneName": 2,
                            "m_iSkillID": 2
                        },
                        {
                            "m_iTuneNumber": 3,
                            "m_iTuneName": 3,
                            "m_iSkillID": 3
                        }
                    ],
                    "m_pNanoTuneStringData": [
                        {
                            "m_strName": "",
                            "m_strComment1": "",
                            "m_strComment": ""
                        },
                        {
                            "m_strName": "MISS FIRE",
                            "m_strComment1": "STUN - CONE",
                            "m_strComment": "Buttercup’s alter-ego Mange uses fire to stun enemies in the target area."
                        },
                        {
                            "m_strName": "RALLYING CRY",
                            "m_strComment1": "HEALTH - GROUP",
                            "m_strComment": "Buttercup’s warcry pumps up your group, healing their injuries."
                        },
                        {
                            "m_strName": "BUTTERCUP BURST",
                            "m_strComment1": "SCAVENGE",
                            "m_strComment": "Buttercup intimidates friend and foe alike to get you even more Fusion Matter!"
                        }
                    ]
                },
                "m_pSkillTable": {
                    "m_pSkillData": [
                        {
                            "m_iSkillNumber": 0,
                            "m_iSkillType": 1,
                            "m_iValueA": [0, 0, 0, 0],
                            "m_iIcon": 0,
                            "m_iBatteryDrainType": 0,
                            "m_iEffectType": 0,
                            "m_iTargetEffect": 0,
                            "m_iEffectTarget": 0,
                            "m_iTargetType": 0,
                            "m_iEffectRange": 0,
                            "m_iEffectArea": 0,
                            "m_iEffectAngle": 0,
                            "m_iTargetNumber": 0,
                            "m_iCoolTime": 0,
                            "m_iCoolType": 0
                        },
                        {
                            "m_iSkillNumber": 1,
                            "m_iSkillType": 1,
                            "m_iValueA": [0, 0, 0, 0],
                            "m_iIcon": 1,
                            "m_iBatteryDrainType": 1,
                            "m_iEffectType": 1,
                            "m_iTargetEffect": 0,
                            "m_iEffectTarget": 3,
                            "m_iTargetType": 1,
                            "m_iEffectRange": 1000,
                            "m_iEffectArea": 0,
                            "m_iEffectAngle": 90,
                            "m_iTargetNumber": 2,
                            "m_iCoolTime": 80,
                            "m_iCoolType": 22
                        },
                        {
                            "m_iSkillNumber": 2,
                            "m_iSkillType": 1,
                            "m_iValueA": [0, 0, 0, 0],
                            "m_iIcon": 2,
                            "m_iBatteryDrainType": 1,
                            "m_iEffectType": 1,
                            "m_iTargetEffect": 0,
                            "m_iEffectTarget": 5,
                            "m_iTargetType": 2,
                            "m_iEffectRange": 0,
                            "m_iEffectArea": 500,
                            "m_iEffectAngle": 0,
                            "m_iTargetNumber": 4,
                            "m_iCoolTime": 60,
                            "m_iCoolType": 7
                        },
                        {
                            "m_iSkillNumber": 3,
                            "m_iSkillType": 1,
                            "m_iValueA": [0, 0, 0, 0],
                            "m_iIcon": 3,
                            "m_iBatteryDrainType": 2,
                            "m_iEffectType": 1,
                            "m_iTargetEffect": 0,
                            "m_iEffectTarget": 2,
                            "m_iTargetType": 3,
                            "m_iEffectRange": 0,
                            "m_iEffectArea": 0,
                            "m_iEffectAngle": 0,
                            "m_iTargetNumber": 1,
                            "m_iCoolTime": 10,
                            "m_iCoolType": 3
                        }
                    ],
                    "m_pSkillIconData": [
                        { "m_iIconNumber": 0, "m_iIconType": 0 },
                        { "m_iIconNumber": 11, "m_iIconType": 2 },
                        { "m_iIconNumber": 5, "m_iIconType": 2 },
                        { "m_iIconNumber": 27, "m_iIconType": 2 }
                    ],
                    "m_pSkillBuffData": []
                }
            }
        }]
    })
}

pub(super) fn compact_item_icon_table(icon_type: u8, icon_number: u32) -> Value {
    json!({
        "m_pItemData": [
            { "m_iItemNumber": 0, "m_iIcon": 0 },
            { "m_iItemNumber": 1, "m_iIcon": 1 }
        ],
        "m_pItemIconData": [
            { "m_iIconType": 0, "m_iIconNumber": 0 },
            { "m_iIconType": icon_type, "m_iIconNumber": icon_number }
        ]
    })
}

pub(super) fn compact_user_equip_document() -> Value {
    let mut document = compact_document();
    let value = document["tables"][0]["value"].as_object_mut().unwrap();
    for (table_key, icon_type, icon_number) in [
        ("m_pBackItemTable", 3, 835),
        ("m_pGlassItemTable", 3, 712),
        ("m_pHatItemTable", 3, 721),
        ("m_pPantsItemTable", 3, 193),
        ("m_pShirtsItemTable", 3, 196),
        ("m_pShoesItemTable", 3, 82),
        ("m_pWeaponItemTable", 0, 13),
        ("m_pVehicleItemTable", 12, 3),
        ("m_pChestItemTable", 7, 0),
    ] {
        value.insert(
            table_key.to_owned(),
            compact_item_icon_table(icon_type, icon_number),
        );
    }
    value.insert(
        "m_pGeneralItemTable".to_owned(),
        json!({
            "m_pItemData": [
                {
                    "m_iItemNumber": 0,
                    "m_iItemType": 0,
                    "m_iIcon": 0
                },
                {
                    "m_iItemNumber": 1,
                    "m_iItemType": 2,
                    "m_iIcon": 1
                }
            ],
            "m_pItemIconData": [
                { "m_iIconType": 0, "m_iIconNumber": 0 },
                { "m_iIconType": 7, "m_iIconNumber": 9 }
            ]
        }),
    );
    value["m_pNanoTable"]["m_pNanoTuneIconData"] = json!([
        { "m_iIconType": 0, "m_iIconNumber": 0 },
        { "m_iIconType": 2, "m_iIconNumber": 19 }
    ]);
    value["m_pNpcTable"]["m_pNpcIconData"] = json!([
        { "m_iIconType": 0, "m_iIconNumber": 0 },
        { "m_iIconType": 10, "m_iIconNumber": 1 }
    ]);
    for npc in value["m_pNpcTable"]["m_pNpcData"].as_array_mut().unwrap() {
        npc["m_iIcon1"] = json!(1);
    }
    value["m_pSkillTable"]["m_pSkillBuffData"] = json!([
        {
            "m_iBuffNumber": 0,
            "m_iBuffIcon": 0,
            "m_iBuffCashIcon": 0
        },
        {
            "m_iBuffNumber": 1,
            "m_iBuffEffect": 0,
            "m_iBuffEffectInstant": 0,
            "m_iBuffIcon": 1,
            "m_iBuffCashIcon": 1
        }
    ]);
    document
}

pub(super) fn mission_rows_mut(document: &mut Value) -> &mut Vec<Value> {
    document
        .pointer_mut("/tables/0/value/m_pMissionTable/m_pMissionData")
        .unwrap()
        .as_array_mut()
        .unwrap()
}

pub(super) fn npc_rows_mut(document: &mut Value) -> &mut Vec<Value> {
    document
        .pointer_mut("/tables/0/value/m_pNpcTable/m_pNpcData")
        .unwrap()
        .as_array_mut()
        .unwrap()
}

pub(super) fn warp_rows_mut(document: &mut Value) -> &mut Vec<Value> {
    document
        .pointer_mut("/tables/0/value/m_pInstanceTable/m_pWarpData")
        .unwrap()
        .as_array_mut()
        .unwrap()
}

pub(super) fn push_gameplay_warp_row(document: &mut Value, npc_type: i32, warp_id: i32, gates: bool) {
    let mut row = warp_rows_mut(document)[0].clone();
    row["m_iNpcNumber"] = json!(npc_type);
    row["m_iWarpNumber"] = json!(warp_id);
    if gates {
        row["m_iWarpGroupType"] = json!(1);
        row["m_iLimit_Level"] = json!(10);
        row["m_iLimit_TaskID"] = json!(123);
        row["m_iLimit_ItemID"] = json!(41);
        row["m_iLimit_ItemType"] = json!(8);
        row["m_iLimit_UseItemID"] = json!(42);
        row["m_iLimit_UseItemType"] = json!(7);
        row["m_iMissionID"] = json!(77);
        row["m_iCost"] = json!(100);
    }
    warp_rows_mut(document).push(row);
}

#[test]
fn normal_warp_resolver_preserves_clean_gate_order_slots_and_group_radius() {
    let mut document = compact_document();
    push_gameplay_warp_row(&mut document, 9_001, 9_001, true);
    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets).unwrap();
    let warp = content.first_gameplay_warp_for_npc(9_001).unwrap();
    let npc_position = GameplayWarpWorldPosition::new(14, 0.0, 100.0, 0.0);
    let no_missions = [];
    let no_group = [];
    let base_input = GameplayWarpEligibilityInput {
        runtime_npc_id: 2_661,
        cash: 100,
        level: 10,
        limit_task_is_active: false,
        active_mission_ids: &no_missions,
        local_pc_uid: 700,
        npc_position,
        group_members: &no_group,
    };

    assert_eq!(
        warp.resolve_normal_eligibility(
            GameplayWarpEligibilityInput {
                cash: 99,
                level: 1,
                ..base_input
            },
            |_| panic!("cash gate must run before inventory lookup")
        ),
        Ok(GameplayWarpEligibility::Denied(
            GameplayWarpDenial::InsufficientCash
        ))
    );
    assert_eq!(
        warp.resolve_normal_eligibility(
            GameplayWarpEligibilityInput {
                level: 9,
                ..base_input
            },
            |_| panic!("level gate must run before inventory lookup")
        ),
        Ok(GameplayWarpEligibility::Denied(
            GameplayWarpDenial::LevelTooLow
        ))
    );

    let mut lookups = Vec::new();
    let missing_limit = warp
        .resolve_normal_eligibility(base_input, |lookup| {
            lookups.push(lookup);
            None
        })
        .unwrap();
    assert_eq!(
        missing_limit,
        GameplayWarpEligibility::Denied(GameplayWarpDenial::MissingLimitItem)
    );
    assert_eq!(
        lookups,
        vec![GameplayWarpItemLookup {
            item_location: GameplayWarpInventoryLocation::Quest,
            start_slot: 0,
            item_id: 41,
            item_type: 8,
        }]
    );

    lookups.clear();
    let missing_use = warp
        .resolve_normal_eligibility(base_input, |lookup| {
            lookups.push(lookup);
            (lookup.item_id == 41).then_some(7)
        })
        .unwrap();
    assert_eq!(
        missing_use,
        GameplayWarpEligibility::Denied(GameplayWarpDenial::MissingUseItem)
    );
    assert_eq!(
        lookups,
        vec![
            GameplayWarpItemLookup {
                item_location: GameplayWarpInventoryLocation::Quest,
                start_slot: 0,
                item_id: 41,
                item_type: 8,
            },
            GameplayWarpItemLookup {
                // Clean uses limit_item_type (8), not use-item type (7),
                // for this second location selector.
                item_location: GameplayWarpInventoryLocation::Quest,
                start_slot: 0,
                item_id: 42,
                item_type: 7,
            },
        ]
    );

    let slots = |lookup: GameplayWarpItemLookup| match lookup.item_id {
        41 => Some(7),
        42 => Some(8),
        _ => None,
    };
    assert_eq!(
        warp.resolve_normal_eligibility(base_input, slots),
        Ok(GameplayWarpEligibility::Denied(
            GameplayWarpDenial::TaskOrMissionLocked
        ))
    );

    let active_missions = [77];
    let far_group = [GameplayWarpGroupMember {
        pc_uid: 701,
        position: GameplayWarpServerPosition::new(14, 501, 0, 10_000),
    }];
    let allowed_by_mission = GameplayWarpEligibilityInput {
        active_mission_ids: &active_missions,
        group_members: &far_group,
        ..base_input
    };
    assert_eq!(
        warp.resolve_normal_eligibility(allowed_by_mission, slots),
        Ok(GameplayWarpEligibility::Denied(
            GameplayWarpDenial::GroupMemberTooFar
        ))
    );

    let near_group = [
        GameplayWarpGroupMember {
            // Clean skips the local member before reading its position.
            pc_uid: 700,
            position: GameplayWarpServerPosition::new(14, i32::MAX, 0, 0),
        },
        GameplayWarpGroupMember {
            pc_uid: 701,
            // Different map and vertical coordinate remain lossless but
            // are not clean gates; horizontal distance is exactly 5.
            position: GameplayWarpServerPosition::new(99, 300, 400, -90_000),
        },
    ];
    assert_eq!(
        near_group[1].position.to_client(),
        GameplayWarpWorldPosition::new(99, 3.0, -900.0, 4.0)
    );
    let ready_input = GameplayWarpEligibilityInput {
        active_mission_ids: &active_missions,
        group_members: &near_group,
        ..base_input
    };
    assert_eq!(
        warp.resolve_normal_eligibility(ready_input, slots),
        Ok(GameplayWarpEligibility::Ready(PcWarpUseNpcRequest0104 {
            npc_id: 2_661,
            warp_id: 9_001,
            e_il_1: 4,
            item_slot_1: 7,
            e_il_2: 4,
            item_slot_2: 8,
        }))
    );
    assert_eq!(
        GameplayWarpDenial::GroupMemberTooFar.system_message_id(),
        115
    );
    assert_eq!(GameplayWarpInventoryLocation::Quest.wire_value(), 2);
}

#[test]
fn normal_warp_group_gate_rejects_a_non_finite_npc_transform() {
    let mut document = compact_document();
    push_gameplay_warp_row(&mut document, 9_001, 9_001, true);
    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(document).assets).unwrap();
    let warp = content.first_gameplay_warp_for_npc(9_001).unwrap();
    let active_missions = [77];
    let group = [];
    let input = GameplayWarpEligibilityInput {
        runtime_npc_id: 2_661,
        cash: 100,
        level: 10,
        limit_task_is_active: false,
        active_mission_ids: &active_missions,
        local_pc_uid: 700,
        npc_position: GameplayWarpWorldPosition::new(14, f32::INFINITY, 0.0, 0.0),
        group_members: &group,
    };

    assert_eq!(
        warp.resolve_normal_eligibility(input, |lookup| match lookup.item_id {
            41 => Some(7),
            42 => Some(8),
            _ => None,
        }),
        Err(GameplayWarpResolveError::NonFiniteNpcPosition { map_number: 14 })
    );
}
