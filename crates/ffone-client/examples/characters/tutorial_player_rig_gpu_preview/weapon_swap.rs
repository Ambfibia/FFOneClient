use super::*;

/// Change two actual hand attachments, then detach through the gameplay queue.
pub(super) fn drive_weapon_swap(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    mut presentation: ResMut<TutorialPlayerPresentationCommandQueue>,
    rigs: Query<&TutorialSelectedPlayerRigStatus>,
    weapons: Query<&TutorialPlayerWeaponAttachment>,
    applied: Query<&TutorialPlayerAnimationApplied>,
    mut exit: MessageWriter<AppExit>,
) {
    if config.case != PreviewCase::WeaponSwap {
        return;
    }
    if state.frames >= TIMEOUT_FRAMES {
        eprintln!("weapon swap timed out at stage {}", state.weapon_stage);
        exit.write(AppExit::error());
        return;
    }
    let Ok(TutorialSelectedPlayerRigStatus::Ready) = rigs.get(config.rig_root) else {
        return;
    };
    let elapsed = state.frames.saturating_sub(state.weapon_step_frame);
    let has_weapon = |id| {
        weapons
            .iter()
            .any(|weapon| weapon.rig_root == config.rig_root && weapon.item_id == id)
    };
    match state.weapon_stage {
        0 => {
            presentation.tutorial_weapon(true, 1).unwrap(); // item 328
            state.weapon_stage = 1;
            state.weapon_step_frame = state.frames;
        }
        1 if elapsed > 120 && has_weapon(328) => {
            presentation.tutorial_weapon(false, 2).unwrap(); // item 43
            state.weapon_stage = 2;
            state.weapon_step_frame = state.frames;
        }
        2 if elapsed > 120 && has_weapon(43) && !has_weapon(328) => {
            state.capture_issued = true;
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_screenshot);
            state.weapon_stage = 3;
        }
        3 if state.capture_saved => {
            let mut detach =
                ffone_client::tutorial_player_presentation::tutorial_weapon_request(true, 1)
                    .unwrap();
            detach.item = EquippedItem0104::default();
            presentation.push_equipment(detach);
            state.weapon_stage = 4;
            state.weapon_step_frame = state.frames;
        }
        4 if elapsed > 120
            && !weapons
                .iter()
                .any(|weapon| weapon.rig_root == config.rig_root) =>
        {
            let pose = applied.get(config.rig_root).expect("unarmed pose");
            assert_eq!(pose.clip, TutorialPlayerClip::Stand1);
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_dismount_screenshot);
            state.weapon_stage = 5;
        }
        5 if state.dismount_capture_saved => {
            println!("weapon swap proof: 328 -> 43 -> unarmed/stand1");
            exit.write(AppExit::Success);
        }
        _ => {}
    }
    std::thread::sleep(Duration::from_millis(16));
}
