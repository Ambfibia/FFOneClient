use super::*;

pub(super) fn animation_event_crossings(
    previous_seek: f32,
    previous_completions: u32,
    current_seek: f32,
    current_completions: u32,
    repeat: RepeatAnimation,
    event_seconds: f32,
) -> u32 {
    if !previous_seek.is_finite()
        || !current_seek.is_finite()
        || !event_seconds.is_finite()
        || event_seconds < 0.0
        || current_completions < previous_completions
    {
        return 0;
    }
    let completion_delta = current_completions - previous_completions;
    if completion_delta == 0 {
        return u32::from(previous_seek < event_seconds && event_seconds <= current_seek);
    }
    if repeat != RepeatAnimation::Forever {
        return u32::from(previous_seek < event_seconds);
    }

    u32::from(previous_seek < event_seconds)
        .saturating_add(completion_delta.saturating_sub(1))
        .saturating_add(u32::from(event_seconds <= current_seek))
}
