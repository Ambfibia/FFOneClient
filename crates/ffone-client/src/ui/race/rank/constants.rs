
pub const RACE_RANK_CAMERA_NEAR: f32 = 0.2;

pub const RACE_RANK_CAMERA_FAR: f32 = 5.0;

pub const RACE_RANK_CAMERA_FOV: f32 = 45.0;

pub const RACE_RANK_CAMERA_CULLING_LAYER: u32 = 1_024;

pub const RACE_RANK_CAMERA_DISTANCE: f32 = 0.5;

pub const RACE_RANK_CAMERA_YAW_DEGREES: f32 = -20.0;

pub const RACE_RANK_CAMERA_TARGET_NAME: &str = "Bip01 Neck";

pub const RACE_RANK_PACKETS_ARE_DORMANT: bool = true;

pub const RACE_RANK_LEGACY_FALLBACK_URL: &str =
    "http://www.fusionfall.com/tools/scripts/client_get.php";

pub const RACE_RANK_RETROBUTION_URL: &str = "http://api.retrobution.xyz/getranks";

pub const RACE_RANK_HTTP_METHOD: &str = "POST";

pub const RACE_RANK_HTTP_FIELD_PCUID: &str = "PCUID";

pub const RACE_RANK_HTTP_FIELD_EP_ID: &str = "EP_ID";

pub const RACE_RANK_HTTP_POLL_SECONDS: f32 = 0.1;

pub const RACE_RANK_LOCATION_COUNT: usize = 32;

pub const RACE_RANK_PAGE_SIZE: usize = 5;

pub const RACE_RANK_PERIOD_COUNT: usize = 4;

pub const RACE_RANK_TOP_COUNT: usize = 10;

pub const RACE_RANK_TITLE_KEY: &str = "ui.race.rank.title";

pub const RACE_RANK_LOCATIONS_KEY: &str = "ui.race.rank.locations";

pub const RACE_RANK_LOCATION_LABEL_KEY: &str = "ui.race.rank.location_label";

pub const RACE_RANK_HEADER_RANK_KEY: &str = "ui.race.rank.header.rank";

pub const RACE_RANK_HEADER_PLAYER_KEY: &str = "ui.race.rank.header.player";

pub const RACE_RANK_HEADER_TOP_SCORE_KEY: &str = "ui.race.rank.header.top_score";

pub const RACE_RANK_NO_SCORE_KEY: &str = "ui.race.rank.no_score";

pub const RACE_RANK_PERIOD_TODAY_KEY: &str = "ui.race.rank.period.today";

pub const RACE_RANK_PERIOD_WEEK_KEY: &str = "ui.race.rank.period.week";

pub const RACE_RANK_PERIOD_MONTH_KEY: &str = "ui.race.rank.period.month";

pub const RACE_RANK_PERIOD_ALL_TIME_KEY: &str = "ui.race.rank.period.all_time";

pub const RACE_RANK_BEST_TODAY_KEY: &str = "ui.race.rank.best.today";

pub const RACE_RANK_BEST_WEEK_KEY: &str = "ui.race.rank.best.week";

pub const RACE_RANK_BEST_MONTH_KEY: &str = "ui.race.rank.best.month";

pub const RACE_RANK_BEST_ALL_TIME_KEY: &str = "ui.race.rank.best.all_time";

pub const RACE_RANK_TOP_TODAY_KEY: &str = "ui.race.rank.top.today";

pub const RACE_RANK_TOP_WEEK_KEY: &str = "ui.race.rank.top.week";

pub const RACE_RANK_TOP_MONTH_KEY: &str = "ui.race.rank.top.month";

pub const RACE_RANK_TOP_ALL_TIME_KEY: &str = "ui.race.rank.top.all_time";

pub const RACE_RANK_NPC_NAME_KEY: &str = "ui.race.rank.npc_name";

pub const RACE_RANK_LOCATION_NAME_KEY: &str = "ui.race.rank.location_name";

pub const RACE_RANK_AREA_NAME_KEY: &str = "ui.race.rank.area_name";

pub const RACE_RANK_ROW_NAME_KEY: &str = "ui.race.rank.location_row_name";

pub const RACE_RANK_ROW_AREA_KEY: &str = "ui.race.rank.location_row_area";

pub const RACE_RANK_PAGE_KEY: &str = "ui.race.rank.page";

pub const RACE_RANK_VALUE_KEY: &str = "ui.race.rank.score.rank_value";

pub const RACE_RANK_PLAYER_VALUE_KEY: &str = "ui.race.rank.score.player_value";

pub const RACE_RANK_SCORE_VALUE_KEY: &str = "ui.race.rank.score.score_value";

pub const RACE_RANK_ROW_STRIDE: f32 = 86.0;

pub const RACE_RANK_UNRESOLVED_BOUNDARIES: [&str; 6] = [
    "HTTP transport and endpoint configuration are external; the model emits exact ordered PCUID/EP_ID form intents",
    "the declared EP-rank packet replies contain only private bytes and are deliberately not decoded or synthesized",
    "the live cnCharRenderCamera NPC portrait is represented by a typed 200x150 camera slot owned by gameplay presentation",
    "mode exit and legacy asset garbage collection remain typed shell effects",
    "the help button is visually interactive but its clean return value is ignored",
    "no accepted clean-primary normalized RaceRankMode screenshot exists; deterministic native captures are acceptance evidence only",
];
