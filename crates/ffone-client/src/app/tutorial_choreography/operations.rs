use super::*;

pub(in super::super) fn tutorial_blocking_wait_is_resolved(
    wait: BlockingWait,
    effect_runtime: &TutorialEffectRuntime,
) -> bool {
    match wait {
        BlockingWait::AssetPreload { effect_id, .. } => {
            effect_runtime.is_native_preload_complete(effect_id)
        }
        BlockingWait::EffectInstantiationRetry { instance_name, .. } => {
            effect_runtime.has_named_native_instance(instance_name)
        }
    }
}

pub(in super::super) fn tutorial_named_world_effect(
    source_line: u32,
    effect_id: i32,
) -> Option<&'static str> {
    match (source_line, effect_id) {
        // `NanoPower_Event_B` keeps both return values so its retry loops can
        // test them. The ES739 reference survives until the explicit
        // `Destroy(gameObject)` at line 4867.
        (4697 | 4841, 739) => Some("Dexter hologram effect 739"),
        (4701 | 4711, 741) => Some("portal effect 741"),
        _ => None,
    }
}

pub(in super::super) fn choreography_client_vector(value: ClientVec3) -> Vec3 {
    unity_to_native_vector(Vec3::new(value.x, value.y, value.z))
}

pub(in super::super) fn choreography_screen_point_to_auxiliary(
    point: ffone_client::tutorial_choreography::TutorialScreenPoint,
) -> TutorialScreenPoint {
    fn axis(axis: ChoreographyScreenAxis) -> AuxiliaryScreenAxis {
        match axis {
            ChoreographyScreenAxis::Pixels(value) => AuxiliaryScreenAxis::Pixels(value),
            ChoreographyScreenAxis::WidthMinus(value) => AuxiliaryScreenAxis::WidthMinus(value),
            ChoreographyScreenAxis::HeightMinus(value) => AuxiliaryScreenAxis::HeightMinus(value),
        }
    }
    TutorialScreenPoint::new(axis(point.x), axis(point.y))
}

pub(in super::super) fn tutorial_nano_choreography_transform(mut native_root: Transform) -> Transform {
    native_root.rotation =
        (native_root.rotation * native_model_forward_child_rotation()).normalize();
    native_root
}

pub(in super::super) fn choreography_character_root_rotation(model_rotation: Quat) -> Quat {
    (model_rotation * native_model_forward_child_rotation()).normalize()
}

pub(in super::super) fn tutorial_scene_angle_heading(angle: i32) -> LegacyUnityHeadingDegrees {
    LegacyUnityHeadingDegrees::new(angle as f32)
}

pub(in super::super) fn tutorial_player_heading_toward(
    transform: &Transform,
    target: Vec3,
) -> Option<LegacyUnityHeadingDegrees> {
    let direction = target - transform.translation;
    if direction.length_squared() <= f32::EPSILON {
        return None;
    }
    let unity = native_to_unity_vector(direction);
    let heading = unity.x.atan2(unity.z).to_degrees();
    Some(LegacyUnityHeadingDegrees::new(heading))
}

pub(in super::super) fn tutorial_cinematic_turn_rotation(
    current: Quat,
    target: Quat,
    delta_seconds: f32,
) -> Quat {
    current.slerp(target, (delta_seconds * 4.0).clamp(0.0, 1.0))
}
