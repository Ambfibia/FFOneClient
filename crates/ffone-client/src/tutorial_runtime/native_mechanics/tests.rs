use crate::tutorial::{MissionStage, NanoPowerStage};
use crate::tutorial_native_mechanics::*;

#[test]
fn stable_stage_metadata_seeds_all_nineteen_live_lock_entries() {
    let stage = TutorialStage::Mission(MissionStage::OpenMenu);
    let mut mechanics = TutorialNativeMechanics::default();
    mechanics.sync_stable_stage(stage);

    assert_eq!(mechanics.bound_stage(), Some(stage));
    for index in 0..TUTORIAL_INPUT_LOCK_COUNT {
        let expected_locked = stage.metadata().input.allowed_bits() & (1_u32 << index) == 0;
        assert_eq!(mechanics.live_input_locks()[index], expected_locked);
    }
    assert!(!mechanics.is_locked(TutorialInputLock::Menu));
    assert!(!mechanics.is_locked(TutorialInputLock::Journal));
    assert!(mechanics.is_locked(TutorialInputLock::Attack));
}

#[test]
fn lock_ui_writes_only_legacy_indices_sixteen_and_eight() {
    let mut mechanics = TutorialNativeMechanics::default();
    mechanics.sync_stable_stage(TutorialStage::Mission(MissionStage::CloseReward));
    let before = *mechanics.live_input_locks();

    assert_eq!(
        mechanics.apply_intent(TutorialIntent::LockUi),
        TutorialNativeIntentApplication::Applied
    );
    for index in 0..TUTORIAL_INPUT_LOCK_COUNT {
        let expected = if index == TutorialInputLock::ModeChange.index()
            || index == TutorialInputLock::Menu.index()
        {
            true
        } else {
            before[index]
        };
        assert_eq!(mechanics.live_input_locks()[index], expected);
    }

    mechanics.apply_intent(TutorialIntent::UnlockUi);
    assert!(!mechanics.is_locked(TutorialInputLock::ModeChange));
    assert!(!mechanics.is_locked(TutorialInputLock::Menu));
}

#[test]
fn push_and_pop_copy_and_restore_all_nineteen_entries_across_stage_rebinds() {
    let mut mechanics = TutorialNativeMechanics::default();
    mechanics.sync_stable_stage(TutorialStage::Mission(MissionStage::OpenMenu));
    let saved = *mechanics.live_input_locks();
    mechanics.apply_intent(TutorialIntent::PushInputFilter);
    assert_eq!(mechanics.saved_input_locks(), &saved);

    mechanics.sync_stable_stage(TutorialStage::NanoPower(NanoPowerStage::SummonNano));
    mechanics.apply_intent(TutorialIntent::UnlockUi);
    assert_ne!(mechanics.live_input_locks(), &saved);
    mechanics.apply_intent(TutorialIntent::PopInputFilter);

    for (index, expected) in saved.into_iter().enumerate() {
        assert_eq!(mechanics.live_input_locks()[index], expected);
    }
}

#[test]
fn all_eight_reference_native_intents_are_typed_and_unknowns_are_unhandled() {
    let mut mechanics = TutorialNativeMechanics::default();
    mechanics.sync_stable_stage(TutorialStage::Mission(MissionStage::OpenMenu));
    mechanics.mark_tutorial_pointer_visible();
    let intents = [
        TutorialIntent::SetFatigueLevel(1),
        TutorialIntent::SetInstanceMap(true),
        TutorialIntent::SetEpisode(0),
        TutorialIntent::LockUi,
        TutorialIntent::UnlockUi,
        TutorialIntent::PushInputFilter,
        TutorialIntent::PopInputFilter,
        TutorialIntent::HideTutorialPointer,
    ];
    for intent in intents {
        assert_ne!(
            mechanics.apply_intent(intent),
            TutorialNativeIntentApplication::Unhandled,
            "reference intent {intent:?} was not consumed"
        );
    }
    assert_eq!(mechanics.fatigue_level(), 1);
    assert!(mechanics.instance_map());
    assert_eq!(mechanics.episode(), 0);
    assert!(!mechanics.tutorial_pointer_visible());
    assert_eq!(
        mechanics.apply_intent(TutorialIntent::SetFusionMatter(7)),
        TutorialNativeIntentApplication::Unhandled
    );
}

#[test]
fn missing_stage_is_fail_closed_and_reset_clears_local_state_and_snapshot() {
    let mut mechanics = TutorialNativeMechanics::default();
    assert!(mechanics.live_input_locks().iter().all(|locked| *locked));
    mechanics.sync_stable_stage(TutorialStage::Mission(MissionStage::OpenMenu));
    mechanics.apply_intent(TutorialIntent::PushInputFilter);
    mechanics.apply_intent(TutorialIntent::SetFatigueLevel(1));
    mechanics.apply_intent(TutorialIntent::SetInstanceMap(true));
    mechanics.mark_tutorial_pointer_visible();

    mechanics.reset();
    assert_eq!(mechanics, TutorialNativeMechanics::default());
    assert!(mechanics.live_input_locks().iter().all(|locked| *locked));
    assert!(mechanics.saved_input_locks().iter().all(|locked| !*locked));
}
