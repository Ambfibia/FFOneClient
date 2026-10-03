use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpsellSourceReachability {
    ReceiveInit,
    RetainedExplicit,
    UnregisteredDead,
}

#[derive(Debug, Default, Resource)]
pub struct UpsellUiOutbox {
    pub(super) actions: VecDeque<UpsellUiAction>,
}

impl UpsellUiOutbox {
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<UpsellUiAction> {
        self.actions.pop_front()
    }

    pub fn clear(&mut self) {
        self.actions.clear();
    }

    pub(super) fn push(&mut self, action: UpsellUiAction) {
        self.actions.push_back(action);
    }
}

#[derive(Clone, Resource)]
pub(super) struct UpsellUiAssets {
    pub(super) panelback: Handle<Image>,
    pub(super) levels: [Handle<Image>; 4],
    pub(super) close_normal: Handle<Image>,
    pub(super) close_hover: Handle<Image>,
    pub(super) get_normal: Handle<Image>,
    pub(super) get_hover: Handle<Image>,
    pub(super) continue_normal: Handle<Image>,
    pub(super) continue_hover: Handle<Image>,
    pub(super) not_now_normal: Handle<Image>,
    pub(super) not_now_hover: Handle<Image>,
    pub(super) advertis: Handle<Image>,
    pub(super) news_button_normal: Handle<Image>,
    pub(super) news_button_hover: Handle<Image>,
    pub(super) jeffe: Handle<Font>,
}

impl UpsellUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            panelback: asset_server.load(UPSELL_PANELBACK_PATH),
            levels: UPSELL_LEVEL_IMAGE_PATHS.map(|path| asset_server.load(path)),
            close_normal: asset_server.load(UPSELL_CLOSE_NORMAL_PATH),
            close_hover: asset_server.load(UPSELL_CLOSE_HOVER_PATH),
            get_normal: asset_server.load(UPSELL_GET_NORMAL_PATH),
            get_hover: asset_server.load(UPSELL_GET_HOVER_PATH),
            continue_normal: asset_server.load(UPSELL_CONTINUE_NORMAL_PATH),
            continue_hover: asset_server.load(UPSELL_CONTINUE_HOVER_PATH),
            not_now_normal: asset_server.load(UPSELL_NOT_NOW_NORMAL_PATH),
            not_now_hover: asset_server.load(UPSELL_NOT_NOW_HOVER_PATH),
            advertis: asset_server.load(UPSELL_ADVERTIS_PATH),
            news_button_normal: asset_server.load(UPSELL_NEWS_BUTTON_NORMAL_PATH),
            news_button_hover: asset_server.load(UPSELL_NEWS_BUTTON_HOVER_PATH),
            jeffe: asset_server.load(UPSELL_FONT_PATH),
        }
    }

    pub(super) fn level_image(&self, level: u8) -> Option<Handle<Image>> {
        self.levels.get(usize::from(level.checked_sub(1)?)).cloned()
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct UpsellUiRoot;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct UpsellUiBackdrop;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct UpsellUiDialog;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct UpsellUiAdvertis;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum UpsellUiSet {
    Interaction,
    Bind,
    Visuals,
}

#[derive(Default)]
pub struct UpsellUiPlugin;

impl Plugin for UpsellUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<UpsellUiModel>()
            .init_resource::<UpsellUiOutbox>()
            .init_resource::<UpsellUiAudioOutbox>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_upsell_ui)
            .configure_sets(
                Update,
                (
                    UpsellUiSet::Interaction,
                    UpsellUiSet::Bind,
                    UpsellUiSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                ((handle_upsell_keyboard, handle_upsell_interactions)
                    .chain()
                    .in_set(UpsellUiSet::Interaction))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    advance_upsell_page_fade,
                    sync_upsell_root_and_layout,
                    sync_upsell_element_geometry,
                )
                    .chain()
                    .in_set(UpsellUiSet::Bind))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (sync_upsell_button_visuals
                    .in_set(UpsellUiSet::Visuals)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
