//! Chat constants, layout, channels/lines, input history and UTF-16 input limits.

use super::actions::GameplayUiAction;
use super::hud::GameplayUiRect;
use super::quick_chat::QuickChatMenuUi;
use crate::text_edit::TextEdit;
use crate::{localization::LocalizedText, option_ui::TextColorSettings};
use bevy::prelude::*;
use std::collections::VecDeque;

pub const CHAT_DEFAULT_WIDTH: f32 = 440.0;
pub const CHAT_DEFAULT_HEIGHT: f32 = 150.0;
pub const CHAT_MIN_WIDTH: f32 = 325.0;
pub const CHAT_MIN_HEIGHT: f32 = 135.0;
pub const CHAT_MAX_WIDTH: f32 = 550.0;
pub const CHAT_MAX_HEIGHT: f32 = 250.0;
pub const CHAT_TAB_WIDTH: f32 = 135.0;
pub const CHAT_TAB_HEIGHT: f32 = 14.0;
pub const CHAT_RESIZE_RECT_WIDTH: f32 = 25.0;
pub const CHAT_RESIZE_RECT_HEIGHT: f32 = 25.0;
pub const CHAT_RESIZE_NORMAL_PATH: &str = "ui/en/gameplay/chat/resize-normal.png";
pub const CHAT_RESIZE_NORMAL_BYTES: u64 = 1_100;
pub const CHAT_RESIZE_NORMAL_SHA256: &str =
    "6993b6d230bd2f14058908603ae0b996ff0bc9534f136f795dd0442cb56213d0";
pub const CHAT_RESIZE_HOVER_PATH: &str = "ui/en/gameplay/chat/resize-hover.png";
pub const CHAT_RESIZE_HOVER_BYTES: u64 = 952;
pub const CHAT_RESIZE_HOVER_SHA256: &str =
    "ec7be526b255e2e70e5367b5b42eed93bb20953563f0b074f213ab238d5259db";
pub const CHAT_ACTIVE_TEXT_FIELD_PATH: &str = "ui/en/launcher/login/ff-textfield-normal.png";
pub const CHAT_ACTIVE_TEXT_FIELD_BYTES: u64 = 217;
pub const CHAT_ACTIVE_TEXT_FIELD_SHA256: &str =
    "f5a79b4fda32a41147172a2ef4cfccbf62beed87b01f7a31dc4acf9e82cd1556";
pub const CHAT_EMPTY_STATE_BACKGROUND_PATH: &str = "ui/en/gameplay/chat/empty-state-background.png";
pub const CHAT_EMPTY_STATE_BACKGROUND_BYTES: u64 = 804;
pub const CHAT_EMPTY_STATE_BACKGROUND_SHA256: &str =
    "ebeae8c153a81e3f47b775acb588c8bdb4409629e0d0af784a12f1f745572f0b";
pub const CHAT_BUDDY_ICON_PATH: &str = "ui/en/gameplay/chat/icons/buddy.png";
pub const CHAT_BUDDY_ICON_BYTES: u64 = 1_002;
pub const CHAT_BUDDY_ICON_SHA256: &str =
    "b5adac803b237d24b70beec8f72521c1a76491f7e9bdac326b3f4282296fa81a";
pub const CHAT_GROUP_ICON_PATH: &str = "ui/en/gameplay/chat/icons/group.png";
pub const CHAT_GROUP_ICON_BYTES: u64 = 1_024;
pub const CHAT_GROUP_ICON_SHA256: &str =
    "900271d59bfc88c24cb508fd719e630ae1ac1c09c5efe0961ea4fdd5d59fc973";

pub(super) const CHAT_HISTORY_CAPACITY: usize = 50;
pub const CHAT_SENT_HISTORY_CAPACITY: usize = 20;
pub const CHAT_INPUT_UTF16_LIMIT: usize = 127;
pub(super) const CHAT_LOG_LINE_GAP: f32 = 4.0;
pub(super) const CHAT_LOG_BOTTOM_PADDING: f32 = 5.0;
pub(super) const CHAT_SCROLL_WHEEL_LINE: f32 = 15.0;
pub(super) const CHAT_SCROLL_TO_LATEST: f32 = 1_000_000.0;
pub(super) const CHAT_SCROLLBAR_WIDTH: f32 = 16.0;
pub(super) const CHAT_SCROLLBAR_ARROW_HEIGHT: f32 = 12.0;
pub(super) const CHAT_SCROLLBAR_THUMB_WIDTH: f32 = 15.0;
pub(super) const CHAT_SCROLLBAR_THUMB_MIN_HEIGHT: f32 = 15.0;
pub(super) const CHAT_SCROLLBAR_TRACK_PATH: &str = "ui/en/gameplay/chat/scrollbar/track.png";
pub const CHAT_SCROLLBAR_TRACK_BYTES: u64 = 225;
pub const CHAT_SCROLLBAR_TRACK_SHA256: &str =
    "94cc317466de205d6336038b99626a3596ea5592298ecdc409f4e93c7fb83355";
pub(super) const CHAT_SCROLLBAR_THUMB_PATH: &str = "ui/en/gameplay/chat/scrollbar/thumb.png";
pub const CHAT_SCROLLBAR_THUMB_BYTES: u64 = 318;
pub const CHAT_SCROLLBAR_THUMB_SHA256: &str =
    "a7afff2d82ddc872cc99bc2695039530ce1e1a554dceb95f33e344804ef589ad";
pub(super) const CHAT_SCROLLBAR_UP_PATH: &str = "ui/en/gameplay/chat/scrollbar/up.png";
pub const CHAT_SCROLLBAR_UP_BYTES: u64 = 496;
pub const CHAT_SCROLLBAR_UP_SHA256: &str =
    "504525e1476d8a753f39821411d9fbdb9bdf7fd299496a0fd251d1dfc9be7463";
pub(super) const CHAT_SCROLLBAR_DOWN_PATH: &str = "ui/en/gameplay/chat/scrollbar/down.png";
pub const CHAT_SCROLLBAR_DOWN_BYTES: u64 = 481;
pub const CHAT_SCROLLBAR_DOWN_SHA256: &str =
    "de13be5d83a1e28ebd312c1c38f293dc28126514f90e92124084e431b5a149fd";
pub(super) const CHAT_INPUT_PADDING: f32 = 5.0;
pub(super) const CHAT_INPUT_FONT_SIZE: f32 = 11.0;
/// Approved vector calibration for primary `ChaletBook-Regular Small` path ID
/// 1018. Eleven pixels restores its fixed-raster advances; the serialized
/// line spacing avoids vertical drift between the clean four-pixel row gaps.
pub(super) const CHAT_CHALET_SMALL_LINE_HEIGHT: f32 = 12.072;
pub(super) const CHAT_CHALET_SMALL_Y_OFFSET: f32 = 3.0;
/// Centered fixed-raster adapter for clean `JEFFE___14` path ID 903. The
/// 14-pixel vector size preserves source horizontal advances; the measured
/// Y-only 0.70 scale restores the seven-pixel source glyph height.
pub(super) const CHAT_JEFFE_14_FONT_SIZE: f32 = 14.0;
pub(super) const CHAT_JEFFE_14_LINE_HEIGHT: f32 = 13.71;
pub(super) const CHAT_JEFFE_14_VERTICAL_SCALE: f32 = 0.7;
pub(super) const CHAT_INACTIVE_TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
pub(super) const CHAT_TAB_NORMAL_TEXT_COLOR: Color = Color::srgb(0.799_270_1, 1.0, 1.0);
pub(super) const CHAT_TAB_HOVER_TEXT_COLOR: Color = Color::srgb(0.0, 0.2, 0.4);
pub(super) const CHAT_TAB_SELECTED_TEXT_COLOR: Color = Color::srgb(0.0, 1.0, 1.0);
pub(super) const CHAT_EMPTY_STATE_TEXT_COLOR: Color = Color::srgb(0.8, 1.0, 1.0);
pub(super) const CHAT_NPC_TEXT_COLOR: Color = Color::WHITE;
pub(super) const CHAT_RECEIVE_TEXT_COLOR: Color = Color::srgb(1.0, 0.5, 0.0);
pub(super) const CHAT_DAMAGE_TEXT_COLOR: Color = Color::srgb(1.0, 0.0, 0.0);
pub(super) const CHAT_ATTACK_TEXT_COLOR: Color = Color::srgb(0.0, 0.0, 1.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ChatLayout {
    pub(super) size: Vec2,
    pub(super) background: GameplayUiRect,
    pub(super) entry: GameplayUiRect,
    pub(super) log: GameplayUiRect,
    pub(super) empty_state: GameplayUiRect,
    pub(super) tabs: [GameplayUiRect; 3],
    pub(super) menu: GameplayUiRect,
    pub(super) emote: GameplayUiRect,
    pub(super) input: GameplayUiRect,
    pub(super) send: GameplayUiRect,
    pub(super) resize: GameplayUiRect,
}

pub(super) fn clamped_chat_size(size: Vec2) -> Vec2 {
    let width = if size.x.is_finite() {
        size.x.clamp(CHAT_MIN_WIDTH, CHAT_MAX_WIDTH)
    } else {
        CHAT_DEFAULT_WIDTH
    };
    let height = if size.y.is_finite() {
        size.y.clamp(CHAT_MIN_HEIGHT, CHAT_MAX_HEIGHT)
    } else {
        CHAT_DEFAULT_HEIGHT
    };
    Vec2::new(width, height)
}

/// Source-neutral projection of fresh Retrobution's `CnGuiChat.DoChatWindow`.
///
/// The source uses one vertical GUILayout area: a fixed 14 px tab row, an
/// expanding chat box, and a fixed 23 px entry row. The input row intentionally
/// changes when chat is inactive because EMOTE and SEND are not laid out then.
pub(super) fn chat_layout(size: Vec2, active: bool) -> ChatLayout {
    let size = clamped_chat_size(size);
    let compact_tabs = size.x < 390.0;
    let background = GameplayUiRect::new(0.0, 14.0, size.x, size.y - 37.0);
    let row_y = size.y - 26.0;
    // GUILayoutUtility.GetRect(228, 23) expands horizontally inside the clean
    // BeginHorizontal row. The requested width is a minimum, not the final
    // rectangle width. CnGuiChat then subtracts 15 px and, while active, the
    // 55 px SEND reservation from that expanded rectangle.
    let (input_x, input_width) = if active {
        (75.0, size.x - 145.0)
    } else {
        (40.0, size.x - 55.0)
    };
    let log = GameplayUiRect::new(
        background.x + 24.0,
        background.y + 4.0,
        background.width - 24.0,
        background.height - 16.0,
    );
    ChatLayout {
        size,
        background,
        entry: GameplayUiRect::new(
            -8.0,
            background.y + background.height - 10.0,
            background.width + 22.0,
            40.0,
        ),
        log,
        // The source intentionally derives this Y from the inset log height,
        // without adding the inset log's own Y coordinate.
        empty_state: GameplayUiRect::new(
            background.x + background.width / 16.0,
            log.height / 2.0 - 15.0,
            background.width - background.width / 8.0,
            60.0,
        ),
        // GUILayout allocates ALL, BUDDY, GROUP in that order, then the
        // source draws by enum index ALL, GROUP, BUDDY.
        tabs: [
            GameplayUiRect::new(-40.0, 1.0, CHAT_TAB_WIDTH, CHAT_TAB_HEIGHT),
            GameplayUiRect::new(
                if compact_tabs { 182.0 } else { 230.0 },
                1.0,
                CHAT_TAB_WIDTH,
                CHAT_TAB_HEIGHT,
            ),
            GameplayUiRect::new(
                if compact_tabs { 71.0 } else { 95.0 },
                1.0,
                CHAT_TAB_WIDTH,
                CHAT_TAB_HEIGHT,
            ),
        ],
        menu: GameplayUiRect::new(6.0, row_y, 31.0, 19.0),
        emote: GameplayUiRect::new(41.0, row_y, 31.0, 19.0),
        input: GameplayUiRect::new(input_x, row_y, input_width, 23.0),
        // The source intentionally uses the input allocation width as an
        // absolute X coordinate, producing a five-pixel field/SEND overlap.
        send: GameplayUiRect::new(size.x - 75.0, row_y, 69.0, 23.0),
        resize: GameplayUiRect::new(
            background.x + background.width - 29.0,
            18.0,
            CHAT_RESIZE_RECT_WIDTH,
            CHAT_RESIZE_RECT_HEIGHT,
        ),
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChatChannel {
    #[default]
    All,
    Group,
    Buddy,
}

impl ChatChannel {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::All => 0,
            Self::Group => 1,
            Self::Buddy => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ChatLineKind {
    #[default]
    Normal,
    System,
    Buddy,
    Group,
    Npc,
    Receive,
    Damage,
    Attack,
    Tutorial,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChatLineUi {
    pub text: String,
    pub kind: ChatLineKind,
    pub localized: Option<LocalizedText>,
}

impl ChatLineUi {
    pub fn normal(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::Normal,
            localized: None,
        }
    }

    pub fn tutorial(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::Tutorial,
            localized: None,
        }
    }

    pub fn system(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::System,
            localized: None,
        }
    }

    pub fn buddy(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::Buddy,
            localized: None,
        }
    }

    pub fn group(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::Group,
            localized: None,
        }
    }

    pub fn npc(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::Npc,
            localized: None,
        }
    }

    pub fn receive(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::Receive,
            localized: None,
        }
    }

    pub fn damage(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::Damage,
            localized: None,
        }
    }

    pub fn attack(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: ChatLineKind::Attack,
            localized: None,
        }
    }

    pub fn with_localized(mut self, localized: LocalizedText) -> Self {
        self.localized = Some(localized);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChatUi {
    pub visible: bool,
    /// Explicit gameplay-state gate. Retrobution clears tutorial text input;
    /// callers must enable this only for a ready main-world session.
    pub input_enabled: bool,
    pub active: bool,
    pub selected: ChatChannel,
    /// Protocol availability still controls GROUP routing, but fresh
    /// Retrobution keeps the GROUP tab visible for solo players.
    pub group_available: bool,
    /// Number of non-blocked Buddy entries. Fresh Retrobution uses this to
    /// replace the Buddy history viewport with its instructional empty state.
    pub buddy_count: usize,
    /// Current fresh-Retrobution chat-box size in source UI pixels.
    pub window_size: Vec2,
    /// Clean `CnGuiChat.bAlert`: one unread marker per ALL/GROUP/FRIEND tab.
    pub alerts: [bool; 3],
    /// Clean `cnTextOption` indices into the 18 reachable chat swatches.
    pub text_colors: TextColorSettings,
    pub quick_menu: QuickChatMenuUi,
    pub input: String,
    pub edit: TextEdit,
    pub lines: Vec<ChatLineUi>,
}

impl Default for ChatUi {
    fn default() -> Self {
        Self {
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
            lines: Vec::new(),
        }
    }
}

impl ChatUi {
    /// Mirrors `CnGuiChat.SetViewMenu(false)` when the shared NanoCom owner
    /// closes through its X button. The clean client leaves an unsent draft in
    /// `ChatString`, but exits the active menu/input state immediately.
    pub fn close_nanocom_menu(&mut self) {
        self.active = false;
        self.quick_menu.close();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ChatKeyboardCommand<'a> {
    Enter,
    PreviousHistory,
    NextHistory,
    Backspace,
    Edit {
        key: KeyCode,
        control: bool,
        shift: bool,
    },
    Text(&'a str),
}

/// `CnGuiChat.ChatHistory` and `iCurHistory`. This is presentation-session
/// state rather than authoritative chat/network state.
#[derive(Debug, Default, Resource)]
pub(super) struct ChatInputHistory {
    pub(super) sent: VecDeque<String>,
    pub(super) cursor: usize,
}

#[derive(Default, Resource)]
pub(super) struct ChatResizeDrag {
    pub(super) start_cursor: Option<Vec2>,
    pub(super) start_size: Vec2,
}

impl ChatInputHistory {
    pub(super) fn reset_cursor(&mut self) {
        self.cursor = self.sent.len();
    }

    pub(super) fn record(&mut self, message: String) {
        self.sent.push_back(message);
        if self.sent.len() > CHAT_SENT_HISTORY_CAPACITY {
            self.sent.pop_front();
        }
        self.reset_cursor();
    }

    pub(super) fn previous(&mut self, input: &mut String) {
        self.cursor = self.cursor.min(self.sent.len()).saturating_sub(1);
        if let Some(message) = self.sent.get(self.cursor) {
            input.clone_from(message);
        }
    }

    pub(super) fn next(&mut self, input: &mut String) {
        self.cursor = self.cursor.saturating_add(1).min(self.sent.len());
        // Clean CnGuiChat leaves ChatString unchanged when iCurHistory reaches
        // ChatHistory.Count; it does not synthesize an empty draft slot.
        if let Some(message) = self.sent.get(self.cursor) {
            input.clone_from(message);
        }
    }
}

/// Pure keyboard transition for the source `CnGuiChat.Update` branches. It can
/// only mutate the chat presentation model/history and return existing UI
/// actions; movement and networking remain outside this module.
pub(super) fn reduce_chat_keyboard(
    chat: &mut ChatUi,
    history: &mut ChatInputHistory,
    command: ChatKeyboardCommand<'_>,
) -> Vec<GameplayUiAction> {
    if !chat.input_enabled {
        return Vec::new();
    }
    truncate_chat_input_utf16(&mut chat.input);

    match command {
        ChatKeyboardCommand::Enter if !chat.active => {
            chat.active = true;
            history.reset_cursor();
            chat.edit.end(&chat.input);
            vec![GameplayUiAction::OpenNanocomMenu]
        }
        ChatKeyboardCommand::Enter => {
            chat.active = false;
            chat.edit = TextEdit::default();
            let mut actions = Vec::with_capacity(2);
            if !chat.input.is_empty() {
                let message = std::mem::take(&mut chat.input);
                history.record(message.clone());
                actions.push(GameplayUiAction::SendChat(message));
            } else {
                chat.input.clear();
            }
            actions.push(GameplayUiAction::CloseNanocomMenu);
            actions
        }
        ChatKeyboardCommand::PreviousHistory if chat.active => {
            history.previous(&mut chat.input);
            chat.edit.end(&chat.input);
            Vec::new()
        }
        ChatKeyboardCommand::NextHistory if chat.active => {
            history.next(&mut chat.input);
            chat.edit.end(&chat.input);
            Vec::new()
        }
        ChatKeyboardCommand::Backspace if chat.active => {
            chat.edit
                .key(&mut chat.input, KeyCode::Backspace, false, false);
            Vec::new()
        }
        ChatKeyboardCommand::Text(text) if chat.active => {
            chat.edit
                .insert(&mut chat.input, text, CHAT_INPUT_UTF16_LIMIT, true);
            Vec::new()
        }
        ChatKeyboardCommand::Edit {
            key,
            control,
            shift,
        } if chat.active => {
            chat.edit.key(&mut chat.input, key, control, shift);
            Vec::new()
        }
        ChatKeyboardCommand::Edit { .. }
        | ChatKeyboardCommand::PreviousHistory
        | ChatKeyboardCommand::NextHistory
        | ChatKeyboardCommand::Backspace
        | ChatKeyboardCommand::Text(_) => Vec::new(),
    }
}

pub(super) fn truncate_chat_input_utf16(input: &mut String) {
    let mut units = 0;
    let mut truncate_at = input.len();
    for (byte_index, character) in input.char_indices() {
        let next = units + character.len_utf16();
        if next > CHAT_INPUT_UTF16_LIMIT {
            truncate_at = byte_index;
            break;
        }
        units = next;
    }
    input.truncate(truncate_at);
}

#[cfg(test)]
pub(super) fn append_chat_text_utf16(input: &mut String, produced: &str) {
    truncate_chat_input_utf16(input);
    let mut units = input.encode_utf16().count();
    for character in produced.chars() {
        if character.is_control() {
            continue;
        }
        let character_units = character.len_utf16();
        if units + character_units > CHAT_INPUT_UTF16_LIMIT {
            break;
        }
        input.push(character);
        units += character_units;
    }
}
