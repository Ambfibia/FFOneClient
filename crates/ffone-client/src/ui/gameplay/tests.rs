use super::active_nano::{
    ActiveNanoInfoImage, ActiveNanoInfoLayer, ActiveNanoInfoName, ActiveNanoInfoRoot,
    ActiveNanoInfoStaminaFill, active_nano_stamina_fill_width,
};
use super::assets::configure_nano_wheel_image;
use super::chat_input::localized_chat_input;
use super::chat_log::{chat_line_color, chat_scroll_position, retained_chat_lines};
use super::chat_model::append_chat_text_utf16;
use super::chat_model::{
    CHAT_CHALET_SMALL_LINE_HEIGHT, CHAT_CHALET_SMALL_Y_OFFSET, CHAT_HISTORY_CAPACITY,
    CHAT_INPUT_FONT_SIZE, CHAT_INPUT_PADDING, CHAT_JEFFE_14_FONT_SIZE, CHAT_JEFFE_14_LINE_HEIGHT,
    CHAT_JEFFE_14_VERTICAL_SCALE, CHAT_LOG_BOTTOM_PADDING, CHAT_LOG_LINE_GAP,
    CHAT_SCROLL_TO_LATEST, CHAT_SCROLLBAR_DOWN_PATH, CHAT_SCROLLBAR_THUMB_PATH,
    CHAT_SCROLLBAR_TRACK_PATH, CHAT_SCROLLBAR_UP_PATH, ChatKeyboardCommand, chat_layout,
    clamped_chat_size, reduce_chat_keyboard, truncate_chat_input_utf16,
};
use super::chat_spawn::{ChatTabButton, ChatTextFieldButton, SendChatButton};
use super::combat_frame::combat_frame_alpha;
use super::combat_target::{
    COMBAT_TARGET_FRAME_HEIGHT, COMBAT_TARGET_MOB_GROUP_HEIGHT, COMBAT_TARGET_MOB_WIDTH,
    COMBAT_TARGET_NPC_WIDTH, CombatTargetInfoLevel, CombatTargetMatchup, GameplayHudNpc,
    GameplayHudNpcQuery, NPC_TALK_TARGET_ICON_PATH, NPC_TALK_TARGET_ICON_SIZE,
    PrimaryTargetIconKind, combat_target_icon_path, combat_target_matchup,
    friendly_target_ui_visible, localized_tutorial_npc_name, primary_target_icon,
};
use super::current_objective::{
    CurrentObjectiveBody, CurrentObjectiveTitle, localized_current_objective_body,
};
use super::minimap::{MinimapNameShadow, MinimapNameText};
use super::minimap_model::MINIMAP_MARKER_CAPACITY;
use super::nano_wheel::{
    BATTERY_COUNTER_FONT_SIZE, BATTERY_COUNTER_LINE_HEIGHT, BATTERY_COUNTER_TEXT_Y_OFFSET,
    NANO_WHEEL_EXACT_SAMPLER_PATHS, NanoBatteryCounterText, NanoCooldownLayers,
    WeaponBatteryCounterText, battery_counter_text, nano_cooldown_layers, nano_stamina_fill_width,
    skill_icon_asset_path,
};
use super::player_status::{PlayerLevelText, PlayerNameText};
use super::quick_chat::{EmoteButton, MenuChatButton};
use super::speech_bubbles::{
    NPC_BARKER_JEFFE_FONT_SIZE, NPC_BARKER_JEFFE_LINE_HEIGHT, NPC_BARKER_JEFFE_VERTICAL_SCALE,
    NpcBarkerActorState, NpcBarkerBubbleLayer, NpcBubbleKind, barker_attempt_succeeds, barker_lifetime,
    chat_string_is_visible, speech_bubble_logical_size, speech_bubble_top_left,
};
use crate::avatar_action::LegacyFocusedTarget;
use crate::avatar_action::LegacyTargetSelection;
use crate::entity_lifecycle::NetworkNpcAppearance0104;
use crate::localization::Localization;
use crate::localization::localized_tabledata_mission_barker;
use crate::localization::localized_tabledata_npc_barker;
use crate::localization::localized_tabledata_npc_greeting;
use crate::localization::localized_tabledata_npc_skill_barker;
use crate::option_ui::TextColorSettings;
use crate::tutorial_actors::TutorialActor;
use crate::tutorial_mission_content::GameplayNpcMinimapDefinition;
use crate::tutorial_mission_content::GameplayNpcUiDefinition;
use crate::tutorial_mission_content::TutorialMissionContent;
use crate::world_map::WORLD_MAP_MARKER_PATHS;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use std::collections::BTreeSet;
use std::path::Path;

use bevy::asset::AssetPlugin;
use bevy::input::keyboard::Key;
use ffone_protocol::NpcAppearance0104;
use sha2::Digest;

use crate::avatar_action::{LegacyAttackTarget, LegacyTargetKind};
use crate::gameplay_ui::*;

mod operations_computress_2555_autonomous_barker_uses_produ;
mod operations_minimap_marker_icons_and_projection_match_re;
mod localization_mission_dialogue_publishes_local;
mod npc_speech_priority;
mod npc_bubble_visibility;
mod layout;
mod interaction;
mod materials;
mod audio;
mod frame;
mod containers;
mod assets_chat_textures_match_the_primary_;
mod textures;
mod state;
mod models;

use operations_computress_2555_autonomous_barker_uses_produ::{
    pressed_key, normal_world_npc_definition, normal_world_npc_appearance
};
