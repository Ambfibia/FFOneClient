use std::{collections::BTreeSet, path::PathBuf};

use ffone_client::{
    assets::AssetLocator,
    gameplay_nano_portraits::GameplayNanoPortraitCatalog,
    nano_free_tuning_runtime::project_nano_free_tuning_content,
    tutorial_mission_content::TutorialMissionContent,
    user_equip_ui::{UserEquipNanoEquippedAuthority, UserEquipNanoModeProjection},
};
use ffone_protocol::Nano0104;
use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game")
}

#[test]
fn restored_nanos_have_independent_models_real_gallery_ids_and_complete_tuning() {
    let locator = AssetLocator::open(root()).unwrap();
    let content = TutorialMissionContent::open(&locator).unwrap();
    let portraits = GameplayNanoPortraitCatalog::open(&locator).unwrap();
    let expected = [
        (37, "Flapjack", "nano_flapjack"),
        (38, "Johnny Bravo", "nano_johnnybravo"),
        (41, "Unstable Nano", "nano_holonano"),
        (67, "Coop", "nano_coop"),
        (68, "Ben Tennyson", "nano_ben"),
        (69, "Ghostfreak", "nano_ghostfreak"),
        (70, "Upgrade", "nano_upgrade"),
    ];
    for (id, name, model) in expected {
        assert_eq!(content.gameplay_nano(id).unwrap().name, name);
        assert_eq!(
            portraits.model_path(id),
            Some(format!("characters/nanos/{model}/{model}.glb").as_str())
        );
        let tuning = project_nano_free_tuning_content(&content, id).unwrap();
        assert!(
            tuning
                .powers
                .iter()
                .all(|power| power.skill_id > 0 && power.tune_id > 0)
        );
    }
    let unstable = project_nano_free_tuning_content(&content, 41).unwrap();
    assert!(
        unstable
            .powers
            .iter()
            .all(|power| power.icon_path.ends_with("skillicon_67.png")
                && root().join(&power.icon_path).is_file())
    );
    assert_eq!(
        unstable.powers.map(|power| (power.tune_id, power.skill_id)),
        [(288, 122), (289, 122), (290, 122)]
    );
    let added = project_nano_free_tuning_content(&content, 68).unwrap();
    for id in [68, 69, 70] {
        assert_eq!(
            project_nano_free_tuning_content(&content, id)
                .unwrap()
                .powers,
            added.powers
        );
    }
    // The rejected Unstable alias is retired; Van Kleiss keeps his own identity.
    assert!(content.gameplay_nano(52).is_none());
    assert!(portraits.model_path(52).is_none());
    assert!(portraits.model_path(66).unwrap().contains("nano_vankleiss"));
    assert!(
        content
            .gameplay_nano(41)
            .unwrap()
            .icon_path
            .as_ref()
            .unwrap()
            .contains("holo-nano")
    );
    assert!(
        content
            .gameplay_nano(66)
            .unwrap()
            .icon_path
            .as_ref()
            .unwrap()
            .contains("van-kleiss")
    );

    let mut bank = vec![
        Nano0104 {
            id: 0,
            skill_id: 0,
            stamina: 0
        };
        71
    ];
    bank[41] = Nano0104 {
        id: 41,
        skill_id: 122,
        stamina: 150,
    };
    // A historical bank record must not resurrect the retired gallery entry.
    bank[52] = Nano0104 {
        id: 52,
        skill_id: 245,
        stamina: 150,
    };
    let gallery = UserEquipNanoModeProjection::from_authoritative(
        &bank,
        [UserEquipNanoEquippedAuthority::default(); 3],
        &content,
    )
    .gallery;
    assert_eq!(gallery.len(), 66);
    assert!(!gallery.iter().any(|nano| nano.nano_id == 52));
    assert_eq!(
        gallery
            .iter()
            .map(|nano| nano.nano_id)
            .collect::<BTreeSet<_>>()
            .len(),
        66
    );
    assert!(
        gallery
            .iter()
            .all(|nano| content.gameplay_nano(nano.nano_id).is_some())
    );
    assert!(
        gallery
            .iter()
            .find(|nano| nano.nano_id == 41)
            .unwrap()
            .owned
    );
    assert!(
        !gallery
            .iter()
            .find(|nano| nano.nano_id == 67)
            .unwrap()
            .owned
    );
}

#[test]
fn quest_reward_and_localized_roster_keep_semantic_identity() {
    let read = |path: &str| -> Value {
        serde_json::from_slice(&std::fs::read(root().join(path)).unwrap()).unwrap()
    };
    let tables = ffone_client::xdt::into_table_set(read("data/tables/xdt.json")).unwrap();
    let nano_table = tables["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|table| table["value"].get("m_pNanoTable"))
        .unwrap();
    assert_eq!(nano_table["m_pNanoData"][52], nano_table["m_pNanoData"][0]);
    assert_eq!(
        nano_table["m_pNanoData"][41]["m_iTune"],
        serde_json::json!([288, 289, 290])
    );
    assert_eq!(
        nano_table["m_pNanoData"][67]["m_iTune"],
        serde_json::json!([210, 211, 212])
    );
    for id in [288, 289, 290] {
        let tune = &nano_table["m_pNanoTuneData"][id];
        assert_eq!(tune["m_iSkillID"], 122);
        assert_eq!(tune["m_iReqItemID"], 37);
        assert_eq!(tune["m_iReqItemCount"], 5);
        assert_eq!(tune["m_iReqFusionMatter"], 100);
        assert_eq!(
            nano_table["m_pNanoTuneStringData"][id]["m_strName"],
            "UNSTABLE POWER"
        );
    }
    let data = tables["tables"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|table| table["value"].get("m_pMissionTable"))
        .unwrap();
    let tasks = data["m_pMissionData"].as_array().unwrap();
    for id in [5213, 5214, 5215] {
        let task = tasks.iter().find(|task| task["m_iHTaskID"] == id).unwrap();
        assert_eq!(task["m_iHMissionID"], 841);
        assert_eq!(task["m_iSTNanoID"], 41);
    }
    let en = read("localization/en.json");
    let ru = read("localization/ru.json");
    assert_eq!(
        en["entries"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ru["entries"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>()
    );
    for locale in [&en, &ru] {
        for id in [37, 38, 41, 67, 68, 69, 70] {
            for field in ["name", "attribute", "description", "acquired_name"] {
                assert!(
                    !locale["entries"][format!("content.nano.{id}.{field}")]
                        .as_str()
                        .unwrap()
                        .is_empty()
                );
            }
        }
        for id in [198, 199, 200, 201, 202, 203, 285, 286, 287, 288, 289, 290] {
            for field in ["name", "type_label", "description"] {
                assert!(
                    !locale["entries"][format!("content.nano_tune.{id}.{field}")]
                        .as_str()
                        .unwrap()
                        .is_empty()
                );
            }
        }
    }
}
