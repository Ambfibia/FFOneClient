use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct RuleUiLabels {
    pub back: String,
    pub previous: String,
    pub next: String,
}

impl Default for RuleUiLabels {
    fn default() -> Self {
        Self {
            back: RULE_UI_BACK_LABEL_KEY.to_owned(),
            previous: RULE_UI_PREVIOUS_LABEL_KEY.to_owned(),
            next: RULE_UI_NEXT_LABEL_KEY.to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum RulePageId {
    Vehicle = 1,
    Combining = 2,
}

impl RulePageId {
    pub const CLEAN_ORDER: [Self; 2] = [Self::Vehicle, Self::Combining];

    #[must_use]
    pub const fn table_index(self) -> usize {
        self as usize
    }

    #[must_use]
    pub const fn slot(self) -> usize {
        self as usize - 1
    }

    #[must_use]
    pub const fn spec(self) -> &'static RulePageSpec {
        &RULE_UI_PAGES[self.slot()]
    }
}

impl TryFrom<i32> for RulePageId {
    type Error = RulePageIndexError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Vehicle),
            2 => Ok(Self::Combining),
            _ => Err(RulePageIndexError(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RulePageSpec {
    pub id: RulePageId,
    pub string_start: usize,
    pub image_start: usize,
    pub previous: Option<RulePageId>,
    pub next: Option<RulePageId>,
    pub strings: [&'static str; 7],
    pub images: [&'static str; 4],
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RuleUiDismissalSource {
    CloseButton,
    BackButton,
    EscapeCloseGate,
}

#[derive(Debug, Default, Resource)]
pub struct RuleUiOutbox {
    pub(super) actions: VecDeque<RuleUiAction>,
}

impl RuleUiOutbox {
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<RuleUiAction> {
        self.actions.pop_front()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }

    pub(super) fn push(&mut self, action: RuleUiAction) {
        self.actions.push_back(action);
    }
}

#[derive(Clone, Resource)]
pub(super) struct RuleUiAssets {
    pub(super) panel_back: Handle<Image>,
    pub(super) rule_back: Handle<Image>,
    pub(super) page_images: [[Handle<Image>; 4]; 2],
    pub(super) back_normal: Handle<Image>,
    pub(super) back_hover: Handle<Image>,
    pub(super) nav_normal: Handle<Image>,
    pub(super) nav_hover: Handle<Image>,
    pub(super) close_normal: Handle<Image>,
    pub(super) close_hover: Handle<Image>,
    pub(super) jeffe: Handle<Font>,
    pub(super) chalet: Handle<Font>,
}

impl RuleUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            panel_back: asset_server.load(RULE_UI_PANEL_BACK_PATH),
            rule_back: asset_server.load(RULE_UI_RULE_BACK_PATH),
            page_images: [
                RULE_UI_VEHICLE_IMAGE_PATHS.map(|path| asset_server.load(path)),
                RULE_UI_COMBINE_IMAGE_PATHS.map(|path| asset_server.load(path)),
            ],
            back_normal: asset_server.load(RULE_UI_BACK_NORMAL_PATH),
            back_hover: asset_server.load(RULE_UI_BACK_HOVER_PATH),
            nav_normal: asset_server.load(RULE_UI_NAV_NORMAL_PATH),
            nav_hover: asset_server.load(RULE_UI_NAV_HOVER_PATH),
            close_normal: asset_server.load(RULE_UI_CLOSE_NORMAL_PATH),
            close_hover: asset_server.load(RULE_UI_CLOSE_HOVER_PATH),
            jeffe: asset_server.load(RULE_UI_JEFFE_FONT_PATH),
            chalet: asset_server.load(RULE_UI_CHALET_FONT_PATH),
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct RuleUiRoot;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct RuleUiBackground;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct RuleUiWindow;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct RuleUiIllustration {
    pub slot: usize,
}

#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum RuleUiTextRole {
    Title,
    Subtitle1,
    Subtitle2,
    Content1,
    Subtitle3,
    Content2,
    LastComment,
    BackButton,
    PreviousButton,
    NextButton,
}

impl RuleUiTextRole {
    #[must_use]
    pub(super) const fn page_string_slot(self) -> Option<usize> {
        match self {
            Self::Title => Some(0),
            Self::Subtitle1 => Some(1),
            Self::Subtitle2 => Some(2),
            Self::Content1 => Some(3),
            Self::Subtitle3 => Some(4),
            Self::Content2 => Some(5),
            Self::LastComment => Some(6),
            Self::BackButton | Self::PreviousButton | Self::NextButton => None,
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct RuleUiTextElement {
    pub role: RuleUiTextRole,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum RuleUiSet {
    Interaction,
    Bind,
    Visuals,
}

#[derive(Default)]
pub struct RuleUiPlugin;

impl Plugin for RuleUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<RuleUiModel>()
            .init_resource::<RuleUiLabels>()
            .init_resource::<RuleUiOutbox>()
            .init_resource::<RuleUiAudioOutbox>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_rule_ui)
            .configure_sets(
                Update,
                (RuleUiSet::Interaction, RuleUiSet::Bind, RuleUiSet::Visuals).chain(),
            )
            .add_systems(
                Update,
                ((handle_rule_ui_keyboard, handle_rule_ui_interactions)
                    .chain()
                    .in_set(RuleUiSet::Interaction))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    sync_rule_ui_root_and_layout,
                    sync_rule_ui_page,
                    sync_rule_ui_localized_labels,
                )
                    .chain()
                    .in_set(RuleUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (sync_rule_ui_button_visuals.in_set(RuleUiSet::Visuals))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
