//! Native Bevy reconstruction of the Retrobution gameplay HUD.
//!
//! The geometry below is an explicit port of the original IMGUI rectangles from
//! `CnGuiCharacter_info`, `CnGuiChat`, `cnGUINanocom`, `cnNanoWheel`, and
//! `cntutorialscript`. It loads only semantic PNG/TTF assets from
//! `assets/game/ui/gameplay`; it never reads a Unity bundle at runtime.
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use minimap_model::CIRCULAR_MINIMAP_TILE_SHADER;
use minimap_model::FUSION_MATTER_METER_SHADER;

use crate::text_edit::TextEditPlugin;

use std::collections::VecDeque;

mod new_mail_icon;

pub mod rewards;

mod trigger_icon;
pub use chat_input::GameplayControllerMenuInput;

pub use new_mail_icon::MinimapNewMailAlarm;

use crate::{
    avatar_action::{LegacyAvatarActionContext, LegacyAvatarActionState},
    legacy_npc_nano_animation::LegacyNanoStandRandomStream,
    localization::{LocalizationSet, LocalizedText},
    movement::LegacyOrbitCamera,
};

// Compatibility re-exports for callers that previously reached tutorial-only
// cue data through the monolithic HUD module. Ownership now lives in the
// independent tutorial overlay plugin.
pub use crate::tutorial_overlay_ui::{
    TUTORIAL_INSTRUCTION_FONT_PATH, TUTORIAL_INSTRUCTION_FONT_SIZE, TUTORIAL_MOUSE_RECT,
    TUTORIAL_RIGHT_RECT, TUTORIAL_TEXT_RECT, TutorialArrowCue, TutorialArrowDirection,
    TutorialCuePosition, TutorialCueScalePivot, TutorialIllustration, TutorialIllustrationCue,
    TutorialUi,
};

#[cfg(test)]
use crate::tutorial_overlay_ui::{legacy_tutorial_instruction_top, tutorial_instruction_layout};

use bevy::{
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    prelude::*,
    text::LineHeight,
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

mod actions;
mod active_nano;
mod assets;
mod chat_input;
mod chat_log;
mod chat_model;
mod chat_scrollbar;
mod chat_spawn;
mod combat_frame;
mod combat_target;
mod current_objective;
mod hud;
mod minimap;
mod minimap_model;
mod model;
mod nano_wheel;
mod player_status;
mod quick_chat;
mod speech_bubbles;
mod text;
pub use actions::{
    GameplayUiAction, GameplayUiAudioCue, GameplayUiAudioOutbox, GameplayUiOutbox, NpcServiceKind,
};
use active_nano::bind_active_nano_info;
pub use active_nano::{
    ACTIVE_NANO_COOLDOWN_RECT, ACTIVE_NANO_INFO_RECT, ACTIVE_NANO_INFO_TEXTURE_BYTES,
    ACTIVE_NANO_INFO_TEXTURE_PATH, ACTIVE_NANO_INFO_TEXTURE_SHA256, ACTIVE_NANO_INFO_WINDOW_RECT,
    ACTIVE_NANO_NAME_RECT, ACTIVE_NANO_SKILL_RECT, ACTIVE_NANO_STAMINA_RECT,
};
use assets::{GameplayUiAssets, configure_nano_wheel_image, load_nano_wheel_image};
use chat_input::{
    bind_chat_button_visuals, bind_chat_edit_visual, handle_chat_edit_pointer,
    handle_chat_keyboard_input, handle_chat_resize,
};
use chat_log::{bind_chat, scroll_chat_history};
pub use chat_model::{
    CHAT_ACTIVE_TEXT_FIELD_BYTES, CHAT_ACTIVE_TEXT_FIELD_PATH, CHAT_ACTIVE_TEXT_FIELD_SHA256,
    CHAT_BUDDY_ICON_BYTES, CHAT_BUDDY_ICON_PATH, CHAT_BUDDY_ICON_SHA256, CHAT_DEFAULT_HEIGHT,
    CHAT_DEFAULT_WIDTH, CHAT_EMPTY_STATE_BACKGROUND_BYTES, CHAT_EMPTY_STATE_BACKGROUND_PATH,
    CHAT_EMPTY_STATE_BACKGROUND_SHA256, CHAT_GROUP_ICON_BYTES, CHAT_GROUP_ICON_PATH,
    CHAT_GROUP_ICON_SHA256, CHAT_INPUT_UTF16_LIMIT, CHAT_MAX_HEIGHT, CHAT_MAX_WIDTH,
    CHAT_MIN_HEIGHT, CHAT_MIN_WIDTH, CHAT_RESIZE_HOVER_BYTES, CHAT_RESIZE_HOVER_PATH,
    CHAT_RESIZE_HOVER_SHA256, CHAT_RESIZE_NORMAL_BYTES, CHAT_RESIZE_NORMAL_PATH,
    CHAT_RESIZE_NORMAL_SHA256, CHAT_RESIZE_RECT_HEIGHT, CHAT_RESIZE_RECT_WIDTH,
    CHAT_SCROLLBAR_DOWN_BYTES, CHAT_SCROLLBAR_DOWN_SHA256, CHAT_SCROLLBAR_THUMB_BYTES,
    CHAT_SCROLLBAR_THUMB_SHA256, CHAT_SCROLLBAR_TRACK_BYTES, CHAT_SCROLLBAR_TRACK_SHA256,
    CHAT_SCROLLBAR_UP_BYTES, CHAT_SCROLLBAR_UP_SHA256, CHAT_SENT_HISTORY_CAPACITY, CHAT_TAB_HEIGHT,
    CHAT_TAB_WIDTH, ChatChannel, ChatLineKind, ChatLineUi, ChatUi,
};
use chat_model::{CHAT_JEFFE_14_VERTICAL_SCALE, ChatInputHistory, ChatResizeDrag};
use chat_scrollbar::{bind_chat_scrollbar, handle_chat_scrollbar};
pub use combat_frame::{
    COMBAT_DANGER_RECT, COMBAT_FRAME_BYTES, COMBAT_FRAME_PATH, COMBAT_FRAME_SHA256,
    CombatModeNotice,
};
use combat_frame::{bind_combat_frame, update_combat_mode_notice};
pub use combat_target::{
    COMBAT_TARGET_AFFINITY_EFFECT_RECT, COMBAT_TARGET_AFFINITY_ICON_RECT, COMBAT_TARGET_SKILL_RECT,
    NANO_SKILL_TARGET_ICON_BYTES, NANO_SKILL_TARGET_ICON_PATH, NANO_SKILL_TARGET_ICON_SHA256,
    NANO_SKILL_TARGET_ICON_SIZE,
};
use combat_target::{
    bind_combat_target_info, bind_nano_skill_target_icons, bind_primary_combat_target_icon,
    bind_primary_combat_target_status, bind_secondary_combat_target_icons,
};
use current_objective::bind_current_objective;
pub use current_objective::{
    CURRENT_OBJECTIVE_RECT, CurrentObjectiveProgressUi, CurrentObjectiveUi,
};
pub use hud::{GameplayUiRect, gameplay_ui_scale};
use hud::{
    advance_gameplay_menu_transition, handle_gameplay_ui_buttons, spawn_gameplay_hud,
    sync_gameplay_camera_activity, update_gameplay_hud_layout,
};
use minimap::{
    bind_custom_minimap_waypoints, bind_hud_player_minimap, bind_minimap_markers,
    bind_minimap_name_shadow, update_minimap_zoom,
};

pub use minimap_model::{
    MINIMAP_ADVANCE_MISSION_PATH, MINIMAP_FRAME_RECT, MINIMAP_FUSION_METER_RECT,
    MINIMAP_FUSION_PATH, MINIMAP_GROUP_PATH, MINIMAP_GROUP_RECT, MINIMAP_MAP_RECT,
    MINIMAP_MOB_PATH, MINIMAP_NAME_RECT, MINIMAP_NEW_MISSION_PATH, MINIMAP_SHINY_PATH,
    MINIMAP_SHOW_NPC_PATH, MINIMAP_WAYPOINT_ABOVE_PATH, MINIMAP_WAYPOINT_BELOW_PATH,
    MINIMAP_WAYPOINT_IN_RANGE_PATH, MINIMAP_WAYPOINT_OUT_OF_RANGE_PATH, MinimapMarkerIcon,
    MinimapMarkerSample, MinimapTableMarkerIcon, MinimapTileSample, MinimapUi, MinimapWaypointIcon,
    MinimapWaypointSample, NormalWorldMinimapMarkerStyle, minimap_marker, minimap_marker_alpha,
    minimap_marker_sized, minimap_tiles, minimap_waypoint, normal_world_minimap_marker_style,
};
pub use model::{
    GameplayMenuTransition, GameplayNanoPortraitImages, GameplayPortraitImage, GameplayUiModel,
    menu_slide_offset,
};
use nano_wheel::bind_nano_wheel;
pub use nano_wheel::{
    NANO_AFFINITY_BACK_RECTS, NANO_AFFINITY_ICON_RECTS, NANO_ANIMATED_PORTRAIT_RECTS,
    NANO_BATTERY_COUNTER_RECT, NANO_BATTERY_LABEL_RECT, NANO_COOLDOWN_RECTS, NANO_EMPTY_SLOTS_RECT,
    NANO_GUMBALL_RECTS, NANO_PORTRAIT_RECTS, NANO_SKILL_RECTS, NANO_SLOT_KEY_RECTS,
    NANO_STAMINA_BACK_RECTS, NANO_STAMINA_FILL_RECTS, NANO_WHEEL_WINDOW_RECT, NanoSlotUi,
    NanoWheelTransientSlotUi, NanoWheelTransientUi, WEAPON_BATTERY_COUNTER_RECT,
    WEAPON_BATTERY_LABEL_RECT,
};
pub use player_status::{
    PLAYER_COMBAT_TOGGLE_RECT, PLAYER_DISK_FRONT_RECT, PLAYER_FRAME_RECT, PLAYER_FREE_CHAT_RECT,
    PLAYER_HEALTH_RECT, PLAYER_LEVEL_RECT, PLAYER_NAME_RECT, PLAYER_PORTRAIT_RECT, PlayerStatusUi,
};
use player_status::{bind_player_auxiliary_status, bind_portrait_image};
pub use quick_chat::{
    EMOTE_CHAT_ITEMS, MENU_CHAT_ITEMS, QUICK_CHAT_BOX_BYTES, QUICK_CHAT_BOX_PATH,
    QUICK_CHAT_BOX_SHA256, QUICK_CHAT_EMOTE_ICON_BYTES, QUICK_CHAT_EMOTE_ICON_PATHS,
    QUICK_CHAT_EMOTE_ICON_SHA256, QUICK_CHAT_EMOTE_ICON_SIZES, QUICK_CHAT_EMOTE_LEVEL0_LEFT,
    QUICK_CHAT_HOVER_BYTES, QUICK_CHAT_HOVER_PATH, QUICK_CHAT_HOVER_SHA256,
    QUICK_CHAT_LEVEL_BOTTOM, QUICK_CHAT_MAX_ROWS, QUICK_CHAT_MENU_LEVEL0_LEFT,
    QUICK_CHAT_NEXT_ARROW_BYTES, QUICK_CHAT_NEXT_ARROW_PATH, QUICK_CHAT_NEXT_ARROW_SHA256,
    QUICK_CHAT_NEXT_HOVER_BYTES, QUICK_CHAT_NEXT_HOVER_PATH, QUICK_CHAT_NEXT_HOVER_SHA256,
    QUICK_CHAT_ROW_HEIGHT, QuickChatDynamicText, QuickChatItem, QuickChatMenuMode, QuickChatMenuUi,
};
use quick_chat::{bind_quick_chat_menu, bind_quick_chat_row_visuals};
pub use speech_bubbles::{
    NPC_BARKER_BOX_BYTES, NPC_BARKER_BOX_PATH, NPC_BARKER_BOX_SHA256, NPC_BARKER_CHANCE_PERCENT,
    NPC_BARKER_DISTANCE, NPC_BARKER_GLOBAL_Z_INDEX, NPC_BARKER_MAX_WIDTH,
    NPC_BARKER_MIN_LIFETIME_SECONDS, NPC_BARKER_MIN_WIDTH, NPC_BARKER_PERIOD_SECONDS,
    NPC_BARKER_TAIL_BYTES, NPC_BARKER_TAIL_PATH, NPC_BARKER_TAIL_SHA256, NpcBarkerBubbleRuntime,
    NpcChatEvent, PLAYER_FREECHAT_BOX_BYTES, PLAYER_FREECHAT_BOX_PATH, PLAYER_FREECHAT_BOX_SHA256,
    PLAYER_FREECHAT_TAIL_BYTES, PLAYER_FREECHAT_TAIL_PATH, PLAYER_FREECHAT_TAIL_SHA256,
    PlayerFreeChatBubbleRuntime,
};
use speech_bubbles::{
    advance_npc_barker_bubbles, advance_player_freechat_bubbles, bind_npc_barker_bubbles,
    bind_player_freechat_bubbles,
};
use text::chat_jeffe_14_font;

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
#[type_path = "ffone_client::gameplay_ui"]
struct CircularMinimapTileMaterial {
    #[uniform(0)]
    source_uv: Vec4,
    #[uniform(0)]
    destination_uv: Vec4,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    alpha_texture: Handle<Image>,
}

impl UiMaterial for CircularMinimapTileMaterial {
    fn fragment_shader() -> ShaderRef {
        CIRCULAR_MINIMAP_TILE_SHADER.into()
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
#[type_path = "ffone_client::gameplay_ui"]
struct FusionMatterMeterMaterial {
    #[uniform(0)]
    parameters: Vec4,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    right_color_texture: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    left_mask_texture: Handle<Image>,
    #[texture(7)]
    #[sampler(8)]
    rotating_mask_texture: Handle<Image>,
}

impl UiMaterial for FusionMatterMeterMaterial {
    fn fragment_shader() -> ShaderRef {
        FUSION_MATTER_METER_SHADER.into()
    }
}

pub const GAMEPLAY_UI_REFERENCE_WIDTH: f32 = 1280.0;
pub const GAMEPLAY_UI_REFERENCE_HEIGHT: f32 = 720.0;
const GAMEPLAY_UI_SCALE_REFERENCE_HEIGHT: f32 = 768.0;
const GAMEPLAY_UI_SCALE_NUDGE: f32 = 1.05;
/// Shared UI pass. Native character cameras render immediately after it,
/// because character-selection panoramas are opaque UI images.
pub const GAMEPLAY_UI_CAMERA_ORDER: isize = 100;
pub const GAMEPLAY_UI_LAYOUT_PATH: &str = "ui/en/gameplay/layout/gameplay_hud.json";

#[derive(Default)]
pub struct GameplayUiPlugin;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum GameplayUiSet {
    /// Chat/Nanocom pointer and keyboard transitions that must observe modal
    /// ownership before consuming this frame's input messages.
    Input,
    BindNanoWheel,
    NpcSpeech,
    Rewards,
    MapNotice,
}

impl Plugin for GameplayUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TextEditPlugin>() {
            app.add_plugins(TextEditPlugin);
        }
        rewards::install(app);
        crate::ui_startup::init_native_ui_startup_phase(app);
        if !app.is_plugin_added::<crate::damage_bar::DamageBarPlugin>() {
            app.add_plugins(crate::damage_bar::DamageBarPlugin);
        }
        app.add_plugins(UiMaterialPlugin::<CircularMinimapTileMaterial>::default())
            .add_plugins(UiMaterialPlugin::<FusionMatterMeterMaterial>::default())
            .init_resource::<GameplayUiModel>()
            .add_message::<NpcChatEvent>()
            .init_resource::<GameplayPortraitImage>()
            .init_resource::<GameplayNanoPortraitImages>()
            .init_resource::<NanoWheelTransientUi>()
            .init_resource::<GameplayMenuTransition>()
            .init_resource::<CombatModeNotice>()
            .init_resource::<crate::ui_icon_variants::UiIconVariants>()
            .init_resource::<GameplayUiOutbox>()
            .init_resource::<GameplayUiAudioOutbox>()
            .init_resource::<ChatInputHistory>()
            .init_resource::<GameplayControllerMenuInput>()
            .init_resource::<ChatResizeDrag>()
            .init_resource::<NpcBarkerBubbleRuntime>()
            .init_resource::<PlayerFreeChatBubbleRuntime>()
            // UnityEngine.Random is app-global: Barker timing must share the
            // same session-seeded stream as legacy animation/event owners.
            .init_resource::<LegacyNanoStandRandomStream>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_gameplay_hud,
            )
            .add_systems(
                Update,
                (advance_gameplay_menu_transition).in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (handle_chat_keyboard_input
                    .in_set(GameplayUiSet::Input)
                    .before(bind_chat)
                    .before(handle_gameplay_ui_buttons))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (
                    handle_chat_edit_pointer.after(handle_gameplay_ui_buttons),
                    bind_chat_edit_visual.after(handle_chat_edit_pointer),
                    handle_chat_resize,
                )
                    .in_set(GameplayUiSet::Input)
                    .before(bind_chat)
                    .before(update_gameplay_hud_layout)
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    (
                        bind_hud_player_minimap.before(LocalizationSet::Apply),
                        bind_minimap_name_shadow.before(LocalizationSet::Apply),
                        bind_current_objective
                            .after(crate::mission_ui::MissionUiSet::Presentation)
                            .before(LocalizationSet::Apply),
                        bind_player_auxiliary_status,
                        bind_minimap_markers,
                        update_gameplay_hud_layout,
                        bind_chat.before(LocalizationSet::Apply),
                        bind_quick_chat_menu.before(LocalizationSet::Apply),
                        scroll_chat_history.after(bind_chat),
                        handle_chat_scrollbar.after(scroll_chat_history),
                        bind_chat_scrollbar.after(handle_chat_scrollbar),
                        bind_active_nano_info.before(LocalizationSet::Apply),
                        bind_nano_wheel
                            .in_set(GameplayUiSet::BindNanoWheel)
                            .before(LocalizationSet::Apply),
                    ),
                    (
                        bind_portrait_image,
                        trigger_icon::bind
                            .after(crate::avatar_action::LegacyAvatarActionSet::Resolve)
                            .after(crate::world_behaviour::process_world_trigger_uses),
                        bind_primary_combat_target_icon,
                        bind_secondary_combat_target_icons,
                        bind_nano_skill_target_icons,
                        bind_primary_combat_target_status.before(LocalizationSet::Apply),
                        bind_combat_frame,
                        bind_combat_target_info.before(LocalizationSet::Apply),
                        (
                            advance_npc_barker_bubbles.in_set(GameplayUiSet::NpcSpeech),
                            bind_npc_barker_bubbles.before(LocalizationSet::Apply),
                        )
                            .chain(),
                        (
                            advance_player_freechat_bubbles,
                            bind_player_freechat_bubbles.before(LocalizationSet::Apply),
                        )
                            .chain(),
                        bind_chat_button_visuals.after(bind_chat),
                        bind_quick_chat_row_visuals.after(bind_quick_chat_menu),
                    ),
                ))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                handle_gameplay_ui_buttons
                    .in_set(GameplayUiSet::Input)
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            // This camera survives when the player returns to character
            // selection. Keep it active only while gameplay owns the native
            // UI phase so it cannot collide with the selection camera's
            // legacy order 100.
            .add_systems(Update, sync_gameplay_camera_activity);
        app.add_systems(Update, update_minimap_zoom.in_set(GameplayUiSet::Input));
        app.add_systems(Update, bind_custom_minimap_waypoints);
        app.init_resource::<MinimapNewMailAlarm>().add_systems(
            Update,
            new_mail_icon::bind.in_set(crate::ui_startup::NativeUiStartupSet),
        );
        app.add_systems(
            Update,
            update_combat_mode_notice
                .in_set(GameplayUiSet::MapNotice)
                .before(LocalizationSet::Apply),
        );
    }
}

#[derive(Component)]
pub struct GameplayHud;
#[derive(Component)]
pub struct GameplayUiCamera;

#[cfg(test)]
mod location_notice_tests;

#[cfg(test)]
mod chat_scrollbar_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "performance_tests.rs"]
mod performance_tests;
