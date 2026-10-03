use super::*;

pub fn read_legacy_avatar_action_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    pointer: Res<crate::input_focus::GameplayPointerCapture>,
    mut input: ResMut<LegacyAvatarActionInput>,
) {
    // UI hit testing runs before Update. Consume the physical mouse press
    // at that boundary, not after a close action has already hidden its modal.
    // Keyboard actions remain governed by the ordinary gameplay input gate.
    *input = action_input_from_devices(
        keyboard.as_deref(),
        if pointer.blocked {
            None
        } else {
            mouse.as_deref()
        },
    );
}

pub(super) fn resolve_legacy_avatar_actions(
    time: Res<Time>,
    input: Res<LegacyAvatarActionInput>,
    cameras: Query<&LegacyOrbitCamera>,
    pointer_cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera)>,
    windows: Query<(&Window, &bevy::window::CursorOptions), With<bevy::window::PrimaryWindow>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    pointer: Res<crate::input_focus::GameplayPointerCapture>,
    mut players: Query<(
        Entity,
        &mut Transform,
        &mut LegacyPlayerController,
        &LegacyAvatarActionContext,
        &LegacyAvatarTargetFeed,
        &LegacyAvatarClipBindings,
        &mut LegacyAvatarActionState,
    )>,
    mut actions: ResMut<LegacyAvatarActionQueue>,
    mut visuals: ResMut<LegacyVisualRequestQueue>,
) {
    for (entity, mut transform, mut controller, context, feed, bindings, mut state) in &mut players
    {
        let mut selection = if context.move_mode == LegacyMoveMode::None {
            select_legacy_targets(feed, context)
        } else {
            state.target_selection.clone()
        };
        // With a visible cursor, select the body under that cursor rather than
        // the nearest player in the camera's narrow targeting cone.
        if !pointer.blocked
            && mouse
                .as_ref()
                .is_some_and(|buttons| buttons.just_pressed(MouseButton::Left))
            && let Ok((window, cursor)) = windows.single()
            && cursor.grab_mode == bevy::window::CursorGrabMode::None
            && let Some(position) = window.cursor_position()
            && let Some((camera, camera_transform, _)) = pointer_cameras
                .iter()
                .find(|(_, _, orbit)| orbit.target == entity)
        {
            selection.focused_player = None;
            if context.player_interaction_allowed && !context.attack_players_enabled {
                let selected = feed
                    .samples
                    .iter()
                    .filter(|sample| {
                        sample.kind == LegacyTargetKind::Player
                            && sample.talk_enabled
                            && crate::world_targeting::world_pc_in_interaction_range(sample.distance)
                    })
                    .filter(|sample| {
                        let bottom = Vec3::from_array(sample.position);
                        let top = bottom + Vec3::Y * sample.height;
                        let center = (bottom + top) * 0.5;
                        let (Ok(a), Ok(b), Ok(edge)) = (
                            camera.world_to_viewport(camera_transform, bottom),
                            camera.world_to_viewport(camera_transform, top),
                            camera.world_to_viewport(
                                camera_transform,
                                center + camera_transform.right() * sample.radius,
                            ),
                        ) else {
                            return false;
                        };
                        let middle = (a.x + b.x) * 0.5;
                        let half_width = (edge.x - middle).abs().max(8.0);
                        position.x >= middle - half_width
                            && position.x <= middle + half_width
                            && position.y >= a.y.min(b.y)
                            && position.y <= a.y.max(b.y)
                    })
                    .min_by(|a, b| a.distance.total_cmp(&b.distance));
                if let Some(sample) = selected {
                    selection.focused_npc = None;
                    selection.trigger = None;
                    selection.check_attack_target = false;
                    selection.focused_player = Some((*sample).into());
                }
            }
        }
        state.target_selection = selection.clone();
        let frame = schedule_legacy_action_frame(
            *input,
            context,
            &selection,
            &mut state,
            bindings,
            time.delta_secs(),
        );
        if frame.force_camera_angle
            && let Some(camera) = cameras.iter().find(|camera| camera.target == entity)
        {
            controller.yaw_degrees = camera.yaw_degrees;
            transform.rotation =
                LegacyUnityHeadingDegrees::new(camera.yaw_degrees).native_root_rotation();
        }
        actions.pending.extend(
            frame
                .actions
                .into_iter()
                .map(|intent| LegacyAvatarActionRequest {
                    actor: entity,
                    intent,
                }),
        );
        visuals.pending.extend(
            frame
                .visuals
                .into_iter()
                .map(|command| LegacyVisualRequest {
                    actor: entity,
                    command,
                }),
        );
    }
}
