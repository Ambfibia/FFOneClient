
pub const LAUNCHER_UI_ROOT_PATH_ID: i64 = 1_315;

pub const LAUNCHER_UI_TRANSFORM_PATH_ID: i64 = 1_236;

pub const LAUNCHER_UI_GUI_COMPONENT_PATH_ID: i64 = 1_546;

pub const LAUNCHER_UI_LOGIC_COMPONENT_PATH_ID: i64 = 1_547;

pub const LAUNCHER_UI_GUI_SCRIPT_PATH_ID: i64 = 1_076;

pub const LAUNCHER_UI_LOGIC_SCRIPT_PATH_ID: i64 = 922;

pub const LAUNCHER_UI_SKIN_PATH_ID: i64 = 1_387;

pub const LAUNCHER_UI_Z_INDEX: i32 = 9_000;

pub const LAUNCHER_UI_BACKDROP_PATH: &str = "ui/en/launcher/backdrop.png";

pub const LAUNCHER_UI_CROSSHAIR_PATH: &str = "ui/en/launcher/crosshair.png";

pub const LAUNCHER_UI_GAUGE_PATH: &str = "ui/en/launcher/gauge.png";

pub const LAUNCHER_UI_GAUGE_BAR_PATH: &str = "ui/en/launcher/gauge-bar.png";

pub const LAUNCHER_UI_FONT_PATH: &str = "fonts/jeffe.otf";

pub const LAUNCHER_UI_DEFAULT_FONT_PATH_ID: i64 = 903;

pub const LAUNCHER_UI_SMALL_FONT_PATH_ID: i64 = 1_066;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LauncherUiAssetContract {
    pub semantic_name: &'static str,
    pub path: &'static str,
    pub source_owner: &'static str,
    pub source_path_id: i64,
    pub source_name: &'static str,
    pub source_texture_format: &'static str,
    pub source_width: u32,
    pub source_height: u32,
    pub source_payload_sha256: &'static str,
    pub png_size: u64,
    pub png_sha256: &'static str,
}

pub const LAUNCHER_UI_ASSET_CONTRACTS: [LauncherUiAssetContract; 4] = [
    LauncherUiAssetContract {
        semantic_name: "backdrop",
        path: LAUNCHER_UI_BACKDROP_PATH,
        source_owner: "main.unity3d/sharedassets0.assets",
        source_path_id: 9,
        source_name: "launchback",
        source_texture_format: "DXT5",
        source_width: 32,
        source_height: 32,
        source_payload_sha256: "EF7CFA542BAFDF727A566B0BCB2D2DD33445B51B471C405CCD45A4CF96BC30A6",
        png_size: 203,
        png_sha256: "EA691BDCE42D7A9D7898B1B70796C3D9C5ED93D0C6E68E226834AC47A0E63A90",
    },
    LauncherUiAssetContract {
        semantic_name: "crosshair",
        path: LAUNCHER_UI_CROSSHAIR_PATH,
        source_owner: "Tutorial.resourceFile",
        source_path_id: 271,
        source_name: "launchcross",
        source_texture_format: "DXT5",
        source_width: 660,
        source_height: 660,
        source_payload_sha256: "039993FAF3CED719C8653C0CD407AD62711EAA11CC71774C5C7C41C6D78559DF",
        png_size: 151_104,
        png_sha256: "499090CA6789F5F364DC044B1BBF28C722648C2FFE5F3806DDDA485D3E006EB7",
    },
    LauncherUiAssetContract {
        semantic_name: "gauge",
        path: LAUNCHER_UI_GAUGE_PATH,
        source_owner: "main.unity3d/sharedassets0.assets",
        source_path_id: 502,
        source_name: "launchgage",
        source_texture_format: "DXT5",
        source_width: 169,
        source_height: 595,
        source_payload_sha256: "74EF88E2C08F38CE4EECB2B79EBE2BBC49EDF123EB6CA5924DF4FBE013AA7801",
        png_size: 42_730,
        png_sha256: "E87DE0DBB6A6D855C36FDB538E25E535352D1CAA00CBA32E4802FB4254D22E65",
    },
    LauncherUiAssetContract {
        semantic_name: "gauge_bar",
        path: LAUNCHER_UI_GAUGE_BAR_PATH,
        source_owner: "main.unity3d/sharedassets0.assets",
        source_path_id: 454,
        source_name: "launchbar",
        source_texture_format: "DXT5",
        source_width: 87,
        source_height: 31,
        source_payload_sha256: "BEDF04E8B60005538082880145A83244F1DD26898679CA3B9AEAD0BB595947DB",
        png_size: 1_428,
        png_sha256: "F6BD0ED2D43FBF795C27D2DA3E91CCF2E83912F7A6D4E14DED912C3878D37383",
    },
];
