use super::*;

pub const RACE_MODE_GAME_OBJECT_PATH_ID: i64 = 1_306;

pub const RACE_MODE_TRANSFORM_PATH_ID: i64 = 1_239;

pub const RACE_MODE_COMPONENT_PATH_ID: i64 = 1_596;

pub const RACE_MODE_SCRIPT_PATH_ID: i64 = 1_073;

pub const RACE_MODE_SKIN_PATH_ID: i64 = 1_374;

pub const RACE_RESULT_UI_Z_INDEX: i32 = 8_120;

pub const RACE_RESULT_BACKGROUND_PATH: &str = "ui/en/race/result/background.png";

pub const RACE_RESULT_BLACK_PATH: &str = "ui/en/race/result/black.png";

pub const RACE_RESULT_FUSION_MATTER_PATH: &str = "ui/en/race/result/fusion_matter.png";

pub const RACE_RESULT_ITEM_BAR_PATH: &str = "ui/en/race/result/item_bar.png";

pub const RACE_RESULT_STAR_PATH: &str = "ui/en/race/result/star.png";

pub const RACE_RESULT_STAR_EMPTY_PATH: &str = "ui/en/race/result/star_empty.png";

#[derive(Clone, Debug, Default, Resource, PartialEq, Eq)]
pub enum RaceModePresentationAssetStatus {
    #[default]
    Loading,
    Ready,
    Failed {
        asset_path: &'static str,
    },
}

pub(super) fn update_race_mode_asset_status(
    asset_server: Res<AssetServer>,
    assets: Res<RaceModePresentationAssets>,
    mut status: ResMut<RaceModePresentationAssetStatus>,
) {
    let failed = assets
        .source_images()
        .into_iter()
        .find_map(|(path, handle)| {
            matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)).then_some(path)
        })
        .or_else(|| {
            matches!(
                asset_server.load_state(assets.jeffe.id()),
                LoadState::Failed(_)
            )
            .then_some(RACE_JEFFE_FONT_PATH)
        })
        .or_else(|| {
            matches!(
                asset_server.load_state(assets.chalet.id()),
                LoadState::Failed(_)
            )
            .then_some(RACE_CHALET_FONT_PATH)
        });
    *status = if let Some(asset_path) = failed {
        RaceModePresentationAssetStatus::Failed { asset_path }
    } else if assets
        .source_images()
        .into_iter()
        .all(|(_, handle)| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(assets.jeffe.id()),
            LoadState::Loaded
        )
        && matches!(
            asset_server.load_state(assets.chalet.id()),
            LoadState::Loaded
        )
    {
        RaceModePresentationAssetStatus::Ready
    } else {
        RaceModePresentationAssetStatus::Loading
    };
}
