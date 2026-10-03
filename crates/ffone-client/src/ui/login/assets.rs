use super::*;

pub const LOGIN_SOURCE_GAME_OBJECT_PATH_ID: i64 = 1_350;

pub const LOGIN_COMPONENT_PATH_ID: i64 = 1_469;

pub const LOGIN_COMPONENT_SCRIPT_PATH_ID: i64 = 1_068;

pub const LOGIN_MODE_COMPONENT_PATH_ID: i64 = 1_471;

pub const LOGIN_MODE_SCRIPT_PATH_ID: i64 = 1_026;

pub const LOGIN_SKIN_PATH_ID: i64 = 1_374;

pub const LOGIN_FALLBACK_BACKGROUND_PATH_ID: i64 = 369;

pub const LOGIN_SOURCE_CREATION_ASSET: &str = "CustomAssetBundle-bd5f53480423447d7bcaed95cb2a96c8";

/// `CnGuiLogin.bgTexture2`, loaded from the clean CharacterCreation asset route.
pub const LOGIN_BACKGROUND_PATH: &str = "ui/en/launcher/login/login_screen_bg_16x10.png";

/// Serialized `CnGuiLogin.bgTexture` path 369 (`load`), shown with ScaleToFit.
pub const LOGIN_FALLBACK_BACKGROUND_PATH: &str = "ui/en/gameplay/loading/load.png";

pub const LOGIN_PANEL_PATH: &str = "ui/en/launcher/login/login_screen_button_bg.png";

pub const LOGIN_TEXT_FIELD_PATH: &str = "ui/en/launcher/login/ff-textfield-normal.png";

/// Approved Cyrillic-capable replacement for clean `JEFFE___16` path 1012.
pub const LOGIN_FONT_PATH: &str = "fonts/jeffe.otf";

/// Approved Cyrillic-capable replacement for clean `ChaletBook-Regular` path 1115.
pub const LOGIN_TEXT_FIELD_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const LOGIN_MUSIC_PATH: &str = "audio/music/00_main_theme.ogg";

// `FusionFallSkin` path 1374 is the authority for every metric below.
// Replacement sizes are the repository-wide validated vector calibration for
// the two serialized fixed-raster Font objects; the exact source line spacing
// remains the Bevy line height.
pub const LOGIN_JEFFE_SOURCE_FONT_PATH_ID: i64 = 1_012;

pub const LOGIN_CHALET_SOURCE_FONT_PATH_ID: i64 = 1_115;

#[derive(Clone, Debug, Resource, PartialEq, Eq)]
pub enum LoginUiAssetStatus {
    Loading { completed: usize, total: usize },
    Ready,
    Failed { path: &'static str, error: String },
}

impl Default for LoginUiAssetStatus {
    fn default() -> Self {
        Self::Loading {
            completed: 0,
            total: 9,
        }
    }
}

pub(super) fn update_login_ui_asset_status(
    asset_server: Res<AssetServer>,
    assets: Res<LoginUiAssets>,
    mut status: ResMut<LoginUiAssetStatus>,
) {
    let handles = [
        (LOGIN_BACKGROUND_PATH, assets.background.id().untyped()),
        (
            LOGIN_FALLBACK_BACKGROUND_PATH,
            assets.fallback_background.id().untyped(),
        ),
        (LOGIN_PANEL_PATH, assets.panel.id().untyped()),
        (LOGIN_BUTTON_PATH, assets.button.id().untyped()),
        (LOGIN_BUTTON_OVER_PATH, assets.button_over.id().untyped()),
        (
            LOGIN_BUTTON_ACTIVE_PATH,
            assets.button_active.id().untyped(),
        ),
        (LOGIN_TEXT_FIELD_PATH, assets.text_field.id().untyped()),
        (LOGIN_FONT_PATH, assets.font.id().untyped()),
        (
            LOGIN_TEXT_FIELD_FONT_PATH,
            assets.text_field_font.id().untyped(),
        ),
    ];
    let total = handles.len();
    let mut completed = 0;
    for (path, id) in handles {
        if asset_server.is_loaded_with_dependencies(id) {
            completed += 1;
            continue;
        }
        if let LoadState::Failed(error) = asset_server.load_state(id) {
            *status = LoginUiAssetStatus::Failed {
                path,
                error: error.to_string(),
            };
            return;
        }
        if let Some(RecursiveDependencyLoadState::Failed(error)) =
            asset_server.get_recursive_dependency_load_state(id)
        {
            *status = LoginUiAssetStatus::Failed {
                path,
                error: error.to_string(),
            };
            return;
        }
    }
    *status = if completed == total {
        LoginUiAssetStatus::Ready
    } else {
        LoginUiAssetStatus::Loading { completed, total }
    };
}
