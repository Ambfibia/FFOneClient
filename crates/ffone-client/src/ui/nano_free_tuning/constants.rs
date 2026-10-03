use super::*;

pub const NANO_TUNE_SUCCESS_SIZE: usize = 168;

pub const NANO_TUNE_FAILURE_SIZE: usize = 8;

pub const NANO_TUNE_ITEM_SLOT_COUNT: usize = 10;

pub const NANO_FREE_TUNING_EFFECT_ID: i32 = 705;

pub const NANO_FREE_TUNING_BULLET_TYPES: [i32; 2] = [76, 77];

pub const NANO_FREE_TUNING_ECOM_ICON: i32 = 5;

pub const NANO_FREE_TUNING_EFFECT_DELAY_SECONDS: f32 = 0.5;

pub const NANO_FREE_TUNING_PROJECTILE_DELAY_SECONDS: f32 = 1.2;

pub const NANO_FREE_TUNING_REVEAL_DELAY_SECONDS: f32 = 0.8;

pub const NANO_FREE_TUNING_CAMERA_APPROACH_SECONDS: f32 = 0.3;

pub const NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS: f32 = 2.0;

pub const NANO_FREE_TUNING_IDLE_SECONDS: f32 = 3.0;

pub const NANO_FREE_TUNING_RESULT_HIDE_SECONDS: f32 = 1.0;

pub const NANO_FREE_TUNING_IDLE_RANDOM_EXCLUSIVE_MAX: i32 = 2;

pub const NANO_FREE_TUNING_BLACK_BAR_RATIO: f32 = 0.156_739_82;

pub const NANO_FREE_TUNING_UI_DEPTH: i32 = 9;

pub const NANO_FREE_TUNING_PANEL_PIVOT: u8 = 4;

pub const NANO_FREE_TUNING_JEFFE_16_FONT_SIZE: f32 = 14.0;

pub const NANO_FREE_TUNING_JEFFE_14_FONT_SIZE: f32 = 12.0;

pub const NANO_FREE_TUNING_JEFFE_08_FONT_SIZE: f32 = 8.0;

/// The approved JEFFE replacement preserves the measured clean top bearing at
/// these sizes. Keep every reached GUIStyle calibration independent: changing
/// one source Font must never silently move another style vertically.
pub const NANO_FREE_TUNING_BIGBLUE_REPLACEMENT_Y_OFFSET: f32 = 0.0;

pub const NANO_FREE_TUNING_BIGYELLOW_REPLACEMENT_Y_OFFSET: f32 = 0.0;

pub const NANO_FREE_TUNING_LIGHT_BLUE_REPLACEMENT_Y_OFFSET: f32 = 0.0;

pub const NANO_FREE_TUNING_YELLOW_SMALL_REPLACEMENT_Y_OFFSET: f32 = 0.0;

pub const NANO_FREE_TUNING_BLUE_REPLACEMENT_Y_OFFSET: f32 = 0.0;

pub const NANO_FREE_TUNING_PRIMARY_MAIN_SIZE: u64 = 7_000_415;

pub const NANO_FREE_TUNING_PRIMARY_MAIN_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const NANO_FREE_TUNING_MANAGED_ASSEMBLY_SIZE: u64 = 1_517_568;

pub const NANO_FREE_TUNING_MANAGED_ASSEMBLY_SHA256: &str =
    "33D6F70216B1C7BA05BCC0F270FBA97E767B129159755AF4C8835922E60ACADB";

pub const NANO_TUNE_SUCCESS_ABI: [NanoFreeTuningAbiField; 5] = [
    NanoFreeTuningAbiField {
        clean_name: "iNanoID",
        offset: 0,
        scalar: NanoFreeTuningAbiScalar::I16,
        count: 1,
    },
    NanoFreeTuningAbiField {
        clean_name: "iSkillID",
        offset: 2,
        scalar: NanoFreeTuningAbiScalar::I16,
        count: 1,
    },
    NanoFreeTuningAbiField {
        clean_name: "iPC_FusionMatter",
        offset: 4,
        scalar: NanoFreeTuningAbiScalar::I32,
        count: 1,
    },
    NanoFreeTuningAbiField {
        clean_name: "aiItemSlotNum",
        offset: 8,
        scalar: NanoFreeTuningAbiScalar::I32,
        count: NANO_TUNE_ITEM_SLOT_COUNT,
    },
    NanoFreeTuningAbiField {
        clean_name: "aItem",
        offset: 48,
        scalar: NanoFreeTuningAbiScalar::ItemBase,
        count: NANO_TUNE_ITEM_SLOT_COUNT,
    },
];

pub const NANO_TUNE_FAILURE_ABI: [NanoFreeTuningAbiField; 2] = [
    NanoFreeTuningAbiField {
        clean_name: "iPC_ID",
        offset: 0,
        scalar: NanoFreeTuningAbiScalar::I32,
        count: 1,
    },
    NanoFreeTuningAbiField {
        clean_name: "iErrorCode",
        offset: 4,
        scalar: NanoFreeTuningAbiScalar::I32,
        count: 1,
    },
];
