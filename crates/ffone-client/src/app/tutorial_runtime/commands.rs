use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in super::super) enum TutorialNanoShortcutAction {
    Summon,
    Dismiss,
}

/// Clean `cnOwnAvatarStatus` treats a Nano shortcut as a toggle. The tutorial
/// virtual-server path has no activation packet, so an existing presentation
/// is also considered active while its call animation is still loading.
pub(in super::super) fn tutorial_nano_shortcut_action(
    locked: bool,
    equipped: bool,
    active: bool,
    presentation_present: bool,
) -> Option<TutorialNanoShortcutAction> {
    if locked || !equipped {
        return None;
    }
    Some(if active || presentation_present {
        TutorialNanoShortcutAction::Dismiss
    } else {
        TutorialNanoShortcutAction::Summon
    })
}

pub(in super::super) fn apply_tutorial_native_intent(
    intent: TutorialIntent,
    native: &mut TutorialNativeMechanics,
    presentation: &mut TutorialAuxiliaryPresentation,
) -> bool {
    match native.apply_intent(intent) {
        TutorialNativeIntentApplication::Applied => true,
        TutorialNativeIntentApplication::HideTutorialPointer => {
            presentation.cursor = None;
            presentation.cursor_touched = true;
            true
        }
        TutorialNativeIntentApplication::Unhandled => false,
    }
}
