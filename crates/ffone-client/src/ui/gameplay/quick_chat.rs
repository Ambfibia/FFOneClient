//! Quick-chat/emote menu items, spawning and binding.

use super::assets::GameplayUiAssets;
use super::model::GameplayUiModel;
use super::text::chat_jeffe_14_font;
use crate::localization::LocalizedText;
use bevy::{
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    ui::widget::NodeImageMode,
};

/// Fresh Retrobution `MenuChatScript` animates the root list to x=40 for
/// Menu Chat and x=72 for Emotes. `DoMenuChat`/`DoEmote` add the shared
/// 149 px draw offset, leaving the visible list area fourteen pixels above
/// the bottom edge of the screen/chat anchor.
pub const QUICK_CHAT_MENU_LEVEL0_LEFT: f32 = 40.0;
pub const QUICK_CHAT_EMOTE_LEVEL0_LEFT: f32 = 72.0;
pub const QUICK_CHAT_LEVEL_BOTTOM: f32 = 14.0;
pub const QUICK_CHAT_ROW_HEIGHT: f32 = 18.0;
pub const QUICK_CHAT_MAX_ROWS: usize = 19;
pub const QUICK_CHAT_BOX_PATH: &str = "ui/en/gameplay/chat/chat_box.png";
pub const QUICK_CHAT_BOX_BYTES: u64 = 1_721;
pub const QUICK_CHAT_BOX_SHA256: &str =
    "9f2d695bf6efc5e82fa4bff491c1ad2a6b99ed0350be50f0d41c5e74d9df7041";
pub const QUICK_CHAT_HOVER_PATH: &str = "ui/en/gameplay/chat/menuchat_sel_over.png";
pub const QUICK_CHAT_HOVER_BYTES: u64 = 179;
pub const QUICK_CHAT_HOVER_SHA256: &str =
    "6b02648f8739d26e520ea46d36f847e2cd9d9240891da4a9b561cd0c7eb5717c";
pub const QUICK_CHAT_NEXT_HOVER_PATH: &str = "ui/en/gameplay/chat/menuchat_over.png";
pub const QUICK_CHAT_NEXT_HOVER_BYTES: u64 = 400;
pub const QUICK_CHAT_NEXT_HOVER_SHA256: &str =
    "5fb67d5524b0c7c261795c0e4a9cc0e3b10a43f136bf45ec5d5ca6838fde98e7";
pub const QUICK_CHAT_NEXT_ARROW_PATH: &str = "ui/en/gameplay/chat/next-arrow.png";
pub const QUICK_CHAT_NEXT_ARROW_BYTES: u64 = 233;
pub const QUICK_CHAT_NEXT_ARROW_SHA256: &str =
    "532501f05c5bf5fd58ded191664969763ca2db091030c078e48388c1b32dc8b2";
pub const QUICK_CHAT_EMOTE_ICON_PATHS: [&str; 19] = [
    "ui/en/gameplay/chat/emotes/hello.png",
    "ui/en/gameplay/chat/emotes/goodbye.png",
    "ui/en/gameplay/chat/emotes/yes.png",
    "ui/en/gameplay/chat/emotes/no.png",
    "ui/en/gameplay/chat/emotes/okay.png",
    "ui/en/gameplay/chat/emotes/thanks.png",
    "ui/en/gameplay/chat/emotes/happy.png",
    "ui/en/gameplay/chat/emotes/sad.png",
    "ui/en/gameplay/chat/emotes/angry.png",
    "ui/en/gameplay/chat/emotes/scared.png",
    "ui/en/gameplay/chat/emotes/love.png",
    "ui/en/gameplay/chat/emotes/laugh.png",
    "ui/en/gameplay/chat/emotes/taunt.png",
    "ui/en/gameplay/chat/emotes/cheer.png",
    "ui/en/gameplay/chat/emotes/clap.png",
    "ui/en/gameplay/chat/emotes/flex.png",
    "ui/en/gameplay/chat/emotes/dance.png",
    "ui/en/gameplay/chat/emotes/chill-out.png",
    "ui/en/gameplay/chat/emotes/extras.png",
];
pub const QUICK_CHAT_EMOTE_ICON_BYTES: [u64; 19] = [
    343, 337, 275, 278, 313, 710, 421, 507, 403, 369, 322, 393, 430, 519, 522, 357, 308, 410, 311,
];
pub const QUICK_CHAT_EMOTE_ICON_SHA256: [&str; 19] = [
    "65ae72411f898c2270b82f1831d197e334c56893ee08f701c419b4fc838ffad8",
    "f4dc1431b9757517f4f2f0a76ce8be0d1593521b0df8c91b8f0b3e7caee25b9a",
    "684984ef38abb9f3fbe8ced53581b0529f3ac7fba11525707c0eed95ffd12662",
    "523a65c3045dcc7ac19c9e160d944fc4128c47b61ba6925bd1df49ece56c0f40",
    "bb2f110c86932b15f69f95ea0e28e9a1ad32ccf75797fa1122e7264d0d8b93ef",
    "e4a719b103e83e5ae490f1a2e23582cc24d336b90f7248df9d93d7a51e65649f",
    "ca966229cb4337a0afbab1a3239a6dd80388946d8e81e6af6cdd9f0d78c405b9",
    "01effdc6091d04207a5244f41cd4d9c74b8124293b59905ae2b87c2688344c62",
    "444c5116ea55df71f69e398f90470905f1ea3e3d8bfa6ecd2f98af84d15a7220",
    "9e75c5f09484dbc62686f5ddf1e4d9b5fcd72e4fc8e43b6ea099ed9cd12f6f88",
    "1dea2b1a64e82b9dc742753a9c365994dc6ba0009ef711e94f9e766f2edd6807",
    "98f9cbaec8eca031a04bdec367770d52fbef7211728bd578d57312ae3425ce01",
    "758ee7cf297d9a22241d3e2c07f39cd51a4c0d1d02ae85c9b133b0f70433d1c8",
    "77fb4bf9d9dd6beb717a361bc1f8e40cd5737d41274044d13ff19660c6ca4d1c",
    "c37ede48a0e3b8d56f6a5b75ed9051ab06b4f3b0d8a7ab261c2bb782f8d8109b",
    "f5988da0dc26468259eed84cbb7cf47a87cbfb015d05e16ce90036cf7ab99626",
    "2d09615d67dbf7dfafd61efabda8eed6e61d4575f270713784e837abfc3c68f5",
    "a614cd0173000e9d9159d50ac5160cc4b80abbd73b1a8c4f9fee94a3a351dc43",
    "6381e2eeb58766cc6aea7f98e9eb334679a7efe6b87165ff736b3202033ac95a",
];
pub const QUICK_CHAT_EMOTE_ICON_SIZES: [(f32, f32); 19] = [
    (12.0, 13.0),
    (12.0, 13.0),
    (14.0, 13.0),
    (14.0, 13.0),
    (12.0, 13.0),
    (26.0, 13.0),
    (14.0, 14.0),
    (24.0, 14.0),
    (14.0, 14.0),
    (14.0, 14.0),
    (16.0, 13.0),
    (14.0, 14.0),
    (14.0, 15.0),
    (22.0, 15.0),
    (24.0, 13.0),
    (17.0, 16.0),
    (16.0, 13.0),
    (15.0, 13.0),
    (16.0, 15.0),
];
pub(super) const QUICK_CHAT_MIN_COLUMN_WIDTH: f32 = 92.0;
pub(super) const QUICK_CHAT_MAX_COLUMN_WIDTH: f32 = 270.0;
pub(super) const QUICK_CHAT_AREA_WIDTH_OVERHEAD: f32 = 38.0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum QuickChatMenuMode {
    #[default]
    Closed,
    MenuChat,
    Emotes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuickChatDynamicText {
    None,
    Location,
    Mission,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuickChatItem {
    pub id: i32,
    pub parent_id: i32,
    pub key: &'static str,
    pub fallback: &'static str,
    pub emote_code: i32,
    pub container: bool,
    pub dynamic: QuickChatDynamicText,
}

macro_rules! quick_item {
    ($id:expr, $parent:expr, $fallback:literal) => {
        QuickChatItem {
            id: $id,
            parent_id: $parent,
            key: concat!(
                "ui.hud.chat.quick.",
                stringify!($parent),
                ".",
                stringify!($id)
            ),
            fallback: $fallback,
            emote_code: 0,
            container: false,
            dynamic: QuickChatDynamicText::None,
        }
    };
    ($id:expr, $parent:expr, $fallback:literal, emote = $emote:expr) => {
        QuickChatItem {
            id: $id,
            parent_id: $parent,
            key: concat!(
                "ui.hud.chat.quick.",
                stringify!($parent),
                ".",
                stringify!($id)
            ),
            fallback: $fallback,
            emote_code: $emote,
            container: false,
            dynamic: QuickChatDynamicText::None,
        }
    };
    ($id:expr, $parent:expr, $fallback:literal, container) => {
        QuickChatItem {
            id: $id,
            parent_id: $parent,
            key: concat!(
                "ui.hud.chat.quick.",
                stringify!($parent),
                ".",
                stringify!($id)
            ),
            fallback: $fallback,
            emote_code: 0,
            container: true,
            dynamic: QuickChatDynamicText::None,
        }
    };
    ($id:expr, $parent:expr, $fallback:literal, dynamic = $dynamic:expr) => {
        QuickChatItem {
            id: $id,
            parent_id: $parent,
            key: concat!(
                "ui.hud.chat.quick.",
                stringify!($parent),
                ".",
                stringify!($id)
            ),
            fallback: $fallback,
            emote_code: 0,
            container: false,
            dynamic: $dynamic,
        }
    };
}

pub const MENU_CHAT_ITEMS: &[QuickChatItem] = &[
    quick_item!(1, 0, "Hello.", emote = 4),
    quick_item!(2, 0, "Goodbye.", emote = 21),
    quick_item!(3, 0, "Yes.", emote = 8),
    quick_item!(4, 0, "No.", emote = 10),
    quick_item!(5, 0, "Maybe."),
    quick_item!(6, 0, "Okay.", emote = 13),
    quick_item!(7, 0, "Combat...", container),
    quick_item!(8, 0, "Unfriendly...", container),
    quick_item!(9, 0, "Friendly...", container),
    quick_item!(10, 0, "General...", container),
    quick_item!(11, 0, "Missions...", container),
    quick_item!(12, 0, "Travel...", container),
    quick_item!(13, 0, "Trade...", container),
    quick_item!(14, 0, "Emote...", container),
    quick_item!(61, 7, "We're ready."),
    quick_item!(62, 7, "They don't stand a chance!"),
    quick_item!(63, 7, "Attack!"),
    quick_item!(64, 7, "Run!"),
    quick_item!(65, 7, "Help me!"),
    quick_item!(66, 7, "Look out!"),
    quick_item!(67, 7, "Use your A Nano!"),
    quick_item!(68, 7, "Use your B Nano!"),
    quick_item!(69, 7, "Use your C Nano!"),
    quick_item!(70, 7, "Heal me!"),
    quick_item!(71, 7, "Use your stun!"),
    quick_item!(73, 7, "I need health."),
    quick_item!(74, 7, "My Nanos are tired."),
    quick_item!(75, 7, "Let's get 'em!"),
    quick_item!(76, 7, "We did it!"),
    quick_item!(77, 7, "Let's take a break."),
    quick_item!(51, 8, "Please stop that."),
    quick_item!(52, 8, "Go away."),
    quick_item!(53, 8, "You're mean."),
    quick_item!(54, 8, "Hey!"),
    quick_item!(55, 8, "Leave me alone."),
    quick_item!(56, 8, "I'm ignoring you."),
    quick_item!(57, 8, "I'm busy right now."),
    quick_item!(58, 8, "Sorry, I'm working on another quest."),
    quick_item!(59, 8, "Sorry, I'm helping a friend."),
    quick_item!(60, 8, "I can't right now!"),
    quick_item!(36, 9, "Hello there."),
    quick_item!(37, 9, "Will you be my buddy?"),
    quick_item!(38, 9, "That was fun."),
    quick_item!(39, 9, "Can I trade with you?"),
    quick_item!(40, 9, "Yay!"),
    quick_item!(41, 9, "Awesome!"),
    quick_item!(42, 9, "Congratulations!"),
    quick_item!(43, 9, "Have fun!"),
    quick_item!(44, 9, "Good luck!"),
    quick_item!(45, 9, "Great job!"),
    quick_item!(46, 9, "Super!"),
    quick_item!(47, 9, "I like your outfit.", emote = 7),
    quick_item!(48, 9, "I like your weapon.", emote = 7),
    quick_item!(49, 9, "I like your Nano.", emote = 7),
    quick_item!(50, 9, "See you around."),
    quick_item!(21, 10, "Hey there.", emote = 5),
    quick_item!(22, 10, "What's up?"),
    quick_item!(23, 10, "Thanks."),
    quick_item!(24, 10, "No thanks."),
    quick_item!(25, 10, "Sorry!"),
    quick_item!(26, 10, "Oops!"),
    quick_item!(27, 10, "No problem."),
    quick_item!(28, 10, "Any time!"),
    quick_item!(29, 10, "I have to go."),
    quick_item!(30, 10, "I have to go soon."),
    quick_item!(31, 10, "I'll be back later."),
    quick_item!(32, 10, "I'll be right back."),
    quick_item!(33, 10, "Hurry up."),
    quick_item!(34, 10, "See you later!"),
    quick_item!(35, 10, "Bye!"),
    quick_item!(13, 11, "Wanna play?"),
    quick_item!(14, 11, "Can I join you?"),
    quick_item!(15, 11, "Let's play."),
    quick_item!(16, 11, "What mission are you doing?"),
    quick_item!(17, 11, "Let's do a different mission."),
    quick_item!(18, 11, "This mission is hard!"),
    quick_item!(19, 11, "Let's get another buddy."),
    quick_item!(20, 11, "We need all the help we can get!"),
    quick_item!(
        102,
        11,
        "I'm doing {mission}.",
        dynamic = QuickChatDynamicText::Mission
    ),
    quick_item!(
        103,
        11,
        "Can you help me with {mission}?",
        dynamic = QuickChatDynamicText::Mission
    ),
    quick_item!(
        104,
        11,
        "I need help with {mission}.",
        dynamic = QuickChatDynamicText::Mission
    ),
    quick_item!(2, 12, "Follow me."),
    quick_item!(3, 12, "I'm lost!"),
    quick_item!(5, 12, "Where are you?"),
    quick_item!(7, 12, "Take the Slider."),
    quick_item!(8, 12, "Warp to me."),
    quick_item!(9, 12, "I'll warp to you."),
    quick_item!(10, 12, "On the way."),
    quick_item!(11, 12, "I need to change Nanos first."),
    quick_item!(
        12,
        12,
        "I am in {location}.",
        dynamic = QuickChatDynamicText::Location
    ),
    quick_item!(78, 13, "Okay."),
    quick_item!(79, 13, "Sounds good."),
    quick_item!(80, 13, "No way!"),
    quick_item!(81, 13, "More?"),
    quick_item!(82, 13, "Less?"),
    quick_item!(83, 13, "Enough?"),
    quick_item!(84, 13, "Good!"),
    quick_item!(85, 13, "Deal!"),
    quick_item!(86, 14, "Boo-hoo!", emote = 1),
    quick_item!(87, 14, "Grr!", emote = 2),
    quick_item!(88, 14, "Yipe!", emote = 3),
    quick_item!(89, 14, "Hi!", emote = 4),
    quick_item!(90, 14, "Thanks!", emote = 5),
    quick_item!(91, 14, "Let's Dance!", emote = 6),
    quick_item!(92, 14, "Awww...!", emote = 7),
    quick_item!(93, 14, "Yes.", emote = 8),
    quick_item!(94, 14, "Ha ha ha!", emote = 9),
    quick_item!(95, 14, "No!", emote = 10),
    quick_item!(96, 14, "I'm strong!", emote = 11),
    quick_item!(97, 14, "Thbbt!", emote = 12),
    quick_item!(98, 14, "Okay.", emote = 13),
    quick_item!(99, 14, "Applaud.", emote = 14),
    quick_item!(100, 14, "That's great!", emote = 15),
    quick_item!(101, 14, "Woo-hoo!", emote = 16),
];

macro_rules! emote_item {
    ($id:expr, $parent:expr, $fallback:literal, $code:expr, $container:expr) => {
        QuickChatItem {
            id: $id,
            parent_id: $parent,
            key: concat!("ui.hud.chat.emote.", stringify!($id)),
            fallback: $fallback,
            emote_code: $code,
            container: $container,
            dynamic: QuickChatDynamicText::None,
        }
    };
}

pub const EMOTE_CHAT_ITEMS: &[QuickChatItem] = &[
    emote_item!(0, 0, "Hello", 4, false),
    emote_item!(1, 0, "Goodbye", 21, false),
    emote_item!(2, 0, "Yes", 8, false),
    emote_item!(3, 0, "No", 10, false),
    emote_item!(4, 0, "Okay", 13, false),
    emote_item!(5, 0, "Thanks", 5, false),
    emote_item!(6, 0, "Happy", 16, false),
    emote_item!(7, 0, "Sad", 1, false),
    emote_item!(8, 0, "Angry", 2, false),
    emote_item!(9, 0, "Scared", 3, false),
    emote_item!(10, 0, "Love", 7, false),
    emote_item!(11, 0, "Laugh", 9, false),
    emote_item!(12, 0, "Taunt", 12, false),
    emote_item!(13, 0, "Cheer", 15, false),
    emote_item!(14, 0, "Clap", 14, false),
    emote_item!(15, 0, "Flex", 11, false),
    emote_item!(16, 0, "Dance...", 6, true),
    emote_item!(17, 0, "Chill Out...", 22, true),
    emote_item!(18, 0, "Retrobution...", 0, true),
    emote_item!(111, 16, "Hipshake", 6, false),
    emote_item!(112, 16, "Point and Shimmy", 17, false),
    emote_item!(113, 16, "Break it Down", 18, false),
    emote_item!(114, 16, "Rock it Out", 19, false),
    emote_item!(115, 16, "All Dances", 20, false),
    emote_item!(211, 17, "Sit", 22, false),
    emote_item!(212, 17, "Lie Down", 23, false),
    emote_item!(213, 17, "Lounge", 24, false),
    // Clean emote table slots 25-30 are empty; 31-44 are native Retrobution clips.
    emote_item!(317, 18, "Dance 02", 31, false),
    emote_item!(318, 18, "Dance 04", 32, false),
    emote_item!(319, 18, "Dance 07", 33, false),
    emote_item!(320, 18, "Dance 08", 34, false),
    emote_item!(321, 18, "Dance 10", 35, false),
    emote_item!(322, 18, "Dance 13", 36, false),
    emote_item!(323, 18, "Dance 15", 37, false),
    emote_item!(324, 18, "Dance 18", 38, false),
    emote_item!(325, 18, "Dance 19", 39, false),
    emote_item!(326, 18, "Dance 20", 40, false),
    emote_item!(327, 18, "Bully Dance", 41, false),
    emote_item!(328, 18, "Tell Me Dance", 42, false),
    emote_item!(329, 18, "Cat Pose", 43, false),
    emote_item!(330, 18, "Idol Pose", 44, false),
];

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct QuickChatMenuUi {
    pub mode: QuickChatMenuMode,
    pub open_level0: Option<i32>,
    pub open_level1: Option<i32>,
}

impl QuickChatMenuUi {
    pub fn toggle(&mut self, mode: QuickChatMenuMode) {
        // Clean small chat renders the selected source button with `ButtonLock`:
        // clicking it again cannot close the list, while the other button swaps
        // MenuChat and Emotes and clears every open child level.
        if self.mode != mode {
            self.mode = mode;
            self.open_level0 = None;
            self.open_level1 = None;
        }
    }

    pub fn close(&mut self) {
        self.mode = QuickChatMenuMode::Closed;
        self.open_level0 = None;
        self.open_level1 = None;
    }

    #[must_use]
    pub fn source_items(&self) -> &'static [QuickChatItem] {
        match self.mode {
            QuickChatMenuMode::Closed => &[],
            QuickChatMenuMode::MenuChat => MENU_CHAT_ITEMS,
            QuickChatMenuMode::Emotes => EMOTE_CHAT_ITEMS,
        }
    }

    #[must_use]
    pub fn visible_items(&self, level: usize) -> Vec<&'static QuickChatItem> {
        let parent = match level {
            0 => Some(0),
            1 => self.open_level0,
            2 => self.open_level1,
            _ => None,
        };
        let Some(parent) = parent else {
            return Vec::new();
        };
        self.source_items()
            .iter()
            .filter(|item| item.parent_id == parent)
            .collect()
    }

    pub fn select(&mut self, item: QuickChatItem) -> Option<QuickChatItem> {
        if item.container {
            if item.parent_id == 0 {
                self.open_level0 = Some(item.id);
                self.open_level1 = None;
            } else {
                self.open_level1 = Some(item.id);
            }
            None
        } else {
            self.close();
            Some(item)
        }
    }
}

impl QuickChatItem {
    #[must_use]
    pub fn localized(
        self,
        location: impl Into<String>,
        mission: impl Into<String>,
    ) -> LocalizedText {
        match self.dynamic {
            QuickChatDynamicText::None => LocalizedText::new(self.key, self.fallback),
            QuickChatDynamicText::Location => {
                LocalizedText::new(self.key, self.fallback).with_arg("location", location)
            }
            QuickChatDynamicText::Mission => {
                LocalizedText::new(self.key, self.fallback).with_arg("mission", mission)
            }
        }
    }
}

#[derive(Component)]
pub(super) struct MenuChatButton;
#[derive(Component)]
pub(super) struct EmoteButton;

#[derive(Component)]
pub(super) struct QuickChatLevelRoot(pub(super) usize);
#[derive(Component)]
pub(super) struct QuickChatRowButton {
    pub(super) level: usize,
    pub(super) row: usize,
    pub(super) item: Option<QuickChatItem>,
}
#[derive(Component)]
pub(super) struct QuickChatRowText {
    pub(super) level: usize,
    pub(super) row: usize,
}
#[derive(Component)]
pub(super) struct QuickChatRowIcon {
    pub(super) level: usize,
    pub(super) row: usize,
}
#[derive(Component)]
pub(super) struct QuickChatRowArrow {
    pub(super) level: usize,
    pub(super) row: usize,
}

pub(super) fn spawn_quick_chat_columns(
    parent: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
) {
    for level in 0..3 {
        parent
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    left: px(QUICK_CHAT_MENU_LEVEL0_LEFT),
                    bottom: px(QUICK_CHAT_LEVEL_BOTTOM),
                    width: px(QUICK_CHAT_MIN_COLUMN_WIDTH + 8.0),
                    height: px(QUICK_CHAT_ROW_HEIGHT),
                    ..default()
                },
                ImageNode {
                    image: assets.quick_chat_box.clone(),
                    image_mode: NodeImageMode::Sliced(TextureSlicer {
                        border: BorderRect {
                            min_inset: Vec2::new(15.0, 15.0),
                            max_inset: Vec2::new(20.0, 20.0),
                        },
                        center_scale_mode: SliceScaleMode::Stretch,
                        sides_scale_mode: SliceScaleMode::Stretch,
                        max_corner_scale: 1.0,
                    }),
                    ..default()
                },
                QuickChatLevelRoot(level),
                // Enter/Nanocom is a separate root at camera order + 10.
                // A local ZIndex cannot lift these HUD descendants above it.
                GlobalZIndex(0),
                ZIndex(super::GAMEPLAY_UI_CAMERA_ORDER as i32 + 11 + level as i32),
            ))
            .with_children(|column| {
                for row in 0..QUICK_CHAT_MAX_ROWS {
                    column
                        .spawn((
                            Button,
                            Node {
                                position_type: PositionType::Absolute,
                                display: Display::None,
                                left: px(0),
                                top: px(row as f32 * QUICK_CHAT_ROW_HEIGHT),
                                width: px(QUICK_CHAT_MIN_COLUMN_WIDTH),
                                height: px(QUICK_CHAT_ROW_HEIGHT),
                                ..default()
                            },
                            ImageNode {
                                color: Color::NONE,
                                image_mode: NodeImageMode::Stretch,
                                ..default()
                            },
                            QuickChatRowButton {
                                level,
                                row,
                                item: None,
                            },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    display: Display::None,
                                    left: px(3),
                                    top: px(6),
                                    ..default()
                                },
                                ImageNode::default(),
                                Pickable::IGNORE,
                                QuickChatRowIcon { level, row },
                            ));
                            button.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(8),
                                    top: px(0),
                                    right: px(2),
                                    height: percent(100),
                                    align_items: AlignItems::Center,
                                    padding: UiRect::bottom(px(2)),
                                    ..default()
                                },
                                Text::new(""),
                                LocalizedText::new("ui.content.passthrough", "{text}")
                                    .with_arg("text", ""),
                                chat_jeffe_14_font(&assets.jeffe_font),
                                TextColor(Color::WHITE),
                                TextLayout::new(Justify::Left, LineBreak::NoWrap),
                                Pickable::IGNORE,
                                QuickChatRowText { level, row },
                            ));
                            button.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    display: Display::None,
                                    right: px(-3),
                                    top: px(3),
                                    width: px(8),
                                    height: px(14),
                                    ..default()
                                },
                                ImageNode::new(assets.quick_chat_next_arrow.clone()),
                                Pickable::IGNORE,
                                QuickChatRowArrow { level, row },
                            ));
                        });
                }
            });
    }
}

pub(super) fn quick_chat_list_width(
    mode: QuickChatMenuMode,
    level: usize,
    items: &[&QuickChatItem],
) -> f32 {
    if level == 0 {
        return match mode {
            QuickChatMenuMode::MenuChat => 112.0,
            QuickChatMenuMode::Emotes => 97.0,
            QuickChatMenuMode::Closed => 0.0,
        };
    }
    let longest = items
        .iter()
        .map(|item| item.fallback.trim_end_matches('.').chars().count())
        .max()
        .unwrap_or_default() as f32;
    (longest * 7.0 + 20.0).clamp(
        QUICK_CHAT_MIN_COLUMN_WIDTH - QUICK_CHAT_AREA_WIDTH_OVERHEAD,
        QUICK_CHAT_MAX_COLUMN_WIDTH - QUICK_CHAT_AREA_WIDTH_OVERHEAD,
    )
}

pub(super) fn quick_chat_level_left(
    mode: QuickChatMenuMode,
    level: usize,
    list_widths: &[f32; 3],
) -> f32 {
    match (mode, level) {
        (QuickChatMenuMode::MenuChat, 0) => QUICK_CHAT_MENU_LEVEL0_LEFT,
        (QuickChatMenuMode::Emotes, 0) => QUICK_CHAT_EMOTE_LEVEL0_LEFT,
        (QuickChatMenuMode::MenuChat, 1) => 60.0 + list_widths[0],
        (QuickChatMenuMode::Emotes, 1) => 95.0 + list_widths[0],
        (QuickChatMenuMode::MenuChat, 2) => 60.0 + list_widths[0] + list_widths[1],
        (QuickChatMenuMode::Emotes, 2) => 95.0 + list_widths[0] + list_widths[1],
        _ => 0.0,
    }
}

pub(super) fn quick_chat_level_bottom(
    mode: QuickChatMenuMode,
    level: usize,
    items: &[Vec<&QuickChatItem>; 3],
) -> f32 {
    if level == 0 {
        return QUICK_CHAT_LEVEL_BOTTOM;
    }
    let previous = &items[level - 1];
    let current = &items[level];
    if previous.is_empty() || current.is_empty() {
        return QUICK_CHAT_LEVEL_BOTTOM;
    }
    let previous_height = previous.len() as f32 * QUICK_CHAT_ROW_HEIGHT;
    let current_height = current.len() as f32 * QUICK_CHAT_ROW_HEIGHT;
    let selected_id = if level == 1 {
        previous
            .iter()
            .position(|item| item.container && current[0].parent_id == item.id)
            .unwrap_or_default()
    } else {
        previous
            .iter()
            .position(|item| item.container && current[0].parent_id == item.id)
            .unwrap_or_default()
    };
    if current_height > previous_height {
        return QUICK_CHAT_LEVEL_BOTTOM;
    }
    let selected_step = if mode == QuickChatMenuMode::MenuChat {
        QUICK_CHAT_ROW_HEIGHT * 0.5
    } else {
        QUICK_CHAT_ROW_HEIGHT
    };
    (previous_height - current_height - selected_id as f32 * selected_step - 4.0)
        .max(QUICK_CHAT_LEVEL_BOTTOM)
}

pub(super) fn bind_quick_chat_menu(
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    mut roots: Query<(&QuickChatLevelRoot, &mut Node), Without<QuickChatRowButton>>,
    mut rows: Query<(&mut QuickChatRowButton, &mut Node), Without<QuickChatLevelRoot>>,
    mut labels: Query<
        (&QuickChatRowText, &mut Node, &mut LocalizedText),
        (
            Without<QuickChatLevelRoot>,
            Without<QuickChatRowButton>,
            Without<QuickChatRowIcon>,
            Without<QuickChatRowArrow>,
        ),
    >,
    mut icons: Query<
        (&QuickChatRowIcon, &mut Node, &mut ImageNode),
        (
            Without<QuickChatLevelRoot>,
            Without<QuickChatRowButton>,
            Without<QuickChatRowText>,
            Without<QuickChatRowArrow>,
        ),
    >,
    mut arrows: Query<
        (&QuickChatRowArrow, &mut Node),
        (
            Without<QuickChatLevelRoot>,
            Without<QuickChatRowButton>,
            Without<QuickChatRowText>,
            Without<QuickChatRowIcon>,
        ),
    >,
) {
    let visible = model.visible
        && model.chat.visible
        && model.chat.input_enabled
        && model.chat.quick_menu.mode != QuickChatMenuMode::Closed;
    let items: [Vec<&QuickChatItem>; 3] = std::array::from_fn(|level| {
        if visible {
            model.chat.quick_menu.visible_items(level)
        } else {
            Vec::new()
        }
    });
    let list_widths = std::array::from_fn(|level| {
        quick_chat_list_width(model.chat.quick_menu.mode, level, &items[level])
    });

    for (marker, mut node) in &mut roots {
        let level_items = &items[marker.0];
        node.display = if visible && !level_items.is_empty() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display == Display::None {
            continue;
        }
        node.left = px(quick_chat_level_left(
            model.chat.quick_menu.mode,
            marker.0,
            &list_widths,
        ));
        node.bottom = px(quick_chat_level_bottom(
            model.chat.quick_menu.mode,
            marker.0,
            &items,
        ));
        node.width = px(list_widths[marker.0] + QUICK_CHAT_AREA_WIDTH_OVERHEAD);
        node.height = px(level_items.len() as f32 * QUICK_CHAT_ROW_HEIGHT + 5.0);
    }

    for (mut marker, mut node) in &mut rows {
        let item = items[marker.level].get(marker.row);

        marker.item = item.copied().copied();
        node.display = if marker.item.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        node.width = px(list_widths[marker.level] + QUICK_CHAT_AREA_WIDTH_OVERHEAD - 20.0);
    }

    for (marker, mut node, mut localized) in &mut labels {
        node.left = px(
            if model.chat.quick_menu.mode == QuickChatMenuMode::Emotes && marker.level == 0 {
                32
            } else {
                8
            },
        );
        *localized = items[marker.level]
            .get(marker.row)
            .map(|item| {
                item.localized(
                    model.minimap.map_name.clone(),
                    model.current_objective.title.clone(),
                )
            })
            .unwrap_or_else(|| {
                LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "")
            });
    }

    for (marker, mut node, mut image) in &mut icons {
        let item = items[marker.level].get(marker.row);
        let emote_root = visible
            && model.chat.quick_menu.mode == QuickChatMenuMode::Emotes
            && marker.level == 0
            && item.is_some();
        node.display = if emote_root {
            Display::Flex
        } else {
            Display::None
        };
        if emote_root {
            let (width, height) = QUICK_CHAT_EMOTE_ICON_SIZES[marker.row];
            node.left = px(3.0 + (26.0 - width) * 0.5);
            node.width = px(width);
            node.height = px(height);
            image.image = assets.quick_chat_emote_icons[marker.row].clone();
        }
    }

    for (marker, mut node) in &mut arrows {
        let item = items[marker.level].get(marker.row);
        let selected = item.is_some_and(|item| {
            marker.level == 0 && model.chat.quick_menu.open_level0 == Some(item.id)
        });
        node.display = if item.is_some_and(|item| item.container) && !selected {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(super) fn bind_quick_chat_row_visuals(
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    mut rows: Query<(&Interaction, &QuickChatRowButton, &mut ImageNode)>,
    mut labels: Query<(&QuickChatRowText, &mut TextColor)>,
) {
    let mut active = [[false; QUICK_CHAT_MAX_ROWS]; 3];
    for (interaction, marker, mut image) in &mut rows {
        let is_hovered = marker.item.is_some()
            && matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
        let is_selected = marker.item.is_some_and(|item| match marker.level {
            0 => model.chat.quick_menu.open_level0 == Some(item.id),
            1 => model.chat.quick_menu.open_level1 == Some(item.id),
            _ => false,
        });
        active[marker.level][marker.row] = is_hovered || is_selected;
        let container_visual =
            marker.item.is_some_and(|item| item.container) && (is_hovered || is_selected);
        image.image = if is_selected {
            assets.quick_chat_next_hover.clone()
        } else if is_hovered {
            if container_visual {
                assets.quick_chat_next_hover.clone()
            } else {
                assets.quick_chat_hover.clone()
            }
        } else {
            default()
        };
        image.image_mode = if container_visual {
            NodeImageMode::Sliced(TextureSlicer {
                border: BorderRect {
                    min_inset: Vec2::new(0.0, 0.0),
                    max_inset: Vec2::new(20.0, 0.0),
                },
                center_scale_mode: SliceScaleMode::Stretch,
                sides_scale_mode: SliceScaleMode::Stretch,
                max_corner_scale: 1.0,
            })
        } else {
            NodeImageMode::Stretch
        };
        image.color = if is_hovered || is_selected {
            Color::WHITE
        } else {
            Color::NONE
        };
    }
    for (marker, mut color) in &mut labels {
        color.0 = if active[marker.level][marker.row] {
            Color::srgb(0.0, 0.34117648, 0.5019608)
        } else {
            Color::WHITE
        };
    }
}
