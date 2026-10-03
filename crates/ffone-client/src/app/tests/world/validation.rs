use super::*;

#[test]
fn transportation_npc_services_require_matching_cat15_or_cat16_catalog_ownership() {
    assert_eq!(
        transportation_npc_service(15, Some(15)),
        Some(TransportationService::Warp)
    );
    assert_eq!(
        transportation_npc_service(16, Some(16)),
        Some(TransportationService::Wyvern)
    );
    assert_eq!(transportation_npc_service(15, Some(16)), None);
    assert_eq!(transportation_npc_service(16, Some(15)), None);
    assert_eq!(transportation_npc_service(17, Some(17)), None);
    assert_eq!(transportation_npc_service(15, None), None);
}

#[test]
fn transportation_registration_and_warp_replies_require_exact_owned_leases() {
    let mut production = TransportationProductionRuntime::default();
    let registration = TransportationRegistrationIntent {
        transportation_type: TransportationService::Warp as i32,
        npc_id: 9_001,
        location_id: 13,
    };
    production.reserve_registration(registration);
    assert_eq!(production.take_registration_reply(2, 13), None);
    assert_eq!(production.pending_registration, Some(registration));
    assert_eq!(
        production.take_registration_reply(1, 13),
        Some(registration)
    );
    assert_eq!(production.pending_registration, None);

    // OpenFusion sends no reply for an already registered stop. That silent
    // request must not block the next monkey (future Candy Cove, location 34).
    production.reserve_registration(registration);
    let next_stop = TransportationRegistrationIntent {
        transportation_type: TransportationService::Wyvern as i32,
        npc_id: 9_002,
        location_id: 34,
    };
    production.reserve_registration(next_stop);
    assert_eq!(production.take_registration_reply(1, 13), None);
    assert_eq!(production.take_registration_reply(2, 34), Some(next_stop));
    assert_eq!(production.pending_registration, None);

    production.begin_npc(9_001, true, TransportationService::Warp, None);
    let travel = TransportationTravelIntent {
        npc_id: 9_001,
        transporation_id: 77,
        e_il: 4,
        slot_number: 0,
        turbo: false,
    };
    assert_eq!(
        production.reserve_warp(travel, TransportationService::Warp),
        Ok(())
    );
    assert_eq!(production.take_warp_success(2), None);
    assert_eq!(production.take_warp_failure(78), None);
    assert_eq!(
        production.take_warp_failure(77),
        Some(TransportationWarpLease0104 {
            intent: travel,
            transportation_type: 1,
        })
    );
    assert_eq!(production.pending_warp, None);

    production.begin_npc(9_002, false, TransportationService::Wyvern, None);
    let stale_npc = TransportationTravelIntent {
        npc_id: 9_001,
        ..travel
    };
    assert!(
        production
            .reserve_warp(stale_npc, TransportationService::Wyvern)
            .is_err()
    );
    let wyvern = TransportationTravelIntent {
        npc_id: 9_002,
        transporation_id: 88,
        turbo: true,
        ..travel
    };
    production
        .reserve_warp(wyvern, TransportationService::Wyvern)
        .unwrap();
    assert_eq!(
        production.take_warp_success(2),
        Some(TransportationWarpLease0104 {
            intent: wyvern,
            transportation_type: 2,
        })
    );
}
