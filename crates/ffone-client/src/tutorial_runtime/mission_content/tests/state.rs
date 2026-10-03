use super::*;

#[test]
fn transmitter_tasks_use_mode_text_final_reward_and_world_type_contracts() {
    let fixture = Fixture::new(compact_document());
    let content = TutorialMissionContent::from_project_assets(&fixture.assets).unwrap();

    let hunt = content.mission(2248).unwrap();
    assert_eq!(hunt.mission_type, TutorialMissionType::World);
    assert_eq!(hunt.mission_type.xdt_value(), 3);
    assert_eq!(hunt.mission_type.legacy_label(), "Quest");
    assert_eq!(hunt.provenance.journal_row_id, 2248);
    assert_eq!(
        hunt.offer_description(),
        "This attack was no accident. An Oil Ogre is using a transmitter to broadcast our position. We need to find that monster and get that transmitter! You ready?"
    );
    assert_eq!(
        hunt.active_description(),
        "Defeat the Oil Ogre.\n\nYou need to find the Oil Ogre that is broadcasting our position. Defeat him and get the transmitter. You can do it!"
    );
    assert_eq!(
        hunt.journal.mission_summary,
        "Recover a transmitter from Fuse's monsters."
    );
    assert_eq!(
        hunt.journal.mission_complete_summary,
        "I defeated an Oil Ogre and recovered its transmitter for Numbuh Two."
    );
    assert_eq!(
        hunt.completion_description(),
        "Thanks! But the battle isn't over. Dexter says that you're the kid from the time machine. He's in trouble. You need to go to the infected zone right away."
    );

    let delivery = content.mission(2249).unwrap();
    assert_eq!(delivery.mission_type, TutorialMissionType::World);
    assert_eq!(delivery.provenance.journal_row_id, 2249);
    assert_eq!(
        delivery.active_description(),
        "Deliver transmitter to Numbuh Two.\n\nYou've really got some skills. Good work defeating that monster! Now bring that transmitter you recovered back to me."
    );
    assert_eq!(content.final_task_id(2248).unwrap(), 2249);
    assert_eq!(content.final_task_id(2249).unwrap(), 2249);

    for task_id in [2248, 2249] {
        let entry = content.mission_entry(task_id, 42, "Tech Square").unwrap();
        assert_eq!(entry.mission_type, 3);
        assert_eq!(entry.has_task_reward, task_id == 2249);
        assert_eq!(
            entry.rewards,
            MissionUiRewards {
                cash: 0,
                fusion_matter: 75,
            }
        );
    }
}
