use super::assets::{
    CENTERED_MENU_JEFFE_FONT_SIZE, CENTERED_MENU_JEFFE_VERTICAL_SCALE, END_DIALOG,
    JEFFE_14_LINE_HEIGHT, JEFFE_14_REPLACEMENT_FONT_SIZE, JEFFE_14_SOURCE_FONT_PATH_ID,
    JOURNAL_ACTIVE_TAB, JOURNAL_ACTIVE_TAB_OVER, JOURNAL_CLOSE, JOURNAL_COMPLETED_PANEL,
    JOURNAL_COMPLETED_TAB, JOURNAL_COMPLETED_TAB_OVER, JOURNAL_REWARD_BOX, JOURNAL_RIGHT_FRAME,
    JOURNAL_WINDOW, MISSION_BODY, MISSION_BUTTON, MISSION_TOP, NPC_MULTI_WINDOW, NPCICON_COMBINE,
    NPCICON_ENCHANT, NPCICON_MONKEY, NPCICON_RANK, NPCICON_RULE, NPCICON_SCAMP, OFFER_DIALOG,
    SYSTEM_DIALOG_BOX,
};
use super::buttons::{menu_interaction_state, tutorial_mission_control_is_locked};
use super::components::{
    MenuButtonStyle, MissionTextVerticalAnchor, MissionUiControl, MissionUiView,
};
use super::geometry::JOURNAL_SCROLL_WHEEL_LINE;
use super::journal::{
    JournalScrollContent, JournalScrollViewport, current_mission, journal_close_uses_hover_asset,
};
use super::labels::{
    JOURNAL_FUSION_MATTER_LABEL, JOURNAL_MISSION_DETAILS_LABEL, JOURNAL_MISSION_OFFER_LABEL,
    JOURNAL_MISSION_SUMMARY_LABEL, JOURNAL_MY_NOTES_LABEL, JOURNAL_REWARD_LABEL,
    MISSION_UI_SOURCE_KEYS, NANOCOM_MENU_LABELS, mission_content_text, mission_ui_localized_text,
    nano_content_text, nano_skill_content_text,
};
use super::layout::{
    JOURNAL_MISSION_ROW_LIMIT, NpcUtilityRow, journal_primary_is_visible, journal_primary_rect,
    journal_primary_text_rect, journal_right_layout, journal_row_has_selected_decorations,
    npc_rows, npc_utility_panel_height, quest_list_end,
};
use super::nanocom::{chat_quick_menu_logical_rect, nanocom_button_source_path, nanocom_control};
use super::system_dialog::system_dialog_button_source_path;
use super::widgets::{
    journal_managed_label, journal_objective_label, mission_glyph_vertical_bounds,
    mission_text_content_rect, mission_text_layout_rect, mission_text_translation_px,
    mission_text_vertical_anchor, mission_text_vertical_offset, skin_style_background_rect,
    skin_style_border, source_style_border,
};
use crate::character_selection_ui::CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH;
use crate::character_selection_ui::CHARACTER_SELECTION_BLUE_BUTTON_PATH;
use crate::character_selection_ui::CHARACTER_SELECTION_CANCEL_NORMAL_PATH;
use crate::character_selection_ui::CHARACTER_SELECTION_RED_BUTTON_OVER_PATH;
use crate::gameplay_ui::GameplayUiAction;
use crate::gameplay_ui::GameplayUiAudioCue;
use crate::gameplay_ui::NpcServiceKind;
use crate::gui_skin::gui_effective_font as mission_gui_effective_font;
use crate::gui_skin::gui_style as mission_gui_style;
use crate::localization::Localization;
use crate::localization::LocalizedText;
use crate::mission_ui::*;
use crate::option_ui::OPTION_JEFFE_FONT_PATH;
use crate::tutorial::TutorialInputLock;
use crate::tutorial_logic::TutorialJournalMode;
use crate::tutorial_mission_content::TutorialWarpTarget;
use crate::user_equip_ui::UserEquipOpenSource;
use crate::{
    assets::AssetLocator,
    gui_skin::GuiInsets as MissionGuiInsets,
    tutorial_mission_content::{TUTORIAL_MISSION_TASK_IDS, TutorialMissionContent},
};
use bevy::input::mouse::MouseScrollUnit;
use bevy::input::mouse::MouseWheel;
use bevy::text::LineHeight;
use bevy::ui::RelativeCursorPosition;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

mod interaction;
mod state;
mod operations_delayed_oil_ogre_accept_is_ignored_until_mou;
mod operations_secondary_closes_allow_and_reward_but_tutori;
mod commands;
mod assets_nanocom_my_stuff_is_the_only_typ;
mod view;
mod input_mission_ui_and_loaded_tutorial_c;
mod layout_enter_quick_menu_uses_clean_retr;
mod containers;
mod frame;

use operations_delayed_oil_ogre_accept_is_ignored_until_mou::{mission, npc_with_available};
use assets_nanocom_my_stuff_is_the_only_typ::asset_root;
