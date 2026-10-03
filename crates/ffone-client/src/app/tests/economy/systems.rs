use super::*;

#[test]
fn apparel_refresh_plan_seeds_without_rebuild_and_commits_only_ready_candidate() {
    let item = |item_type, item_id| ItemBase0104 {
        item_type,
        item_id,
        option: 0,
        time_limit: 0,
    };
    let initial = [
        item(1, 100),
        item(2, 200),
        item(3, 300),
        item(4, 400),
        item(5, 500),
        item(6, 600),
    ];
    let changed = [
        item(1, 101),
        item(2, 201),
        item(3, 301),
        item(4, 401),
        item(5, 501),
        item(6, 601),
    ];

    assert_eq!(
        world_player_apparel_refresh_plan_0104(None, None, None, initial),
        WorldPlayerApparelRefreshPlan::Seed
    );
    assert_eq!(
        world_player_apparel_refresh_plan_0104(
            Some(initial),
            Some((changed, WorldPlayerApparelCandidateReadiness::Loading)),
            None,
            changed,
        ),
        WorldPlayerApparelRefreshPlan::Await
    );
    assert_eq!(
        world_player_apparel_refresh_plan_0104(
            Some(initial),
            Some((changed, WorldPlayerApparelCandidateReadiness::Ready)),
            None,
            changed,
        ),
        WorldPlayerApparelRefreshPlan::Promote
    );
    assert_eq!(
        world_player_apparel_refresh_plan_0104(
            Some(initial),
            Some((changed, WorldPlayerApparelCandidateReadiness::Blocked)),
            None,
            changed,
        ),
        WorldPlayerApparelRefreshPlan::Reject
    );
}

#[test]
fn apparel_refresh_plan_cancels_or_replaces_superseded_rapid_replies() {
    let item = |item_type, item_id| ItemBase0104 {
        item_type,
        item_id,
        option: 0,
        time_limit: 0,
    };
    let active = [
        item(1, 100),
        item(2, 200),
        item(3, 300),
        item(4, 400),
        item(5, 500),
        item(6, 600),
    ];
    let first_reply = [
        item(1, 101),
        item(2, 201),
        item(3, 301),
        item(4, 401),
        item(5, 501),
        item(6, 601),
    ];
    let later_reply = [
        item(1, 102),
        item(2, 202),
        item(3, 302),
        item(4, 402),
        item(5, 502),
        item(6, 602),
    ];

    assert_eq!(
        world_player_apparel_refresh_plan_0104(
            Some(active),
            Some((first_reply, WorldPlayerApparelCandidateReadiness::Loading,)),
            None,
            active,
        ),
        WorldPlayerApparelRefreshPlan::Cancel
    );
    assert_eq!(
        world_player_apparel_refresh_plan_0104(
            Some(active),
            Some((first_reply, WorldPlayerApparelCandidateReadiness::Loading,)),
            None,
            later_reply,
        ),
        WorldPlayerApparelRefreshPlan::Replace
    );
    assert_eq!(
        world_player_apparel_refresh_plan_0104(Some(active), None, Some(later_reply), later_reply,),
        WorldPlayerApparelRefreshPlan::Stable,
        "a rejected exact candidate must not respawn every frame"
    );
}

#[test]
fn quick_slot_roster_and_registration_success_update_the_exact_slots() {
    let mut model = QuickSlotUiModel::default();
    let roster = ffone_protocol::QuickSlotInfo0104 {
        slots: std::array::from_fn(|index| ffone_protocol::QuickSlotEntry0104 {
            item_type: 7,
            item_id: if index % 2 == 0 {
                100 + index as i16
            } else {
                0
            },
        }),
    };
    let roster_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_PC_QUICK_SLOT_INFO,
        flags: 0,
        checksum: 0,
        payload: roster.encode(),
    };
    assert_eq!(apply_quick_slot_frame(&roster_frame, &mut model), Ok(true));
    assert!(model.ready_for_play);
    assert_eq!(
        model
            .slots
            .iter()
            .map(|slot| slot.item_id)
            .collect::<Vec<_>>(),
        vec![100, 0, 102, 0, 104, 0, 106, 0]
    );
    assert!(
        model
            .slots
            .iter()
            .all(|slot| slot.icon_path.is_none() && !slot.inventory_empty)
    );

    let success_frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_REGIST_QUICK_SLOT_SUCC,
        flags: 0,
        checksum: 0,
        payload: ffone_protocol::QuickSlotRegisterSuccess0104 {
            slot_num: 7,
            item_type: 7,
            item_id: 777,
        }
        .encode(),
    };
    assert_eq!(apply_quick_slot_frame(&success_frame, &mut model), Ok(true));
    assert_eq!(model.slots[7].item_id, 777);
}
