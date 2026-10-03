use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanocomGuiInsets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl NanocomGuiInsets {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub(super) fn ui_rect(self) -> UiRect {
        UiRect::new(px(self.left), px(self.right), px(self.top), px(self.bottom))
    }
}

/// Reached clean `FusionFallHUDSkin` styles. Alignment values are serialized
/// `TextAnchor` values (`0` UpperLeft, `1` UpperCenter, `3` MiddleLeft,
/// `4` MiddleCenter). All reached styles serialize `contentOffset=(0,0)`, so
/// the approved replacement faces require no additional per-style Y shift.
#[derive(Clone, Copy, Debug, Component, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NanocomGuiStyleRole {
    BigFont14,
    MessageText,
    MessageTitle,
    CenterBox2,
    Button,
    RedButton,
}

/// Clean immediate-mode call order, made explicit so Bevy sibling order stays
/// reviewable. The modal dialog background is painted by its parent before
/// the numbered child calls.
#[derive(Clone, Copy, Debug, Component, Eq, Hash, PartialEq)]
pub enum NanocomDrawRole {
    CompactFrame,
    CompactTitle,
    CompactBody,
    CompactIcon,
    ModalOverlay,
    ModalDialogBox,
    ModalMessageArea,
    ModalIcon,
    ModalTitle,
    ModalTextArea,
    ModalAccept,
    ModalDecline,
}

impl NanocomDrawRole {
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::CompactFrame => 0,
            Self::CompactTitle => 1,
            Self::CompactBody => 2,
            Self::CompactIcon => 3,
            Self::ModalOverlay => 0,
            Self::ModalDialogBox => 1,
            Self::ModalMessageArea => 2,
            Self::ModalIcon => 3,
            Self::ModalTitle => 4,
            Self::ModalTextArea => 5,
            Self::ModalAccept => 6,
            Self::ModalDecline => 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanocomGuiStyleEvidence {
    pub skin_path_id: i64,
    pub style_name: &'static str,
    pub source_font_name: &'static str,
    pub source_font_path_id: i64,
    pub replacement_font_path: &'static str,
    pub replacement_font_size: f32,
    pub source_line_height: f32,
    pub alignment: i32,
    pub word_wrap: bool,
    pub text_clipping: i32,
    pub padding: NanocomGuiInsets,
    pub margin: NanocomGuiInsets,
    pub content_offset: [f32; 2],
    pub replacement_y_offset: f32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum NanocomMessageKind {
    Npc = 9,
    Nano = 10,
    TradeInvite = 12,
    BuddyInvite = 13,
    GroupInvite = 14,
    ClubInvite = 15,
    Skill = 16,
}

impl NanocomMessageKind {
    #[must_use]
    pub const fn raw(self) -> i32 {
        self as i32
    }

    /// Clean `AddMessage` treats every type above 11 as interactive priority.
    #[must_use]
    pub const fn is_interactive(self) -> bool {
        self.raw() > 11
    }

    /// Only these branches have a production producer in the current native
    /// client. Retain the remaining wire values in the queue contract, but do
    /// not fabricate their unavailable clean content in presentation.
    #[must_use]
    pub const fn is_reached_presentation(self) -> bool {
        matches!(
            self,
            Self::Npc | Self::Nano | Self::BuddyInvite | Self::GroupInvite
        )
    }

    #[must_use]
    pub const fn has_reached_modal(self) -> bool {
        matches!(self, Self::BuddyInvite | Self::GroupInvite)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QueuedNanocomMessage {
    pub request: NanocomMessageRequest,
    pub remaining_seconds: f32,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NanocomMessageChoice {
    Accept,
    Decline,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NanocomMessageResolution {
    Accepted,
    Declined,
    TimedOut,
}

#[derive(Debug, Default, Resource)]
pub struct NanocomMessageUiOutbox {
    pub(super) actions: VecDeque<NanocomMessageUiAction>,
}

impl NanocomMessageUiOutbox {
    pub fn push(&mut self, action: NanocomMessageUiAction) {
        self.actions.push_back(action);
    }

    pub fn pop_front(&mut self) -> Option<NanocomMessageUiAction> {
        self.actions.pop_front()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }
}

/// Chat copy of one accepted message. Clean `cnGUINanocom.SetMessageBox`
/// writes `TitleString + ": " + OutString` through `CnGuiChat.AddEventString`:
/// type 4 (NPC/events-in-chat) below button 11 and type 1 for interactive
/// invitations. The chat owner resolves both texts at receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct NanocomChatEcho {
    pub request_id: u64,
    pub interactive: bool,
    pub title: LocalizedText,
    pub body: LocalizedText,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NanocomBindingKey {
    pub(super) viewport: [u32; 2],
    pub(super) scale: u32,
    pub(super) reveal: u32,
    pub(super) active_request: Option<NanocomMessageRequest>,
    pub(super) rounded_remaining_seconds: Option<String>,
    pub(super) compact_visible: bool,
    pub(super) expanded_visible: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum NanocomMessageUiSet {
    Tick,
    Bind,
    Input,
    Audio,
}

#[derive(Clone, Copy, Debug, Component, Eq, PartialEq)]
pub(super) enum NanocomMessageElement {
    CompactPanel,
    CompactFrame,
    CompactIcon,
    CompactTitle,
    CompactBody,
    ExpandedRoot,
    ExpandedDialog,
    ExpandedIcon,
    ExpandedTitle,
    ExpandedBody,
    ExpandedExpiration,
}

#[derive(Component)]
pub struct NanocomMessageUiRoot;

#[derive(Component)]
pub struct NanocomMessageCompactPanel;

#[derive(Component)]
pub struct NanocomMessageExpandedRoot;

#[derive(Component)]
pub struct NanocomMessageExpandedDialog;

#[derive(Clone, Resource)]
pub(super) struct NanocomMessageUiAssets {
    pub(super) buddy_frame: Handle<Image>,
    pub(super) buddy_icon: Handle<Image>,
    pub(super) _group_icon: Handle<Image>,
    pub(super) _type_9_frame: Handle<Image>,
    pub(super) _nano_frame: Handle<Image>,
    pub(super) _numbuh_two_icon: Handle<Image>,
    pub(super) dialog: Handle<Image>,
    pub(super) message_area: Handle<Image>,
    pub(super) blue: Handle<Image>,
    pub(super) blue_over: Handle<Image>,
    pub(super) red: Handle<Image>,
    pub(super) red_over: Handle<Image>,
    pub(super) jeffe: Handle<Font>,
    pub(super) chalet: Handle<Font>,
}

impl NanocomMessageUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            buddy_frame: asset_server.load(NANOCOM_BUDDY_FRAME_PATH),
            buddy_icon: asset_server.load(NANOCOM_BUDDY_ICON_PATH),
            _group_icon: asset_server.load(NANOCOM_GROUP_ICON_PATH),
            _type_9_frame: asset_server.load(NANOCOM_TYPE_9_FRAME_PATH),
            _nano_frame: asset_server.load(NANOCOM_NANO_FRAME_PATH),
            _numbuh_two_icon: asset_server.load(NANOCOM_NUMBUH_TWO_ICON_PATH),
            dialog: asset_server.load(NANOCOM_DIALOG_PATH),
            message_area: asset_server.load(NANOCOM_MESSAGE_AREA_PATH),
            blue: asset_server.load(NANOCOM_BLUE_BUTTON_PATH),
            blue_over: asset_server.load(NANOCOM_BLUE_BUTTON_OVER_PATH),
            red: asset_server.load(NANOCOM_RED_BUTTON_PATH),
            red_over: asset_server.load(NANOCOM_RED_BUTTON_OVER_PATH),
            jeffe: asset_server.load(NANOCOM_JEFFE_FONT_PATH),
            chalet: asset_server.load(NANOCOM_CHALET_FONT_PATH),
        }
    }

    pub(super) fn button_image(
        &self,
        choice: NanocomMessageChoice,
        interaction: Interaction,
    ) -> Handle<Image> {
        match (choice, interaction) {
            (NanocomMessageChoice::Accept, Interaction::Hovered) => self.blue_over.clone(),
            (NanocomMessageChoice::Accept, Interaction::None | Interaction::Pressed) => {
                self.blue.clone()
            }
            (NanocomMessageChoice::Decline, Interaction::Hovered) => self.red_over.clone(),
            (NanocomMessageChoice::Decline, Interaction::None | Interaction::Pressed) => {
                self.red.clone()
            }
        }
    }
}

pub struct NanocomMessageUiPlugin;

impl Plugin for NanocomMessageUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<NanocomMessageUiModel>()
            .init_resource::<NanocomMessageUiOutbox>()
            .configure_sets(
                Update,
                (
                    NanocomMessageUiSet::Tick,
                    NanocomMessageUiSet::Bind,
                    NanocomMessageUiSet::Input,
                    NanocomMessageUiSet::Audio,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_nanocom_message_ui,
            )
            .add_systems(
                Update,
                (tick_nanocom_messages.in_set(NanocomMessageUiSet::Tick))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (bind_nanocom_message_ui
                    .in_set(NanocomMessageUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    handle_nanocom_buttons,
                    update_nanocom_button_visuals,
                    latch_nanocom_button_request_ids,
                )
                    .chain()
                    .in_set(NanocomMessageUiSet::Input))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (play_nanocom_message_sounds.in_set(NanocomMessageUiSet::Audio))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
