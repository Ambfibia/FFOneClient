use super::*;

pub const SERVER_SELECTION_SOURCE_BUILD: &str = "retrobution-20260613";

pub const SERVER_SELECTION_SOURCE_ARCHIVE: &str = "main.unity3d";

pub const SERVER_SELECTION_SOURCE_ARCHIVE_SIZE: u64 = 7_000_415;

pub const SERVER_SELECTION_SOURCE_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const SERVER_SELECTION_MANAGED_MODE_SHA256: &str =
    "1B965D6656CDD9241D4D2AD030D519CB190015291DE9B99A8C88EC17EEAE70CD";

pub const SERVER_SELECTION_MANAGED_GUI_SHA256: &str =
    "B4FBE78836205A73E4579ADB9078E5981D3D7D2C7852E33D1E640D78F4228568";

pub const SERVER_SELECTION_GAME_MODE_SLOT: usize = 26;

pub const SERVER_SELECTION_DEFAULT_US_REACHABLE: bool = false;

pub const SERVER_SELECTION_ROOT_NAME: &str = "ServerSelectionMode";

pub const SERVER_SELECTION_ROOT_INITIALLY_ACTIVE: bool = false;

pub const SERVER_SELECTION_SERVER_COUNT: usize = 1;

pub const SERVER_SELECTION_ALLOCATED_SERVER_RECORDS: usize = 2;

pub const SERVER_SELECTION_SHARD_ARRAY_LEN: usize = 26;

pub const SERVER_SELECTION_FIRST_SHARD: u8 = 1;

pub const SERVER_SELECTION_LAST_SHARD: u8 = 25;

pub const SERVER_SELECTION_REFRESH_SECONDS: f32 = 60.0;

pub const SERVER_SELECTION_STATUS_MESSAGE_ID: i32 = 250;

pub const SERVER_SELECTION_URL: &str = "http://www.fusionfall.co.kr";

pub const SERVER_SELECTION_UI_DEPTH: i32 = 10;

pub const SERVER_SELECTION_JEFFE_16_FONT_SIZE: f32 = 14.0;

pub const SERVER_SELECTION_LABEL_TOP_PADDING: f32 = 3.0;

pub const SERVER_SELECTION_JEFFE_14_FONT_SIZE: f32 = 12.0;

pub const SERVER_SELECTION_PARITY_STATUS: &str = "partial";

pub const SERVER_SELECTION_PARITY_CAVEAT: &str = "The clean fixed-pixel screen, exact native PNGs, one-server/twenty-five-channel state machine, refresh timing, clicks, transitions, source quirks, silent audio contract, and packet boundary are represented. Clean Retrobution fixes localized.local to US (0), whose LOGIN_SUCC route bypasses GameMode 26; only the dormant KOREA (2) branch opens ServerSelection. Production intentionally keeps this root hidden unless an explicit Korea extension owns login-server transport. Browser/ExternalCall execution and a normalized live clean-client golden comparison remain shell integrations.";

pub const SERVER_SELECTION_HEADER_SERVER_CHANNEL_KEY: &str =
    "ui.server_selection.header.server_channel";

pub const SERVER_SELECTION_HEADER_STATUS_KEY: &str = "ui.server_selection.header.status";

pub const SERVER_SELECTION_SERVER_COLLAPSED_KEY: &str = "ui.server_selection.server.collapsed";

pub const SERVER_SELECTION_SERVER_EXPANDED_KEY: &str = "ui.server_selection.server.expanded";

pub const SERVER_SELECTION_CHANNEL_ROW_KEY: &str = "ui.server_selection.channel.row";

pub const SERVER_SELECTION_STATUS_NONE_KEY: &str = "ui.server_selection.status.none";

pub const SERVER_SELECTION_STATUS_CLOSED_KEY: &str = "ui.server_selection.status.closed";

pub const SERVER_SELECTION_STATUS_EMPTY_KEY: &str = "ui.server_selection.status.empty";

pub const SERVER_SELECTION_STATUS_NORMAL_KEY: &str = "ui.server_selection.status.normal";

pub const SERVER_SELECTION_STATUS_BUSY_KEY: &str = "ui.server_selection.status.busy";

pub const SERVER_SELECTION_CONNECT_KEY: &str = "ui.server_selection.connect";

pub const SERVER_SELECTION_MY_ACCOUNT_KEY: &str = "ui.server_selection.my_account";

pub const SERVER_SELECTION_HOMEPAGE_KEY: &str = "ui.server_selection.homepage";

pub const SERVER_SELECTION_QUIT_KEY: &str = "ui.server_selection.quit";

pub const SERVER_SELECTION_EXTERNAL_CALL_FUNCTION: &str = "HomePage";

pub const SERVER_SELECTION_EXTERNAL_CALL_ARGUMENT: &str = "gameObject";

pub const SERVER_SELECTION_EXTERNAL_CALL_ARGUMENT_COUNT: usize = 1;

pub const SERVER_SELECTION_IMAGE_PATHS: [&str; 14] = [
    SERVER_SELECTION_BACKGROUND_PATH,
    SERVER_SELECTION_PANEL_PATH,
    SERVER_SELECTION_INNER_PATH,
    SERVER_SELECTION_ROW_HOVER_PATH,
    SERVER_SELECTION_ROW_SELECTED_PATH,
    SERVER_SELECTION_BUTTON_NORMAL_PATH,
    SERVER_SELECTION_BUTTON_HOVER_PATH,
    SERVER_SELECTION_BUTTON_ACTIVE_PATH,
    SERVER_SELECTION_RED_NORMAL_PATH,
    SERVER_SELECTION_RED_HOVER_PATH,
    SERVER_SELECTION_SCROLL_TRACK_PATH,
    SERVER_SELECTION_SCROLL_THUMB_PATH,
    SERVER_SELECTION_SCROLL_UP_PATH,
    SERVER_SELECTION_SCROLL_DOWN_PATH,
];

pub const SERVER_SELECTION_FIRST_CHANNEL_TOP: f32 = 29.0;

pub const SERVER_SELECTION_CHANNEL_STRIDE: f32 = 18.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ServerSelectionPopulation {
    Closed = 0,
    Empty = 1,
    Normal = 2,
    Busy = 3,
}

impl ServerSelectionPopulation {
    #[must_use]
    pub const fn from_raw(raw: i32) -> Option<Self> {
        match raw {
            0 => Some(Self::Closed),
            1 => Some(Self::Empty),
            2 => Some(Self::Normal),
            3 => Some(Self::Busy),
            _ => None,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Closed => SERVER_SELECTION_COPY_STATUS_CLOSED,
            Self::Empty => SERVER_SELECTION_COPY_STATUS_EMPTY,
            Self::Normal => SERVER_SELECTION_COPY_STATUS_NORMAL,
            Self::Busy => SERVER_SELECTION_COPY_STATUS_BUSY,
        }
    }

    #[must_use]
    pub const fn localization_key(self) -> &'static str {
        match self {
            Self::Closed => SERVER_SELECTION_STATUS_CLOSED_KEY,
            Self::Empty => SERVER_SELECTION_STATUS_EMPTY_KEY,
            Self::Normal => SERVER_SELECTION_STATUS_NORMAL_KEY,
            Self::Busy => SERVER_SELECTION_STATUS_BUSY_KEY,
        }
    }

    #[must_use]
    pub fn localized(self) -> LocalizedText {
        LocalizedText::new(self.localization_key(), self.label())
    }

    #[must_use]
    pub fn color(self) -> Color {
        match self {
            Self::Closed => Color::srgb(0.5, 0.5, 0.5),
            Self::Empty => Color::srgb(0.0, 1.0, 0.0),
            Self::Normal => Color::srgb(1.0, 0.921_568_6, 0.015_686_275),
            Self::Busy => Color::srgb(1.0, 0.0, 0.0),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerSelectionLoginSnapshot {
    pub character_count: i8,
    pub slot_number: i8,
    pub payment_flag: i8,
    pub server_time: u64,
    pub account_id: String,
    pub open_beta_flag: i32,
}

impl Default for ServerSelectionLoginSnapshot {
    fn default() -> Self {
        Self {
            character_count: 1,
            slot_number: 0,
            payment_flag: 0,
            server_time: 0,
            account_id: String::new(),
            open_beta_flag: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ServerSelectionInit {
    pub login: ServerSelectionLoginSnapshot,
    pub auto_login: bool,
    pub warp_shard: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ServerSelectionPhase {
    #[default]
    Hidden,
    Visible,
    AwaitingQuitGate,
    Transitioning,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServerSelectionUiEffect {
    RequestShardList {
        packet_id: u32,
        packet_size: usize,
    },
    SystemMessage {
        message_id: i32,
    },
    SetGameMode(usize),
    InitCharacterSelection {
        login: ServerSelectionLoginSnapshot,
        account_id: String,
        shard: i32,
        zero: i32,
        auto_login: bool,
        warp_shard: bool,
    },
    InitCharacterCreation {
        mode: usize,
        login: ServerSelectionLoginSnapshot,
        account_id: String,
        shard: i32,
        zero: i32,
        inverted_auto_login: bool,
        warp_shard: bool,
    },
    ApplySoundOptions,
    InitDexterNameCreate {
        character_id: i64,
        event_scene_name: &'static str,
        slot_number: i32,
    },
    LegacyEvent {
        manager: u8,
        function: u8,
        element_function: Option<u8>,
    },
    SendServerSelect {
        server_number: i8,
        packet_id: u32,
        packet_size: usize,
    },
    OpenUrl {
        source: ServerSelectionWebSource,
        url: &'static str,
    },
    RequestQuitGate {
        manager: u8,
        function: u8,
    },
    SetSystemFocusOut(bool),
    SkipWorldUpdate(bool),
    SetSaveResolution(bool),
    ExternalCallHomePage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerSelectionWebSource {
    MyAccount,
    Homepage,
}

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct ServerSelectionUiOutbox(pub(super) VecDeque<ServerSelectionUiEffect>);

impl ServerSelectionUiOutbox {
    pub fn push(&mut self, effect: ServerSelectionUiEffect) {
        self.0.push_back(effect);
    }

    pub fn pop_front(&mut self) -> Option<ServerSelectionUiEffect> {
        self.0.pop_front()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = ServerSelectionUiEffect> + '_ {
        self.0.drain(..)
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Resource)]
pub struct ServerSelectionUiAssets {
    pub background: Handle<Image>,
    pub panel: Handle<Image>,
    pub inner: Handle<Image>,
    pub row_hover: Handle<Image>,
    pub row_selected: Handle<Image>,
    pub button_normal: Handle<Image>,
    pub button_hover: Handle<Image>,
    pub button_active: Handle<Image>,
    pub red_normal: Handle<Image>,
    pub red_hover: Handle<Image>,
    pub scroll_track: Handle<Image>,
    pub scroll_thumb: Handle<Image>,
    pub scroll_up: Handle<Image>,
    pub scroll_down: Handle<Image>,
    pub font: Handle<Font>,
}

impl ServerSelectionUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            background: asset_server.load(SERVER_SELECTION_BACKGROUND_PATH),
            panel: asset_server.load(SERVER_SELECTION_PANEL_PATH),
            inner: asset_server.load(SERVER_SELECTION_INNER_PATH),
            row_hover: asset_server.load(SERVER_SELECTION_ROW_HOVER_PATH),
            row_selected: asset_server.load(SERVER_SELECTION_ROW_SELECTED_PATH),
            button_normal: asset_server.load(SERVER_SELECTION_BUTTON_NORMAL_PATH),
            button_hover: asset_server.load(SERVER_SELECTION_BUTTON_HOVER_PATH),
            button_active: asset_server.load(SERVER_SELECTION_BUTTON_ACTIVE_PATH),
            red_normal: asset_server.load(SERVER_SELECTION_RED_NORMAL_PATH),
            red_hover: asset_server.load(SERVER_SELECTION_RED_HOVER_PATH),
            scroll_track: asset_server.load(SERVER_SELECTION_SCROLL_TRACK_PATH),
            scroll_thumb: asset_server.load(SERVER_SELECTION_SCROLL_THUMB_PATH),
            scroll_up: asset_server.load(SERVER_SELECTION_SCROLL_UP_PATH),
            scroll_down: asset_server.load(SERVER_SELECTION_SCROLL_DOWN_PATH),
            font: asset_server.load(SERVER_SELECTION_FONT_PATH),
        }
    }

    #[must_use]
    pub fn image_handles(&self) -> [&Handle<Image>; 14] {
        [
            &self.background,
            &self.panel,
            &self.inner,
            &self.row_hover,
            &self.row_selected,
            &self.button_normal,
            &self.button_hover,
            &self.button_active,
            &self.red_normal,
            &self.red_hover,
            &self.scroll_track,
            &self.scroll_thumb,
            &self.scroll_up,
            &self.scroll_down,
        ]
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum ServerSelectionUiElement {
    Root,
    Background,
    Panel,
    ScrollContent,
    ChannelRow(u8),
    ScrollTrack,
    ScrollThumb,
    ScrollUp,
    ScrollDown,
    AccountButton,
    HomepageButton,
    QuitButton,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum ServerSelectionUiControl {
    ServerToggle,
    Shard(u8),
    Connect,
    MyAccount,
    Homepage,
    Quit,
    ScrollUp,
    ScrollDown,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum ServerSelectionUiTextRole {
    ServerHeading,
    ServerStatus,
    ShardStatus(u8),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum ServerSelectionUiSet {
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct ServerSelectionUiPlugin;

impl Plugin for ServerSelectionUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ServerSelectionUiModel>()
            .init_resource::<ServerSelectionUiOutbox>()
            .add_systems(Startup, spawn_server_selection_ui)
            .configure_sets(
                Update,
                (
                    ServerSelectionUiSet::Interaction,
                    ServerSelectionUiSet::Bind,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    advance_server_selection_refresh,
                    collect_server_selection_input,
                    handle_server_selection_controls,
                )
                    .in_set(ServerSelectionUiSet::Interaction),
            )
            .add_systems(
                Update,
                (
                    bind_server_selection_layout,
                    bind_server_selection_text,
                    bind_server_selection_button_images,
                )
                    .chain()
                    .in_set(ServerSelectionUiSet::Bind)
                    .before(LocalizationSet::Apply),
            );
    }
}

pub(super) fn advance_server_selection_refresh(
    time: Res<Time>,
    mut model: ResMut<ServerSelectionUiModel>,
    mut outbox: ResMut<ServerSelectionUiOutbox>,
) {
    model.tick(time.delta_secs(), &mut outbox);
}

pub(super) fn handle_server_selection_controls(
    controls: Query<(&Interaction, &ServerSelectionUiControl), Changed<Interaction>>,
    mut model: ResMut<ServerSelectionUiModel>,
    mut outbox: ResMut<ServerSelectionUiOutbox>,
) {
    for (interaction, control) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *control {
            ServerSelectionUiControl::ServerToggle => model.toggle_server(),
            ServerSelectionUiControl::Shard(shard) => {
                model.select_shard(1, i32::from(shard));
            }
            ServerSelectionUiControl::Connect => {
                model.connect(&mut outbox);
            }
            ServerSelectionUiControl::MyAccount => {
                model.open_web(ServerSelectionWebSource::MyAccount, &mut outbox);
            }
            ServerSelectionUiControl::Homepage => {
                model.open_web(ServerSelectionWebSource::Homepage, &mut outbox);
            }
            ServerSelectionUiControl::Quit => {
                model.request_quit(&mut outbox);
            }
            ServerSelectionUiControl::ScrollUp => {
                model.scroll_by(-SERVER_SELECTION_SCROLL_STEP);
            }
            ServerSelectionUiControl::ScrollDown => {
                model.scroll_by(SERVER_SELECTION_SCROLL_STEP);
            }
        }
    }
}

pub(super) fn bind_server_selection_text(
    model: Res<ServerSelectionUiModel>,
    mut texts: Query<(
        &ServerSelectionUiTextRole,
        &mut TextColor,
        &mut LocalizedText,
    )>,
) {
    for (role, mut color, mut localized_component) in &mut texts {
        let population = match *role {
            ServerSelectionUiTextRole::ServerHeading => {
                *localized_component =
                    server_selection_server_heading_localized(model.server_expanded, 1);
                color.0 = Color::WHITE;
                continue;
            }
            ServerSelectionUiTextRole::ServerStatus => model.server_population(),
            ServerSelectionUiTextRole::ShardStatus(shard) => model.shard_population(shard),
        };
        if let Some(population) = population {
            *localized_component = population.localized();
            color.0 = population.color();
        } else {
            // Clean branch has no fallback label for an unknown raw status.
            *localized_component = LocalizedText::new(SERVER_SELECTION_STATUS_NONE_KEY, "");
            color.0 = Color::WHITE;
        }
    }
}
