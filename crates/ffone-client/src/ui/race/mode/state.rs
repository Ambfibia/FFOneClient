use super::*;

pub const RACE_MODE_GAME_MODE_ID: u8 = 16;

pub const RACE_RANK_GAME_MODE_ID: u8 = 17;

pub const RACE_RECORD_MODE: i32 = 1;

pub const RACE_MODE_TIME_KEY: &str = "ui.race.mode.time";

pub const RACE_MODE_PODS_KEY: &str = "ui.race.mode.pods";

pub const RACE_MODE_SCORE_KEY: &str = "ui.race.mode.score";

pub const RACE_MODE_MY_BEST_KEY: &str = "ui.race.mode.my_best_record";

pub const RACE_MODE_REWARD_KEY: &str = "ui.race.mode.reward";

pub const RACE_MODE_ACCEPT_KEY: &str = "ui.race.mode.accept";

pub const RACE_MODE_INVENTORY_FULL_KEY: &str = "ui.race.mode.inventory_full";

pub const RACE_MODE_TIME_VALUE_KEY: &str = "ui.race.mode.time_value";

pub const RACE_MODE_PODS_VALUE_KEY: &str = "ui.race.mode.pods_value";

pub const RACE_MODE_SCORE_VALUE_KEY: &str = "ui.race.mode.score_value";

pub const RACE_MODE_FUSION_MATTER_KEY: &str = "ui.race.mode.fusion_matter_reward";

pub const RACE_MODE_ITEM_NAME_KEY: &str = "ui.race.mode.reward_item_name";

pub const RACE_MODE_ITEM_LEVEL_KEY: &str = "ui.race.mode.reward_item_level";

pub const RACE_MODE_RATING_NONE_KEY: &str = "ui.race.mode.rating.none";

pub const RACE_MODE_RATING_GENIUS_KEY: &str = "ui.race.mode.rating.genius";

pub const RACE_MODE_RATING_AWESOME_KEY: &str = "ui.race.mode.rating.awesome";

pub const RACE_MODE_RATING_GOOD_KEY: &str = "ui.race.mode.rating.good";

pub const RACE_MODE_RATING_NOT_BAD_KEY: &str = "ui.race.mode.rating.not_bad";

pub const RACE_MODE_RATING_BLEH_KEY: &str = "ui.race.mode.rating.bleh";

pub const RACE_MODE_UNRESOLVED_BOUNDARIES: [&str; 7] = [
    "packet serialization and transport are external; request intents preserve clean packet IDs, sizes and field order",
    "world-ring activation and authoritative race status remain external effects",
    "ECom icon manager, cursor, camera sub-target, sound, voice and game-mode exit remain shell-owned effects",
    "reward item name, level and icon are supplied by the native item table presentation boundary",
    "the clean inventory-full packet branch is dormant under the normal GameFrame router and is not synthesized",
    "the serialized StartWindow and OnFailGUI exist but are unreachable from clean OnGUI and are not rendered",
    "no accepted clean-primary normalized RaceMode screenshot exists; deterministic native captures are acceptance evidence only",
];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RacePlayerState {
    pub ring_race_active: bool,
    pub instance_race_mode: i32,
    pub ring_count: i32,
    pub local_start_time: f32,
    pub race_limit_time: i32,
    pub current_ep_id: i32,
    pub top_record: RaceTopRecord,
    pub fatigue: i32,
    pub fatigue_level: i32,
    pub fusion_matter: i32,
    pub cursor_was_locked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaceModeOpenContext {
    pub ecom_type: RaceEcomType,
    pub npc: Option<RaceNpcContext>,
    pub player: RacePlayerState,
    /// Mirrors the clean TableData lookup gate around stars/copy.
    pub current_ep_instance_exists: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceModeReply {
    StartSuccess { start_tick: u64, limit_time: i32 },
    StartFailure { error_code: i32 },
    EndSuccess(RaceEndSuccess),
    EndFailure { error_code: i32 },
    CancelSuccess { temporary: i32 },
    CancelFailure { error_code: i32 },
}

impl RaceModeReply {
    #[must_use]
    pub const fn kind(self) -> RaceRequestKind {
        match self {
            Self::StartSuccess { .. } | Self::StartFailure { .. } => RaceRequestKind::Start,
            Self::EndSuccess(_) | Self::EndFailure { .. } => RaceRequestKind::End,
            Self::CancelSuccess { .. } | Self::CancelFailure { .. } => RaceRequestKind::Cancel,
        }
    }

    #[must_use]
    pub const fn packet_id(self) -> u32 {
        match self {
            Self::StartSuccess { .. } => RACE_START_SUCCESS_PACKET_ID,
            Self::StartFailure { .. } => RACE_START_FAILURE_PACKET_ID,
            Self::EndSuccess(_) => RACE_END_SUCCESS_PACKET_ID,
            Self::EndFailure { .. } => RACE_END_FAILURE_PACKET_ID,
            Self::CancelSuccess { .. } => RACE_CANCEL_SUCCESS_PACKET_ID,
            Self::CancelFailure { .. } => RACE_CANCEL_FAILURE_PACKET_ID,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RaceModeEffect {
    SetCursorLocked(bool),
    EndCameraSubTarget,
    ExitMode,
    ActivateRings,
    DeactivateRings,
    PlaySound(RaceSound),
    PlayNpcRaceStartVoice {
        npc_instance_id: i32,
    },
    Ecom(RaceEcomOperation),
    SystemMessage {
        message_id: i32,
        key: &'static str,
    },
    MessageBox {
        message_id: i32,
        box_type: i32,
        copy: &'static str,
    },
    ReceiveRewardItem {
        inventory_location: i32,
        inventory_slot: i32,
        item: RaceRewardItem,
    },
    RefreshInventory,
    CheckFirstUseCondition(i32),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RaceModeOutput {
    Request(RaceRequestIntent),
    Effect(RaceModeEffect),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceModePhase {
    Hidden,
    AwaitingStart,
    AwaitingEnd,
    AwaitingCancel,
    Result,
    FailSystemMessage,
    DormantRank,
}

impl Default for RaceModePhase {
    fn default() -> Self {
        Self::Hidden
    }
}

impl RaceModePhase {
    #[must_use]
    pub const fn is_busy(self) -> bool {
        matches!(
            self,
            Self::AwaitingStart | Self::AwaitingEnd | Self::AwaitingCancel
        )
    }

    #[must_use]
    pub const fn paints(self) -> bool {
        // Wait for the authoritative result before showing reward rows.
        matches!(self, Self::Result)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceModeUiCommand {
    Accept,
}

#[derive(Debug, Default, Resource)]
pub struct RaceModeUiCommandOutbox(pub(super) VecDeque<RaceModeUiCommand>);

impl RaceModeUiCommandOutbox {
    pub fn push(&mut self, command: RaceModeUiCommand) {
        self.0.push_back(command);
    }

    pub fn pop(&mut self) -> Option<RaceModeUiCommand> {
        self.0.pop_front()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Resource, Clone)]
pub(super) struct RaceModePresentationAssets {
    pub(super) background: Handle<Image>,
    pub(super) black: Handle<Image>,
    pub(super) button_normal: Handle<Image>,
    pub(super) button_hover: Handle<Image>,
    pub(super) button_active: Handle<Image>,
    pub(super) fusion_matter: Handle<Image>,
    pub(super) item_bar: Handle<Image>,
    pub(super) star: Handle<Image>,
    pub(super) star_empty: Handle<Image>,
    pub(super) jeffe: Handle<Font>,
    pub(super) chalet: Handle<Font>,
}

impl RaceModePresentationAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            background: asset_server.load(RACE_RESULT_BACKGROUND_PATH),
            black: asset_server.load(RACE_RESULT_BLACK_PATH),
            button_normal: asset_server.load(RACE_RESULT_BUTTON_NORMAL_PATH),
            button_hover: asset_server.load(RACE_RESULT_BUTTON_HOVER_PATH),
            button_active: asset_server.load(RACE_RESULT_BUTTON_ACTIVE_PATH),
            fusion_matter: asset_server.load(RACE_RESULT_FUSION_MATTER_PATH),
            item_bar: asset_server.load(RACE_RESULT_ITEM_BAR_PATH),
            star: asset_server.load(RACE_RESULT_STAR_PATH),
            star_empty: asset_server.load(RACE_RESULT_STAR_EMPTY_PATH),
            jeffe: asset_server.load(RACE_JEFFE_FONT_PATH),
            chalet: asset_server.load(RACE_CHALET_FONT_PATH),
        }
    }

    pub(super) fn source_images(&self) -> [(&'static str, &Handle<Image>); 9] {
        [
            (RACE_RESULT_BACKGROUND_PATH, &self.background),
            (RACE_RESULT_BLACK_PATH, &self.black),
            (RACE_RESULT_BUTTON_NORMAL_PATH, &self.button_normal),
            (RACE_RESULT_BUTTON_HOVER_PATH, &self.button_hover),
            (RACE_RESULT_BUTTON_ACTIVE_PATH, &self.button_active),
            (RACE_RESULT_FUSION_MATTER_PATH, &self.fusion_matter),
            (RACE_RESULT_ITEM_BAR_PATH, &self.item_bar),
            (RACE_RESULT_STAR_PATH, &self.star),
            (RACE_RESULT_STAR_EMPTY_PATH, &self.star_empty),
        ]
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceModePresentationRoot;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceModePresentationPanel;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceModePresentationStar {
    pub index: usize,
    pub filled: bool,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceModePresentationBar {
    pub(super) bottom: bool,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceModePresentationText(pub(super) RaceModeTextRole);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RaceModeTextRole {
    RankCopy,
    CurrentTime,
    CurrentPods,
    CurrentScore,
    BestTime,
    BestPods,
    BestScore,
    ItemName,
    ItemLevel,
    InventoryFull,
    FusionMatter,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceModeOptionalItemNode;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceModeOptionalItemIcon;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RaceModeFusionMatterNode;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum RaceModePresentationSet {
    Preload,
    Input,
    Bind,
    Visuals,
}

/// Passive, hidden-by-default RaceMode result presentation. Interaction only
/// queues `RaceModeUiCommand`; it never sends a packet or applies a reward.
pub struct RaceModeUiPlugin;

impl Plugin for RaceModeUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<RaceModeModel>()
            .init_resource::<RaceRewardPresentation>()
            .init_resource::<RaceModePresentationInput>()
            .init_resource::<RaceModeUiCommandOutbox>()
            .init_resource::<RaceModePresentationAssetStatus>()
            .configure_sets(
                Update,
                (
                    RaceModePresentationSet::Preload,
                    RaceModePresentationSet::Input,
                    RaceModePresentationSet::Bind,
                    RaceModePresentationSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_race_mode_presentation,
            )
            .add_systems(
                Update,
                (update_race_mode_asset_status.in_set(RaceModePresentationSet::Preload))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (queue_race_mode_controls.in_set(RaceModePresentationSet::Input))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    sync_race_mode_layout,
                    sync_race_mode_content,
                    sync_race_mode_optional_reward,
                )
                    .chain()
                    .in_set(RaceModePresentationSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (sync_race_mode_button_visual.in_set(RaceModePresentationSet::Visuals))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

pub(super) fn queue_race_mode_controls(
    model: Res<RaceModeModel>,
    input: Res<RaceModePresentationInput>,
    buttons: Query<&Interaction, (With<RaceModeAcceptButton>, Changed<Interaction>)>,
    mut commands: ResMut<RaceModeUiCommandOutbox>,
) {
    if !model.controls_enabled(input.system_popup_active) {
        return;
    }
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        commands.push(RaceModeUiCommand::Accept);
    }
}

pub(super) fn sync_race_mode_content(
    model: Res<RaceModeModel>,
    mut texts: Query<
        (&RaceModePresentationText, &mut LocalizedText, &mut Node),
        Without<RaceModePresentationStar>,
    >,
    mut stars: Query<(&RaceModePresentationStar, &mut Node), Without<RaceModePresentationText>>,
) {
    let result = model.result();
    for (role, mut localized_component, mut node) in &mut texts {
        let localized = match role.0 {
            RaceModeTextRole::RankCopy => {
                if model.current_ep_instance_exists() {
                    node.display = Display::Flex;
                    race_rank_localized(result.rank)
                } else {
                    node.display = Display::None;
                    LocalizedText::new(RACE_MODE_RATING_NONE_KEY, "")
                }
            }
            RaceModeTextRole::CurrentTime => race_time_localized(result.race_time_seconds),
            RaceModeTextRole::CurrentPods => race_pods_localized(result.ring_count),
            RaceModeTextRole::CurrentScore => race_score_localized(result.score),
            RaceModeTextRole::BestTime => {
                race_time_localized(model.player().top_record.time_seconds)
            }
            RaceModeTextRole::BestPods => race_pods_localized(model.player().top_record.rings),
            RaceModeTextRole::BestScore => race_score_localized(model.player().top_record.score),
            RaceModeTextRole::FusionMatter => {
                race_fusion_matter_localized(result.reward_fusion_matter)
            }
            RaceModeTextRole::InventoryFull => {
                LocalizedText::new(RACE_MODE_INVENTORY_FULL_KEY, RACE_COPY_INVENTORY_FULL)
            }
            RaceModeTextRole::ItemName | RaceModeTextRole::ItemLevel => continue,
        };
        *localized_component = localized;
    }
    let filled = race_filled_star_count(result.rank);
    for (star, mut node) in &mut stars {
        node.display = if !star.filled
            || (model.current_ep_instance_exists() && (star.index as i32) < filled)
        {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(super) fn sync_race_mode_optional_reward(
    asset_server: Res<AssetServer>,
    model: Res<RaceModeModel>,
    presentation: Res<RaceRewardPresentation>,
    mut optional_nodes: Query<
        &mut Node,
        (
            With<RaceModeOptionalItemNode>,
            Without<RaceModeFusionMatterNode>,
            Without<RaceModePresentationText>,
        ),
    >,
    mut optional_text: Query<
        (&RaceModePresentationText, &mut LocalizedText, &mut Node),
        (
            With<RaceModeOptionalItemNode>,
            Without<RaceModeFusionMatterNode>,
        ),
    >,
    mut optional_icon: Query<&mut ImageNode, With<RaceModeOptionalItemIcon>>,
    mut fusion_nodes: Query<
        &mut Node,
        (
            With<RaceModeFusionMatterNode>,
            Without<RaceModeOptionalItemNode>,
        ),
    >,
    mut fusion_texts: Query<
        (&RaceModePresentationText, &mut Node),
        (
            Without<RaceModeFusionMatterNode>,
            Without<RaceModeOptionalItemNode>,
        ),
    >,
) {
    let item = model.result().reward_item;
    let has_item = item.was_granted();
    let resolved = has_item && presentation.matches(item);
    for mut node in &mut optional_nodes {
        node.display = if has_item {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (role, mut localized_component, mut node) in &mut optional_text {
        let (localized, visible) = match role.0 {
            RaceModeTextRole::ItemName => (
                if resolved {
                    presentation.name_key.as_ref()
                        .map(|key| LocalizedText::new(key.clone(), presentation.name.clone()))
                        .unwrap_or_else(|| race_reward_item_name_localized(&presentation.name))
                } else {
                    race_reward_item_name_localized("")
                },
                resolved,
            ),
            RaceModeTextRole::ItemLevel => (
                race_reward_item_level_localized(if resolved { presentation.level } else { 0 }),
                resolved && item.item_type < 7,
            ),
            RaceModeTextRole::InventoryFull => (
                LocalizedText::new(RACE_MODE_INVENTORY_FULL_KEY, RACE_COPY_INVENTORY_FULL),
                has_item && model.result().inventory_full,
            ),
            _ => (initial_race_mode_localized(role.0), false),
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *localized_component = localized;
    }
    for mut image in &mut optional_icon {
        image.image = if resolved {
            presentation
                .icon_path
                .as_ref()
                .map(|path| asset_server.load(path.clone()))
                .unwrap_or_default()
        } else {
            Handle::default()
        };
        image.image_mode = NodeImageMode::Stretch;
    }
    let shift = if has_item {
        RACE_REWARD_ROW_STRIDE
    } else {
        0.0
    };
    for mut node in &mut fusion_nodes {
        node.top = px(match node.left {
            Val::Px(left) if left == RACE_ITEM_BAR_RECT.x => RACE_ITEM_BAR_RECT.y + shift,
            _ => RACE_ITEM_RECT.y + shift,
        });
    }
    for (role, mut node) in &mut fusion_texts {
        if role.0 == RaceModeTextRole::FusionMatter {
            node.top = px(RACE_ITEM_RECT.y + shift);
        }
    }
}
