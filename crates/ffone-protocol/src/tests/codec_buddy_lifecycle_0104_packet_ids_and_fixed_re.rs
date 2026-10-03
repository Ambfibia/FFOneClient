use super::*;

#[test]
fn buddy_lifecycle_0104_packet_ids_and_fixed_registry_match_clean_client() {
    assert_eq!(packet::P_CL2FE_REQ_REQUEST_MAKE_BUDDY, 0x1300_0035);
    assert_eq!(packet::P_CL2FE_REQ_ACCEPT_MAKE_BUDDY, 0x1300_0036);
    assert_eq!(packet::P_CL2FE_REQ_SET_BUDDY_BLOCK, 0x1300_003a);
    assert_eq!(packet::P_CL2FE_REQ_REMOVE_BUDDY, 0x1300_003b);
    assert_eq!(packet::P_CL2FE_REQ_GET_BUDDY_STATE, 0x1300_003c);
    assert_eq!(packet::P_CL2FE_REQ_PC_BUDDY_WARP, 0x1300_0051);
    assert_eq!(packet::P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY, 0x1300_008e);
    assert_eq!(packet::P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY, 0x1300_008f);
    assert_eq!(packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC, 0x3100_0063);
    assert_eq!(
        packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC,
        0x8300_0065,
        "clean Retrobution csDefines is authoritative over OpenFusion's sentinel"
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC,
        0x3100_0101
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC,
        0x3100_00fe
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL,
        0x3100_00ff
    );
    assert_eq!(
        packet::P_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL,
        0x3100_0100
    );

    for (packet_id, expected) in [
        (
            packet::P_CL2FE_REQ_REQUEST_MAKE_BUDDY,
            BuddyMakeRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_ACCEPT_MAKE_BUDDY,
            BuddyAcceptRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_GET_BUDDY_STATE,
            BuddyStateRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_SET_BUDDY_BLOCK,
            BuddySetBlockRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_REMOVE_BUDDY,
            BuddyRemoveRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_BUDDY_WARP,
            BuddyWarpRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY,
            BuddyFindNameRequest0104::SIZE,
        ),
        (
            packet::P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY,
            BuddyFindNameAcceptRequest0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC,
            BuddyMakeSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_ACCEPT_MAKE_BUDDY_SUCC,
            BuddyAcceptSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_GET_BUDDY_STATE_SUCC,
            BuddyStateSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_SET_BUDDY_BLOCK_SUCC,
            BuddyBlockSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_REMOVE_BUDDY_SUCC,
            BuddyRemoveSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC_TO_ACCEPTER,
            BuddyIncomingRequest0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_BUDDY_WARP_FAIL,
            BuddyWarpFailure0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC,
            BuddyWarpOtherShardSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC,
            BuddyWarpSameShardSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_SUCC,
            BuddyFindNameSuccess0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_FIND_NAME_MAKE_BUDDY_FAIL,
            BuddyFindNameFailure0104::SIZE,
        ),
        (
            packet::P_FE2CL_REP_PC_FIND_NAME_ACCEPT_BUDDY_FAIL,
            BuddyFindNameAcceptFailure0104::SIZE,
        ),
    ] {
        assert_eq!(fixed_payload_size(packet_id), Some(expected));
    }
    assert_eq!(
        fixed_payload_size(packet::P_FE2CL_REP_PC_BUDDYLIST_INFO_SUCC),
        None,
        "buddy-list success owns a variable sBuddyBaseInfo tail"
    );
}
