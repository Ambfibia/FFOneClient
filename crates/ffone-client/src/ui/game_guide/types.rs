use super::*;

#[derive(Clone, Debug)]
pub(super) struct HelpRange {
    pub(super) start: usize,
    pub(super) end: usize,
}

#[derive(Clone, Debug)]
pub(super) struct HelpContent {
    pub(super) kind: u8,
    pub(super) size: u8,
    pub(super) string_id: usize,
}

#[derive(Component)]
pub struct GameGuideUiRoot;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GameGuideControl {
    Main(usize),
    Sub(usize),
    Previous,
    Next,
    ScrollUp,
    ScrollDown,
    Close,
}

#[derive(Component)]
pub(super) struct GameGuideMainLabel(pub(super) usize);

#[derive(Component)]
pub(super) struct GameGuideSubLabel(pub(super) usize);

#[derive(Component)]
pub(super) struct GameGuideWindow;

#[derive(Component)]
pub(super) struct GameGuideBackdrop;

#[derive(Component)]
pub(super) struct GameGuideContentStack {
    pub(super) page_id: usize,
}

#[derive(Resource)]
pub(super) struct GameGuideAssets {
    pub(super) backdrop: Handle<Image>,
    pub(super) background: Handle<Image>,
    pub(super) close: Handle<Image>,
    pub(super) close_over: Handle<Image>,
    pub(super) help_button: Handle<Image>,
    pub(super) help_button_over: Handle<Image>,
    pub(super) title_bar: Handle<Image>,
    pub(super) nav_button: Handle<Image>,
    pub(super) nav_button_over: Handle<Image>,
    pub(super) scroll_up: Handle<Image>,
    pub(super) scroll_down: Handle<Image>,
    pub(super) scroll_track: Handle<Image>,
    pub(super) scroll_thumb: Handle<Image>,
    pub(super) jeffe_font: Handle<Font>,
    pub(super) chalet_font: Handle<Font>,
}

impl FromWorld for GameGuideAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            backdrop: assets.load(GAME_GUIDE_BACKDROP_PATH),
            background: assets.load(GAME_GUIDE_BACKGROUND_PATH),
            close: assets.load(GAME_GUIDE_CLOSE_PATH),
            close_over: assets.load(GAME_GUIDE_CLOSE_OVER_PATH),
            help_button: assets.load(GAME_GUIDE_HELP_BUTTON_PATH),
            help_button_over: assets.load(GAME_GUIDE_HELP_BUTTON_OVER_PATH),
            title_bar: assets.load(GAME_GUIDE_TITLE_BAR_PATH),
            nav_button: assets.load(GAME_GUIDE_NAV_BUTTON_PATH),
            nav_button_over: assets.load(GAME_GUIDE_NAV_BUTTON_OVER_PATH),
            scroll_up: assets.load(GAME_GUIDE_SCROLL_UP_PATH),
            scroll_down: assets.load(GAME_GUIDE_SCROLL_DOWN_PATH),
            scroll_track: assets.load(GAME_GUIDE_SCROLL_TRACK_PATH),
            scroll_thumb: assets.load(GAME_GUIDE_SCROLL_THUMB_PATH),
            jeffe_font: assets.load(GAME_GUIDE_JEFFE_FONT_PATH),
            chalet_font: assets.load(GAME_GUIDE_CHALET_FONT_PATH),
        }
    }
}

#[derive(Default)]
pub struct GameGuideUiPlugin;

impl Plugin for GameGuideUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<GameGuideUiModel>()
            .init_resource::<GameGuideCatalog>()
            .init_resource::<GameGuideAssets>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_game_guide_ui,
            )
            .add_systems(
                Update,
                (
                    handle_game_guide_controls,
                    rebuild_game_guide_content,
                    sync_game_guide_layout,
                    sync_game_guide_labels,
                    sync_game_guide_scroll.before(LocalizationSet::Apply),
                    sync_game_guide_button_visuals,
                )
                    .chain()
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
