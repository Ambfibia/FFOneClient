use std::collections::{BTreeMap, BTreeSet};

use super::*;

#[test]
fn rustyfusion_barber_requests_accept_typed_wire_payloads_and_reject_wrong_sizes() {
    use crate::{
        WirePayload, packet,
        wire_0104::{PcBarberConfirmRequest0104, PcBarberOpenRequest0104},
    };

    let open = PcBarberOpenRequest0104 { npc_id: 900_347 };
    let confirm = PcBarberConfirmRequest0104::decode(&[0; 76]).unwrap();
    for (id, payload) in [
        (packet::P_CL2FE_REQ_PC_BARBER_OPEN, open.encode()),
        (packet::P_CL2FE_REQ_PC_BARBER_CONFIRM, confirm.encode()),
    ] {
        let request = RegisteredGameplayRequest0104::new(id, payload.clone())
            .expect("RustyFusion registers the barber handler");
        assert_eq!(request.payload(), payload);
        assert_eq!(
            request.registration().family,
            ShardRequestFamily0104::NpcManager
        );
        for size in [payload.len() - 1, payload.len() + 1] {
            assert!(matches!(
                RegisteredGameplayRequest0104::new(id, vec![0; size]),
                Err(RegisteredGameplayRequestError0104::WrongPayloadSize { .. })
            ));
        }
    }
}

#[test]
fn paired_openfusion_inventory_has_131_unique_requests_and_exact_family_counts() {
    assert_eq!(OPENFUSION_SHARD_REQUESTS_0104.len(), 131);
    let ids = OPENFUSION_SHARD_REQUESTS_0104
        .iter()
        .map(|request| request.packet_type)
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 131);

    let mut counts = BTreeMap::<u8, usize>::new();
    for request in OPENFUSION_SHARD_REQUESTS_0104 {
        *counts.entry(request.family as u8).or_default() += 1;
    }
    let expected = [9, 22, 9, 8, 2, 8, 4, 5, 6, 9, 7, 10, 9, 4, 10, 2, 7];
    let actual = (0..expected.len() as u8)
        .map(|family| counts.get(&family).copied().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn every_registered_request_has_an_exact_layout() {
    for request in OPENFUSION_SHARD_REQUESTS_0104 {
        let layout = registered_request_layout_0104(request.packet_type)
            .unwrap_or_else(|| panic!("missing layout for {}", request.name));
        let payload = match layout {
            RegisteredRequestLayout0104::Fixed { size } => vec![0; size],
            RegisteredRequestLayout0104::Counted { base_size, .. } => vec![0; base_size],
        };
        RegisteredGameplayRequest0104::new(request.packet_type, payload).unwrap_or_else(
            |error| panic!("invalid zero-value ABI for {}: {error}", request.name),
        );
    }
}

#[test]
fn registered_envelope_rejects_unknown_oversized_and_wrong_abi_payloads() {
    let request = RegisteredGameplayRequest0104::new(0x1300_005b, vec![0; 12])
        .expect("race start is registered");
    assert_eq!(request.registration().name, "P_CL2FE_REQ_EP_RACE_START");

    assert!(matches!(
        RegisteredGameplayRequest0104::new(0x1300_ffff, Vec::new()),
        Err(RegisteredGameplayRequestError0104::UnregisteredPacket { .. })
    ));
    assert!(matches!(
        RegisteredGameplayRequest0104::new(
            0x1300_005b,
            vec![0; RegisteredGameplayRequest0104::MAX_PAYLOAD_SIZE + 1]
        ),
        Err(RegisteredGameplayRequestError0104::PayloadTooLarge { .. })
    ));
    assert!(matches!(
        RegisteredGameplayRequest0104::new(0x1300_005b, vec![0; 3]),
        Err(RegisteredGameplayRequestError0104::WrongPayloadSize { .. })
    ));
}

#[test]
fn counted_layouts_validate_count_trailer_and_server_cap() {
    let mut attack = vec![0; 12];
    attack[0..4].copy_from_slice(&2i32.to_le_bytes());
    assert!(RegisteredGameplayRequest0104::new(0x1300_0006, attack).is_ok());

    let mut mismatch = vec![0; 8];
    mismatch[0..4].copy_from_slice(&2i32.to_le_bytes());
    assert!(matches!(
        RegisteredGameplayRequest0104::new(0x1300_0006, mismatch),
        Err(RegisteredGameplayRequestError0104::WrongPayloadSize { .. })
    ));

    let mut too_many = vec![0; 20];
    too_many[0..4].copy_from_slice(&4i32.to_le_bytes());
    assert!(matches!(
        RegisteredGameplayRequest0104::new(0x1300_0006, too_many),
        Err(RegisteredGameplayRequestError0104::CountTooLarge { maximum: 3, .. })
    ));
}
