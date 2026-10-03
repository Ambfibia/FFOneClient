use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuddyFontRole {
    Jeffe,
    Chalet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuddyTextAnchor {
    UpperLeft,
    MiddleLeft,
    MiddleCenter,
}

/// Text-bearing GUIStyles reached by the clean Buddy list and its ADD popup.
///
/// `FusionFallChatSkin` contains many unrelated chat styles and the shared
/// popup skin contains the rest of character selection. Keeping this enum to
/// the actual draw routes prevents unused legacy UI from entering the native
/// tree.
#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum BuddyTextStyle {
    BuddyWindow,
    BuddyItem,
    BlueButton,
    RedButton2,
    Transparent3,
    DeleteText,
    Cancel,
    QuitButton,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BuddyTextStyleSpec {
    pub source_skin_path_id: i64,
    pub source_font_path_id: i64,
    pub font_role: BuddyFontRole,
    pub font_size: f32,
    pub line_height: f32,
    /// Left, right, top, bottom in clean IMGUI pixels.
    pub padding: [f32; 4],
    pub anchor: BuddyTextAnchor,
    pub normal_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub active_color: [f32; 4],
    pub word_wrap: bool,
    pub y_offset: f32,
}

impl BuddyTextStyle {
    #[must_use]
    pub const fn spec(self) -> BuddyTextStyleSpec {
        use BuddyFontRole::{Chalet, Jeffe};
        use BuddyTextAnchor::{MiddleCenter, MiddleLeft, UpperLeft};

        let (source_skin_path_id, source_font_path_id, font_role, font_size, line_height) =
            match self {
                Self::BuddyWindow
                | Self::BlueButton
                | Self::RedButton2
                | Self::Transparent3
                | Self::QuitButton => (
                    if matches!(self, Self::Transparent3 | Self::QuitButton) {
                        BUDDY_POP_SKIN_PATH_ID
                    } else {
                        BUDDY_CHAT_SKIN_PATH_ID
                    },
                    BUDDY_JEFFE_14_SOURCE_FONT_PATH_ID,
                    Jeffe,
                    BUDDY_JEFFE_14_FONT_SIZE,
                    BUDDY_JEFFE_14_LINE_HEIGHT,
                ),
                Self::Cancel => (
                    BUDDY_POP_SKIN_PATH_ID,
                    BUDDY_JEFFE_12_SOURCE_FONT_PATH_ID,
                    Jeffe,
                    BUDDY_JEFFE_12_FONT_SIZE,
                    BUDDY_JEFFE_12_LINE_HEIGHT,
                ),
                Self::BuddyItem | Self::DeleteText => (
                    if matches!(self, Self::DeleteText) {
                        BUDDY_POP_SKIN_PATH_ID
                    } else {
                        BUDDY_CHAT_SKIN_PATH_ID
                    },
                    BUDDY_CHALET_SMALL_SOURCE_FONT_PATH_ID,
                    Chalet,
                    BUDDY_CHALET_SMALL_FONT_SIZE,
                    BUDDY_CHALET_SMALL_LINE_HEIGHT,
                ),
            };
        let padding = match self {
            Self::BuddyWindow => [10.0, 2.0, 1.0, 2.0],
            Self::BuddyItem => [2.0, 2.0, 0.0, 2.0],
            Self::BlueButton => [2.0; 4],
            Self::RedButton2 => [0.0; 4],
            Self::Transparent3 | Self::DeleteText | Self::QuitButton => [10.0, 6.0, 4.0, 6.0],
            Self::Cancel => [0.0, 0.0, 4.0, 7.0],
        };
        let anchor = match self {
            Self::BuddyWindow => UpperLeft,
            Self::BuddyItem => MiddleLeft,
            Self::BlueButton
            | Self::RedButton2
            | Self::Transparent3
            | Self::DeleteText
            | Self::Cancel
            | Self::QuitButton => MiddleCenter,
        };
        let normal_color = match self {
            Self::Transparent3 | Self::DeleteText | Self::Cancel => [0.8, 1.0, 1.0, 1.0],
            _ => [1.0; 4],
        };
        let hover_color = match self {
            Self::BlueButton => [0.0, 0.342_741_94, 0.528_225_8, 1.0],
            _ => normal_color,
        };
        let active_color = match self {
            Self::BlueButton => [0.0; 4],
            Self::RedButton2 => [1.0, 1.0, 1.0, 0.0],
            Self::QuitButton => [0.898_039_2, 0.898_039_2, 0.898_039_2, 1.0],
            _ => normal_color,
        };
        let y_offset = match self {
            Self::BuddyWindow => BUDDY_WINDOW_TEXT_Y_OFFSET,
            Self::BuddyItem => BUDDY_ITEM_TEXT_Y_OFFSET,
            Self::BlueButton => BUDDY_BLUE_BUTTON_TEXT_Y_OFFSET,
            Self::RedButton2 => BUDDY_RED_BUTTON_TEXT_Y_OFFSET,
            Self::Transparent3 => BUDDY_TRANSPARENT3_TEXT_Y_OFFSET,
            Self::DeleteText => BUDDY_DELETE_TEXT_Y_OFFSET,
            Self::Cancel => BUDDY_CANCEL_TEXT_Y_OFFSET,
            Self::QuitButton => BUDDY_QUIT_BUTTON_TEXT_Y_OFFSET,
        };
        BuddyTextStyleSpec {
            source_skin_path_id,
            source_font_path_id,
            font_role,
            font_size,
            line_height,
            padding,
            anchor,
            normal_color,
            hover_color,
            active_color,
            word_wrap: !matches!(
                self,
                Self::BlueButton
                    | Self::Transparent3
                    | Self::DeleteText
                    | Self::Cancel
                    | Self::QuitButton
            ),
            y_offset,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum BuddyChatWindowStyle {
    #[default]
    Small,
    Large,
}

impl BuddyChatWindowStyle {
    pub(super) const fn legacy_group_x(self) -> f32 {
        match self {
            Self::Small => 300.0,
            Self::Large => 440.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum BuddyPresence {
    #[default]
    Offline,
    Online,
}

impl BuddyPresence {
    #[must_use]
    pub const fn from_legacy(raw: i8) -> Self {
        if raw == 0 {
            Self::Offline
        } else {
            Self::Online
        }
    }

    #[must_use]
    pub const fn is_online(self) -> bool {
        matches!(self, Self::Online)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BuddyEntry {
    pub runtime_pc_id: i32,
    pub pc_uid: i64,
    pub blocked: bool,
    pub free_chat: bool,
    pub presence: BuddyPresence,
    pub first_name: String,
    pub last_name: String,
    pub gender: i8,
    pub name_check_flag: i8,
}

impl BuddyEntry {
    /// Clean buddy, option, and chat paths accept a name only when the flag is
    /// exactly one. Every other value renders `Player <PCUID>`.
    #[must_use]
    pub fn display_name(&self) -> String {
        if self.name_check_flag == 1 {
            format!("{} {}", self.first_name, self.last_name)
        } else {
            format!("Player {}", self.pc_uid)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BuddyTarget {
    pub slot: usize,
    pub pc_uid: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BuddyUiNotice {
    PresenceChanged {
        target: BuddyTarget,
        display_name: String,
        presence: BuddyPresence,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuddyInvite {
    /// Native correlation token supplied by the integration layer.
    pub invite_id: u64,
    pub requester_pc_id: i32,
    pub requester_pc_uid: i64,
    pub first_name: String,
    pub last_name: String,
    pub name_check_flag: i8,
}

impl BuddyInvite {
    #[must_use]
    pub fn display_name(&self) -> String {
        BuddyEntry {
            pc_uid: self.requester_pc_uid,
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            name_check_flag: self.name_check_flag,
            ..default()
        }
        .display_name()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuddyConfirmation {
    Remove(BuddyTarget),
    Warp(BuddyTarget),
    LeaveGroupForWarp(BuddyTarget),
}

#[derive(Default, Resource)]
pub struct BuddyUiOutbox {
    pub(super) actions: VecDeque<BuddyUiAction>,
}

impl BuddyUiOutbox {
    pub fn push(&mut self, action: BuddyUiAction) {
        self.actions.push_back(action);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = BuddyUiAction> + '_ {
        self.actions.drain(..)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BuddyRowView {
    pub slot: usize,
    pub target: BuddyTarget,
    pub display_name: String,
    pub name_is_verified: bool,
    pub free_chat: bool,
    pub presence: BuddyPresence,
    pub selected: bool,
    pub top: f32,
    pub fully_visible: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BuddyUiView {
    pub visible: bool,
    pub rows: Vec<BuddyRowView>,
    pub delete_enabled: bool,
    pub warp_enabled: bool,
    pub add_dialog_open: bool,
}

#[derive(Clone, Resource)]
pub(super) struct BuddyUiAssets {
    pub(super) window: Handle<Image>,
    pub(super) selection: Handle<Image>,
    pub(super) freechat: Handle<Image>,
    pub(super) list_background: Handle<Image>,
    pub(super) large_list_background: Handle<Image>,
    pub(super) blue: Handle<Image>,
    pub(super) blue_over: Handle<Image>,
    pub(super) red: Handle<Image>,
    pub(super) red_over: Handle<Image>,
    pub(super) add_dialog: Handle<Image>,
    pub(super) add_overlay: Handle<Image>,
    pub(super) cancel: Handle<Image>,
    pub(super) scroll_track: Handle<Image>,
    pub(super) scroll_thumb: Handle<Image>,
    pub(super) scroll_up: Handle<Image>,
    pub(super) scroll_down: Handle<Image>,
    pub(super) jeffe_font: Handle<Font>,
    pub(super) chalet_font: Handle<Font>,
}

impl BuddyUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            window: asset_server.load(BUDDY_BOX_PATH),
            selection: asset_server.load(BUDDY_SELECT_PATH),
            freechat: asset_server.load(BUDDY_FREECHAT_PATH),
            list_background: asset_server.load(BUDDY_LIST_BACKGROUND_PATH),
            large_list_background: asset_server.load(BUDDY_LARGE_LIST_BACKGROUND_PATH),
            blue: asset_server.load(BUDDY_BLUE_BUTTON_PATH),
            blue_over: asset_server.load(BUDDY_BLUE_BUTTON_OVER_PATH),
            red: asset_server.load(BUDDY_RED_BUTTON_PATH),
            red_over: asset_server.load(BUDDY_RED_BUTTON_OVER_PATH),
            add_dialog: asset_server.load(BUDDY_ADD_DIALOG_PATH),
            add_overlay: asset_server.load(BUDDY_ADD_OVERLAY_PATH),
            cancel: asset_server.load(BUDDY_CANCEL_BUTTON_PATH),
            scroll_track: asset_server.load(BUDDY_SCROLL_TRACK_PATH),
            scroll_thumb: asset_server.load(BUDDY_SCROLL_THUMB_PATH),
            scroll_up: asset_server.load(BUDDY_SCROLL_UP_PATH),
            scroll_down: asset_server.load(BUDDY_SCROLL_DOWN_PATH),
            jeffe_font: asset_server.load(BUDDY_FONT_PATH),
            chalet_font: asset_server.load(BUDDY_CHALET_FONT_PATH),
        }
    }

    pub(super) fn control_image(&self, control: BuddyControl, interaction: Interaction) -> Handle<Image> {
        match (control, interaction) {
            (
                BuddyControl::Delete | BuddyControl::Warp | BuddyControl::Add,
                Interaction::Pressed,
            ) => Handle::default(),
            (BuddyControl::Delete | BuddyControl::ModalAdd, Interaction::Hovered) => {
                self.red_over.clone()
            }
            (BuddyControl::Delete | BuddyControl::ModalAdd, _) => self.red.clone(),
            (BuddyControl::ModalCancel, Interaction::None) => self.cancel.clone(),
            // Clean `cancel` uses BlueButton normal, not BlueButton hover,
            // for its hover/active background.
            (BuddyControl::ModalCancel, Interaction::Hovered) => self.blue.clone(),
            (_, Interaction::Hovered) => self.blue_over.clone(),
            _ => self.blue.clone(),
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct BuddyUiRoot;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct BuddyPanel;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct BuddyListBackground;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct BuddyModalRoot;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct BuddyAddDialog;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct BuddyRow {
    pub(super) slot: usize,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) enum BuddyRowPartKind {
    Selection,
    SelectionSpike,
    FreeChat,
    Label,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct BuddyRowPart {
    pub(super) slot: usize,
    pub(super) kind: BuddyRowPartKind,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) enum BuddyControl {
    Delete,
    Warp,
    Add,
    ModalCancel,
    ModalAdd,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct BuddyControlMarker(pub(super) BuddyControl);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum BuddyUiSet {
    Time,
    Interaction,
    Bind,
    Visuals,
}

#[derive(Default)]
pub struct BuddyUiPlugin;

impl Plugin for BuddyUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<BuddyUiModel>()
            .init_resource::<BuddyUiOutbox>()
            .init_resource::<GameplayUiAudioOutbox>()
            .init_resource::<BuddyKeyboardCapture>()
            .init_resource::<BuddyScrollbarDrag>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_buddy_ui)
            .configure_sets(
                Update,
                (
                    BuddyUiSet::Time,
                    BuddyUiSet::Interaction,
                    BuddyUiSet::Bind,
                    BuddyUiSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                ((
                    begin_buddy_keyboard_capture,
                    handle_buddy_keyboard,
                    clear_captured_buddy_keyboard_messages,
                )
                    .chain()
                    .after(InputSystems))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (tick_buddy_ui.in_set(BuddyUiSet::Time))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((handle_buddy_scroll_input, handle_buddy_interactions)
                    .chain()
                    .in_set(BuddyUiSet::Interaction))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((bind_buddy_layout, bind_buddy_ui)
                    .chain()
                    .in_set(BuddyUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (update_buddy_control_visuals.in_set(BuddyUiSet::Visuals))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
