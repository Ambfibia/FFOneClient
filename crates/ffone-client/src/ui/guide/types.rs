use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum GuideMentor {
    BenTennyson = 4,
    Dexter = 2,
    MojoJojo = 3,
    Edd = 1,
}

impl GuideMentor {
    pub const CLEAN_ORDER: [Self; 4] = [Self::BenTennyson, Self::Dexter, Self::MojoJojo, Self::Edd];

    #[must_use]
    pub const fn from_wire_id(value: i16) -> Option<Self> {
        match value {
            4 => Some(Self::BenTennyson),
            2 => Some(Self::Dexter),
            3 => Some(Self::MojoJojo),
            1 => Some(Self::Edd),
            _ => None,
        }
    }

    #[must_use]
    pub const fn wire_id(self) -> i16 {
        self as i16
    }

    #[must_use]
    pub const fn slot(self) -> usize {
        match self {
            Self::BenTennyson => 0,
            Self::Dexter => 1,
            Self::MojoJojo => 2,
            Self::Edd => 3,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::BenTennyson => "BEN TENNYSON",
            Self::Dexter => "DEXTER",
            Self::MojoJojo => "MOJO JOJO",
            Self::Edd => "EDD",
        }
    }

    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::BenTennyson => {
                "Ben must prevent Fuse from getting his claws on hidden Plumber technology, and \
only you can help him!"
            }
            Self::Dexter => {
                "The world's heroes are disappearing! Help Dexter track them down, and find out \
if Fuse is responsible."
            }
            Self::MojoJojo => {
                "Help Mojo Jojo in his quest for a new-and-improved monkey minion to fight Fuse."
            }
            Self::Edd => {
                "Edd is on the hunt for hidden candy treasure! Help him dig up some sweet riches, \
and dirty secrets!"
            }
        }
    }

    #[must_use]
    pub const fn guide_position(self) -> Vec2 {
        match self {
            Self::BenTennyson => Vec2::new(135.0, 530.0),
            Self::Dexter => Vec2::new(390.0, 523.0),
            Self::MojoJojo => Vec2::new(652.0, 523.0),
            Self::Edd => Vec2::new(897.0, 540.0),
        }
    }

    #[must_use]
    pub const fn portrait_size(self) -> Vec2 {
        match self {
            Self::BenTennyson => Vec2::new(160.0, 287.0),
            Self::Dexter => Vec2::new(224.0, 280.0),
            Self::MojoJojo => Vec2::new(281.0, 272.0),
            Self::Edd => Vec2::new(180.0, 295.0),
        }
    }

    #[must_use]
    pub const fn icon_size(self) -> Vec2 {
        match self {
            Self::BenTennyson => Vec2::new(34.0, 34.0),
            Self::Dexter => Vec2::new(32.0, 30.0),
            Self::MojoJojo => Vec2::new(30.0, 35.0),
            Self::Edd => Vec2::new(28.0, 29.0),
        }
    }

    #[must_use]
    pub const fn confirm_size(self) -> Vec2 {
        match self {
            Self::BenTennyson => Vec2::new(116.0, 182.0),
            Self::Dexter => Vec2::new(160.0, 195.0),
            Self::MojoJojo => Vec2::new(285.0, 196.0),
            Self::Edd => Vec2::new(109.0, 162.0),
        }
    }

    #[must_use]
    pub const fn portrait_path(self) -> &'static str {
        match self {
            Self::BenTennyson => "ui/en/gameplay/guide/ben.png",
            Self::Dexter => "ui/en/gameplay/guide/dexter.png",
            Self::MojoJojo => "ui/en/gameplay/guide/mojo.png",
            Self::Edd => "ui/en/gameplay/guide/edd.png",
        }
    }

    #[must_use]
    pub const fn icon_path(self) -> &'static str {
        match self {
            Self::BenTennyson => "ui/en/gameplay/guide/ben_icon.png",
            Self::Dexter => "ui/en/gameplay/guide/dexter_icon.png",
            Self::MojoJojo => "ui/en/gameplay/guide/mojo_icon.png",
            Self::Edd => "ui/en/gameplay/guide/edd_icon.png",
        }
    }

    #[must_use]
    pub const fn confirm_path(self) -> &'static str {
        match self {
            Self::BenTennyson => "ui/en/gameplay/guide/ben_confirm.png",
            Self::Dexter => "ui/en/gameplay/guide/dexter_confirm.png",
            Self::MojoJojo => "ui/en/gameplay/guide/mojo_confirm.png",
            Self::Edd => "ui/en/gameplay/guide/edd_confirm.png",
        }
    }

    #[must_use]
    pub fn card_rect(self) -> GuideUiRect {
        GUIDE_CARD_RECT.translated(self.guide_position().x - 100.0, 0.0)
    }

    #[must_use]
    pub fn current_frame_rect(self) -> GuideUiRect {
        let card = self.card_rect();
        GuideUiRect::new(card.x, card.y - 15.0, card.width, card.height + 15.0)
    }

    #[must_use]
    pub fn portrait_rect(self) -> GuideUiRect {
        let size = self.portrait_size();
        let position = self.guide_position();
        GuideUiRect::new(
            position.x - (size.x as i32 / 2) as f32,
            position.y - size.y,
            size.x,
            size.y,
        )
    }

    #[must_use]
    pub fn selected_effect_rect(self) -> GuideUiRect {
        let position = self.guide_position();
        GuideUiRect::new(position.x - 133.0, position.y - 414.0, 267.0, 414.0)
    }

    #[must_use]
    pub fn confirm_art_rect(self) -> GuideUiRect {
        let size = self.confirm_size();
        GuideUiRect::new(
            (GUIDE_WINDOW_RECT.width - size.x) * 0.5,
            GUIDE_MODAL_RECT.y - size.y + 2.0,
            size.x,
            size.y,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideUiPurpose {
    InitialSelection,
    ChangeMentor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideUiPhase {
    WarpWarning,
    MentorSelection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideUiBlocker {
    Hidden,
    ControlsDisabled,
    AwaitingServer,
    ConfirmationOpen,
    NoMentorSelected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideUiDismissalSource {
    WarpCancelButton,
    CancelButton,
    CloseButton,
    EscapeKey,
}

#[derive(Debug, Default, Resource)]
pub struct GuideUiOutbox {
    pub(super) actions: VecDeque<GuideUiAction>,
}

impl GuideUiOutbox {
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<GuideUiAction> {
        self.actions.pop_front()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }

    pub fn push(&mut self, action: GuideUiAction) {
        self.actions.push_back(action);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuideChangeSuccess {
    pub mentor: GuideMentor,
    pub mentor_count: i16,
    pub fusion_matter: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GuideMentorView {
    pub mentor: GuideMentor,
    pub card_rect: GuideUiRect,
    pub icon_rect: GuideUiRect,
    pub toggle_rect: GuideUiRect,
    pub portrait_rect: GuideUiRect,
    pub selected_effect_rect: GuideUiRect,
    pub current_frame_rect: GuideUiRect,
    pub selected: bool,
    pub current: bool,
    pub portrait_path: &'static str,
    pub icon_path: &'static str,
    pub toggle_path: &'static str,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GuideConfirmationView {
    pub mentor: GuideMentor,
    pub art_path: &'static str,
    pub art_rect: GuideUiRect,
    pub title: &'static str,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GuideUiView {
    pub layout: GuideUiLayout,
    pub purpose: GuideUiPurpose,
    pub phase: GuideUiPhase,
    pub backdrop_path: &'static str,
    pub heading: Option<&'static str>,
    pub intro: Option<&'static str>,
    pub primary_label: Option<&'static str>,
    pub displayed_cost: Option<String>,
    pub mentors: [GuideMentorView; 4],
    pub confirmation: Option<GuideConfirmationView>,
}

#[derive(Clone, Resource)]
pub(super) struct GuideUiAssets {
    pub(super) select_background: Handle<Image>,
    pub(super) change_background: Handle<Image>,
    pub(super) panel: Handle<Image>,
    pub(super) member_back: Handle<Image>,
    pub(super) selected_effect: Handle<Image>,
    pub(super) alert_icon: Handle<Image>,
    pub(super) confirm_icon: Handle<Image>,
    pub(super) black: Handle<Image>,
    pub(super) cost_bar: Handle<Image>,
    pub(super) fusion_matter_icon: Handle<Image>,
    pub(super) computress_icon: Handle<Image>,
    pub(super) card_frame: Handle<Image>,
    pub(super) current_frame: Handle<Image>,
    pub(super) computress_frame: Handle<Image>,
    pub(super) icon_frame: Handle<Image>,
    pub(super) dialog: Handle<Image>,
    pub(super) toggle_off: Handle<Image>,
    pub(super) toggle_on: Handle<Image>,
    pub(super) blue_button: Handle<Image>,
    pub(super) blue_button_over: Handle<Image>,
    pub(super) cancel_button: Handle<Image>,
    pub(super) close_button: Handle<Image>,
    pub(super) close_button_over: Handle<Image>,
    pub(super) help_button: Handle<Image>,
    pub(super) help_button_over: Handle<Image>,
    pub(super) portraits: [Handle<Image>; 4],
    pub(super) icons: [Handle<Image>; 4],
    pub(super) confirmation_art: [Handle<Image>; 4],
    pub(super) jeffe: Handle<Font>,
    pub(super) chalet: Handle<Font>,
}

impl GuideUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            select_background: asset_server.load(GUIDE_SELECT_BACKGROUND_PATH),
            change_background: asset_server.load(GUIDE_CHANGE_BACKGROUND_PATH),
            panel: asset_server.load(GUIDE_PANEL_PATH),
            member_back: asset_server.load(GUIDE_MEMBER_BACK_PATH),
            selected_effect: asset_server.load(GUIDE_SELECTED_EFFECT_PATH),
            alert_icon: asset_server.load(GUIDE_ALERT_ICON_PATH),
            confirm_icon: asset_server.load(GUIDE_CONFIRM_ICON_PATH),
            black: asset_server.load(GUIDE_BLACK_TEXTURE_PATH),
            cost_bar: asset_server.load(GUIDE_COST_BAR_PATH),
            fusion_matter_icon: asset_server.load(GUIDE_FUSION_MATTER_ICON_PATH),
            computress_icon: asset_server.load(GUIDE_COMPUTRESS_ICON_PATH),
            card_frame: asset_server.load(GUIDE_CARD_FRAME_PATH),
            current_frame: asset_server.load(GUIDE_CURRENT_FRAME_PATH),
            computress_frame: asset_server.load(GUIDE_COMPUTRESS_FRAME_PATH),
            icon_frame: asset_server.load(GUIDE_ICON_FRAME_PATH),
            dialog: asset_server.load(GUIDE_DIALOG_PATH),
            toggle_off: asset_server.load(GUIDE_TOGGLE_OFF_PATH),
            toggle_on: asset_server.load(GUIDE_TOGGLE_ON_PATH),
            blue_button: asset_server.load(GUIDE_BLUE_BUTTON_PATH),
            blue_button_over: asset_server.load(GUIDE_BLUE_BUTTON_OVER_PATH),
            cancel_button: asset_server.load(GUIDE_CANCEL_BUTTON_PATH),
            close_button: asset_server.load(GUIDE_CLOSE_BUTTON_PATH),
            close_button_over: asset_server.load(GUIDE_CLOSE_BUTTON_OVER_PATH),
            help_button: asset_server.load(GUIDE_HELP_BUTTON_PATH),
            help_button_over: asset_server.load(GUIDE_HELP_BUTTON_OVER_PATH),
            portraits: GuideMentor::CLEAN_ORDER
                .map(|mentor| asset_server.load(mentor.portrait_path())),
            icons: GuideMentor::CLEAN_ORDER.map(|mentor| asset_server.load(mentor.icon_path())),
            confirmation_art: GuideMentor::CLEAN_ORDER
                .map(|mentor| asset_server.load(mentor.confirm_path())),
            jeffe: asset_server.load(GUIDE_JEFFE_FONT_PATH),
            chalet: asset_server.load(GUIDE_CHALET_FONT_PATH),
        }
    }
}

#[derive(Component, Debug)]
pub struct GuideUiRoot;

#[derive(Component, Debug)]
pub struct GuideUiBackground;

#[derive(Component, Debug)]
pub struct GuideUiWarpWindow;

#[derive(Component, Debug)]
pub struct GuideUiConfirmationOverlay;

#[derive(Component, Debug)]
pub struct GuideUiConfirmationWindow;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct GuideUiMentorCard {
    pub mentor: GuideMentor,
}

#[derive(Component, Debug)]
pub(super) struct GuideUiCostGroup;

#[derive(Component, Debug)]
pub(super) struct GuideUiCostText;

#[derive(Component, Debug)]
pub(super) struct GuideUiCostIcon;

#[derive(Component, Debug)]
pub(super) struct GuideUiConfirmationArt;

#[derive(Component, Debug)]
pub(super) struct GuideUiConfirmationTitle;

#[derive(Component, Debug)]
pub(super) struct GuideUiConfirmationBody;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GuideUiTextRole {
    Computress,
    SelectionHeading,
    SelectionIntro,
    MentorName(GuideMentor),
    MentorDescription(GuideMentor),
    CurrentGuide,
    Cost,
    PrimaryButton,
    WarpTitle,
    WarpBody,
    WarpButton,
    ConfirmationTitle,
    ConfirmationBody,
    CommonCancel,
    CommonConfirm,
}

#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub struct GuideUiTextElement {
    pub role: GuideUiTextRole,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum GuideUiSet {
    Interaction,
    Bind,
    Visuals,
}

#[derive(Default)]
pub struct GuideUiPlugin;

impl Plugin for GuideUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<GuideUiModel>()
            .init_resource::<GuideUiOutbox>()
            .init_resource::<GuideUiAudioOutbox>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_guide_ui)
            .configure_sets(
                Update,
                (
                    GuideUiSet::Interaction,
                    GuideUiSet::Bind,
                    GuideUiSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (handle_guide_ui_interactions.in_set(GuideUiSet::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    sync_guide_ui_root_and_layout,
                    sync_guide_ui_selection,
                    sync_guide_ui_confirmation,
                    sync_guide_ui_cost_geometry,
                )
                    .chain()
                    .in_set(GuideUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (sync_guide_ui_button_visuals.in_set(GuideUiSet::Visuals))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
