//! Neutral gameplay HUD model and menu slide transition.

use super::chat_model::{CHAT_DEFAULT_HEIGHT, CHAT_DEFAULT_WIDTH, ChatChannel, ChatLineUi, ChatUi};
use super::current_objective::CurrentObjectiveUi;
use super::minimap_model::{MinimapUi, minimap_tiles};
use super::nano_wheel::NanoSlotUi;
use super::player_status::PlayerStatusUi;
use super::quick_chat::QuickChatMenuUi;
use crate::option_ui::TextColorSettings;
use crate::text_edit::TextEdit;
use bevy::prelude::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct GameplayUiModel {
    pub visible: bool,
    /// Retrobution's FFGUIUtility scale, applied about each source screen pivot.
    pub ui_scale: f32,
    /// Clean `cnDisplayOption.bNpcName`. `PrintName.PositionUpdate` moves a
    /// chat rectangle up by 15 pixels only while the NPC name is drawable.
    pub npc_names_visible: bool,
    /// Clean `cnDisplayOption.bBaloonChat` (legacy spelling). New NPC chat
    /// entries are discarded while this option is disabled.
    pub balloon_chat_visible: bool,
    pub player: PlayerStatusUi,
    pub minimap: MinimapUi,
    pub current_objective: CurrentObjectiveUi,
    pub chat: ChatUi,
    pub nanos: [NanoSlotUi; 3],
    pub nano_battery: i32,
    pub weapon_battery: i32,
}

impl Default for GameplayUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            ui_scale: 1.0,
            // Exact clean `cnDisplayOption.SetDefault` values.
            npc_names_visible: false,
            balloon_chat_visible: true,
            player: PlayerStatusUi::default(),
            minimap: MinimapUi::default(),
            current_objective: CurrentObjectiveUi::default(),
            chat: ChatUi::default(),
            nanos: std::array::from_fn(|_| NanoSlotUi::default()),
            nano_battery: 0,
            weapon_battery: 0,
        }
    }
}

impl GameplayUiModel {
    /// Deterministic state matching the saved 1280x720 Retrobution comparison frame.
    pub fn retrobution_reference_frame() -> Self {
        Self {
            visible: true,
            ui_scale: 1.0,
            npc_names_visible: false,
            balloon_chat_visible: true,
            player: PlayerStatusUi {
                owner: 1,
                name: "Test Ser".to_owned(),
                level: 1,
                hp: 1000,
                max_hp: 1000,
                free_chat: false,
                allow_player_interaction: true,
            },
            minimap: MinimapUi {
                ratio: 8.0,
                map_name: "Tech Square".to_owned(),
                fusion_matter: 220,
                max_fusion_matter: 220,
                tiles: minimap_tiles(6320.32, 1871.77, 8.0),
                camera_heading_degrees: 0.0,
                avatar_heading_degrees: 0.0,
                waypoint: None,
                custom_waypoints: Vec::new(),
                markers: Vec::new(),
                player_marker_alpha: 1.0,
            },
            current_objective: CurrentObjectiveUi::default(),
            chat: ChatUi {
                visible: true,
                input_enabled: false,
                active: false,
                selected: ChatChannel::All,
                group_available: false,
                buddy_count: 0,
                window_size: Vec2::new(CHAT_DEFAULT_WIDTH, CHAT_DEFAULT_HEIGHT),
                alerts: [false; 3],
                text_colors: TextColorSettings::default(),
                quick_menu: QuickChatMenuUi::default(),
                input: String::new(),
                edit: TextEdit::default(),
                lines: vec![
                    ChatLineUi::normal("BUTTERCUP: Watch out!!"),
                    ChatLineUi::normal("NUMBUH FIVE: Holy cannoli! Where did you come from?"),
                    ChatLineUi::normal("BEN: Not now, Numbuh Five. We gotta move! Follow us!"),
                    ChatLineUi::normal("COMPUTRESS: Initiate training."),
                    ChatLineUi::normal(
                        "COMPUTRESS: Please move the mouse to the right to look at the marker.",
                    ),
                    ChatLineUi::tutorial("Move your mouse to the right and find the marker."),
                ],
            },
            nanos: std::array::from_fn(|_| NanoSlotUi::default()),
            nano_battery: 0,
            weapon_battery: 0,
        }
    }
}

/// Shared Enter-menu transition used by both `CnGuiChat` and `cnGUINanocom`.
///
/// Retrobution resets `fSlideMenu` to `1` whenever the menu target changes and
/// subtracts `Time.deltaTime * 2`, so the complete transition lasts 0.5 s.
#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct GameplayMenuTransition {
    pub open: bool,
    pub remaining: f32,
}

impl Default for GameplayMenuTransition {
    fn default() -> Self {
        Self {
            open: false,
            remaining: 0.0,
        }
    }
}

impl GameplayMenuTransition {
    pub fn set_open(&mut self, open: bool) {
        if self.open != open {
            self.open = open;
            self.remaining = 1.0;
        }
    }

    pub fn advance(&mut self, delta_seconds: f32) {
        self.remaining = (self.remaining - delta_seconds.max(0.0) * 2.0).max(0.0);
    }

    pub fn slide_offset(self) -> f32 {
        menu_slide_offset(self.open, self.remaining)
    }

    pub const fn visible_or_transitioning(self) -> bool {
        self.open || self.remaining > 0.0
    }
}

/// Exact easing used by `CnGuiChat.OnGUI` and `cnGUINanocom.OnGUI`.
pub fn menu_slide_offset(open: bool, remaining: f32) -> f32 {
    let squared = remaining.clamp(0.0, 1.0).powi(2);
    let phase = if open { squared } else { 1.0 - squared };
    (phase * std::f32::consts::FRAC_PI_2).sin().powi(2)
}

/// Runtime-generated offscreen avatar portrait. The HUD deliberately does not fake it with a PNG.
#[derive(Default, Resource)]
pub struct GameplayPortraitImage(pub Option<Handle<Image>>);

/// Three runtime-generated Nano camera targets used by `cnNanoWheel`.
#[derive(Default, Resource)]
pub struct GameplayNanoPortraitImages(pub [Option<Handle<Image>>; 3]);
