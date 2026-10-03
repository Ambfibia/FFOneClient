use super::*;

/// Every text-bearing GUIStyle actually reached by clean
/// `CnGuiNanoFreeTuning.OnGUI`. Values are the serialized
/// `FusionFallNanoSkin` (path ID 1379) contract; only glyph coverage is
/// replaced by `fonts/jeffe.otf`.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum NanoFreeTuningUiTextStyle {
    BigBlueMiddleLeft,
    BigYellowMiddleLeft,
    TransparentLightBlueUpperLeft,
    TransparentYellowSmallUpperLeft,
    TransparentBlueUpperLeft,
    ButtonMiddleCenter,
}

impl NanoFreeTuningUiTextStyle {
    #[must_use]
    pub const fn source_style_name(self) -> &'static str {
        match self {
            Self::BigBlueMiddleLeft => "bigblue",
            Self::BigYellowMiddleLeft => "bigyellow",
            Self::TransparentLightBlueUpperLeft => "TransparentLightBlue",
            Self::TransparentYellowSmallUpperLeft => "TransparentYellowSmall",
            Self::TransparentBlueUpperLeft => "TransparentBlue",
            Self::ButtonMiddleCenter => "button",
        }
    }

    #[must_use]
    pub const fn source_skin_path_id(self) -> i64 {
        NANO_FREE_TUNING_SKIN_PATH_ID
    }

    #[must_use]
    pub const fn source_font_path_id(self) -> i64 {
        match self {
            Self::BigBlueMiddleLeft | Self::BigYellowMiddleLeft => {
                NANO_FREE_TUNING_JEFFE_16_FONT_PATH_ID
            }
            Self::TransparentLightBlueUpperLeft | Self::ButtonMiddleCenter => {
                NANO_FREE_TUNING_JEFFE_14_FONT_PATH_ID
            }
            Self::TransparentYellowSmallUpperLeft | Self::TransparentBlueUpperLeft => {
                NANO_FREE_TUNING_JEFFE_08_FONT_PATH_ID
            }
        }
    }

    #[must_use]
    pub const fn replacement_font_path(self) -> &'static str {
        NANO_FREE_TUNING_FONT_PATH
    }

    /// Unity `TextAnchor`: 0 UpperLeft, 3 MiddleLeft, 4 MiddleCenter.
    #[must_use]
    pub const fn legacy_alignment(self) -> i32 {
        match self {
            Self::BigBlueMiddleLeft | Self::BigYellowMiddleLeft => 3,
            Self::TransparentLightBlueUpperLeft
            | Self::TransparentYellowSmallUpperLeft
            | Self::TransparentBlueUpperLeft => 0,
            Self::ButtonMiddleCenter => 4,
        }
    }

    /// Serialized `[left, right, top, bottom]` `RectOffset`.
    #[must_use]
    pub const fn padding(self) -> [f32; 4] {
        match self {
            Self::ButtonMiddleCenter => [6.0, 6.0, 3.0, 3.0],
            _ => [0.0; 4],
        }
    }

    #[must_use]
    pub const fn content_offset(self) -> [f32; 2] {
        [0.0, 0.0]
    }

    #[must_use]
    pub const fn word_wrap(self) -> bool {
        matches!(
            self,
            Self::BigBlueMiddleLeft | Self::BigYellowMiddleLeft | Self::TransparentBlueUpperLeft
        )
    }

    /// Serialized `TextClipping`: 0 Overflow, 1 Clip.
    #[must_use]
    pub const fn legacy_text_clipping(self) -> i32 {
        if matches!(self, Self::ButtonMiddleCenter) {
            1
        } else {
            0
        }
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        match self.source_font_path_id() {
            NANO_FREE_TUNING_JEFFE_16_FONT_PATH_ID => NANO_FREE_TUNING_JEFFE_16_FONT_SIZE,
            NANO_FREE_TUNING_JEFFE_14_FONT_PATH_ID => NANO_FREE_TUNING_JEFFE_14_FONT_SIZE,
            _ => NANO_FREE_TUNING_JEFFE_08_FONT_SIZE,
        }
    }

    #[must_use]
    pub const fn line_height(self) -> f32 {
        match self.source_font_path_id() {
            NANO_FREE_TUNING_JEFFE_16_FONT_PATH_ID => NANO_FREE_TUNING_JEFFE_16_LINE_HEIGHT,
            NANO_FREE_TUNING_JEFFE_14_FONT_PATH_ID => NANO_FREE_TUNING_JEFFE_14_LINE_HEIGHT,
            _ => NANO_FREE_TUNING_JEFFE_08_LINE_HEIGHT,
        }
    }

    #[must_use]
    pub const fn replacement_y_offset(self) -> f32 {
        match self {
            Self::BigBlueMiddleLeft => NANO_FREE_TUNING_BIGBLUE_REPLACEMENT_Y_OFFSET,
            Self::BigYellowMiddleLeft => NANO_FREE_TUNING_BIGYELLOW_REPLACEMENT_Y_OFFSET,
            Self::TransparentLightBlueUpperLeft => NANO_FREE_TUNING_LIGHT_BLUE_REPLACEMENT_Y_OFFSET,
            Self::TransparentYellowSmallUpperLeft => {
                NANO_FREE_TUNING_YELLOW_SMALL_REPLACEMENT_Y_OFFSET
            }
            Self::TransparentBlueUpperLeft => NANO_FREE_TUNING_BLUE_REPLACEMENT_Y_OFFSET,
            Self::ButtonMiddleCenter => NANO_FREE_TUNING_BUTTON_REPLACEMENT_Y_OFFSET,
        }
    }

    #[must_use]
    pub const fn text_color(self) -> [f32; 4] {
        match self {
            Self::BigYellowMiddleLeft | Self::TransparentYellowSmallUpperLeft => {
                [1.0, 1.0, 0.0, 1.0]
            }
            Self::TransparentBlueUpperLeft => [0.0, 1.0, 1.0, 1.0],
            _ => [0.8, 1.0, 1.0, 1.0],
        }
    }

    pub(super) fn font(self, font: &Handle<Font>) -> (TextFont, LineHeight) {
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (self.font_size()).into(),
                ..default()
            },
            LineHeight::Px(self.line_height()),
        )
    }

    pub(super) fn layout(self) -> TextLayout {
        let justify = if matches!(self, Self::ButtonMiddleCenter) {
            Justify::Center
        } else {
            Justify::Left
        };
        TextLayout::new(
            justify,
            if self.word_wrap() {
                LineBreak::WordBoundary
            } else {
                LineBreak::NoWrap
            },
        )
    }

    pub(super) fn color(self) -> TextColor {
        let [red, green, blue, alpha] = self.text_color();
        TextColor(Color::srgba(red, green, blue, alpha))
    }

    pub(super) fn apply_to_container(self, node: &mut Node) {
        node.justify_content = match self.legacy_alignment() {
            4 => JustifyContent::Center,
            _ => JustifyContent::FlexStart,
        };
        node.align_items = match self.legacy_alignment() {
            3 | 4 => AlignItems::Center,
            _ => AlignItems::FlexStart,
        };
        let [left, right, top, bottom] = self.padding();
        node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
        node.overflow = if self.legacy_text_clipping() == 1 {
            Overflow::clip()
        } else {
            Overflow::visible()
        };
    }
}

/// Reached default `GUI.skin.label` used only by `GUI.Label(Rect, Texture)`.
/// Its outer Rect remains unchanged; this padding defines the image content
/// box inside that Rect.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub struct NanoFreeTuningIconLabelStyle;

impl NanoFreeTuningIconLabelStyle {
    pub const SOURCE_STYLE_NAME: &'static str = "label";
    pub const SOURCE_SKIN_PATH_ID: i64 = NANO_FREE_TUNING_SKIN_PATH_ID;
    pub const SOURCE_FONT_PATH_ID: i64 = NANO_FREE_TUNING_JEFFE_14_FONT_PATH_ID;
    pub const LEGACY_ALIGNMENT: i32 = 0;
    pub const PADDING: [f32; 4] = [0.0, 0.0, 3.0, 3.0];
    pub const CONTENT_OFFSET: [f32; 2] = [0.0, 0.0];
    pub const WORD_WRAP: bool = true;
    pub const LEGACY_TEXT_CLIPPING: i32 = 1;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningAbiScalar {
    I16,
    I32,
    ItemBase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningAbiField {
    pub clean_name: &'static str,
    pub offset: usize,
    pub scalar: NanoFreeTuningAbiScalar,
    pub count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningPower {
    pub tune_id: i16,
    pub skill_id: i16,
    pub icon_path: String,
    pub name: String,
    pub power_type: String,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningContent {
    pub nano_id: i16,
    /// XDT style metadata. Acquisition does not spawn the world summon column.
    pub nano_style: u8,
    pub nano_name: String,
    pub powers: [NanoFreeTuningPower; 3],
}

impl NanoFreeTuningContent {
    pub fn validate(&self) -> Result<(), NanoFreeTuningContentError> {
        if self.nano_id <= 0 {
            return Err(NanoFreeTuningContentError::InvalidNanoId(self.nano_id));
        }
        if self.nano_style > 2 {
            return Err(NanoFreeTuningContentError::InvalidNanoStyle(
                self.nano_style,
            ));
        }
        if self.nano_name.trim().is_empty() {
            return Err(NanoFreeTuningContentError::EmptyNanoName);
        }
        for (index, power) in self.powers.iter().enumerate() {
            if power.tune_id <= 0 {
                return Err(NanoFreeTuningContentError::InvalidTuneId {
                    index,
                    tune_id: power.tune_id,
                });
            }
            if power.skill_id <= 0 {
                return Err(NanoFreeTuningContentError::InvalidSkillId {
                    index,
                    skill_id: power.skill_id,
                });
            }
            if power.icon_path.trim().is_empty() {
                return Err(NanoFreeTuningContentError::EmptyIconPath(index));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NanoFreeTuningTransform {
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NanoFreeTuningWorldSnapshot {
    pub player: NanoFreeTuningTransform,
    /// The first dead member encountered in `NpcContainer.member`.
    pub first_defeated_fusion: Option<NanoFreeTuningTransform>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NanoFreeTuningOpenContext {
    pub player_id: i32,
    pub killed_fusion: bool,
    pub content: NanoFreeTuningContent,
    pub world: NanoFreeTuningWorldSnapshot,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NanoTuneItemBase {
    pub item_type: i16,
    pub item_id: i16,
    pub option: i32,
    pub time_limit: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NanoTuneSuccess {
    pub nano_id: i16,
    pub skill_id: i16,
    pub fusion_matter: i32,
    pub item_slots: [i32; NANO_TUNE_ITEM_SLOT_COUNT],
    pub items: [NanoTuneItemBase; NANO_TUNE_ITEM_SLOT_COUNT],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoTuneFailure {
    pub player_id: i32,
    pub error_code: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningProtocolFault {
    Timeout,
    NoPendingRequest,
    StaleRequestToken {
        expected: u64,
        actual: u64,
    },
    UnexpectedPacketId(u32),
    PayloadSize {
        packet_id: u32,
        expected: usize,
        actual: usize,
    },
    BodyKindMismatch {
        packet_id: u32,
    },
    NanoMismatch {
        expected: i16,
        actual: i16,
    },
    SkillMismatch {
        expected: i16,
        actual: i16,
    },
    PlayerMismatch {
        expected: i32,
        actual: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningFault {
    ServerRejected { player_id: i32, error_code: i32 },
    Protocol(NanoFreeTuningProtocolFault),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub enum NanoFreeTuningPhase {
    Closed = -1,
    EffectDelay = 0,
    ProjectileDelay = 1,
    RevealDelay = 2,
    CameraApproach = 3,
    CameraSettle = 4,
    PowerSelection = 5,
    ResultSkill = 6,
    ResultHide = 7,
}

impl NanoFreeTuningPhase {
    #[must_use]
    pub const fn legacy_index(self) -> Option<i32> {
        match self {
            Self::Closed => None,
            _ => Some(self as i8 as i32),
        }
    }
}

#[derive(Clone, Resource)]
pub(super) struct NanoFreeTuningPresentationAssets {
    pub(super) panel: Handle<Image>,
    pub(super) black: Handle<Image>,
    pub(super) select_normal: Handle<Image>,
    pub(super) select_hover: Handle<Image>,
    pub(super) font: Handle<Font>,
}

impl NanoFreeTuningPresentationAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            panel: asset_server.load(NANO_FREE_TUNING_PANEL_PATH),
            black: asset_server.load(NANO_FREE_TUNING_BLACK_PATH),
            select_normal: asset_server.load(NANO_FREE_TUNING_SELECT_NORMAL_PATH),
            select_hover: asset_server.load(NANO_FREE_TUNING_SELECT_HOVER_PATH),
            font: asset_server.load(NANO_FREE_TUNING_FONT_PATH),
        }
    }

    pub(super) fn source_images(&self) -> [(&'static str, &Handle<Image>); 4] {
        [
            (NANO_FREE_TUNING_PANEL_PATH, &self.panel),
            (NANO_FREE_TUNING_BLACK_PATH, &self.black),
            (NANO_FREE_TUNING_SELECT_NORMAL_PATH, &self.select_normal),
            (NANO_FREE_TUNING_SELECT_HOVER_PATH, &self.select_hover),
        ]
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningPresentationRoot;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningPresentationPanel;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoFreeTuningPresentationBar {
    pub bottom: bool,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NanoFreeTuningAnnouncement;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NanoFreeTuningPowerIcon(pub(super) usize);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct NanoFreeTuningText(pub(super) NanoFreeTuningTextRole);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum NanoFreeTuningTextRole {
    AcquiredPrefix,
    NanoName,
    Bang,
    PowerName(usize),
    PowerType(usize),
    PowerDescription(usize),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum NanoFreeTuningPresentationSet {
    Preload,
    Input,
    Bind,
    Visuals,
}

/// Passive presentation: buttons queue semantic commands; they never dispatch
/// packets or commit an authoritative Nano update directly.
pub struct NanoFreeTuningUiPlugin;

impl Plugin for NanoFreeTuningUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<NanoFreeTuningModel>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<NanoFreeTuningUiCommandOutbox>()
            .init_resource::<NanoFreeTuningPresentationAssetStatus>()
            .configure_sets(
                Update,
                (
                    NanoFreeTuningPresentationSet::Preload,
                    NanoFreeTuningPresentationSet::Input,
                    NanoFreeTuningPresentationSet::Bind,
                    NanoFreeTuningPresentationSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_nano_free_tuning_presentation,
            )
            .add_systems(
                Update,
                (update_nano_free_tuning_asset_status
                    .in_set(NanoFreeTuningPresentationSet::Preload))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (queue_nano_free_tuning_controls.in_set(NanoFreeTuningPresentationSet::Input))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((sync_nano_free_tuning_layout, sync_nano_free_tuning_content)
                    .chain()
                    .in_set(NanoFreeTuningPresentationSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (sync_nano_free_tuning_button_visuals
                    .in_set(NanoFreeTuningPresentationSet::Visuals))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
