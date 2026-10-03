use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkNpcAnimationSoundEvent0104 {
    pub clip: String,
    pub time: f32,
    pub payload: String,
}

pub fn parse_network_npc_animation_sound_events(
    bytes: &[u8],
) -> Result<Vec<NetworkNpcAnimationSoundEvent0104>, String> {
    let document = parse_network_npc_animation_document(bytes)?;
    let animations = document
        .get("animations")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let mut parsed = Vec::new();
    for (animation_index, animation) in animations.iter().enumerate() {
        let context = format!("animations[{animation_index}]");
        let clip = required_string(animation, "name", &context)?;
        let Some(events) = animation
            .pointer("/extras/nonTrs/events")
            .and_then(Value::as_array)
        else {
            continue;
        };
        for (event_index, event) in events.iter().enumerate() {
            let event_context = format!("{context}.events[{event_index}]");
            if required_string(event, "functionName", &event_context)? != "sound" {
                continue;
            }
            let payload = required_string(event, "stringParameter", &event_context)?;
            if payload.trim().is_empty() {
                // Some clean GLBs serialize an empty `sound` AnimationEvent.
                // AssetLoader cannot resolve it, so Retrobution produces no
                // source; retaining it would only enqueue the same no-op.
                continue;
            }
            let seconds = event
                .get("time")
                .and_then(Value::as_f64)
                .ok_or_else(|| format!("{event_context}.time must be numeric"))?;
            if !seconds.is_finite() || seconds < 0.0 || seconds > f64::from(f32::MAX) {
                return Err(format!(
                    "{event_context}.time must be finite and non-negative"
                ));
            }
            parsed.push(NetworkNpcAnimationSoundEvent0104 {
                clip: clip.to_owned(),
                time: seconds as f32,
                payload: payload.to_owned(),
            });
        }
    }
    Ok(parsed)
}

#[derive(Clone, Debug, PartialEq, Component)]
pub(super) struct NetworkNpcAnimationSoundCursor0104 {
    pub(super) root: Entity,
    pub(super) clip: String,
    pub(super) combat_revision: Option<u64>,
    pub(super) node: AnimationNodeIndex,
    pub(super) seek_time: f32,
    pub(super) completions: u32,
}

#[derive(Default, Component)]
pub(super) struct NetworkNpcAnimationSoundCursors0104(pub(super) [Option<NetworkNpcAnimationSoundCursor0104>; 3]);

/// Tracks sound events on the currently applied modular HNPC animation.
#[derive(Component)]
pub(super) struct NetworkHnpcAnimationSoundCursor0104 {
    pub(super) revision: u64,
    pub(super) seek_time: f32,
    pub(super) completions: u32,
}

pub(super) fn network_npc_sound_ancestor<'a>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    roots: &'a Query<(
        Entity,
        &NetworkNpc0104,
        &NetworkNpcVisual0104,
        &GlobalTransform,
    )>,
) -> Option<(
    Entity,
    &'a NetworkNpc0104,
    &'a NetworkNpcVisual0104,
    &'a GlobalTransform,
)> {
    loop {
        if let Ok(root) = roots.get(entity) {
            return Some(root);
        }
        entity = parents.get(entity).ok()?.parent();
    }
}
