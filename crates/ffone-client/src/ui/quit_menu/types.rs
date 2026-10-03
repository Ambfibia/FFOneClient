use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuitMenuTextStyleSpec {
    pub source_style: &'static str,
    pub source_font_path_id: i64,
    pub font_size: f32,
    pub line_height: f32,
    /// Unity `RectOffset` order: left, right, top, bottom.
    pub padding: [f32; 4],
    pub word_wrap: bool,
    pub clips_text: bool,
    pub y_offset: f32,
}

/// Exact serialized text style selected by the owning legacy button.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub struct QuitMenuTextStyle(pub QuitMenuButtonVisual);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QuitMenuDismissalSource {
    CancelButton,
    EscapeKey,
}

#[derive(Debug, Default, Resource)]
pub struct QuitMenuUiOutbox {
    pub(super) actions: VecDeque<QuitMenuUiAction>,
}

impl QuitMenuUiOutbox {
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<QuitMenuUiAction> {
        self.actions.pop_front()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }

    pub(super) fn push(&mut self, action: QuitMenuUiAction) {
        self.actions.push_back(action);
    }
}

#[derive(Clone, Resource)]
pub(super) struct QuitMenuUiAssets {
    pub(super) backdrop: Handle<Image>,
    pub(super) dialog: Handle<Image>,
    pub(super) button_normal: Handle<Image>,
    pub(super) button_hover: Handle<Image>,
    pub(super) button_active: Option<Handle<Image>>,
    pub(super) cancel_normal: Handle<Image>,
    pub(super) cancel_hover: Handle<Image>,
    pub(super) cancel_active: Option<Handle<Image>>,
    pub(super) font: Handle<Font>,
}

impl QuitMenuUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            backdrop: asset_server.load(QUIT_MENU_BACKDROP_PATH),
            dialog: asset_server.load(QUIT_MENU_DIALOG_PATH),
            button_normal: asset_server.load(QUIT_MENU_BUTTON_NORMAL_PATH),
            button_hover: asset_server.load(QUIT_MENU_BUTTON_HOVER_PATH),
            button_active: None,
            cancel_normal: asset_server.load(QUIT_MENU_CANCEL_NORMAL_PATH),
            cancel_hover: asset_server.load(QUIT_MENU_CANCEL_HOVER_PATH),
            cancel_active: None,
            font: asset_server.load(QUIT_MENU_FONT_PATH),
        }
    }

    pub(super) fn button_image(
        &self,
        visual: QuitMenuButtonVisual,
        interaction: Interaction,
    ) -> Handle<Image> {
        match (visual, interaction) {
            (QuitMenuButtonVisual::Standard, Interaction::Pressed)
                if self.button_active.is_some() =>
            {
                self.button_active.clone().unwrap()
            }
            (QuitMenuButtonVisual::Cancel, Interaction::Pressed)
                if self.cancel_active.is_some() =>
            {
                self.cancel_active.clone().unwrap()
            }
            (QuitMenuButtonVisual::Standard, Interaction::Hovered) => self.button_hover.clone(),
            (QuitMenuButtonVisual::Standard, Interaction::None | Interaction::Pressed) => {
                self.button_normal.clone()
            }
            (QuitMenuButtonVisual::Cancel, Interaction::None) => self.cancel_normal.clone(),
            (QuitMenuButtonVisual::Cancel, Interaction::Hovered | Interaction::Pressed) => {
                self.cancel_hover.clone()
            }
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct QuitMenuUiRoot;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct QuitMenuBackdrop;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct QuitMenuDialog;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum QuitMenuUiSet {
    Interaction,
    Bind,
    Visuals,
}

#[derive(Default)]
pub struct QuitMenuUiPlugin;

impl Plugin for QuitMenuUiPlugin {
    fn build(&self, app: &mut App) {
        document::install(app);
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<QuitMenuUiModel>()
            .init_resource::<QuitMenuUiOutbox>()
            .init_resource::<QuitMenuAudioOutbox>()
            .init_resource::<QuitMenuClickSoundSequence>()
            .init_resource::<QuitMenuPresentationState>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_quit_menu_ui,
            )
            .configure_sets(
                Update,
                (
                    QuitMenuUiSet::Interaction,
                    QuitMenuUiSet::Bind,
                    QuitMenuUiSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                ((handle_quit_menu_keyboard, handle_quit_menu_interactions)
                    .chain()
                    .in_set(QuitMenuUiSet::Interaction))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((sync_quit_menu_visibility, update_quit_menu_layout)
                    .chain()
                    .in_set(QuitMenuUiSet::Bind))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (update_quit_menu_button_visuals.in_set(QuitMenuUiSet::Visuals))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
