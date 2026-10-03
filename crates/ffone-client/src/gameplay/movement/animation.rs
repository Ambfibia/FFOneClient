use super::*;

pub fn update_legacy_camera_pose(
    // `cnPlayerCamera.SetSubTarget` can temporarily target a Nano/NPC mode
    // transform. Requiring the gameplay-controller component here made those
    // exact modal targets impossible even though the camera stores an Entity.
    targets: Query<&Transform, Without<LegacyOrbitCamera>>,
    mut cameras: Query<(&LegacyOrbitCamera, &mut Transform), Without<LegacyPlayerController>>,
) {
    for (camera, mut camera_transform) in &mut cameras {
        let Ok(target_transform) = targets.get(camera.target) else {
            continue;
        };
        let target = target_transform.translation + Vec3::Y * camera.height;
        if let Some(forward) = camera.sub_target_forward {
            camera_transform.translation = target - forward * camera.distance;
            camera_transform.look_at(target, Vec3::Y);
            continue;
        }
        let desired_rotation =
            legacy_camera_rotation_to_native(camera.yaw_degrees, camera.pitch_degrees);
        let rotation = if camera.force_player_angle_this_frame {
            desired_rotation
        } else {
            // cnPlayerCamera.PositionUpdate uses Quaternion.Slerp with this
            // exact sensitivity-derived factor. Unity clamps Slerp's t.
            let factor = (0.5 + camera.configurable_sensitivity * 0.1).clamp(0.0, 1.0);
            camera_transform.rotation.slerp(desired_rotation, factor)
        };
        let native_forward = rotation * Vec3::NEG_Z;
        let ideal_position = target - native_forward * camera.distance;

        // cnPlayerCamera always applies its final inward correction, even when
        // the raycast does not hit. Native geometry raycasts are not in this
        // slice yet, but preserving this no-hit one-unit quirk is deterministic.
        let inward_distance = 1.0_f32.min((camera.distance - 0.1).max(0.0));
        camera_transform.translation =
            ideal_position + (target - ideal_position).normalize_or_zero() * inward_distance;
        camera_transform.rotation = rotation;
        camera_transform.look_at(target, Vec3::Y);
    }
}
