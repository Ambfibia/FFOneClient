use super::*;

pub const LAUNCHER_UI_SOURCE_BUILD: &str = "retrobution-20260613";

pub const LAUNCHER_UI_ROOT_NAME: &str = "LuncherMode";

pub const LAUNCHER_UI_ROOT_INITIALLY_ACTIVE: bool = false;

pub const LAUNCHER_UI_CAMERA_LAUNCH_OFFSET: f32 = 5.0;

pub const LAUNCHER_UI_DEPTH: i32 = 9;

pub const LAUNCHER_UI_MAIN_SIZE: u64 = 7_000_415;

pub const LAUNCHER_UI_MAIN_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const LAUNCHER_UI_TUTORIAL_SIZE: u64 = 27_003_826;

pub const LAUNCHER_UI_TUTORIAL_SHA256: &str =
    "49A684FF4236848D0B882A5D725FFBE99D5CFB5D2090DC8705350E97DD3FD024";

pub const LAUNCHER_UI_CHARACTER_CREATION_SIZE: u64 = 8_974_798;

pub const LAUNCHER_UI_CHARACTER_CREATION_SHA256: &str =
    "78785925E716027DE4BEC897C402C352BB411B7D627DE1783E6FBDA8BF34E59E";

pub const LAUNCHER_UI_MANAGED_GUI_SHA256: &str =
    "A0D191F2DCE2D4E00F1B49BD8A596EB727281E077FA135CAB1AF6554759A5399";

pub const LAUNCHER_UI_MANAGED_LOGIC_SHA256: &str =
    "352368F13C26F8CEBDE79BA582F6E11E186B7907E430EC2549C12CB41F158EF7";

pub const LAUNCHER_UI_PARITY_CAVEAT: &str = "The clean in-client Launcher screen, power/aim/exit state machine, native PNGs, legacy audio routes, deterministic GPU acceptance harness, production world-trigger discovery, camera/renderer/movement application, and launch packet transport are represented. Remaining integration work is the clean ConfigurableInput fire mapping, live HP/system-popup synchronization, RequestEscapeCloseGate resolution, combat-icon/name-visibility effects, centralized GameFrame link-mode transitions, and a normalized live clean-client golden comparison. CnGuiEUALA belongs to LoginMode and is intentionally not claimed by this slice.";

pub const LAUNCHER_UI_IMAGE_PATHS: [&str; 4] = [
    LAUNCHER_UI_BACKDROP_PATH,
    LAUNCHER_UI_CROSSHAIR_PATH,
    LAUNCHER_UI_GAUGE_PATH,
    LAUNCHER_UI_GAUGE_BAR_PATH,
];

pub const LAUNCHER_UI_POWER_LABEL_KEY: &str = "POWER";

pub const LAUNCHER_UI_TIP_LABEL_KEY: &str = "HOLD SPACE BAR TO SET POWER AND RELEASE TO FIRE";

pub const LAUNCHER_UI_DEFAULT_FONT_SIZE: f32 = 14.0;

pub const LAUNCHER_UI_SMALL_FONT_SIZE: f32 = 6.0;

pub const LAUNCHER_UI_CROSS_SIZE: f32 = 660.0;

pub const LAUNCHER_UI_GAUGE_AREA_X: f32 = 42.0;

pub const LAUNCHER_UI_GAUGE_AREA_Y: f32 = 30.0;
