use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankDeclaredAbiField {
    pub clean_name: &'static str,
    pub offset: usize,
    pub byte_width: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct RaceRankLocation {
    pub ep_id: i32,
    pub sort_index: i32,
    pub instance_name_id: i32,
    pub name: String,
    pub zone_x: i32,
    pub zone_y: i32,
    pub dong_name: String,
    pub area_name: String,
    pub score_max: i32,
    pub big_image: String,
    pub small_image: String,
}

#[repr(usize)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum RaceRankPeriod {
    #[default]
    Today = 0,
    Week = 1,
    Month = 2,
    AllTime = 3,
}

impl RaceRankPeriod {
    pub const ALL: [Self; 4] = [Self::Today, Self::Week, Self::Month, Self::AllTime];

    #[must_use]
    pub const fn tab_copy(self) -> &'static str {
        match self {
            Self::Today => "today",
            Self::Week => "this week",
            Self::Month => "this month",
            Self::AllTime => "all time",
        }
    }

    #[must_use]
    pub const fn best_copy(self) -> &'static str {
        match self {
            Self::Today => "MY BEST SCORE TODAY:",
            Self::Week => "MY BEST SCORE THIS WEEK:",
            Self::Month => "MY BEST SCORE THIS MONTH:",
            Self::AllTime => "MY ALL TIME BEST SCORE:",
        }
    }

    #[must_use]
    pub const fn top_copy(self) -> &'static str {
        match self {
            Self::Today => " TODAY'S TOP 10:",
            Self::Week => " THIS WEEK'S TOP 10:",
            Self::Month => " THIS MONTH'S TOP 10:",
            Self::AllTime => " ALL TIME TOP 10:",
        }
    }

    #[must_use]
    pub fn tab_localized(self) -> LocalizedText {
        let key = match self {
            Self::Today => RACE_RANK_PERIOD_TODAY_KEY,
            Self::Week => RACE_RANK_PERIOD_WEEK_KEY,
            Self::Month => RACE_RANK_PERIOD_MONTH_KEY,
            Self::AllTime => RACE_RANK_PERIOD_ALL_TIME_KEY,
        };
        LocalizedText::new(key, self.tab_copy())
    }

    #[must_use]
    pub fn best_localized(self) -> LocalizedText {
        let key = match self {
            Self::Today => RACE_RANK_BEST_TODAY_KEY,
            Self::Week => RACE_RANK_BEST_WEEK_KEY,
            Self::Month => RACE_RANK_BEST_MONTH_KEY,
            Self::AllTime => RACE_RANK_BEST_ALL_TIME_KEY,
        };
        LocalizedText::new(key, self.best_copy())
    }

    #[must_use]
    pub fn top_localized(self) -> LocalizedText {
        let key = match self {
            Self::Today => RACE_RANK_TOP_TODAY_KEY,
            Self::Week => RACE_RANK_TOP_WEEK_KEY,
            Self::Month => RACE_RANK_TOP_MONTH_KEY,
            Self::AllTime => RACE_RANK_TOP_ALL_TIME_KEY,
        };
        LocalizedText::new(key, self.top_copy())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RaceRankScore {
    pub pcuid: i32,
    pub rank: i32,
    pub player: String,
    pub score: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RaceRankScores {
    pub personal: [Option<RaceRankScore>; RACE_RANK_PERIOD_COUNT],
    pub top: [[Option<RaceRankScore>; RACE_RANK_TOP_COUNT]; RACE_RANK_PERIOD_COUNT],
}

impl Default for RaceRankScores {
    fn default() -> Self {
        Self {
            personal: std::array::from_fn(|_| None),
            top: std::array::from_fn(|_| std::array::from_fn(|_| None)),
        }
    }
}

impl RaceRankScores {
    pub(super) fn clear_all(&mut self) {
        *self = Self::default();
    }

    /// Exact page-button quirk: only `topScores[period, 0]` and the personal
    /// row are cleared. Rendering stops at the first zero entry, hiding any
    /// stale entries 1..9 without actually clearing them.
    pub(super) fn clear_page_visible_heads(&mut self) {
        for period in 0..RACE_RANK_PERIOD_COUNT {
            self.personal[period] = None;
            self.top[period][0] = None;
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaceRankEffect {
    BindNpcCamera { target_instance_id: i32 },
    ReleaseNpcCamera,
    FreeLegacyAssets,
    ExitMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaceRankOutput {
    Http(RaceRankHttpIntent),
    Effect(RaceRankEffect),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RaceRankOpenContext {
    pub pcuid: i32,
    pub current_ep_id: i32,
    pub npc_name: String,
    pub npc_target_instance_id: i32,
    pub rank_url: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RaceRankPhase {
    #[default]
    Hidden,
    Loading,
    Browsing,
}

impl RaceRankPhase {
    #[must_use]
    pub const fn visible(self) -> bool {
        !matches!(self, Self::Hidden)
    }
}

#[derive(Resource, Clone)]
pub(super) struct RaceRankPresentationAssets {
    pub(super) images: BTreeMap<String, Handle<Image>>,
    pub(super) jeffe: Handle<Font>,
    pub(super) chalet: Handle<Font>,
}

impl RaceRankPresentationAssets {
    pub(super) fn load(asset_server: &AssetServer, catalog: &RaceRankCatalog) -> Self {
        let paths =
            RACE_RANK_STATIC_ASSET_PATHS
                .into_iter()
                .map(str::to_owned)
                .chain(catalog.locations().iter().flat_map(|location| {
                    [location.big_image.clone(), location.small_image.clone()]
                }));
        Self {
            images: paths
                .map(|path| {
                    let handle = asset_server.load(path.clone());
                    (path, handle)
                })
                .collect(),
            jeffe: asset_server.load(RACE_JEFFE_FONT_PATH),
            chalet: asset_server.load(RACE_CHALET_FONT_PATH),
        }
    }

    pub(super) fn image(&self, path: &str) -> Handle<Image> {
        self.images.get(path).cloned().unwrap_or_default()
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankPresentationRoot;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankPresentationLeft;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankPresentationRight;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankPresentationNpcCameraSlot;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankPresentationLocationRow(pub usize);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankPresentationScoreRow(pub usize);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceRankPresentationBackdrop;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceRankPresentationShell;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceRankPresentationControl(pub(super) RaceRankControl);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RaceRankControl {
    Location(usize),
    Previous,
    Next,
    Period(RaceRankPeriod),
    Close,
    HelpIgnored,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceRankPresentationRowPart {
    pub(super) slot: usize,
    pub(super) part: RaceRankRowPart,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RaceRankRowPart {
    Icon,
    Name,
    Area,
    Selection,
    Outline,
    Point,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceRankPresentationPersonalText(pub(super) RaceRankScoreColumn);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceRankPresentationTopText {
    pub(super) row: usize,
    pub(super) column: RaceRankScoreColumn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RaceRankScoreColumn {
    Rank,
    Player,
    Score,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceRankPresentationTabVisual(pub(super) RaceRankPeriod);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum RaceRankPresentationSet {
    Preload,
    Animation,
    Input,
    Bind,
    Visuals,
}

/// Passive, hidden-by-default RaceRankMode presentation. It queues typed
/// commands and never performs HTTP, packet, camera, or mode-exit work.
pub struct RaceRankUiPlugin;

impl Plugin for RaceRankUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<RaceRankCatalog>()
            .init_resource::<RaceRankModel>()
            .init_resource::<RaceRankPresentationInput>()
            .init_resource::<RaceRankUiCommandOutbox>()
            .init_resource::<RaceRankPresentationAssetStatus>()
            .configure_sets(
                Update,
                (
                    RaceRankPresentationSet::Preload,
                    RaceRankPresentationSet::Animation,
                    RaceRankPresentationSet::Input,
                    RaceRankPresentationSet::Bind,
                    RaceRankPresentationSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_race_rank_presentation,
            )
            .add_systems(
                Update,
                (update_race_rank_asset_status.in_set(RaceRankPresentationSet::Preload))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (advance_race_rank_slide.in_set(RaceRankPresentationSet::Animation))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (queue_race_rank_controls.in_set(RaceRankPresentationSet::Input))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    sync_race_rank_layout,
                    sync_race_rank_rows,
                    sync_race_rank_selected,
                    sync_race_rank_scores,
                    sync_race_rank_tabs,
                )
                    .chain()
                    .in_set(RaceRankPresentationSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (sync_race_rank_control_visuals.in_set(RaceRankPresentationSet::Visuals))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
