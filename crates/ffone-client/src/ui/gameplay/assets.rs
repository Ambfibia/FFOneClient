//! Loaded HUD image/font set and Nano wheel exact-sampler image loading.

use super::active_nano::ACTIVE_NANO_INFO_TEXTURE_PATH;
use super::chat_model::{
    CHAT_ACTIVE_TEXT_FIELD_PATH, CHAT_BUDDY_ICON_PATH, CHAT_EMPTY_STATE_BACKGROUND_PATH,
    CHAT_GROUP_ICON_PATH, CHAT_RESIZE_HOVER_PATH, CHAT_RESIZE_NORMAL_PATH,
    CHAT_SCROLLBAR_DOWN_PATH, CHAT_SCROLLBAR_THUMB_PATH, CHAT_SCROLLBAR_TRACK_PATH,
    CHAT_SCROLLBAR_UP_PATH,
};
use super::combat_frame::{COMBAT_DANGER_PATH, COMBAT_FRAME_PATH};
use super::combat_target::{
    COMBAT_TARGET_HP_FILL_PATH, COMBAT_TARGET_MATCHUP_PATHS, COMBAT_TARGET_MOB_INFO_PATH,
    COMBAT_TARGET_NPC_INFO_PATH, CombatTargetMatchup, NANO_SKILL_TARGET_ICON_PATH,
    NPC_TALK_TARGET_ICON_PATH, PRIMARY_COMBAT_TARGET_ICON_PATH, SECONDARY_COMBAT_TARGET_ICON_PATH,
};
use super::minimap_model::{MinimapMarkerIcon, MinimapWaypointIcon};
use super::nano_wheel::NANO_WHEEL_EXACT_SAMPLER_PATHS;
use super::new_mail_icon;
use super::quick_chat::{
    QUICK_CHAT_BOX_PATH, QUICK_CHAT_EMOTE_ICON_PATHS, QUICK_CHAT_HOVER_PATH,
    QUICK_CHAT_NEXT_ARROW_PATH, QUICK_CHAT_NEXT_HOVER_PATH,
};
use super::speech_bubbles::{
    NPC_BARKER_BOX_PATH, NPC_BARKER_TAIL_PATH, NPC_QUEST_BOX_PATH, NPC_QUEST_TAIL_PATH,
    PLAYER_FREECHAT_BOX_PATH, PLAYER_FREECHAT_TAIL_PATH,
};
use crate::world_map::WORLD_MAP_MARKER_PATHS;
use bevy::{
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    prelude::*,
};

#[derive(Clone, Resource)]
pub(super) struct GameplayUiAssets {
    pub(super) player_frame: Handle<Image>,
    pub(super) disk_front: Handle<Image>,
    pub(super) health_bar: Handle<Image>,
    pub(super) free_chat: Handle<Image>,
    pub(super) combat_toggle: Handle<Image>,
    pub(super) minimap_zoom_in: Handle<Image>,
    pub(super) minimap_zoom_out: Handle<Image>,
    pub(super) minimap_frame: Handle<Image>,
    pub(super) minimap_fusion_meter: Handle<Image>,
    pub(super) minimap_fusion_meter_right: Handle<Image>,
    pub(super) minimap_fusion_meter_left_mask: Handle<Image>,
    pub(super) minimap_fusion_meter_rotating_mask: Handle<Image>,
    pub(super) minimap_line: Handle<Image>,
    pub(super) minimap_alpha: Handle<Image>,
    pub(super) minimap_tiles: [Handle<Image>; 16],
    pub(super) minimap_camera: Handle<Image>,
    pub(super) minimap_player: Handle<Image>,
    pub(super) minimap_new_mail: Handle<Image>,
    pub(super) minimap_waypoint_icons: [Handle<Image>; 4],
    pub(super) minimap_marker_icons: [Handle<Image>; 7],
    pub(super) minimap_table_marker_icons: [Handle<Image>; WORLD_MAP_MARKER_PATHS.len()],
    pub(super) chat_background: Handle<Image>,
    pub(super) chat_entry: Handle<Image>,
    pub(super) chat_active_text_field: Handle<Image>,
    pub(super) chat_inactive_text_field: Handle<Image>,
    pub(super) chat_empty_state_background: Handle<Image>,
    pub(super) chat_buddy_icon: Handle<Image>,
    pub(super) chat_group_icon: Handle<Image>,
    pub(super) chat_scroll_track: Handle<Image>,
    pub(super) chat_scroll_thumb: Handle<Image>,
    pub(super) chat_scroll_up: Handle<Image>,
    pub(super) chat_scroll_down: Handle<Image>,
    pub(super) chat_tab_normal: Handle<Image>,
    pub(super) chat_tab_over: Handle<Image>,
    pub(super) chat_tab_selected: Handle<Image>,
    pub(super) chat_alert: Handle<Image>,
    pub(super) menu_chat: Handle<Image>,
    pub(super) menu_chat_over: Handle<Image>,
    pub(super) emote: Handle<Image>,
    pub(super) emote_over: Handle<Image>,
    pub(super) quick_chat_box: Handle<Image>,
    pub(super) quick_chat_hover: Handle<Image>,
    pub(super) quick_chat_next_hover: Handle<Image>,
    pub(super) quick_chat_next_arrow: Handle<Image>,
    pub(super) quick_chat_emote_icons: [Handle<Image>; 19],
    pub(super) blue_button: Handle<Image>,
    pub(super) blue_button_over: Handle<Image>,
    pub(super) chat_resize_normal: Handle<Image>,
    pub(super) chat_resize_hover: Handle<Image>,
    pub(super) active_nano_info: Handle<Image>,
    pub(super) nano_empty: Handle<Image>,
    pub(super) nano_slot_keys: [Handle<Image>; 3],
    pub(super) nano_affinity_backs: [Handle<Image>; 3],
    pub(super) nano_affinity_icons: [Handle<Image>; 3],
    pub(super) nano_gumballs: [Handle<Image>; 3],
    pub(super) nano_skill_back: Handle<Image>,
    pub(super) nano_cooldown_slices: [Handle<Image>; 7],
    pub(super) nano_cooldown_fine: Handle<Image>,
    pub(super) nano_stamina_back: Handle<Image>,
    pub(super) nano_stamina_fill: Handle<Image>,
    pub(super) nano_battery_counter: Handle<Image>,
    pub(super) weapon_battery_counter: Handle<Image>,
    pub(super) primary_combat_target_icon: Handle<Image>,
    pub(super) secondary_combat_target_icon: Handle<Image>,
    pub(super) nano_skill_target_icon: Handle<Image>,
    pub(super) npc_talk_target_icon: Handle<Image>,
    pub(super) combat_target_hp_fill: Handle<Image>,
    pub(super) combat_target_mob_info: Handle<Image>,
    pub(super) combat_target_npc_info: Handle<Image>,
    pub(super) combat_target_matchups: [Handle<Image>; 3],
    pub(super) combat_frame: Handle<Image>,
    pub(super) combat_danger: Handle<Image>,
    pub(super) npc_barker_box: Handle<Image>,
    pub(super) npc_barker_tail: Handle<Image>,
    pub(super) npc_quest_box: Handle<Image>,
    pub(super) npc_quest_tail: Handle<Image>,
    pub(super) player_freechat_box: Handle<Image>,
    pub(super) player_freechat_tail: Handle<Image>,
    pub(super) chalet_font: Handle<Font>,
    pub(super) jeffe_font: Handle<Font>,
}

pub(super) fn nano_wheel_sampler_descriptor() -> ImageSamplerDescriptor {
    // All ten audited primary Texture2D objects use FilterMode=Bilinear,
    // WrapMode=Repeat, no mipmaps and anisotropy 1. Bevy's global linear
    // default clamps, which leaves a small rim mismatch on scaled/rotated
    // cooldown wedges.
    ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Nearest,
        anisotropy_clamp: 1,
        ..default()
    }
}

pub(super) fn configure_nano_wheel_image(settings: &mut ImageLoaderSettings) {
    settings.sampler = ImageSampler::Descriptor(nano_wheel_sampler_descriptor());
}

pub(super) fn load_nano_wheel_image(
    asset_server: &AssetServer,
    path: &'static str,
) -> Handle<Image> {
    asset_server
        .load_builder()
        .with_settings::<ImageLoaderSettings>(configure_nano_wheel_image)
        .load::<Image>(path)
}

impl GameplayUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            player_frame: asset_server.load("ui/en/gameplay/player/character_info.png"),
            disk_front: asset_server.load("ui/en/gameplay/player/DiskFront.png"),
            health_bar: asset_server.load("ui/en/gameplay/player/HP_BAR.png"),
            free_chat: asset_server.load("ui/en/gameplay/player/freechat_icon.png"),
            minimap_zoom_in: asset_server.load("ui/en/gameplay/minimap/zoom-in.png"),
            minimap_zoom_out: asset_server.load("ui/en/gameplay/minimap/zoom-out.png"),
            combat_toggle: asset_server.load("ui/en/gameplay/player/combat_toggle.png"),
            minimap_frame: asset_server.load("ui/en/gameplay/minimap/fusionmatter_us.png"),
            minimap_fusion_meter: asset_server
                .load("ui/en/gameplay/minimap/fusionmeter/regular.png"),
            minimap_fusion_meter_right: asset_server
                .load("ui/en/gameplay/minimap/fusionmeter/regular_right.png"),
            minimap_fusion_meter_left_mask: asset_server
                .load("ui/en/gameplay/minimap/fusionmeter/back_left.png"),
            minimap_fusion_meter_rotating_mask: asset_server
                .load("ui/en/gameplay/minimap/fusionmeter/back_right.png"),
            minimap_line: asset_server.load("ui/en/gameplay/minimap/line_minimap.png"),
            minimap_alpha: asset_server.load("ui/en/gameplay/minimap/nanocom_minimap_a.png"),
            minimap_tiles: std::array::from_fn(|index| {
                asset_server.load(format!(
                    "ui/en/gameplay/minimap/tiles/all_minimap_{:02}.png",
                    index + 1
                ))
            }),
            minimap_camera: asset_server.load("ui/en/gameplay/minimap/camera.png"),
            minimap_player: asset_server.load("ui/en/gameplay/minimap/map_icon_00.png"),
            // `mail_but` uses the same audited Bilinear/Repeat, no-mip sampler.
            minimap_new_mail: load_nano_wheel_image(asset_server, new_mail_icon::TEXTURE_PATH),
            minimap_waypoint_icons: MinimapWaypointIcon::ALL
                .map(|icon| asset_server.load(icon.asset_path())),
            minimap_marker_icons: MinimapMarkerIcon::ALL
                .map(|icon| asset_server.load(icon.asset_path())),
            minimap_table_marker_icons: std::array::from_fn(|index| {
                asset_server.load(WORLD_MAP_MARKER_PATHS[index])
            }),
            chat_background: asset_server.load("ui/en/gameplay/chat/darkenedChatArea.png"),
            chat_entry: asset_server.load("ui/en/gameplay/chat/chatEntryArea.png"),
            chat_active_text_field: asset_server.load(CHAT_ACTIVE_TEXT_FIELD_PATH),
            chat_inactive_text_field: asset_server
                .load("ui/en/gameplay/chat/largeChatTextfield-normal.png"),
            chat_empty_state_background: asset_server.load(CHAT_EMPTY_STATE_BACKGROUND_PATH),
            chat_buddy_icon: asset_server.load(CHAT_BUDDY_ICON_PATH),
            chat_group_icon: asset_server.load(CHAT_GROUP_ICON_PATH),
            chat_scroll_track: asset_server.load(CHAT_SCROLLBAR_TRACK_PATH),
            chat_scroll_thumb: asset_server.load(CHAT_SCROLLBAR_THUMB_PATH),
            chat_scroll_up: asset_server.load(CHAT_SCROLLBAR_UP_PATH),
            chat_scroll_down: asset_server.load(CHAT_SCROLLBAR_DOWN_PATH),
            chat_tab_normal: asset_server.load("ui/en/gameplay/chat/chatTab_normal.png"),
            chat_tab_over: asset_server.load("ui/en/gameplay/chat/chatTab_over.png"),
            chat_tab_selected: asset_server.load("ui/en/gameplay/chat/chatTab_selected.png"),
            chat_alert: asset_server.load("ui/en/gameplay/chat/alert.png"),
            menu_chat: asset_server.load("ui/en/gameplay/chat/menuChatButton.png"),
            menu_chat_over: asset_server.load("ui/en/gameplay/chat/menuChatButton_Over.png"),
            emote: asset_server.load("ui/en/gameplay/chat/emoteButton.png"),
            emote_over: asset_server.load("ui/en/gameplay/chat/emoteButton_Over.png"),
            quick_chat_box: asset_server.load(QUICK_CHAT_BOX_PATH),
            quick_chat_hover: asset_server.load(QUICK_CHAT_HOVER_PATH),
            quick_chat_next_hover: asset_server.load(QUICK_CHAT_NEXT_HOVER_PATH),
            quick_chat_next_arrow: asset_server.load(QUICK_CHAT_NEXT_ARROW_PATH),
            quick_chat_emote_icons: QUICK_CHAT_EMOTE_ICON_PATHS.map(|path| asset_server.load(path)),
            blue_button: asset_server.load("ui/en/gameplay/chat/blue_button_normal.png"),
            blue_button_over: asset_server.load("ui/en/gameplay/chat/blue_button_over.png"),
            chat_resize_normal: asset_server.load(CHAT_RESIZE_NORMAL_PATH),
            chat_resize_hover: asset_server.load(CHAT_RESIZE_HOVER_PATH),
            active_nano_info: asset_server.load(ACTIVE_NANO_INFO_TEXTURE_PATH),
            nano_empty: asset_server.load("ui/en/gameplay/nano/nano_whell.png"),
            nano_slot_keys: std::array::from_fn(|index| {
                asset_server.load(format!("ui/en/gameplay/nano/slots/but_{}.png", index + 1))
            }),
            nano_affinity_backs: [
                asset_server.load("ui/en/gameplay/nano/affinity/blue_icon.png"),
                asset_server.load("ui/en/gameplay/nano/affinity/red_icon.png"),
                asset_server.load("ui/en/gameplay/nano/affinity/yellow_icon.png"),
            ],
            nano_affinity_icons: [
                asset_server.load("ui/en/gameplay/nano/affinity/a_icon.png"),
                asset_server.load("ui/en/gameplay/nano/affinity/b_icon.png"),
                asset_server.load("ui/en/gameplay/nano/affinity/c_icon.png"),
            ],
            nano_gumballs: [
                asset_server.load("ui/en/gameplay/nano/gumball/NanoGumballBlue.png"),
                asset_server.load("ui/en/gameplay/nano/gumball/NanoGumballRed.png"),
                asset_server.load("ui/en/gameplay/nano/gumball/NanoGumballYellow.png"),
            ],
            nano_skill_back: asset_server.load("ui/en/gameplay/nano/skill/buff_icon_back.png"),
            nano_cooldown_slices: std::array::from_fn(|index| {
                load_nano_wheel_image(asset_server, NANO_WHEEL_EXACT_SAMPLER_PATHS[index])
            }),
            nano_cooldown_fine: load_nano_wheel_image(
                asset_server,
                NANO_WHEEL_EXACT_SAMPLER_PATHS[7],
            ),
            nano_stamina_back: load_nano_wheel_image(
                asset_server,
                NANO_WHEEL_EXACT_SAMPLER_PATHS[8],
            ),
            nano_stamina_fill: load_nano_wheel_image(
                asset_server,
                NANO_WHEEL_EXACT_SAMPLER_PATHS[9],
            ),
            nano_battery_counter: asset_server
                .load("ui/en/gameplay/nano/counters/potion_counter.png"),
            weapon_battery_counter: asset_server
                .load("ui/en/gameplay/nano/counters/boost_counter.png"),
            primary_combat_target_icon: asset_server.load(PRIMARY_COMBAT_TARGET_ICON_PATH),
            secondary_combat_target_icon: asset_server.load(SECONDARY_COMBAT_TARGET_ICON_PATH),
            nano_skill_target_icon: asset_server.load(NANO_SKILL_TARGET_ICON_PATH),
            npc_talk_target_icon: asset_server.load(NPC_TALK_TARGET_ICON_PATH),
            combat_target_hp_fill: asset_server.load(COMBAT_TARGET_HP_FILL_PATH),
            combat_target_mob_info: asset_server.load(COMBAT_TARGET_MOB_INFO_PATH),
            combat_target_npc_info: asset_server.load(COMBAT_TARGET_NPC_INFO_PATH),
            combat_target_matchups: CombatTargetMatchup::ALL
                .map(|result| asset_server.load(COMBAT_TARGET_MATCHUP_PATHS[result.index()])),
            combat_frame: asset_server.load(COMBAT_FRAME_PATH),
            combat_danger: asset_server.load(COMBAT_DANGER_PATH),
            npc_barker_box: asset_server.load(NPC_BARKER_BOX_PATH),
            npc_barker_tail: asset_server.load(NPC_BARKER_TAIL_PATH),
            npc_quest_box: asset_server.load(NPC_QUEST_BOX_PATH),
            npc_quest_tail: asset_server.load(NPC_QUEST_TAIL_PATH),
            player_freechat_box: asset_server.load(PLAYER_FREECHAT_BOX_PATH),
            player_freechat_tail: asset_server.load(PLAYER_FREECHAT_TAIL_PATH),
            chalet_font: asset_server.load("fonts/chaletbook-regular.ttf"),
            jeffe_font: asset_server.load("fonts/jeffe.otf"),
        }
    }
}
