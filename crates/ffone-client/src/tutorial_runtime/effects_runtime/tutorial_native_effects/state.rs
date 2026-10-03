use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum NativePreloadHandleState {
    Pending,
    Complete,
    Failed(String),
}

pub(super) fn native_preload_handle_state<A: Asset>(
    asset_server: &AssetServer,
    handle: &Handle<A>,
    label: &str,
) -> NativePreloadHandleState {
    if asset_server.is_loaded_with_dependencies(handle.id()) {
        return NativePreloadHandleState::Complete;
    }
    if let LoadState::Failed(error) = asset_server.load_state(handle.id()) {
        return NativePreloadHandleState::Failed(format!("{label} load failed: {error}"));
    }
    if let Some(RecursiveDependencyLoadState::Failed(error)) =
        asset_server.get_recursive_dependency_load_state(handle.id())
    {
        return NativePreloadHandleState::Failed(format!(
            "{label} dependency load failed: {error}"
        ));
    }
    NativePreloadHandleState::Pending
}

pub(super) fn legacy_particle_initial_state(
    plan: &EmitterPlan,
    random: &mut NativeParticleRandomStream,
) -> (Vec3, Vec3) {
    let (unity_position, unity_velocity) = legacy_particle_initial_state_in_unity(plan, random);
    (
        unity_to_native_vector(unity_position),
        unity_to_native_vector(unity_velocity),
    )
}

pub(super) fn script_state(keys: &[ScriptKey], time: f32) -> (Vec3, bool) {
    if time <= keys[0].time {
        return (keys[0].translate, keys[0].emit);
    }
    for pair in keys.windows(2) {
        if time <= pair[1].time {
            let span = pair[1].time - pair[0].time;
            let factor = if span <= f32::EPSILON {
                1.0
            } else {
                (time - pair[0].time) / span
            };
            return (
                pair[0].translate.lerp(pair[1].translate, factor),
                pair[0].emit,
            );
        }
    }
    let last = *keys.last().unwrap();
    (last.translate, last.emit)
}
