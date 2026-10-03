use super::*;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TransportationPresentationAnimation {
    pub(super) large_phase: f32,
    pub(super) small_phase: f32,
}

pub(super) fn advance_transportation_animation(
    time: Res<Time>,
    model: Res<TransportationModel>,
    mut animation: ResMut<TransportationPresentationAnimation>,
) {
    if model.phase() != TransportationPhase::Browsing {
        return;
    }
    animation.large_phase += time.delta_secs() * 0.15;
    if animation.large_phase > 1.0 + 64.0 / TRANSPORTATION_MAP_RECT.height {
        animation.large_phase = 0.0;
    }
    animation.small_phase += time.delta_secs() * 0.5;
    if animation.small_phase > 1.0 {
        animation.small_phase = 0.0;
    }
}

pub(super) fn sync_transportation_animation(
    animation: Res<TransportationPresentationAnimation>,
    mut effects: Query<(&TransportationPresentationLineEffect, &mut Node)>,
) {
    if !animation.is_changed() {
        return;
    }
    for (effect, mut node) in &mut effects {
        node.top = if effect.0 == 0 {
            px(animation.large_phase * TRANSPORTATION_MAP_RECT.height - 64.0)
        } else {
            px((animation.small_phase * TRANSPORTATION_MAP_RECT.height)
                .min(TRANSPORTATION_MAP_RECT.height - 4.0))
        };
    }
}
