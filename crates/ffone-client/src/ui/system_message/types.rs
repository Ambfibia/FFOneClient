use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemMessageFontRole {
    Jeffe,
    Chalet,
}

#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum SystemMessageTextStyle {
    Label,
    ImageWindow,
    Button,
    CancelButton,
    RedButton,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemMessageTextStyleSpec {
    pub source_style: &'static str,
    pub source_font_path_id: i64,
    pub font_role: SystemMessageFontRole,
    pub font_size: f32,
    pub line_height: f32,
    /// Unity `RectOffset` order: left, right, top, bottom.
    pub margin: [f32; 4],
    /// Unity `RectOffset` order: left, right, top, bottom.
    pub padding: [f32; 4],
    pub justify: Justify,
    pub linebreak: LineBreak,
    pub stretch_width: bool,
    /// Replacement-font baseline compensation. Serialized contentOffset.y is
    /// zero for every reached style, so the clean value remains zero here.
    pub y_offset: f32,
}

impl SystemMessageTextStyle {
    #[must_use]
    pub const fn spec(self) -> SystemMessageTextStyleSpec {
        match self {
            Self::Label => SystemMessageTextStyleSpec {
                source_style: "label",
                source_font_path_id: SYSTEM_MESSAGE_JEFFE_14_PATH_ID,
                font_role: SystemMessageFontRole::Jeffe,
                font_size: SYSTEM_MESSAGE_JEFFE_14_FONT_SIZE,
                line_height: SYSTEM_MESSAGE_JEFFE_14_LINE_HEIGHT,
                margin: [4.0; 4],
                padding: [0.0, 0.0, 3.0, 3.0],
                justify: Justify::Left,
                linebreak: LineBreak::WordBoundary,
                stretch_width: false,
                y_offset: 0.0,
            },
            Self::ImageWindow => SystemMessageTextStyleSpec {
                source_style: "Imagewindow",
                source_font_path_id: SYSTEM_MESSAGE_CHALET_SMALL_PATH_ID,
                font_role: SystemMessageFontRole::Chalet,
                font_size: SYSTEM_MESSAGE_CHALET_SMALL_FONT_SIZE,
                line_height: SYSTEM_MESSAGE_CHALET_SMALL_LINE_HEIGHT,
                margin: [0.0; 4],
                padding: [0.0; 4],
                justify: Justify::Left,
                linebreak: LineBreak::WordBoundary,
                stretch_width: true,
                y_offset: 0.0,
            },
            Self::Button => SystemMessageTextStyleSpec {
                source_style: "button",
                source_font_path_id: SYSTEM_MESSAGE_JEFFE_14_PATH_ID,
                font_role: SystemMessageFontRole::Jeffe,
                font_size: SYSTEM_MESSAGE_JEFFE_14_FONT_SIZE,
                line_height: SYSTEM_MESSAGE_JEFFE_14_LINE_HEIGHT,
                margin: [0.0; 4],
                padding: [0.0; 4],
                justify: Justify::Center,
                linebreak: LineBreak::NoWrap,
                stretch_width: true,
                y_offset: 0.0,
            },
            Self::CancelButton => SystemMessageTextStyleSpec {
                source_style: "CancelButton",
                source_font_path_id: SYSTEM_MESSAGE_JEFFE_12_PATH_ID,
                font_role: SystemMessageFontRole::Jeffe,
                font_size: SYSTEM_MESSAGE_JEFFE_12_FONT_SIZE,
                line_height: SYSTEM_MESSAGE_JEFFE_12_LINE_HEIGHT,
                margin: [4.0; 4],
                padding: [10.0, 6.0, 4.0, 6.0],
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
                stretch_width: true,
                y_offset: 0.0,
            },
            Self::RedButton => SystemMessageTextStyleSpec {
                source_style: "RedButton",
                source_font_path_id: SYSTEM_MESSAGE_JEFFE_14_PATH_ID,
                font_role: SystemMessageFontRole::Jeffe,
                font_size: SYSTEM_MESSAGE_JEFFE_14_FONT_SIZE,
                line_height: SYSTEM_MESSAGE_JEFFE_14_LINE_HEIGHT,
                margin: [4.0; 4],
                padding: [10.0, 6.0, 4.0, 6.0],
                justify: Justify::Center,
                linebreak: LineBreak::WordBoundary,
                stretch_width: true,
                y_offset: 0.0,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemMessageChoice {
    Primary,
    Secondary,
}

#[derive(Default, Resource)]
pub struct SystemMessageUiOutbox {
    pub(super) actions: VecDeque<SystemMessageUiAction>,
}

impl SystemMessageUiOutbox {
    pub fn push(&mut self, action: SystemMessageUiAction) {
        self.actions.push_back(action);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = SystemMessageUiAction> + '_ {
        self.actions.drain(..)
    }

    /// Removes only actions owned by one correlated consumer while
    /// preserving every unrelated action and its original FIFO order.
    pub fn drain_matching(
        &mut self,
        mut predicate: impl FnMut(&SystemMessageUiAction) -> bool,
    ) -> Vec<SystemMessageUiAction> {
        let mut matching = Vec::new();
        let mut retained = VecDeque::with_capacity(self.actions.len());
        while let Some(action) = self.actions.pop_front() {
            if predicate(&action) {
                matching.push(action);
            } else {
                retained.push_back(action);
            }
        }
        self.actions = retained;
        matching
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

#[derive(Clone, Resource)]
pub(super) struct SystemMessageUiAssets {
    pub(super) dialog: Handle<Image>,
    pub(super) item_box: Handle<Image>,
    pub(super) combined_badge: Handle<Image>,
    pub(super) jeffe_font: Handle<Font>,
    pub(super) chalet_font: Handle<Font>,
    pub(super) blue: Handle<Image>,
    pub(super) blue_over: Handle<Image>,
    pub(super) red: Handle<Image>,
    pub(super) red_over: Handle<Image>,
    pub(super) cancel: Handle<Image>,
}

impl SystemMessageUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            dialog: asset_server.load(SYSTEM_MESSAGE_DIALOG_PATH),
            item_box: asset_server.load(SYSTEM_MESSAGE_ITEM_BOX_PATH),
            combined_badge: asset_server.load(SYSTEM_MESSAGE_COMBINED_BADGE_PATH),
            jeffe_font: asset_server.load(SYSTEM_MESSAGE_FONT_PATH),
            chalet_font: asset_server.load(SYSTEM_MESSAGE_BODY_FONT_PATH),
            blue: asset_server.load(SYSTEM_MESSAGE_BLUE_BUTTON_PATH),
            blue_over: asset_server.load(SYSTEM_MESSAGE_BLUE_BUTTON_OVER_PATH),
            red: asset_server.load(SYSTEM_MESSAGE_RED_BUTTON_PATH),
            red_over: asset_server.load(SYSTEM_MESSAGE_RED_BUTTON_OVER_PATH),
            cancel: asset_server.load(SYSTEM_MESSAGE_CANCEL_BUTTON_PATH),
        }
    }

    pub(super) fn button_image(
        &self,
        visual: SystemMessageButtonVisual,
        interaction: Interaction,
    ) -> Handle<Image> {
        match (visual, interaction) {
            (SystemMessageButtonVisual::Standard, Interaction::Hovered) => self.blue_over.clone(),
            (SystemMessageButtonVisual::Standard, _) => self.blue.clone(),
            (SystemMessageButtonVisual::Destructive, Interaction::Hovered) => self.red_over.clone(),
            (SystemMessageButtonVisual::Destructive, _) => self.red.clone(),
            (SystemMessageButtonVisual::Cancel, Interaction::None) => self.cancel.clone(),
            (SystemMessageButtonVisual::Cancel, _) => self.blue.clone(),
        }
    }

    pub(super) fn text_font(&self, style: SystemMessageTextStyle) -> Handle<Font> {
        match style.spec().font_role {
            SystemMessageFontRole::Jeffe => self.jeffe_font.clone(),
            SystemMessageFontRole::Chalet => self.chalet_font.clone(),
        }
    }
}

/// Marker for the single full-viewport modal/input-blocking root.
#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct SystemMessageUiRoot;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct SystemMessageLayer(pub(super) usize);

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct SystemMessageBody;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct SystemMessageBodyLine {
    pub index: usize,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct SystemMessagePrimaryIcon;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct SystemMessageIconQuantity;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct SystemMessageComparisonIcon(pub(super) usize);

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub(super) struct SystemMessageCombinedBadge(pub(super) usize);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum SystemMessageUiSet {
    Rebuild,
    Layout,
    Interaction,
    Visuals,
    Cursor,
}

#[derive(Default)]
pub struct SystemMessageUiPlugin;

impl Plugin for SystemMessageUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SystemMessageUiModel>()
            .init_resource::<SystemMessageUiOutbox>()
            .init_resource::<SystemMessageUiAudioOutbox>()
            .init_resource::<SystemMessageCursorLease>()
            .add_systems(Startup, spawn_system_message_ui)
            .configure_sets(
                Update,
                (
                    SystemMessageUiSet::Rebuild,
                    SystemMessageUiSet::Layout,
                    SystemMessageUiSet::Interaction,
                    SystemMessageUiSet::Visuals,
                    SystemMessageUiSet::Cursor,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                rebuild_system_message_ui
                    .in_set(SystemMessageUiSet::Rebuild)
                    .before(LocalizationSet::Apply),
            )
            .add_systems(
                Update,
                update_system_message_layout.in_set(SystemMessageUiSet::Layout),
            )
            .add_systems(
                Update,
                scroll_system_message_body.in_set(SystemMessageUiSet::Interaction),
            )
            .add_systems(
                Update,
                handle_system_message_buttons.in_set(SystemMessageUiSet::Interaction),
            )
            .add_systems(
                Update,
                update_system_message_button_visuals.in_set(SystemMessageUiSet::Visuals),
            )
            .add_systems(
                Update,
                sync_system_message_cursor.in_set(SystemMessageUiSet::Cursor),
            );
    }
}
