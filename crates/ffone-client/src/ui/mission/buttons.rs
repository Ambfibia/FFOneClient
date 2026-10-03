//! Mission UI button handling and tutorial control locks.

use super::NpcInteractionPressGuard;
use super::components::MissionUiControl;
use super::journal::current_mission;
use super::layout::journal_right_layout;
use super::model::{JournalListTab, MissionJournalUi, MissionUiModel};
use super::nanocom::{
    mission_ui_control_is_nanocom_menu_row, mission_ui_control_uses_nanocom_surface,
};
use crate::{
    gameplay_ui::{GameplayUiAudioCue, GameplayUiAudioOutbox, GameplayUiOutbox},
    tutorial::TutorialInputLock,
    tutorial_native_mechanics::TutorialNativeMechanics,
};
use bevy::prelude::*;

pub(super) fn menu_interaction_state(interaction: &Interaction) -> &'static str {
    match interaction {
        Interaction::None => "normal",
        Interaction::Hovered => "hover",
        Interaction::Pressed => "active",
    }
}

pub(super) fn handle_mission_ui_buttons(
    controls: Query<(&Interaction, &MissionUiControl), Changed<Interaction>>,
    primary_mouse: Option<Res<ButtonInput<MouseButton>>>,
    native: Res<TutorialNativeMechanics>,
    mut model: ResMut<MissionUiModel>,
    mut outbox: ResMut<GameplayUiOutbox>,
    mut audio: ResMut<GameplayUiAudioOutbox>,
    mut npc_press_guard: Local<NpcInteractionPressGuard>,
) {
    let visible_npc_id = (model.enabled && model.npc_icon_mode_visible)
        .then(|| model.npc_interaction.as_ref().map(|npc| npc.npc_id))
        .flatten();
    if npc_press_guard.npc_id != visible_npc_id {
        npc_press_guard.npc_id = visible_npc_id;
        // NpcIconMode can become visible on the frame after the world click
        // which queued TalkNpc. If that button is still held, Bevy may mark a
        // newly revealed row Pressed and accidentally accept it with the same
        // physical click. The clean IMGUI event is consumed by the talk edge,
        // so arm the new NPC menu only after that press has been released.
        npc_press_guard.wait_for_primary_release = visible_npc_id.is_some()
            && primary_mouse
                .as_ref()
                .is_some_and(|mouse| mouse.pressed(MouseButton::Left));
    }
    if npc_press_guard.wait_for_primary_release
        && primary_mouse
            .as_ref()
            .is_none_or(|mouse| !mouse.pressed(MouseButton::Left))
    {
        npc_press_guard.wait_for_primary_release = false;
    }

    for (interaction, control) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if npc_press_guard.wait_for_primary_release
            && matches!(
                *control,
                MissionUiControl::NpcMissionRow(_)
                    | MissionUiControl::NpcUtility(_)
                    | MissionUiControl::NpcClose
            )
        {
            continue;
        }
        if tutorial_mission_control_is_locked(*control, &native) {
            // `cnGUINanocom.RenderMenu` plays the click before each callback
            // checks its own eLock. NpcIcon/journal controls put the lock in
            // the GUI.Button condition and therefore remain silent here.
            if mission_ui_control_is_nanocom_menu_row(*control) {
                audio.push(GameplayUiAudioCue::ButtonSound);
            }
            continue;
        }
        if model.nanocom_foreign_modal_suppressed()
            && mission_ui_control_uses_nanocom_surface(*control)
        {
            continue;
        }
        match *control {
            MissionUiControl::NpcMissionRow(slot) => {
                if model.select_npc_mission(slot, &mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::NpcUtility(slot) => {
                if model.activate_npc_utility(slot, &mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::NpcClose => {
                // The button's Mouse gate and EndMode's ModeChange gate are
                // distinct: a locked EndMode may click but cannot close/speak.
                if native.bound_stage().is_some() && native.is_locked(TutorialInputLock::ModeChange)
                {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                    continue;
                }
                if model.close_npc_interaction(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::JournalPrimary => {
                if matches!(model.journal, MissionJournalUi::Allow(_)) {
                    if model.accept_mission(&mut outbox) {
                        audio.push(GameplayUiAudioCue::MissionAccepted);
                    }
                } else {
                    model.complete_mission(&mut outbox);
                }
            }
            MissionUiControl::JournalClose => {
                if model.close_journal(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::JournalRewardIcon => {
                audio.push(GameplayUiAudioCue::ButtonSound);
            }
            MissionUiControl::JournalMissionRow(row) => {
                if model.select_journal_mission(row) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::JournalTrackMission(row) => {
                if model.pending.is_none()
                    && model.journal_tab == JournalListTab::Active
                    && matches!(model.journal, MissionJournalUi::Other(_))
                    && let Some(task_id) = journal_right_layout(&model)
                        .rows
                        .get(row)
                        .map(|layout| layout.mission.task_id)
                {
                    // Tracking is independent of the mission open in the detail panel.
                    model.selected_journal_task_id = Some(task_id);
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::JournalMakeCurrent => {
                if model.pending.is_none() && model.journal_tab == JournalListTab::Active {
                    model.selected_journal_task_id = model
                        .viewed_journal_task_id
                        .or_else(|| current_mission(&model).map(|mission| mission.task_id));
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::JournalActiveTab => {
                if model.journal_tab != JournalListTab::Active
                    && model.select_journal_tab(JournalListTab::Active)
                {
                    audio.push(GameplayUiAudioCue::TabClick01);
                }
            }
            MissionUiControl::JournalCompletedTab => {
                if model.journal_tab != JournalListTab::Completed
                    && model.select_journal_tab(JournalListTab::Completed)
                {
                    audio.push(GameplayUiAudioCue::TabClick01);
                }
            }
            MissionUiControl::JournalCategoryToggle(section) => {
                if model.toggle_completed_category(section) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::JournalHelp => {
                audio.push(GameplayUiAudioCue::ButtonSound);
            }
            MissionUiControl::JournalSecondary => {
                if matches!(model.journal, MissionJournalUi::Allow(_)) {
                    if model.close_journal(&mut outbox) {
                        audio.push(GameplayUiAudioCue::MissionDecline);
                    }
                } else if matches!(model.journal, MissionJournalUi::Reward { .. }) {
                    if model.close_journal(&mut outbox) {
                        audio.push(GameplayUiAudioCue::ButtonSound);
                    }
                } else if matches!(model.journal, MissionJournalUi::Other(_))
                    && native.bound_stage().is_none()
                    && model.journal_tab == JournalListTab::Active
                    && model.request_task_stop_confirmation(&mut outbox)
                {
                    audio.push(GameplayUiAudioCue::AbandonMission);
                }
            }
            MissionUiControl::NanocomMyStuff => {
                if model.open_user_equip_from_nanocom(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                    audio.push(GameplayUiAudioCue::CloseScreen);
                }
            }
            MissionUiControl::NanocomJournal => {
                let journal = model.nanocom_journal.clone();
                if model.open_journal_from_nanocom(journal, &mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                    audio.push(GameplayUiAudioCue::CloseScreen);
                }
            }
            MissionUiControl::NanocomEmail => {
                if model.open_email_from_nanocom(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                    audio.push(GameplayUiAudioCue::CloseScreen);
                }
            }
            MissionUiControl::NanocomMap => {
                if model.open_world_map_from_nanocom(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                    audio.push(GameplayUiAudioCue::CloseScreen);
                }
            }
            MissionUiControl::NanocomSettings => {
                if model.open_option_from_nanocom(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                    audio.push(GameplayUiAudioCue::CloseScreen);
                }
            }
            MissionUiControl::NanocomGameGuide => {
                if model.open_game_guide_from_nanocom(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::NanocomExitGame => {
                if model.open_quit_from_nanocom(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                }
            }
            MissionUiControl::NanocomClose => {
                if model.close_nanocom_menu(&mut outbox) {
                    audio.push(GameplayUiAudioCue::ButtonSound);
                    audio.push(GameplayUiAudioCue::CloseScreen);
                }
            }
            MissionUiControl::WarpAway => {
                model.begin_warp_away(&mut outbox);
            }
            MissionUiControl::SystemDialogOkay => {
                if model.confirm_tutorial_exit_dialog(&mut outbox) {
                    audio.push(GameplayUiAudioCue::YesButton);
                }
            }
            MissionUiControl::SystemDialogCancel => {
                if model.cancel_tutorial_exit_dialog() {
                    audio.push(GameplayUiAudioCue::NoButton);
                }
            }
        }
    }
}

pub(super) fn tutorial_mission_control_is_locked(
    control: MissionUiControl,
    native: &TutorialNativeMechanics,
) -> bool {
    // Outside the local tutorial the native lock array is intentionally
    // unbound/fail-closed and must not disable future world NPC UI.
    if native.bound_stage().is_none() {
        return false;
    }
    match control {
        // These exact GUI.Button sites check `!IsLock(Mouse)` in
        // NpcIconMode/cnMissionJournal.
        MissionUiControl::NpcMissionRow(_)
        | MissionUiControl::NpcUtility(_)
        | MissionUiControl::NpcClose
        | MissionUiControl::JournalPrimary
        | MissionUiControl::JournalMissionRow(_)
        | MissionUiControl::JournalTrackMission(_)
        | MissionUiControl::JournalActiveTab
        | MissionUiControl::JournalCompletedTab
        | MissionUiControl::JournalCategoryToggle(_)
        | MissionUiControl::JournalHelp
        | MissionUiControl::JournalMakeCurrent => native.is_locked(TutorialInputLock::Mouse),
        // Clean `cnGUINanocom.RenderMenu` always renders live GUI.Button rows.
        // Each callback gates itself only with its specific eLock; eLock.Mouse
        // controls the avatar camera and is not an additional Nanocom gate.
        MissionUiControl::NanocomMyStuff
        | MissionUiControl::NanocomEmail
        | MissionUiControl::NanocomGameGuide => native.is_locked(TutorialInputLock::Inventory),
        MissionUiControl::NanocomJournal => native.is_locked(TutorialInputLock::Journal),
        MissionUiControl::NanocomMap => native.is_locked(TutorialInputLock::WorldMap),
        MissionUiControl::NanocomSettings | MissionUiControl::NanocomExitGame => {
            native.is_locked(TutorialInputLock::Option)
        }
        // cnMissionJournal.EndMode is guarded by the mode-change lock.
        MissionUiControl::JournalClose => native.is_locked(TutorialInputLock::ModeChange),
        MissionUiControl::JournalRewardIcon
        | MissionUiControl::JournalSecondary
        | MissionUiControl::NanocomClose
        | MissionUiControl::SystemDialogOkay
        | MissionUiControl::SystemDialogCancel => false,
        // Clean `MenuChatScript.OnOwnGUI` hard-disables Warp Away throughout
        // the tutorial, independent of the tutorial input-lock array.
        MissionUiControl::WarpAway => true,
    }
}
