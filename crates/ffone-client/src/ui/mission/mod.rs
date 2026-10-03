//! Shared Retrobution mission/Nanocom/NPC-icon UI contract.
//!
//! State transitions and reference-space rectangles are ports of
//! `cntutorialscript`, `cnMissionJournal`, `cnGUINanocom`, and `NpcIconMode`.
//! Asset routes, style borders, and reward-card geometry are recovered from
//! `cnMissionJournal` plus the serialized `FusionFallMissionSkin` assets.

use bevy::prelude::*;

use crate::{
    gameplay_ui::{
        GAMEPLAY_UI_CAMERA_ORDER, GameplayMenuTransition, GameplayUiAudioOutbox, GameplayUiOutbox,
    },
    localization::LocalizationSet,
    tutorial_native_mechanics::TutorialNativeMechanics,
};

mod assets;
mod binding;
mod buttons;
mod components;
mod geometry;
mod journal;
mod journal_spawn;
mod labels;
mod layout;
mod model;
mod nanocom;
mod npc_interaction;
mod requests;
mod system_dialog;
mod warp_away;
mod widgets;
use assets::MissionUiAssets;
use binding::{bind_mission_ui, update_mission_ui_layout};
use buttons::handle_mission_ui_buttons;
pub use geometry::{
    CHAT_EMOTE_CONTAINER_HEIGHT, CHAT_QUICK_GROUP_RECT, CHAT_QUICK_MENU_RECT,
    CHAT_QUICK_VEHICLE_RECT, CHAT_QUICK_WARP_RECT, CHAT_WINDOW_HEIGHT, JOURNAL_ACCEPT_RECT,
    JOURNAL_ACTIVE_DELETE_RECT, JOURNAL_ACTIVE_GROUP_RECT, JOURNAL_ACTIVE_MAKE_CURRENT_RECT,
    JOURNAL_ACTIVE_TAB_RECT, JOURNAL_ALLOW_GROUP_RECT, JOURNAL_ALLOW_SECONDARY_RECT,
    JOURNAL_CLOSE_RECT, JOURNAL_COMPLETE_RECT, JOURNAL_COMPLETED_BANNER_RECT,
    JOURNAL_COMPLETED_GROUP_RECT, JOURNAL_COMPLETED_MISSION_DIFFICULTY_RECT,
    JOURNAL_COMPLETED_MISSION_TITLE_RECT, JOURNAL_COMPLETED_NANO_GROUP_RECT,
    JOURNAL_COMPLETED_NANO_LABEL_RECT, JOURNAL_COMPLETED_NOTES_HEADER_RECT,
    JOURNAL_COMPLETED_NOTES_RECT, JOURNAL_COMPLETED_NPC_FRAME_RECT,
    JOURNAL_COMPLETED_NPC_NAME_RECT, JOURNAL_COMPLETED_NPC_POSITION_RECT,
    JOURNAL_COMPLETED_REWARD_HEADER_RECT, JOURNAL_COMPLETED_REWARD_VIEW_RECT,
    JOURNAL_COMPLETED_TAB_RECT, JOURNAL_DESCRIPTION_RECT, JOURNAL_FM_AMOUNT_RECT,
    JOURNAL_FM_CAPTION_RECT, JOURNAL_FM_CARD_RECT, JOURNAL_FM_ICON_RECT, JOURNAL_HELP_RECT,
    JOURNAL_LEFT_BOX_RECT, JOURNAL_MISSION_DIFFICULTY_RECT, JOURNAL_MISSION_TITLE_RECT,
    JOURNAL_NPC_FRAME_RECT, JOURNAL_NPC_NAME_RECT, JOURNAL_NPC_POSITION_RECT,
    JOURNAL_OBJECTIVE_HEADER_RECT, JOURNAL_REWARD_GROUP_RECT, JOURNAL_REWARD_HEADER_RECT,
    JOURNAL_RIGHT_BOX_RECT, JOURNAL_SCROLL_VIEW_RECT, JOURNAL_TAROS_AMOUNT_RECT,
    JOURNAL_TAROS_CAPTION_RECT, JOURNAL_TAROS_CARD_RECT, JOURNAL_TAROS_ICON_RECT,
    JOURNAL_WINDOW_RECT, NANOCOM_CLOSE_RECT, NANOCOM_JOURNAL_RECT, NANOCOM_MENU_RECT,
    NPC_CLOSE_RECT, NPC_SINGLE_MISSION_RECT, NPC_WARP_AND_CLOSE_RECT, SYSTEM_DIALOG_CANCEL_RECT,
    SYSTEM_DIALOG_CONTENT_RECT, SYSTEM_DIALOG_ICON_RECT, SYSTEM_DIALOG_OKAY_RECT,
    SYSTEM_DIALOG_OVERLAY_ALPHA, SYSTEM_DIALOG_PANEL_RECT,
};
use journal::{
    bind_journal_aux_button_hover, bind_journal_category_toggle_hover, bind_journal_close_hover,
    bind_journal_content_layout, bind_journal_mission_button_hover,
    bind_journal_nano_portrait_image, bind_journal_primary_hover, bind_journal_tab_hover,
    scroll_mission_journal, sync_journal_nano_portrait_request,
};
use journal_spawn::spawn_journal;
pub use layout::MissionUiRect;
use layout::{NpcMissionRowHeights, measure_npc_mission_rows};
pub(crate) use model::gameplay_chrome_visible;
pub use model::{
    JournalListTab, JournalOtherUi, MissionJournalUi, MissionUiEntry, MissionUiModel,
    MissionUiRewards, NpcInteractionUi, NpcServiceUiEntry, WarpUiEntry,
};
use nanocom::{
    bind_chat_quick_button_hover, bind_chat_quick_warp_state, bind_nanocom_button_hover,
    bind_nanocom_close_hover, spawn_chat_quick_menu, spawn_nanocom, toggle_nanocom_with_enter,
};
use npc_interaction::{
    bind_mission_button_hover, bind_npc_action_button_hover, spawn_npc_letterbox, spawn_npc_quest,
    spawn_npc_warp,
};
pub use requests::{MissionRequestCodecError0104, PendingMissionUiRequest};
pub use system_dialog::{
    TUTORIAL_EXIT_KEY, TUTORIAL_EXIT_SYSTEM_MESSAGE_BUTTON_TYPE, TUTORIAL_EXIT_SYSTEM_MESSAGE_ID,
    TUTORIAL_EXIT_SYSTEM_MESSAGE_TEXT, TutorialExitDialogGate, TutorialSystemDialogUi,
};
use system_dialog::{bind_system_dialog_button_hover, spawn_system_dialog};
pub use warp_away::{WARP_AWAY_COOLDOWN_SECONDS, WARP_AWAY_DELAY_SECONDS};
use warp_away::{advance_warp_away, spawn_warp_away_countdown};
use widgets::align_mission_text_vertically;

#[derive(Default)]
pub struct MissionUiPlugin;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum MissionUiSet {
    /// Pointer actions that commit NpcIcon/journal state and enqueue
    /// `GameplayUiAction` handoffs.
    Interaction,
    Presentation,
}

#[derive(Default)]
struct NpcInteractionPressGuard {
    npc_id: Option<i32>,
    wait_for_primary_release: bool,
}

#[derive(Component)]
pub struct MissionUiRoot;

impl Plugin for MissionUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<MissionUiModel>()
            .init_resource::<NpcMissionRowHeights>()
            .add_systems(Update, measure_npc_mission_rows.before(update_mission_ui_layout).before(bind_mission_ui))
            .init_resource::<TutorialNativeMechanics>()
            .init_resource::<GameplayMenuTransition>()
            .init_resource::<GameplayUiOutbox>()
            .init_resource::<GameplayUiAudioOutbox>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_mission_ui)
            .add_systems(
                Update,
                ((
                    sync_journal_nano_portrait_request,
                    toggle_nanocom_with_enter,
                    handle_mission_ui_buttons.in_set(MissionUiSet::Interaction),
                    (advance_warp_away, sync_gameplay_menu_transition_target).chain(),
                    update_mission_ui_layout,
                    (scroll_mission_journal, bind_journal_content_layout).chain(),
                    (
                        bind_mission_ui
                            .in_set(MissionUiSet::Presentation)
                            .before(LocalizationSet::Apply),
                        align_mission_text_vertically,
                    )
                        .chain(),
                    bind_journal_nano_portrait_image,
                    bind_mission_button_hover,
                    bind_npc_action_button_hover,
                    bind_journal_mission_button_hover,
                    bind_journal_close_hover,
                    bind_journal_primary_hover,
                    bind_journal_aux_button_hover,
                    bind_journal_tab_hover,
                    bind_journal_category_toggle_hover,
                    bind_nanocom_button_hover,
                    (bind_chat_quick_warp_state, bind_chat_quick_button_hover).chain(),
                    bind_nanocom_close_hover,
                    bind_system_dialog_button_hover,
                )
                    .chain())
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

fn spawn_mission_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = MissionUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            ZIndex(GAMEPLAY_UI_CAMERA_ORDER as i32 + 10),
            Pickable::IGNORE,
            MissionUiRoot,
        ))
        .with_children(|root| {
            spawn_npc_letterbox(root, &assets);
            spawn_npc_quest(root, &assets);
            spawn_npc_warp(root, &assets);
            spawn_journal(root, &assets);
            spawn_nanocom(root, &assets);
            spawn_chat_quick_menu(root, &assets);
            spawn_warp_away_countdown(root, &assets);
        });
    // `cnSystemMessageManager` renders at GUI.depth 0, above the tutorial's
    // depth 5 surface. Keep it outside MissionUiRoot so no tutorial
    // arrow or instruction can sort over the modal.
    spawn_system_dialog(&mut commands, &assets);
}

fn sync_gameplay_menu_transition_target(
    model: Res<MissionUiModel>,
    mut transition: ResMut<GameplayMenuTransition>,
) {
    transition.set_open(model.nanocom_menu_presented());
}

#[cfg(test)]
#[path = "../../mission_ui_warp_input_tests.rs"]
mod warp_input_tests;

#[cfg(test)]
mod tests;
