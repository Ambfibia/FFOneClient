use super::*;

pub const ENCHANT_SOURCE_BUILD: &str = "retrobution-20260613";

pub const ENCHANT_SOURCE_MAIN_ARCHIVE: &str = "main.unity3d";

pub const ENCHANT_SOURCE_MAIN_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const ENCHANT_SKIN_CUSTOM_STYLE_COUNT: usize = 31;

pub const ENCHANT_GUI_MANAGED_SOURCE_SHA256: &str =
    "EF546C88A834322152A9D7E5ECC3CE4D327F9F90B991546D73AD4699F77F21AE";

pub const ENCHANT_SUCCESS_MANAGED_SOURCE_SHA256: &str =
    "1DA2BE120614736B3076D0A137A4075B7F6B4DEE306168F754A44B3F0A7607FB";

pub const ENCHANT_FAILURE_MANAGED_SOURCE_SHA256: &str =
    "F713E437ABF851003D1611778746C7887DE951477120988F8D3E90A7F473B56B";

pub const ENCHANT_ITEM_BASE_MANAGED_SOURCE_SHA256: &str =
    "00CBB45AF59D8E43DE276259D506A8E0A65A50C1D4FF4F0014EA8FC7C64A5AEF";

pub const ENCHANT_PC_STUFF_MANAGED_SOURCE_SHA256: &str =
    "03653A1002F9DD0C6C58FCB1C4D6172A66D3ABF0CF1DB34F3970CAA3D64EA70F";

pub const ENCHANT_EQUIPMENT_PANEL_MANAGED_SOURCE_SHA256: &str =
    "A0EA72A50035A48E3B0C5762BFE67BD50872A8E3A657EBDE5CCE978ABF797C60";

pub const ENCHANT_NPC_TYPE_0104: i32 = 27;

pub const ENCHANT_POPUP_SOURCE_SLOT_0104: i32 = 21;

pub const ENCHANT_WAIT_SECONDS_0104: f32 = 4.0;

/// `Panel_Equip.DoEquipPanelType` forces both drag and drop flags off while
/// `InventoryManagerScript.InvenMode` is Enchant.
pub const ENCHANT_EQUIPMENT_STRIP_INTERACTIVE_0104: bool = false;

pub const ENCHANT_HELP_ITEM_1_ID_0104: i16 = 141;

pub const ENCHANT_HELP_ITEM_2_ID_0104: i16 = 142;

pub const ENCHANT_RECIPE_ROW_COUNT_0104: usize = 11;

pub const ENCHANT_REDEEM_WINDOW_ID_0104: i32 = 99;

pub const ENCHANT_REDEEM_CODE_MAX_CHARS_0104: usize = 32;

pub const ENCHANT_REDEEM_CODE_MIN_CHARS_0104: usize = 3;

pub const ENCHANT_REDEEM_SHADE_ALPHA_0104: f32 = 0.75;

pub const ENCHANT_ITEM_BASE_SIZE_0104: usize = 12;

pub const ENCHANT_FAILURE_TARGET_SLOT_OFFSET_0104: usize = 4;

pub const ENCHANT_FAILURE_WEAPON_SLOT_OFFSET_0104: usize = 8;

pub const ENCHANT_FAILURE_ARMOR_SLOT_OFFSET_0104: usize = 12;

pub const ENCHANT_FAILURE_CASH_SLOT_1_OFFSET_0104: usize = 16;

pub const ENCHANT_FAILURE_CASH_SLOT_2_OFFSET_0104: usize = 20;

pub const ENCHANT_SUCCESS_TARGET_SLOT_OFFSET_0104: usize = 0;

pub const ENCHANT_SUCCESS_TARGET_ITEM_OFFSET_0104: usize = 4;

pub const ENCHANT_SUCCESS_WEAPON_SLOT_OFFSET_0104: usize = 16;

pub const ENCHANT_SUCCESS_WEAPON_ITEM_OFFSET_0104: usize = 20;

pub const ENCHANT_SUCCESS_ARMOR_SLOT_OFFSET_0104: usize = 32;

pub const ENCHANT_SUCCESS_ARMOR_ITEM_OFFSET_0104: usize = 36;

pub const ENCHANT_SUCCESS_CASH_SLOT_1_OFFSET_0104: usize = 48;

pub const ENCHANT_SUCCESS_CASH_SLOT_2_OFFSET_0104: usize = 52;

pub const ENCHANT_SUCCESS_TAROS_OFFSET_0104: usize = 56;

pub const ENCHANT_SUCCESS_FLAG_OFFSET_0104: usize = 60;

const _: [(); ENCHANT_ITEM_BASE_SIZE_0104] = [(); ItemBase0104::SIZE];

pub const ENCHANT_MESSAGE_CONFIRM_0104: i32 = 254;

pub const ENCHANT_MESSAGE_NOT_ENOUGH_TAROS_0104: i32 = 256;

pub const ENCHANT_MESSAGE_FAILURE_0104: i32 = 259;

pub const ENCHANT_MESSAGE_DELETE_ITEM_0104: i32 = 153;

pub const ENCHANT_MESSAGE_DISASSEMBLE_ITEM_0104: i32 = 257;

pub const ENCHANT_MESSAGE_CONFIRM_TEXT_0104: &str = "ATTEMPTING TO COMBINE!\nIf successful, the two items you combine will be replaced with a brand-new item. If unsuccessful, you will keep your original items but still spend Taros. You can always make another combination attempt later.";

pub const ENCHANT_MESSAGE_NOT_ENOUGH_TAROS_TEXT_0104: &str =
    "NOT ENOUGH TAROS \nYou do not have enough Taros to make this combination attempt.";

// Message 259 is shared with inventory equip rejection.  The clean enchant
// callback nevertheless passes its level-loss argument to this exact row.
pub const ENCHANT_MESSAGE_FAILURE_TEXT_0104: &str = "CANNOT EQUIP\nYou cannot equip items here. Exit and go to MY STUFF to change your equipped items.";

pub const ENCHANT_RECIPE_TABLE_SHA256: &str =
    "EA0BC1B274979D819982EFC5651E44EF26645D1B29D529DFEC4069B96D306031";

pub const ENCHANT_SOURCE_TEXTURES_0104: [EnchantTextureEvidence0104; 12] = [
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::Panel,
        source_path_id: 381,
        source_name: "EnchantBG",
        runtime_path: ENCHANT_PANEL_PATH,
        width: 481,
        height: 600,
        png_bytes: 41_005,
        png_sha256: "0FB8ECD060EE81A1463D6FF279BCEF413C7B3CE05F297AEE70BF51E1DD00FFBD",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::RawCover,
        source_path_id: 183,
        source_name: "EnchantCover",
        runtime_path: ENCHANT_RAW_COVER_PATH,
        width: 479,
        height: 237,
        png_bytes: 7_604,
        png_sha256: "CD4426A7FB21685EA146A1D04BAA0C1BFFC4490326002C26D5A64D58301CC1AC",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::ItemCover,
        source_path_id: 615,
        source_name: "EnchantItemCover",
        runtime_path: ENCHANT_ITEM_COVER_PATH,
        width: 385,
        height: 68,
        png_bytes: 2_387,
        png_sha256: "64B7A533F5EDC15EA52D5893BEF56D1959729725E939157DAC968F0D8B38B22B",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::QuantityCover,
        source_path_id: 163,
        source_name: "EnchantRawCover",
        runtime_path: ENCHANT_QUANTITY_COVER_PATH,
        width: 147,
        height: 68,
        png_bytes: 1_798,
        png_sha256: "58F2E049C8050344A1FD117F26655015C0090BC464DC14CD1807D3D0CAE72704",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::XMark,
        source_path_id: 275,
        source_name: "EnchantXmark",
        runtime_path: ENCHANT_X_MARK_PATH,
        width: 65,
        height: 65,
        png_bytes: 1_711,
        png_sha256: "9B7F92D1A324BB5F6A96F39C7AC6C509EDC4DAED2F6306E934232066F155199A",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::LevelBadge,
        source_path_id: 632,
        source_name: "EnchantLevel",
        runtime_path: ENCHANT_LEVEL_BADGE_PATH,
        width: 25,
        height: 12,
        png_bytes: 212,
        png_sha256: "DD4D6DE2FE2E23F5892F56EF1075F628F828FAB65526F7EAEC8C91AF296B6FD0",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::Waiting,
        source_path_id: 564,
        source_name: "EnchantWaiting",
        runtime_path: ENCHANT_WAITING_PATH,
        width: 434,
        height: 435,
        png_bytes: 8_710,
        png_sha256: "31131418EB3EC5802EB943234E6C6B534C1DD6BD09FE1F8F28178CD4F5327890",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::WaitProgress,
        source_path_id: 662,
        source_name: "EnchantWaitProgress",
        runtime_path: ENCHANT_WAIT_PROGRESS_PATH,
        width: 414,
        height: 13,
        png_bytes: 600,
        png_sha256: "11DA8CB28FD84A6DB23EC2E828068F0A0985AE129EA7659011B3C42B3E83B277",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::DexlabsBanner,
        source_path_id: 451,
        source_name: "dexlabsbut",
        runtime_path: ENCHANT_DEXLABS_PATH,
        width: 203,
        height: 67,
        png_bytes: 11_819,
        png_sha256: "FD6757A9872AB53250D23A56F83E398E1FC76A27F3BC4F2D7003B1805689F49D",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::TarosCounter,
        source_path_id: 326,
        source_name: "Taros",
        runtime_path: ENCHANT_TAROS_COUNTER_PATH,
        width: 149,
        height: 33,
        png_bytes: 3_201,
        png_sha256: "E280860ECE99E9D3EAA5C2F33CD0A3FB633AB4D0301492AFA4DB18B4D9A0ECD0",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::BoostIcon,
        source_path_id: 87,
        source_name: "BoostIcon",
        runtime_path: ENCHANT_BOOST_ICON_PATH,
        width: 62,
        height: 62,
        png_bytes: 1_580,
        png_sha256: "B56576EB911194BB87BF6716C8F59864513626AAA71D964A1DFF0B3548B77E19",
    },
    EnchantTextureEvidence0104 {
        role: EnchantTextureRole0104::PotionIcon,
        source_path_id: 99,
        source_name: "PostionIcon",
        runtime_path: ENCHANT_POTION_ICON_PATH,
        width: 62,
        height: 62,
        png_bytes: 2_231,
        png_sha256: "08B9E922C7D1BC774404E76590F783259F20B323A45A640AEF61933376E7BFD2",
    },
];

pub const ENCHANT_LEGACY_STATES_0104: [EnchantLegacyStateKind0104; 13] = [
    EnchantLegacyStateKind0104::UnreachableCashWarning,
    EnchantLegacyStateKind0104::ArmorPreviewUsesWeaponTexture,
    EnchantLegacyStateKind0104::HelperTwoPreviewChecksHelperOneTexture,
    EnchantLegacyStateKind0104::HelperTwoIdleDragCarriesTarget,
    EnchantLegacyStateKind0104::IntendedHelperIdsRejected,
    EnchantLegacyStateKind0104::OtherGeneralItemsAcceptedAsHelpers,
    EnchantLegacyStateKind0104::FailureMutatesMinusOneMaterialSlots,
    EnchantLegacyStateKind0104::UnsupportedSuccessFlagLeavesSendLock,
    EnchantLegacyStateKind0104::EnchantMoreLeavesSlotObjects,
    EnchantLegacyStateKind0104::MaterialLabelsUseEquipmentTable,
    EnchantLegacyStateKind0104::DeclaredScrollDelegateNeverAssigned,
    EnchantLegacyStateKind0104::KoreanHammerControlHiddenByCleanConfiguration,
    EnchantLegacyStateKind0104::SuccessEquipCheckBranchesUseSameColors,
];

pub const ENCHANT_FAILURE_ABI_0104: [EnchantAbiField0104; 6] = [
    EnchantAbiField0104 {
        name: "iErrorCode",
        offset: 0,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iEnchantItemSlot",
        offset: 4,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iWeaponMaterialItemSlot",
        offset: 8,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iDefenceMaterialItemSlot",
        offset: 12,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iCashItemSlot1",
        offset: 16,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iCashItemSlot2",
        offset: 20,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
];

pub const ENCHANT_SUCCESS_ABI_0104: [EnchantAbiField0104; 10] = [
    EnchantAbiField0104 {
        name: "iEnchantItemSlot",
        offset: 0,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "sEnchantItem",
        offset: 4,
        scalar: EnchantAbiScalar0104::ItemBase,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iWeaponMaterialItemSlot",
        offset: 16,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "sWeaponMaterialItem",
        offset: 20,
        scalar: EnchantAbiScalar0104::ItemBase,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iDefenceMaterialItemSlot",
        offset: 32,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "sDefenceMaterialItem",
        offset: 36,
        scalar: EnchantAbiScalar0104::ItemBase,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iCashItemSlot1",
        offset: 48,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iCashItemSlot2",
        offset: 52,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iCandy",
        offset: 56,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
    EnchantAbiField0104 {
        name: "iSuccessFlag",
        offset: 60,
        scalar: EnchantAbiScalar0104::I32,
        count: 1,
    },
];

const _: [(); ENCHANT_REDEEM_REQUEST_PACKET_SIZE_0104] = [(); FreeChatRequest0104::SIZE];

pub const CLEAN_ENCHANT_RECIPES_0104: [EnchantRecipe0104; ENCHANT_RECIPE_ROW_COUNT_0104] = [
    EnchantRecipe0104 {
        enchant_grade: 0,
        cost: 0,
        class: 0,
        weapon_matter: 0,
        costume_matter: 0,
        probability: 0,
        offence_up: 0,
        defence_up: 0,
        fail_type: 0,
        no_drop_probability: 0,
        one_drop_probability: 0,
        two_drop_probability: 0,
        three_drop_probability: 0,
        four_drop_probability: 0,
    },
    EnchantRecipe0104 {
        enchant_grade: 1,
        cost: 100,
        class: 0,
        weapon_matter: 10,
        costume_matter: 10,
        probability: 100,
        offence_up: 1,
        defence_up: 1,
        fail_type: 1,
        no_drop_probability: 100,
        one_drop_probability: 0,
        two_drop_probability: 0,
        three_drop_probability: 0,
        four_drop_probability: 0,
    },
    EnchantRecipe0104 {
        enchant_grade: 2,
        cost: 200,
        class: 0,
        weapon_matter: 12,
        costume_matter: 12,
        probability: 100,
        offence_up: 2,
        defence_up: 2,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 50,
        two_drop_probability: 50,
        three_drop_probability: 0,
        four_drop_probability: 0,
    },
    EnchantRecipe0104 {
        enchant_grade: 3,
        cost: 300,
        class: 0,
        weapon_matter: 15,
        costume_matter: 15,
        probability: 95,
        offence_up: 3,
        defence_up: 3,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 50,
        two_drop_probability: 50,
        three_drop_probability: 0,
        four_drop_probability: 0,
    },
    EnchantRecipe0104 {
        enchant_grade: 4,
        cost: 500,
        class: 1,
        weapon_matter: 10,
        costume_matter: 10,
        probability: 70,
        offence_up: 5,
        defence_up: 5,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 50,
        two_drop_probability: 50,
        three_drop_probability: 0,
        four_drop_probability: 0,
    },
    EnchantRecipe0104 {
        enchant_grade: 5,
        cost: 700,
        class: 1,
        weapon_matter: 12,
        costume_matter: 12,
        probability: 60,
        offence_up: 6,
        defence_up: 6,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 33,
        two_drop_probability: 33,
        three_drop_probability: 34,
        four_drop_probability: 0,
    },
    EnchantRecipe0104 {
        enchant_grade: 6,
        cost: 1_000,
        class: 1,
        weapon_matter: 15,
        costume_matter: 15,
        probability: 50,
        offence_up: 7,
        defence_up: 7,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 33,
        two_drop_probability: 33,
        three_drop_probability: 34,
        four_drop_probability: 0,
    },
    EnchantRecipe0104 {
        enchant_grade: 7,
        cost: 1_200,
        class: 1,
        weapon_matter: 18,
        costume_matter: 18,
        probability: 30,
        offence_up: 10,
        defence_up: 10,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 33,
        two_drop_probability: 33,
        three_drop_probability: 34,
        four_drop_probability: 0,
    },
    EnchantRecipe0104 {
        enchant_grade: 8,
        cost: 1_500,
        class: 1,
        weapon_matter: 20,
        costume_matter: 20,
        probability: 20,
        offence_up: 11,
        defence_up: 11,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 0,
        two_drop_probability: 33,
        three_drop_probability: 33,
        four_drop_probability: 34,
    },
    EnchantRecipe0104 {
        enchant_grade: 9,
        cost: 1_800,
        class: 1,
        weapon_matter: 25,
        costume_matter: 25,
        probability: 10,
        offence_up: 12,
        defence_up: 12,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 0,
        two_drop_probability: 33,
        three_drop_probability: 33,
        four_drop_probability: 34,
    },
    EnchantRecipe0104 {
        enchant_grade: 10,
        cost: 2_000,
        class: 1,
        weapon_matter: 30,
        costume_matter: 30,
        probability: 5,
        offence_up: 20,
        defence_up: 15,
        fail_type: 1,
        no_drop_probability: 0,
        one_drop_probability: 0,
        two_drop_probability: 33,
        three_drop_probability: 33,
        four_drop_probability: 34,
    },
];

pub const ENCHANT_EQUIPMENT_SLOT_COUNT_0104: usize = 9;

pub const ENCHANT_EQUIPMENT_SLOT_SIZE_0104: f32 = 64.0;

pub const ENCHANT_EQUIPMENT_SLOT_STRIDE_0104: f32 = 62.0;

pub const ENCHANT_EQUIPMENT_LABELS_0104: [&str; ENCHANT_EQUIPMENT_SLOT_COUNT_0104] = [
    "HEAD", "FACE", "BACK", "CHEST", "LEGS", "FEET", "WEAPON 1", "WEAPON 2", "VEHICLE",
];

pub const ENCHANT_UI_DEFAULT_IMAGE_PATHS_0104: [&str; EnchantStaticAssetRole0104::COUNT] = [
    ENCHANT_BACKDROP_PATH,
    ENCHANT_PANEL_PATH,
    ENCHANT_RIGHT_PANEL_PATH,
    ENCHANT_BLACK_SHADE_PATH,
    ENCHANT_RAW_COVER_PATH,
    ENCHANT_QUANTITY_COVER_PATH,
    ENCHANT_X_MARK_PATH,
    ENCHANT_LEVEL_BADGE_PATH,
    ENCHANT_WAITING_PATH,
    ENCHANT_WAIT_PROGRESS_PATH,
    ENCHANT_SUCCESS_PATH,
    ENCHANT_NPC_ICON_PATH,
    ENCHANT_BUTTON_NORMAL_PATH,
    ENCHANT_BUTTON_HOVER_PATH,
    ENCHANT_COMBINED_PATH,
    ENCHANT_INVENTORY_PANEL_PATH,
    ENCHANT_SLOT_OCCUPIED_PATH,
    ENCHANT_SLOT_EMPTY_PATH,
    ENCHANT_RESTRICTED_ITEM_FRAME_PATH,
    ENCHANT_EQUIP_TITLE_PATH,
    ENCHANT_CLOSE_PATH,
    ENCHANT_TRASH_PATH,
    ENCHANT_HELP_PATH,
    ENCHANT_DEXLABS_PATH,
    ENCHANT_TAROS_COUNTER_PATH,
    ENCHANT_BOOST_ICON_PATH,
    ENCHANT_POTION_ICON_PATH,
];
