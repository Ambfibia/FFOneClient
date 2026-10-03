use super::*;

pub const CHARACTER_CREATION_SKIN_PATH_ID: i64 = 1_382;

pub const CHARACTER_CREATION_BSD_PATH_ID: i64 = 1_378;

pub const CHARACTER_CREATION_APPEARANCE_GAME_OBJECT_PATH_ID: i64 = 1_281;

pub const CHARACTER_CREATION_APPEARANCE_GUI_COMPONENT_PATH_ID: i64 = 1_538;

pub const CHARACTER_CREATION_APPEARANCE_MODE_COMPONENT_PATH_ID: i64 = 1_539;

pub const CHARACTER_CREATION_NAME_GAME_OBJECT_PATH_ID: i64 = 1_358;

pub const CHARACTER_CREATION_NAME_GUI_COMPONENT_PATH_ID: i64 = 1_570;

pub const CHARACTER_CREATION_NAME_MODE_COMPONENT_PATH_ID: i64 = 1_571;

pub const CHARACTER_CREATION_NAME_SUBMIT_COMPONENT_PATH_ID: i64 = 1_572;

pub const CHARACTER_CREATION_SIMPLE_CAMERA_COMPONENT_PATH_ID: i64 = 1_541;

pub const CHARACTER_CREATION_JEFFE_14_PATH_ID: i64 = 903;

pub const CHARACTER_CREATION_JEFFE_16_PATH_ID: i64 = 1_012;

pub const CHARACTER_CREATION_CHALET_SMALL_PATH_ID: i64 = 1_018;

pub const CHARACTER_CREATION_CHALET_REGULAR_PATH_ID: i64 = 1_115;

pub const CHARACTER_CREATION_TEXT_FIELD_PATH: &str = "ui/en/launcher/login/ff-textfield-normal.png";

pub const CHARACTER_CREATION_TEXT_FIELD_SOURCE_PATH_ID: i64 = 11_024;

pub const CHARACTER_CREATION_BACKGROUND_PATH: &str =
    "ui/en/character/creation/background/CharCreationBG.png";

pub const CHARACTER_CREATION_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const CHARACTER_CREATION_DISPLAY_FONT_PATH: &str = "fonts/jeffe.otf";

pub const CHARACTER_CREATION_MUSIC_PATH: &str = "audio/music/charactercreation_loop.ogg";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CharacterCreationAssetStatus {
    #[default]
    Loading,
    Ready,
    Failed {
        path: &'static str,
    },
}

pub(super) fn update_character_creation_asset_status(
    asset_server: Res<AssetServer>,
    assets: Option<Res<CharacterCreationAssets>>,
    mut model: ResMut<CharacterCreationUiModel>,
) {
    let Some(assets) = assets else {
        return;
    };
    let mut loading = false;
    macro_rules! observe_asset {
        ($path:expr, $handle:expr) => {
            match asset_server.load_state($handle.id()) {
                LoadState::Failed(_) => {
                    model.asset_status = CharacterCreationAssetStatus::Failed { path: $path };
                    return;
                }
                LoadState::Loaded => {}
                LoadState::NotLoaded | LoadState::Loading => loading = true,
            }
        };
    }
    for spec in CHARACTER_CREATION_IMAGE_SPECS
        .iter()
        .chain(CHARACTER_CREATION_SHARED_IMAGE_SPECS.iter())
        .chain(CHARACTER_CREATION_ENGINE_IMAGE_SPECS.iter())
    {
        let handle = assets.image(spec.path);
        observe_asset!(spec.path, handle);
    }
    for (cue, handle) in &assets.sounds {
        observe_asset!(cue.path(), handle);
    }
    for (path, handle) in CHARACTER_CREATION_BUTTON_SOUND_PATHS
        .iter()
        .copied()
        .zip(assets.button_sounds.iter())
    {
        observe_asset!(path, handle);
    }
    observe_asset!(CHARACTER_CREATION_MUSIC_PATH, assets.music);
    observe_asset!(CHARACTER_CREATION_FONT_PATH, assets.font);
    observe_asset!(CHARACTER_CREATION_DISPLAY_FONT_PATH, assets.display_font);
    model.asset_status = if loading {
        CharacterCreationAssetStatus::Loading
    } else {
        CharacterCreationAssetStatus::Ready
    };
}
