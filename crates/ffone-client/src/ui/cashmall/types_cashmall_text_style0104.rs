use super::*;

/// Exact clean `FusionFallInvenSkin` text role attached to every Cash Mall
/// `Text`. Localization changes copy only; Rect and source-style metrics stay
/// fixed and the approved JEFFE replacement remains the runtime font.
#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum CashmallTextStyle0104 {
    LabelUpperLeft,
    BlankBoxUpperLeft,
    BlankBoxMiddleRight,
    ButtonMiddleCenter,
    EquipBarMiddleCenter,
    EquipFontMiddleRight,
}

impl CashmallTextStyle0104 {
    #[must_use]
    pub const fn source_font_path_id(self) -> i64 {
        match self {
            Self::ButtonMiddleCenter => CASHMALL_BUTTON_FONT_PATH_ID,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => CASHMALL_SMALL_FONT_PATH_ID,
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => {
                CASHMALL_LABEL_FONT_PATH_ID
            }
        }
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        match self {
            Self::ButtonMiddleCenter => CASHMALL_BUTTON_FONT_SIZE,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => CASHMALL_SMALL_FONT_SIZE,
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => {
                CASHMALL_LABEL_FONT_SIZE
            }
        }
    }

    #[must_use]
    pub const fn line_height(self) -> f32 {
        match self {
            Self::ButtonMiddleCenter => CASHMALL_BUTTON_FONT_LINE_HEIGHT,
            Self::EquipBarMiddleCenter | Self::EquipFontMiddleRight => {
                CASHMALL_SMALL_FONT_LINE_HEIGHT
            }
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft | Self::BlankBoxMiddleRight => {
                CASHMALL_LABEL_FONT_LINE_HEIGHT
            }
        }
    }

    /// Unity `TextAnchor` numeric value serialized by the source style.
    #[must_use]
    pub const fn legacy_alignment(self) -> i32 {
        match self {
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft => 0,
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => 4,
            Self::BlankBoxMiddleRight | Self::EquipFontMiddleRight => 5,
        }
    }

    /// `[left, right, top, bottom]` from the serialized GUIStyle.
    #[must_use]
    pub const fn padding(self) -> [f32; 4] {
        match self {
            Self::LabelUpperLeft => [
                0.0,
                0.0,
                CASHMALL_LABEL_PADDING_TOP,
                CASHMALL_LABEL_PADDING_BOTTOM,
            ],
            Self::ButtonMiddleCenter => [
                CASHMALL_BUTTON_PADDING_LEFT,
                CASHMALL_BUTTON_PADDING_RIGHT,
                CASHMALL_BUTTON_PADDING_TOP,
                CASHMALL_BUTTON_PADDING_BOTTOM,
            ],
            Self::BlankBoxUpperLeft
            | Self::BlankBoxMiddleRight
            | Self::EquipBarMiddleCenter
            | Self::EquipFontMiddleRight => [0.0; 4],
        }
    }

    #[must_use]
    pub const fn content_offset(self) -> [f32; 2] {
        [0.0, CASHMALL_REPLACEMENT_FONT_Y_OFFSET]
    }

    pub(super) fn font(self, font: Handle<Font>) -> (TextFont, LineHeight) {
        (
            TextFont {
                font: (font).into(),
                font_size: (self.font_size()).into(),
                ..default()
            },
            LineHeight::Px(self.line_height()),
        )
    }

    pub(super) fn node(self, rect: CashmallUiRect0104) -> Node {
        let mut node = rect.node();
        self.apply_to_node(&mut node);
        node
    }

    pub(super) fn apply_to_node(self, node: &mut Node) {
        let [left, right, top, bottom] = self.padding();
        node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
        match self {
            Self::LabelUpperLeft | Self::BlankBoxUpperLeft => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::FlexStart;
            }
            Self::BlankBoxMiddleRight | Self::EquipFontMiddleRight => {
                node.justify_content = JustifyContent::FlexEnd;
                node.align_items = AlignItems::Center;
            }
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => {
                node.justify_content = JustifyContent::Center;
                node.align_items = AlignItems::Center;
            }
        }
        node.overflow = match self {
            Self::LabelUpperLeft | Self::ButtonMiddleCenter => Overflow::clip(),
            Self::BlankBoxUpperLeft
            | Self::BlankBoxMiddleRight
            | Self::EquipBarMiddleCenter
            | Self::EquipFontMiddleRight => Overflow::visible(),
        };
    }

    pub(super) fn layout(self) -> TextLayout {
        let (justify, linebreak) = match self {
            Self::LabelUpperLeft => (Justify::Left, LineBreak::WordBoundary),
            Self::BlankBoxUpperLeft => (Justify::Left, LineBreak::NoWrap),
            Self::BlankBoxMiddleRight | Self::EquipFontMiddleRight => {
                (Justify::Right, LineBreak::NoWrap)
            }
            Self::ButtonMiddleCenter | Self::EquipBarMiddleCenter => {
                (Justify::Center, LineBreak::NoWrap)
            }
        };
        TextLayout::new(justify, linebreak)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum CashmallTab0104 {
    #[default]
    New = 0,
    Scroll = 1,
    Potion = 2,
    Equipment = 3,
    Etc = 4,
}

impl CashmallTab0104 {
    pub const ALL: [Self; CASHMALL_TAB_COUNT] = [
        Self::New,
        Self::Scroll,
        Self::Potion,
        Self::Equipment,
        Self::Etc,
    ];

    #[must_use]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub const fn legacy_name(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Scroll => "Scroll",
            Self::Potion => "Potion",
            Self::Equipment => "Equipment",
            Self::Etc => "Etc",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CashmallTabLabelSource0104 {
    /// Inactive tabs call `TextManager.GetStr(eCMTab.ToString())`.
    LocalizedKey,
    /// The selected tab passes `eCMTab.ToString()` directly.
    RawEnumName,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CashmallTabVisual0104 {
    #[default]
    Normal,
    Hover,
    Selected,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CashmallTabView0104 {
    pub tab: CashmallTab0104,
    pub rect: CashmallUiRect0104,
    pub hit_rect: CashmallUiRect0104,
    pub visual: CashmallTabVisual0104,
    pub label: &'static str,
    pub label_source: CashmallTabLabelSource0104,
    /// Clean has no pressed texture. Holding an inactive tab keeps the hover
    /// box; holding the selected tab keeps the selected box.
    pub pressed_visual_is_distinct: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashmallIconRef0104(pub(super) String);

impl CashmallIconRef0104 {
    pub fn new(path: impl Into<String>) -> Result<Self, CashmallIconRefError0104> {
        let path = path.into();
        if !cashmall_safe_relative_asset_path_0104(&path)
            || !path.starts_with("icons/")
            || !path.ends_with(".png")
        {
            return Err(CashmallIconRefError0104(path));
        }
        Ok(Self(path))
    }

    #[must_use]
    pub fn runtime_path(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashmallIconRefError0104(pub String);

impl fmt::Display for CashmallIconRefError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unsafe Cash Mall icon path: {}", self.0)
    }
}

impl Error for CashmallIconRefError0104 {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CashmallPresentationIcon0104 {
    Resolved(CashmallIconRef0104),
    MissingChecker,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashmallRowProjection0104 {
    pub scan_slot_id: usize,
    pub source: CashmallPlayerInventoryItem0104,
    pub icon: CashmallPresentationIcon0104,
    pub frame_visual: CashmallSlotFrameVisual0104,
    /// The insufficient-Taros branch draws the frame at 0.4 alpha after the
    /// icon. It never dims the already-drawn icon.
    pub frame_alpha_percent: u8,
    pub icon_alpha_percent: u8,
    pub level_label: String,
    /// `DrawItemInfo` updates the price rect/color but never emits a label.
    pub price_text_visible: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CashmallLifecyclePhase0104 {
    #[default]
    Hidden,
    Opening,
    Visible,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct CashmallModalState0104 {
    pub help_active: bool,
    pub system_popup_active: bool,
    pub generic_popup_active: bool,
    pub inventory_popup_modal: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct CashmallCloseGate0104 {
    /// Result of clean `(2, 24)` for the configurable close key.
    pub mode_accepts_escape: bool,
    /// `true` when clean `(11, 13)[0] == 0`.
    pub exit_arbitration_clear: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CashmallOpenSource0104 {
    HiddenChatCommand,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CashmallHiddenChatBoundary0104 {
    pub request_game_mode_event: [i32; 2],
    pub requested_game_mode: i32,
    pub receive_init_event: [i32; 3],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CashmallCloseSource0104 {
    PcStuffCloseButton,
    ConfigurableKey4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CashmallTabActivation0104 {
    Changed,
    SelectedButtonReturnDiscarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CashmallGoToStuffBoundary0104 {
    pub close_inventory_event: [i32; 3],
    pub request_game_mode_event: [i32; 2],
    pub next_game_mode: i32,
    pub receive_init_event: [i32; 3],
    pub next_mode_init_argument: i32,
    pub first_use_condition: i32,
    pub final_refresh_event: [i32; 2],
}

impl Default for CashmallGoToStuffBoundary0104 {
    fn default() -> Self {
        Self {
            close_inventory_event: [2, 3, 5],
            request_game_mode_event: [2, 0],
            next_game_mode: 6,
            receive_init_event: [2, 3, 0],
            next_mode_init_argument: 2,
            first_use_condition: 3,
            final_refresh_event: [11, 18],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CashmallCloseBoundary0104 {
    pub ui_input_event: [i32; 2],
    pub ui_input_exit_value: i32,
    pub restore_cursor_locked: bool,
    pub notify_game_mode_exit: [i32; 2],
    pub stop_ui_mode_sound: bool,
    pub request_asset_gc: bool,
    pub loaded_textures_actually_cleared: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CashmallLocalEffect0104 {
    EnterMode {
        ui_input_event: [i32; 2],
        ui_input_enter_value: i32,
        force_inventory_tab: i32,
        force_cursor_unlocked: bool,
    },
    PlayAudio(CashmallAudioCue0104),
    VendorClickItem(CashmallVendorClickBoundary0104),
    /// The right-button equipment branch emits a local legacy event. This is
    /// deliberately not represented as a packet or purchase contract.
    RetainedRightClickEquipmentEvent {
        event: [i32; 3],
        item_id: i16,
        item_type: i16,
        option: i32,
    },
    GoToMyStuff(CashmallGoToStuffBoundary0104),
    Close(CashmallCloseBoundary0104),
    /// `Panel_PCStuffScript` sends `ClickHelp`, but none of the four components
    /// on clean `CashmallMode` implements that receiver.
    DeadHelpSendMessage,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct CashmallUiOutbox0104 {
    pub(super) effects: VecDeque<CashmallLocalEffect0104>,
}

impl CashmallUiOutbox0104 {
    pub fn push(&mut self, effect: CashmallLocalEffect0104) {
        self.effects.push_back(effect);
    }

    pub fn pop_front(&mut self) -> Option<CashmallLocalEffect0104> {
        self.effects.pop_front()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = CashmallLocalEffect0104> + '_ {
        self.effects.drain(..)
    }

    pub fn clear(&mut self) {
        self.effects.clear();
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.effects.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct CashmallUiState0104 {
    pub(super) phase: CashmallLifecyclePhase0104,
    pub(super) opening_elapsed_seconds: f32,
    pub(super) tab: CashmallTab0104,
    pub(super) cashmall_scroll_y: f32,
    pub(super) inventory_scroll_y: f32,
    pub(super) scroll_target: CashmallScrollTarget0104,
    pub(super) previous_cursor_locked: bool,
    pub(super) cursor_locked: bool,
    pub(super) ui_input_active: bool,
}

impl Default for CashmallUiState0104 {
    fn default() -> Self {
        Self {
            phase: CashmallLifecyclePhase0104::Hidden,
            opening_elapsed_seconds: 0.0,
            tab: CashmallTab0104::New,
            cashmall_scroll_y: 0.0,
            inventory_scroll_y: 0.0,
            scroll_target: CashmallScrollTarget0104::PcStuff,
            previous_cursor_locked: false,
            cursor_locked: false,
            ui_input_active: false,
        }
    }
}
