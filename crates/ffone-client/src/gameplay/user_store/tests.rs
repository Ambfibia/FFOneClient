use crate::user_store_runtime::*;
use crate::user_store_ui::{
    EMPTY_ITEM_0104, UserStorePendingRequest0104, UserStoreUiCommand0104,
};
use ffone_protocol::RegisteredGameplayRequestError0104;

fn valid_other() -> UserStoreOtherPlayerEntry0104 {
    UserStoreOtherPlayerEntry0104 {
        owner_pc_id: 7,
        target_pc_id: 19,
        authoritative_store_open: true,
        player_menu_owner: UserStoreOwnerReachability0104::Proven,
        previous_cursor_locked: true,
    }
}

fn valid_own() -> UserStoreOwnEntry0104 {
    UserStoreOwnEntry0104 {
        owner_pc_id: 7,
        inventory_slot: 4,
        item_type: 7,
        general_item_type: Some(11),
        map_flags: 1,
        player_y: 20.49,
        map_height: 20.0,
        nearest_other_player_squared_distance: Some(1.0),
        nearest_store_squared_distance: None,
        inventory_use_owner: UserStoreOwnerReachability0104::Proven,
        previous_cursor_locked: true,
    }
}

fn assert_owner_unavailable(
    result: Result<RegisteredGameplayRequest0104, UserStoreProductionError0104>,
    _packet_type: u32,
) {
    assert_eq!(
        result,
        Err(UserStoreProductionError0104::ProductionOwnerBridgeUnavailable)
    );
}

#[test]
fn all_clean_request_structs_match_registered_gm_store_abi() {
    let packets = [
        UserStorePacket0104::ready(4),
        UserStorePacket0104::cancel(7),
        UserStorePacket0104::register_item(0, 4, EMPTY_ITEM_0104, 100),
        UserStorePacket0104::unregister_item(0),
        UserStorePacket0104::sale_start(4),
        UserStorePacket0104::item_list(19),
        UserStorePacket0104::item_buy(19, 0, 5),
    ];
    for (packet, capability) in packets.iter().zip(USER_STORE_REQUEST_CAPABILITIES_0104) {
        assert_eq!(packet.packet_id, capability.packet_type);
        assert_eq!(packet.payload.len(), capability.payload_size);
        assert_eq!(
            prove_user_store_packet_0104(packet).unwrap().packet_type(),
            packet.packet_id
        );
    }
}

#[test]
fn valid_looking_entries_leave_state_authority_and_outbox_untouched() {
    let mut runtime = UserStoreProductionRuntime0104::default();
    let mut state = UserStoreUiState0104::default();
    let mut authority = UserStoreAuthority0104::default();
    authority.owner_pc_id = 123;
    authority.taros = 456;
    let mut outbox = UserStoreUiOutbox0104::default();
    let state_before = state.clone();
    let authority_before = authority.clone();
    let outbox_before = outbox.clone();

    assert_owner_unavailable(
        runtime.attempt_other_player_entry(
            &mut state,
            &mut authority,
            &mut outbox,
            valid_other(),
        ),
        STREETSTALL_REQ_ITEM_LIST,
    );
    assert_eq!(state, state_before);
    assert_eq!(authority, authority_before);
    assert_eq!(outbox, outbox_before);

    assert_owner_unavailable(
        runtime.attempt_own_entry(&mut state, &mut authority, &mut outbox, valid_own()),
        STREETSTALL_REQ_READY,
    );
    assert_eq!(state, state_before);
    assert_eq!(authority, authority_before);
    assert_eq!(outbox, outbox_before);
}

#[test]
fn clean_entry_gates_are_strict_and_do_not_replace_registry_proof() {
    assert_eq!(validate_other_player_entry_0104(valid_other()), Ok(()));
    let mut other = valid_other();
    other.authoritative_store_open = false;
    assert_eq!(
        validate_other_player_entry_0104(other),
        Err(UserStoreEntryGate0104::TargetStoreClosed)
    );

    assert_eq!(validate_own_entry_0104(valid_own()), Ok(()));
    let mut own = valid_own();
    own.item_type = 6;
    assert_eq!(
        validate_own_entry_0104(own),
        Err(UserStoreEntryGate0104::WrongOuterItemType(6))
    );
    let mut own = valid_own();
    own.general_item_type = Some(10);
    assert_eq!(
        validate_own_entry_0104(own),
        Err(UserStoreEntryGate0104::WrongGeneralItemType(Some(10)))
    );
    let mut own = valid_own();
    own.map_flags = 0;
    assert_eq!(
        validate_own_entry_0104(own),
        Err(UserStoreEntryGate0104::RequiredMapFlagMissing)
    );
    let mut own = valid_own();
    own.map_flags = 5;
    assert_eq!(
        validate_own_entry_0104(own),
        Err(UserStoreEntryGate0104::ForbiddenMapFlagSet)
    );
    let mut own = valid_own();
    own.player_y = 20.5;
    assert_eq!(
        validate_own_entry_0104(own),
        Err(UserStoreEntryGate0104::TooFarFromMapHeight {
            absolute_delta: 0.5
        })
    );
    let mut own = valid_own();
    own.nearest_other_player_squared_distance = Some(0.999);
    assert_eq!(
        validate_own_entry_0104(own),
        Err(UserStoreEntryGate0104::NearbyPlayer {
            squared_distance: 0.999
        })
    );
    let mut own = valid_own();
    own.nearest_store_squared_distance = Some(0.0);
    assert_eq!(
        validate_own_entry_0104(own),
        Err(UserStoreEntryGate0104::NearbyStore {
            squared_distance: 0.0
        })
    );
}

#[test]
fn manual_ui_packet_is_dropped_and_shell_returns_inactive() {
    let mut runtime = UserStoreProductionRuntime0104::default();
    let mut state = UserStoreUiState0104 {
        active: true,
        pending: Some(UserStorePendingRequest0104::ItemList { target_pc_id: 19 }),
        ..Default::default()
    };
    let mut popup = UserStorePopupPresentation0104::default();
    let mut outbox = UserStoreUiOutbox0104::default();
    outbox.push(UserStoreUiCommand0104::UnlockCursor);
    outbox.push(UserStoreUiCommand0104::SendPacket(
        UserStorePacket0104::item_list(19),
    ));

    let rejection = runtime
        .guard_unowned_ui(&mut state, &mut popup, &mut outbox)
        .expect("manual shell must be rejected");
    assert_eq!(rejection.dropped_commands, 2);
    assert_eq!(
        rejection.error,
        UserStoreProductionError0104::NoRegisteredProductionSession
    );
    assert_eq!(state, UserStoreUiState0104::default());
    assert_eq!(popup, UserStorePopupPresentation0104::default());
    assert!(outbox.0.is_empty());
    assert_eq!(runtime.rejected_boundaries(), 1);
}

#[test]
fn every_inbound_frame_is_byte_exact_passthrough_without_request_owner() {
    let runtime = UserStoreProductionRuntime0104::default();
    for frame in [
        DecodedFrame {
            packet_type: STREETSTALL_REP_ITEM_LIST,
            flags: 0x1234,
            checksum: 0xabcd,
            payload: vec![1, 2, 3],
        },
        DecodedFrame {
            packet_type: STREETSTALL_REP_READY_SUCCESS,
            flags: 7,
            checksum: 9,
            payload: vec![0; 16],
        },
        DecodedFrame {
            packet_type: 0x3100_dead,
            flags: 2,
            checksum: 3,
            payload: vec![0xff, 0x00],
        },
    ] {
        let expected = frame.clone();
        let passthrough = runtime.ingest(frame);
        assert_eq!(passthrough.frame, expected);
        assert_eq!(
            passthrough.reason,
            if USER_STORE_REPLY_PACKET_TYPES_0104.contains(&expected.packet_type) {
                UserStoreInboundPassthroughReason0104::NoRegisteredRequestOwner
            } else {
                UserStoreInboundPassthroughReason0104::UnrelatedPacket
            }
        );
    }
}
