use super::*;

pub(super) fn tutorial_actor_native_combat_blend(role: LegacyNpcAnimationRole) -> LegacyAnimationBlend {
    if role.is_additive() {
        // Delta-ready packages use Bevy's Add node; packages which have not
        // completed that publication gate retain the safe full-body adapter.
        // Both preserve Retrobution's short high-layer transition timing.
        LegacyAnimationBlend::CrossFade100Ms
    } else {
        LegacyAnimationBlend::CrossFade300Ms
    }
}
