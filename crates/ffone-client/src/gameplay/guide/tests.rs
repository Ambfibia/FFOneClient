use crate::guide_runtime::*;

fn pc_load(raw_mentor: i16, raw_mentor_count: i16) -> PcLoadData0104 {
    let mut load = PcLoadData0104::zeroed();
    load.as_bytes_mut()[PcLoadData0104::MENTOR_OFFSET..PcLoadData0104::MENTOR_OFFSET + 2]
        .copy_from_slice(&raw_mentor.to_le_bytes());
    load.as_bytes_mut()
        [PcLoadData0104::MENTOR_COUNT_OFFSET..PcLoadData0104::MENTOR_COUNT_OFFSET + 2]
        .copy_from_slice(&raw_mentor_count.to_le_bytes());
    load
}

#[test]
fn only_one_through_four_are_selectable_and_five_remains_a_sentinel() {
    for raw in 1..=4 {
        let recognized = GuideRawMentor::from_raw(raw).unwrap();
        assert_eq!(recognized.raw(), raw);
        assert!(matches!(recognized, GuideRawMentor::Selectable(_)));
    }
    assert_eq!(
        GuideRawMentor::from_raw(5),
        Some(GuideRawMentor::ComputressFuture)
    );
    assert_eq!(GuideRawMentor::from_raw(0), None);
    assert_eq!(GuideRawMentor::from_raw(6), None);
    assert_eq!(GuideRawMentor::from_raw(-1), None);
}

#[test]
fn npc_categories_preserve_exact_guide_and_paid_past_warp_routes() {
    let expected = [
        GuideRawMentor::Selectable(GuideMentor::Edd),
        GuideRawMentor::Selectable(GuideMentor::Dexter),
        GuideRawMentor::Selectable(GuideMentor::MojoJojo),
        GuideRawMentor::Selectable(GuideMentor::BenTennyson),
        GuideRawMentor::ComputressFuture,
    ];
    for (offset, passed_mentor) in expected.into_iter().enumerate() {
        assert_eq!(
            guide_npc_service_route(18 + offset as i32, 0),
            Some(GuideNpcServiceRoute::GuideChanger { passed_mentor })
        );
    }
    assert_eq!(
        guide_npc_service_route(23, 0),
        Some(GuideNpcServiceRoute::UpsellRequired)
    );
    assert_eq!(
        guide_npc_service_route(23, 1),
        Some(GuideNpcServiceRoute::PastWarp)
    );
    assert_eq!(guide_npc_service_route(23, 2), None);
    assert_eq!(guide_npc_service_route(17, 1), None);
    assert_eq!(guide_npc_service_route(24, 1), None);
}

#[test]
fn init_uses_owned_mentor_override_and_source_backed_initial_phases() {
    assert_eq!(
        guide_init_intent(2, -999),
        Ok(GuideInitIntent::ChangeExisting {
            current: GuideMentor::Dexter
        })
    );
    assert_eq!(
        guide_init_intent(5, 0),
        Ok(GuideInitIntent::InitialWarpWarning)
    );
    assert_eq!(
        guide_init_intent(5, 4),
        Ok(GuideInitIntent::InitialSelection {
            passed_mentor: GuideRawMentor::Selectable(GuideMentor::BenTennyson)
        })
    );
    assert_eq!(
        guide_init_intent(5, 5),
        Ok(GuideInitIntent::InitialSelection {
            passed_mentor: GuideRawMentor::ComputressFuture
        })
    );
    assert_eq!(
        guide_init_intent(5, 6),
        Err(GuideInitError::UnsupportedPassedMentor { raw_mentor: 6 })
    );
    assert_eq!(
        guide_init_intent(0, 1),
        Err(GuideInitError::UnsupportedOwnedMentor { raw_mentor: 0 })
    );
}

#[test]
fn pc_load_mentor_and_count_remain_lossless_even_when_unsupported() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::Clean0104);
    runtime.load_pc_state(&pc_load(-7, 301));
    let authoritative = runtime.authoritative().unwrap();
    assert_eq!(authoritative.raw_mentor(), -7);
    assert_eq!(authoritative.raw_mentor_count(), 301);
    assert_eq!(authoritative.recognized_mentor(), None);
    assert_eq!(
        runtime.request_change(GuideMentor::Edd),
        Err(GuideCorrelationError::UnsupportedOwnedMentor { raw_mentor: -7 })
    );
}

#[test]
fn cancelling_a_local_send_preserves_authoritative_mentor_state() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    runtime.load_pc_state(&pc_load(2, 7));
    runtime.request_change(GuideMentor::Edd).unwrap();
    runtime.cancel_pending_change();
    assert_eq!(runtime.pending(), None);
    assert_eq!(
        runtime.authoritative(),
        Some(GuideAuthoritativeState {
            raw_mentor: 2,
            raw_mentor_count: 7,
        })
    );
}

#[test]
fn clean_profile_uses_the_raw_reply_count_for_first_change() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::Clean0104);
    runtime.load_pc_state(&pc_load(5, 0));
    assert_eq!(
        runtime.request_change(GuideMentor::Edd),
        Ok(PcChangeMentorRequest0104 { mentor: 1 })
    );
    let applied = runtime
        .accept_success(PcChangeMentorSuccess0104 {
            mentor: 1,
            mentor_count: 1,
            fusion_matter: 123,
        })
        .unwrap();
    assert_eq!(
        applied,
        GuideChangeApplied {
            mentor: GuideMentor::Edd,
            raw_mentor_count: 1,
            fusion_matter: 123,
            intent: GuidePostChangeIntent::WarpToNpc {
                npc_table_id: 1_425
            }
        }
    );
    assert_eq!(
        runtime.authoritative(),
        Some(GuideAuthoritativeState {
            raw_mentor: 1,
            raw_mentor_count: 1
        })
    );
}

#[test]
fn clean_profile_treats_a_non_one_raw_count_as_later() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::Clean0104);
    runtime.load_pc_state(&pc_load(2, 1));
    runtime.request_change(GuideMentor::BenTennyson).unwrap();
    let applied = runtime
        .accept_success(PcChangeMentorSuccess0104 {
            mentor: 4,
            mentor_count: 7,
            fusion_matter: 80,
        })
        .unwrap();
    assert_eq!(applied.intent, GuidePostChangeIntent::RefreshGuideMissions);
    assert_eq!(applied.raw_mentor_count, 7);
}

#[test]
fn openfusion_count_one_after_existing_mentor_is_later_and_stays_raw() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    runtime.load_pc_state(&pc_load(2, 9));
    runtime.request_change(GuideMentor::BenTennyson).unwrap();
    let applied = runtime
        .accept_success(PcChangeMentorSuccess0104 {
            mentor: 4,
            mentor_count: 1,
            fusion_matter: 77,
        })
        .unwrap();

    assert_eq!(applied.intent, GuidePostChangeIntent::RefreshGuideMissions);
    assert_eq!(applied.raw_mentor_count, 1);
    assert_eq!(runtime.authoritative().unwrap().raw_mentor_count(), 1);
    assert_eq!(
        runtime.last_success_diagnostics(),
        Some(GuideReplyDiagnostics {
            profile: GuideServerProfile::OpenFusion0104,
            requested_mentor: GuideMentor::BenTennyson,
            previous_raw_mentor: 2,
            reply_raw_mentor: 4,
            raw_reply_mentor_count: 1,
            interpreted_as: GuideChangeKind::Later,
        })
    );
}

#[test]
fn openfusion_count_one_after_sentinel_is_first() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    runtime.load_pc_state(&pc_load(5, 17));
    runtime.request_change(GuideMentor::MojoJojo).unwrap();
    let applied = runtime
        .accept_success(PcChangeMentorSuccess0104 {
            mentor: 3,
            mentor_count: 1,
            fusion_matter: 5,
        })
        .unwrap();
    assert_eq!(
        applied.intent,
        GuidePostChangeIntent::WarpToNpc {
            npc_table_id: GUIDE_FIRST_CHANGE_WARP_NPC_TABLE_ID
        }
    );
    assert_eq!(
        runtime.last_success_diagnostics().unwrap().interpreted_as,
        GuideChangeKind::First
    );
}

#[test]
fn mismatched_success_and_failure_replies_are_transactional() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    runtime.load_pc_state(&pc_load(2, 3));
    runtime.request_change(GuideMentor::BenTennyson).unwrap();

    let before_success = runtime.clone();
    assert_eq!(
        runtime.accept_success(PcChangeMentorSuccess0104 {
            mentor: 3,
            mentor_count: 1,
            fusion_matter: 90,
        }),
        Err(GuideCorrelationError::ReplyMentorMismatch {
            expected: GuideMentor::BenTennyson,
            actual: GuideMentor::MojoJojo,
        })
    );
    assert_eq!(runtime, before_success);

    let before_failure = runtime.clone();
    assert_eq!(
        runtime.accept_failure(PcChangeMentorFailure0104 {
            mentor: 1,
            error_code: 44,
        }),
        Err(GuideCorrelationError::ReplyMentorMismatch {
            expected: GuideMentor::BenTennyson,
            actual: GuideMentor::Edd,
        })
    );
    assert_eq!(runtime, before_failure);
}

#[test]
fn changed_previous_mentor_rejects_reply_without_any_further_mutation() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    runtime.load_pc_state(&pc_load(2, 3));
    runtime.request_change(GuideMentor::BenTennyson).unwrap();
    runtime.authoritative = Some(GuideAuthoritativeState {
        raw_mentor: 3,
        raw_mentor_count: 4,
    });
    let before = runtime.clone();

    assert_eq!(
        runtime.accept_success(PcChangeMentorSuccess0104 {
            mentor: 4,
            mentor_count: 1,
            fusion_matter: 90,
        }),
        Err(GuideCorrelationError::PreviousMentorChanged {
            expected_raw_mentor: 2,
            actual_raw_mentor: 3,
        })
    );
    assert_eq!(runtime, before);
}

#[test]
fn accepted_failure_clears_pending_but_preserves_authority() {
    let mut runtime = GuideRuntime::new(GuideServerProfile::Clean0104);
    runtime.load_pc_state(&pc_load(2, 8));
    let before = runtime.authoritative();
    runtime.request_change(GuideMentor::Edd).unwrap();
    assert_eq!(
        runtime.accept_failure(PcChangeMentorFailure0104 {
            mentor: 1,
            error_code: 12,
        }),
        Ok(GuideChangeFailed {
            mentor: GuideMentor::Edd,
            error_code: 12,
        })
    );
    assert_eq!(runtime.authoritative(), before);
    assert_eq!(runtime.pending(), None);
}
