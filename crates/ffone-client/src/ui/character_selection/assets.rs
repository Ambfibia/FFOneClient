use super::*;

pub const CHARACTER_SELECTION_MUSIC_PATH: &str = "audio/music/00_main_theme.ogg";

pub const CHARACTER_SELECTION_CHROME_PATH: &str = "ui/en/character/selection/background/CSBG.png";

pub const CHARACTER_SELECTION_SLOT_EMPTY_PATH: &str =
    "ui/en/character/selection/controls/CSCharButtonEmpty.png";

pub const CHARACTER_SELECTION_SLOT_NORMAL_PATH: &str =
    "ui/en/character/selection/controls/CSCharButtonNormal.png";

pub const CHARACTER_SELECTION_SLOT_OVER_PATH: &str =
    "ui/en/character/selection/controls/CSCharButtonOver.png";

pub const CHARACTER_SELECTION_SLOT_LOCKED_PATH: &str =
    "ui/en/character/selection/controls/CSCharButtonSubscription.png";

pub const CHARACTER_SELECTION_LOCK_PATH: &str = "ui/en/character/selection/controls/CSCharLock.png";

pub const CHARACTER_SELECTION_ENTER_PATH: &str =
    "ui/en/character/selection/controls/CSEnterButton.png";

pub const CHARACTER_SELECTION_ENTER_OVER_PATH: &str =
    "ui/en/character/selection/controls/CSEnterGameOver.png";

pub const CHARACTER_SELECTION_FULLSCREEN_PATH: &str =
    "ui/en/character/selection/controls/CSFullScreenButton.png";

pub const CHARACTER_SELECTION_WINDOWED_PATH: &str =
    "ui/en/character/selection/controls/CSFullToWindow.png";

pub const CHARACTER_SELECTION_DELETE_WINDOW_PATH: &str =
    "ui/en/character/selection/controls/CSDeleteWindow.png";

pub const CHARACTER_SELECTION_DELETE_BACKDROP_PATH: &str =
    "ui/en/character/selection/controls/CSDeleteBG.png";

pub const CHARACTER_SELECTION_CANCEL_NORMAL_PATH: &str =
    "ui/en/character/selection/controls/CancelNormal.png";

pub const CHARACTER_SELECTION_MUSIC_TOGGLE_ON_PATH: &str =
    "ui/en/character/selection/controls/musicon.png";

pub const CHARACTER_SELECTION_MUSIC_TOGGLE_OFF_PATH: &str =
    "ui/en/character/selection/controls/musicoff.png";

pub const CHARACTER_SELECTION_FULLSCREEN_OVER_PATH: &str =
    "ui/en/character/selection/controls/FullscreenModeButton_Over.png";

pub const CHARACTER_SELECTION_WINDOWED_OVER_PATH: &str =
    "ui/en/character/selection/controls/WindowModeButton_Over.png";

pub const CHARACTER_SELECTION_ROTATE_LEFT_PATH: &str =
    "ui/en/character/creation/camera/CCRotateLeftNormal.png";

pub const CHARACTER_SELECTION_ROTATE_LEFT_OVER_PATH: &str =
    "ui/en/character/creation/camera/CCRotateLeftOver.png";

pub const CHARACTER_SELECTION_ROTATE_RIGHT_PATH: &str =
    "ui/en/character/creation/camera/CCRotateRightNormal.png";

pub const CHARACTER_SELECTION_ROTATE_RIGHT_OVER_PATH: &str =
    "ui/en/character/creation/camera/CCRoatateRightOver.png";

pub const CHARACTER_SELECTION_DISK_BACK_PATH: &str = "ui/en/gameplay/player/DiskBack.png";

pub const CHARACTER_SELECTION_DISK_FRONT_PATH: &str = "ui/en/gameplay/player/DiskFront.png";

pub const CHARACTER_SELECTION_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const CHARACTER_SELECTION_JEFFE_FONT_PATH: &str = "fonts/jeffe.otf";

pub const CHARACTER_SELECTION_PRIMARY_ASSET_FILE: &str = "sharedassets0.assets";

pub const CHARACTER_SELECTION_GAME_OBJECT_PATH_ID: i64 = 1_334;

pub const CHARACTER_SELECTION_SKIN_PATH_ID: i64 = 1_382;

pub const CHARACTER_SELECTION_GUI_COMPONENT_PATH_ID: i64 = 1_523;

pub const CHARACTER_SELECTION_MODE_COMPONENT_PATH_ID: i64 = 1_524;

pub const CHARACTER_SELECTION_GUI_SCRIPT_PATH_ID: i64 = 1_161;

pub const CHARACTER_SELECTION_MODE_SCRIPT_PATH_ID: i64 = 921;

pub const CHARACTER_SELECTION_CHROME_SOURCE_PATH_ID: i64 = 78;

pub const CHARACTER_SELECTION_DELETE_BACKDROP_SOURCE_PATH_ID: i64 = 49;

pub const CHARACTER_SELECTION_DELETE_WINDOW_SOURCE_PATH_ID: i64 = 97;

pub const CHARACTER_SELECTION_SLOT_EMPTY_SOURCE_PATH_ID: i64 = 202;

pub const CHARACTER_SELECTION_SLOT_NORMAL_SOURCE_PATH_ID: i64 = 430;

pub const CHARACTER_SELECTION_SLOT_OVER_SOURCE_PATH_ID: i64 = 147;

pub const CHARACTER_SELECTION_SLOT_LOCKED_SOURCE_PATH_ID: i64 = 221;

pub const CHARACTER_SELECTION_LOCK_SOURCE_PATH_ID: i64 = 142;

pub const CHARACTER_SELECTION_ENTER_SOURCE_PATH_ID: i64 = 611;

pub const CHARACTER_SELECTION_ENTER_OVER_SOURCE_PATH_ID: i64 = 120;

pub const CHARACTER_SELECTION_FULLSCREEN_SOURCE_PATH_ID: i64 = 527;

pub const CHARACTER_SELECTION_FULLSCREEN_OVER_SOURCE_PATH_ID: i64 = 42;

pub const CHARACTER_SELECTION_WINDOWED_SOURCE_PATH_ID: i64 = 628;

pub const CHARACTER_SELECTION_WINDOWED_OVER_SOURCE_PATH_ID: i64 = 461;

pub const CHARACTER_SELECTION_MUSIC_NORMAL_SOURCE_PATH_ID: i64 = 200;

pub const CHARACTER_SELECTION_MUSIC_ON_SOURCE_PATH_ID: i64 = 438;

pub const CHARACTER_SELECTION_RED_NORMAL_SOURCE_PATH_ID: i64 = 282;

pub const CHARACTER_SELECTION_RED_OVER_SOURCE_PATH_ID: i64 = 190;

pub const CHARACTER_SELECTION_BLUE_NORMAL_SOURCE_PATH_ID: i64 = 640;

pub const CHARACTER_SELECTION_BLUE_OVER_SOURCE_PATH_ID: i64 = 309;

pub const CHARACTER_SELECTION_CANCEL_NORMAL_SOURCE_PATH_ID: i64 = 178;

pub const CHARACTER_SELECTION_ROTATE_LEFT_SOURCE_PATH_ID: i64 = 20;

pub const CHARACTER_SELECTION_ROTATE_LEFT_OVER_SOURCE_PATH_ID: i64 = 400;

pub const CHARACTER_SELECTION_ROTATE_RIGHT_SOURCE_PATH_ID: i64 = 15;

pub const CHARACTER_SELECTION_ROTATE_RIGHT_OVER_SOURCE_PATH_ID: i64 = 261;

pub const CHARACTER_SELECTION_DISK_BACK_SOURCE_PATH_ID: i64 = 558;

pub const CHARACTER_SELECTION_DISK_FRONT_SOURCE_PATH_ID: i64 = 43;

pub const CHARACTER_SELECTION_JEFFE_14_PATH_ID: i64 = 903;

pub const CHARACTER_SELECTION_JEFFE_12_PATH_ID: i64 = 953;

pub const CHARACTER_SELECTION_JEFFE_16_PATH_ID: i64 = 1_012;

pub const CHARACTER_SELECTION_CHALET_SMALL_PATH_ID: i64 = 1_018;

pub const CHARACTER_SELECTION_CHALET_REGULAR_PATH_ID: i64 = 1_115;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CharacterSelectionAssetStatus {
    #[default]
    Loading,
    Ready,
    Failed {
        path: String,
    },
}

pub(super) fn update_character_selection_asset_status(
    asset_server: Res<AssetServer>,
    assets: Option<Res<CharacterSelectionAssets>>,
    mut model: ResMut<CharacterSelectionUiModel>,
) {
    let Some(assets) = assets else {
        return;
    };
    model.asset_status = assets.status(&asset_server);
}
