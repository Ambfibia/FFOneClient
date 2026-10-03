use super::*;

pub const RACE_START_SUCCESS_SIZE: usize = 12;

pub const RACE_END_SUCCESS_SIZE: usize = 72;

pub const RACE_REWARD_ITEM_SIZE: usize = 20;

pub const RACE_TICKET_SLOT: i32 = 0;

pub const RACE_START_SUCCESS_ABI: [RaceAbiField; 2] = [
    RaceAbiField {
        clean_name: "iStartTick",
        offset: 0,
        scalar: RaceAbiScalar::U64,
    },
    RaceAbiField {
        clean_name: "iLimitTime",
        offset: 8,
        scalar: RaceAbiScalar::I32,
    },
];

pub const RACE_END_SUCCESS_ABI: [RaceAbiField; 19] = [
    RaceAbiField {
        clean_name: "iEPRaceMode",
        offset: 0,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPRaceTime",
        offset: 4,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPRingCnt",
        offset: 8,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPScore",
        offset: 12,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPRank",
        offset: 16,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPRewardFM",
        offset: 20,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPTopScore",
        offset: 24,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPTopRank",
        offset: 28,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPTopTime",
        offset: 32,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPTopRingCount",
        offset: 36,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iFusionMatter",
        offset: 40,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "RewardItem.sItem.iType",
        offset: 44,
        scalar: RaceAbiScalar::I16,
    },
    RaceAbiField {
        clean_name: "RewardItem.sItem.iID",
        offset: 46,
        scalar: RaceAbiScalar::I16,
    },
    RaceAbiField {
        clean_name: "RewardItem.sItem.iOpt",
        offset: 48,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "RewardItem.sItem.iTimeLimit",
        offset: 52,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "RewardItem.eIL",
        offset: 56,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "RewardItem.iSlotNum",
        offset: 60,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iFatigue",
        offset: 64,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iFatigue_Level",
        offset: 68,
        scalar: RaceAbiScalar::I32,
    },
];

pub const RACE_START_ECOM_ICON: i32 = 13;

pub const RACE_END_ECOM_ICON: i32 = 14;

pub const RACE_ECOM_MANAGER_ID: i32 = 4;

pub const RACE_ECOM_MAKE_FUNCTION: i32 = 12;

pub const RACE_ECOM_DELETE_FUNCTION: i32 = 13;

pub const RACE_FAIL_SYSTEM_MESSAGE_ID: i32 = 159;

pub const RACE_FAIL_SYSTEM_MESSAGE_KEY: &str = "StartRace";

pub const RACE_FATIGUE_MESSAGE_BOX_ID: i32 = 730;

pub const RACE_FATIGUE_MESSAGE_BOX_TYPE: i32 = 9;

pub const RACE_FIRST_USE_REWARD_CONDITION: i32 = 2;

pub const RACE_FIRST_USE_LOW_ENERGY_CONDITION: i32 = 13;

pub const RACE_LOW_ENERGY_WARNING: &str = "Warning: your NanoCom is low on energy. You will begin to collect less Fusion Matter and fewer Taros. Take d break from playing to recharge your NanoCom.";

pub const RACE_EMPTY_ENERGY_WARNING: &str = "Warning: your NanoCom is about to lose its charge. You will no longer be able to collect Fusion Matter and Taros. Take abreak from playing to recharge your NanoCom.";

pub const RACE_RESULT_PIVOT: u8 = 8;

pub const RACE_RESULT_STAR_COUNT: usize = 5;

pub const RACE_RESULT_TEXTURES: [RaceResultTextureContract; 9] = [
    RaceResultTextureContract {
        role: RaceResultTextureRole::Background,
        runtime_path: RACE_RESULT_BACKGROUND_PATH,
        source_path_id: 630,
        source_name: "EPRaceBack",
        source_width: 398,
        source_height: 434,
        sha256: "90BAD3A2D2047F696803B817AAE1B8FF460E1260C02846F7C5910C1CDD60D2CD",
    },
    RaceResultTextureContract {
        role: RaceResultTextureRole::Black,
        runtime_path: RACE_RESULT_BLACK_PATH,
        source_path_id: 350,
        source_name: "black.dds",
        source_width: 16,
        source_height: 16,
        sha256: "8A5258C042C17B715357247D79117B62EE53632572F25F7962BAFA69138EE6CC",
    },
    RaceResultTextureContract {
        role: RaceResultTextureRole::ButtonNormal,
        runtime_path: RACE_RESULT_BUTTON_NORMAL_PATH,
        source_path_id: 411,
        source_name: "ff-button-normal",
        source_width: 11,
        source_height: 26,
        sha256: "6205BA81A517B3861976BCDC68B622F83E04C6A005B416F4CECA9862156ACCA5",
    },
    RaceResultTextureContract {
        role: RaceResultTextureRole::ButtonHover,
        runtime_path: RACE_RESULT_BUTTON_HOVER_PATH,
        source_path_id: 320,
        source_name: "ff-button-hover",
        source_width: 11,
        source_height: 26,
        sha256: "180EADB13B03A8DC5F1A5CCEBEED9193ED932F87715DA6B424EFE20FE5A79BE8",
    },
    RaceResultTextureContract {
        role: RaceResultTextureRole::ButtonActive,
        runtime_path: RACE_RESULT_BUTTON_ACTIVE_PATH,
        source_path_id: 567,
        source_name: "ff-button-active",
        source_width: 13,
        source_height: 31,
        sha256: "AB17A0BD908F6C8F533CEFF18080C486B6AB2E4C4224F8291D8E359F04840847",
    },
    RaceResultTextureContract {
        role: RaceResultTextureRole::FusionMatter,
        runtime_path: RACE_RESULT_FUSION_MATTER_PATH,
        source_path_id: 108,
        source_name: "fusionmatter",
        source_width: 64,
        source_height: 64,
        sha256: "EF537B90C39058BF4C58E70B4B49FEE727EF8D041D0EAADEAA2D0E1B2FCAEF36",
    },
    RaceResultTextureContract {
        role: RaceResultTextureRole::ItemBar,
        runtime_path: RACE_RESULT_ITEM_BAR_PATH,
        source_path_id: 637,
        source_name: "EPRaceItemBar",
        source_width: 367,
        source_height: 76,
        sha256: "31DDFFC7D1C6A2721706901800AFF14B534C50D6F0F349CF57BB744B61E1CA4D",
    },
    RaceResultTextureContract {
        role: RaceResultTextureRole::Star,
        runtime_path: RACE_RESULT_STAR_PATH,
        source_path_id: 164,
        source_name: "EPStar",
        source_width: 19,
        source_height: 17,
        sha256: "5643B0AE96886C4D7C34F07579E51945A7CE051976EE2F363F84E139640B8196",
    },
    RaceResultTextureContract {
        role: RaceResultTextureRole::StarEmpty,
        runtime_path: RACE_RESULT_STAR_EMPTY_PATH,
        source_path_id: 403,
        source_name: "EPStarBack",
        source_width: 19,
        source_height: 17,
        sha256: "1F407BD1500B46BBEC21DCF15295AD6DDC351A82778DDB12CD9CADDBF1715140",
    },
];

pub const RACE_REWARD_ROW_STRIDE: f32 = 85.0;
