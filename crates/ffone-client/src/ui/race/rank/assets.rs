use super::*;

pub const RACE_RANK_GAME_OBJECT_PATH_ID: i64 = 1_320;

pub const RACE_RANK_TRANSFORM_PATH_ID: i64 = 1_240;

pub const RACE_RANK_COMPONENT_PATH_ID: i64 = 1_496;

pub const RACE_RANK_SCRIPT_PATH_ID: i64 = 978;

pub const RACE_RANK_SKIN_PATH_ID: i64 = 1_390;

pub const RACE_RANK_CAMERA_PATH_ID: i64 = 1_497;

pub const RACE_RANK_CAMERA_GAME_OBJECT_PATH_ID: i64 = 1_318;

pub const RACE_RANK_CAMERA_TRANSFORM_PATH_ID: i64 = 1_242;

pub const RACE_RANK_CAMERA_COMPONENT_PATH_ID: i64 = 1_498;

pub const RACE_RANK_CAMERA_SCRIPT_PATH_ID: i64 = 1_154;

pub const RACE_RANK_CATALOG_SCHEMA: &str = "ffone.race-rank-locations.v1";

pub const RACE_RANK_CATALOG_PATH: &str = "ui/en/race/rank/locations.json";

pub const RACE_RANK_CATALOG_SHA256: &str =
    "9BEAAE20C635CD1F1D0CA4A2167B9BF052E3C3478738566D4B603CE0525D38E0";

pub const RACE_RANK_UI_Z_INDEX: i32 = 8_110;

pub const RACE_RANK_BACKDROP_PATH: &str = "ui/en/race/rank/skin/panel_backdrop.png";

pub const RACE_RANK_SHELL_PATH: &str = "ui/en/race/rank/skin/shell.png";

pub const RACE_RANK_TITLE_PATH: &str = "ui/en/race/rank/skin/title.png";

pub const RACE_RANK_LOCATION_BACK_PATH: &str = "ui/en/race/rank/skin/location_back.png";

pub const RACE_RANK_BLACK_PATH: &str = "ui/en/race/rank/skin/black.png";

pub const RACE_RANK_LOCATION_ROW_PATH: &str = "ui/en/race/rank/skin/location_row.png";

pub const RACE_RANK_LOCATION_SELECTED_PATH: &str = "ui/en/race/rank/skin/location_row_selected.png";

pub const RACE_RANK_SELECTION_OUTLINE_PATH: &str = "ui/en/race/rank/skin/selection_outline.png";

pub const RACE_RANK_SELECTION_POINT_PATH: &str = "ui/en/race/rank/skin/selection_point.png";

pub const RACE_RANK_RIGHT_BACK_PATH: &str = "ui/en/race/rank/skin/right_back.png";

pub const RACE_RANK_ROW_PATH: &str = "ui/en/race/rank/skin/row.png";

pub const RACE_RANK_ROW_SELECTED_PATH: &str = "ui/en/race/rank/skin/row_selected.png";

pub const RACE_RANK_MY_BACK_PATH: &str = "ui/en/race/rank/skin/my_rank_back.png";

pub const RACE_RANK_SKY_PATH: &str = "ui/en/race/rank/skin/sky.png";

pub const RACE_RANK_HIGHLIGHT_PATH: &str = "ui/en/race/rank/skin/highlight.png";

pub const RACE_RANK_TAB_BAR_PATH: &str = "ui/en/race/rank/skin/tab_bar.png";

pub const RACE_RANK_TAB_BOX_PATH: &str = "ui/en/race/rank/skin/tab_box.png";

pub const RACE_RANK_TODAY_ACTIVE_PATH: &str = "ui/en/race/rank/skin/today_active.png";

pub const RACE_RANK_TODAY_IDLE_PATH: &str = "ui/en/race/rank/skin/today_idle.png";

pub const RACE_RANK_WIDE_ACTIVE_PATH: &str = "ui/en/race/rank/skin/wide_active.png";

pub const RACE_RANK_WIDE_IDLE_PATH: &str = "ui/en/race/rank/skin/wide_idle.png";

pub const RACE_RANK_PREVIOUS_PATH: &str = "ui/en/race/rank/controls/previous.png";

pub const RACE_RANK_NEXT_PATH: &str = "ui/en/race/rank/controls/next.png";

pub const RACE_RANK_LEFT_ARROW_PATH: &str = "ui/en/race/rank/controls/arrow_left.png";

pub const RACE_RANK_RIGHT_ARROW_PATH: &str = "ui/en/race/rank/controls/arrow_right.png";

pub const RACE_RANK_CLOSE_PATH: &str = "ui/en/race/rank/controls/close.png";

pub const RACE_RANK_HELP_PATH: &str = "ui/en/race/rank/controls/help.png";

pub const RACE_RANK_STATIC_ASSET_PATHS: [&str; 37] = [
    RACE_RANK_BACKDROP_PATH,
    RACE_RANK_SHELL_PATH,
    RACE_RANK_TITLE_PATH,
    RACE_RANK_LOCATION_BACK_PATH,
    RACE_RANK_BLACK_PATH,
    RACE_RANK_LOCATION_ROW_PATH,
    RACE_RANK_LOCATION_SELECTED_PATH,
    RACE_RANK_ICON_FRAME_PATH,
    RACE_RANK_SELECTION_OUTLINE_PATH,
    RACE_RANK_SELECTION_POINT_PATH,
    RACE_RANK_RIGHT_BACK_PATH,
    RACE_RANK_ROW_PATH,
    RACE_RANK_ROW_SELECTED_PATH,
    RACE_RANK_MY_BACK_PATH,
    RACE_RANK_SKY_PATH,
    RACE_RANK_HIGHLIGHT_PATH,
    RACE_RANK_TAB_BAR_PATH,
    RACE_RANK_TAB_BOX_PATH,
    RACE_RANK_TAB_TEXTURE_PATH,
    RACE_RANK_TODAY_ACTIVE_PATH,
    RACE_RANK_TODAY_IDLE_PATH,
    RACE_RANK_TODAY_HOVER_PATH,
    RACE_RANK_WIDE_ACTIVE_PATH,
    RACE_RANK_WIDE_IDLE_PATH,
    RACE_RANK_WIDE_HOVER_PATH,
    RACE_RANK_PREVIOUS_PATH,
    RACE_RANK_PREVIOUS_HOVER_PATH,
    RACE_RANK_NEXT_PATH,
    RACE_RANK_NEXT_HOVER_PATH,
    RACE_RANK_LEFT_ARROW_PATH,
    RACE_RANK_LEFT_ARROW_HOVER_PATH,
    RACE_RANK_RIGHT_ARROW_PATH,
    RACE_RANK_RIGHT_ARROW_HOVER_PATH,
    RACE_RANK_CLOSE_PATH,
    RACE_RANK_CLOSE_HOVER_PATH,
    RACE_RANK_HELP_PATH,
    RACE_RANK_HELP_HOVER_PATH,
];

pub const RACE_RANK_INVENTORY_SKIN_PATH_ID: i64 = 1_366;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct RaceRankCatalogSource {
    pub build: String,
    pub table_archive: String,
    pub table_archive_sha256: String,
    pub serialized_asset: String,
    pub instance_table_path_id: i64,
    pub world_name_path_id: i64,
    pub ordering: Vec<String>,
    pub area_copy: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct RaceRankCatalogFile {
    pub(super) schema: String,
    pub(super) source: RaceRankCatalogSource,
    pub(super) locations: Vec<RaceRankLocation>,
}

#[derive(Clone, Debug, Resource, Eq, PartialEq)]
pub struct RaceRankCatalog {
    pub(super) schema: String,
    pub(super) source: RaceRankCatalogSource,
    pub(super) locations: Vec<RaceRankLocation>,
}

impl Default for RaceRankCatalog {
    fn default() -> Self {
        Self::embedded().expect("embedded clean race-rank catalog must validate")
    }
}

impl RaceRankCatalog {
    pub fn embedded() -> Result<Self, RaceRankCatalogError> {
        let file: RaceRankCatalogFile = serde_json::from_str(include_str!(
            "../../../../../../assets/game/ui/en/race/rank/locations.json"
        ))
        .map_err(|error| RaceRankCatalogError::Json(error.to_string()))?;
        let catalog = Self {
            schema: file.schema,
            source: file.source,
            locations: file.locations,
        };
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn from_locations_for_test(locations: Vec<RaceRankLocation>) -> Self {
        Self {
            schema: RACE_RANK_CATALOG_SCHEMA.to_owned(),
            source: RaceRankCatalogSource {
                build: "test".to_owned(),
                table_archive: String::new(),
                table_archive_sha256: String::new(),
                serialized_asset: String::new(),
                instance_table_path_id: 7,
                world_name_path_id: 8,
                ordering: Vec::new(),
                area_copy: "ZoneName".to_owned(),
            },
            locations,
        }
    }

    pub(super) fn validate(&self) -> Result<(), RaceRankCatalogError> {
        if self.schema != RACE_RANK_CATALOG_SCHEMA {
            return Err(RaceRankCatalogError::WrongSchema(self.schema.clone()));
        }
        if self.source.build != "retrobution-20260613" {
            return Err(RaceRankCatalogError::WrongSourceBuild(
                self.source.build.clone(),
            ));
        }
        if self.locations.len() != RACE_RANK_LOCATION_COUNT {
            return Err(RaceRankCatalogError::WrongLocationCount(
                self.locations.len(),
            ));
        }
        let mut ids = HashSet::new();
        let mut previous: Option<&RaceRankLocation> = None;
        for location in &self.locations {
            if location.ep_id <= 0 {
                return Err(RaceRankCatalogError::NonPositiveEpId(location.ep_id));
            }
            if !ids.insert(location.ep_id) {
                return Err(RaceRankCatalogError::DuplicateEpId(location.ep_id));
            }
            if let Some(previous) = previous {
                if (location.sort_index, location.ep_id) < (previous.sort_index, previous.ep_id) {
                    return Err(RaceRankCatalogError::WrongOrdering {
                        previous_ep: previous.ep_id,
                        current_ep: location.ep_id,
                    });
                }
            }
            for path in [&location.big_image, &location.small_image] {
                let expected = format!("ep_{:02}_", location.ep_id);
                if !path.starts_with("ui/en/race/rank/locations/")
                    || !path.contains(&expected)
                    || !path.ends_with(".png")
                {
                    return Err(RaceRankCatalogError::WrongImagePath {
                        ep_id: location.ep_id,
                        path: path.clone(),
                    });
                }
            }
            previous = Some(location);
        }
        Ok(())
    }

    #[must_use]
    pub fn source(&self) -> &RaceRankCatalogSource {
        &self.source
    }

    #[must_use]
    pub fn locations(&self) -> &[RaceRankLocation] {
        &self.locations
    }

    #[must_use]
    pub fn location_by_ep(&self, ep_id: i32) -> Option<(usize, &RaceRankLocation)> {
        self.locations
            .iter()
            .enumerate()
            .find(|(_, location)| location.ep_id == ep_id)
    }
}

#[derive(Clone, Debug, Default, Resource, Eq, PartialEq)]
pub enum RaceRankPresentationAssetStatus {
    #[default]
    Loading,
    Ready,
    Failed {
        asset_path: String,
    },
}

pub(super) fn update_race_rank_asset_status(
    asset_server: Res<AssetServer>,
    assets: Res<RaceRankPresentationAssets>,
    mut status: ResMut<RaceRankPresentationAssetStatus>,
) {
    if let Some((path, _)) = assets
        .images
        .iter()
        .find(|(_, handle)| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
    {
        *status = RaceRankPresentationAssetStatus::Failed {
            asset_path: path.clone(),
        };
        return;
    }
    for (path, handle) in [
        (RACE_JEFFE_FONT_PATH, &assets.jeffe),
        (RACE_CHALET_FONT_PATH, &assets.chalet),
    ] {
        if matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)) {
            *status = RaceRankPresentationAssetStatus::Failed {
                asset_path: path.to_owned(),
            };
            return;
        }
    }
    *status = if assets
        .images
        .values()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(assets.jeffe.id()),
            LoadState::Loaded
        )
        && matches!(
            asset_server.load_state(assets.chalet.id()),
            LoadState::Loaded
        ) {
        RaceRankPresentationAssetStatus::Ready
    } else {
        RaceRankPresentationAssetStatus::Loading
    };
}
