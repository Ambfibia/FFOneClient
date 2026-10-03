use super::*;

pub const TUTORIAL_MISSION_TASK_IDS: [i32; 7] = [2248, 2249, 2250, 2251, 2252, 2253, 2254];

pub const TUTORIAL_MISSION_NPC_TYPES: [i32; 7] = [2671, 2672, 2673, 2694, 2695, 2696, 2800];

pub const TUTORIAL_WARP_NPC_TYPES: [i32; 3] = [2694, 2695, 2696];

pub(super) const TABLE_SET_SCHEMA: &str = "ffone.table-set.v1";

pub(super) const CONSOLIDATED_TABLE: &str = "npc_imports_consolidated";

pub(super) const OPTIONAL_USER_EQUIP_ITEM_TABLES: [UserEquipIconTableSpec; 9] = [
    UserEquipIconTableSpec {
        table_key: "m_pBackItemTable",
        item_table: 17,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
    UserEquipIconTableSpec {
        table_key: "m_pGlassItemTable",
        item_table: 19,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
    UserEquipIconTableSpec {
        table_key: "m_pHatItemTable",
        item_table: 20,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
    UserEquipIconTableSpec {
        table_key: "m_pPantsItemTable",
        item_table: 22,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
    UserEquipIconTableSpec {
        table_key: "m_pShirtsItemTable",
        item_table: 23,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
    UserEquipIconTableSpec {
        table_key: "m_pShoesItemTable",
        item_table: 24,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
    UserEquipIconTableSpec {
        table_key: "m_pWeaponItemTable",
        item_table: 25,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
    UserEquipIconTableSpec {
        table_key: "m_pVehicleItemTable",
        item_table: 26,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
    UserEquipIconTableSpec {
        table_key: "m_pChestItemTable",
        item_table: 28,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: true,
    },
];

pub(super) const EXISTING_USER_EQUIP_ICON_SUBTABLES: [UserEquipIconTableSpec; 6] = [
    UserEquipIconTableSpec {
        table_key: "m_pNanoTable",
        item_table: 9,
        item_rows_key: "m_pNanoData",
        declared_item_field: "m_iNanoNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon1",
        icon_rows_key: "m_pNanoIconData",
        icon_subtable: 3,
        require_icon_rows_when_table_present: false,
    },
    UserEquipIconTableSpec {
        table_key: "m_pNanoTable",
        item_table: 9,
        item_rows_key: "m_pNanoData",
        declared_item_field: "m_iNanoNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon1",
        icon_rows_key: "m_pNanoTuneIconData",
        icon_subtable: 6,
        require_icon_rows_when_table_present: false,
    },
    UserEquipIconTableSpec {
        table_key: "m_pNpcTable",
        item_table: 10,
        item_rows_key: "m_pNpcData",
        declared_item_field: "m_iNpcNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon1",
        icon_rows_key: "m_pNpcIconData",
        icon_subtable: 3,
        require_icon_rows_when_table_present: false,
    },
    UserEquipIconTableSpec {
        table_key: "m_pSkillTable",
        item_table: 12,
        item_rows_key: "m_pSkillData",
        declared_item_field: "m_iSkillNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pSkillIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: false,
    },
    UserEquipIconTableSpec {
        table_key: "m_pSkillTable",
        item_table: 12,
        item_rows_key: "m_pSkillBuffData",
        declared_item_field: "m_iBuffNumber",
        item_subtable: 1,
        item_icon_field: "m_iBuffIcon",
        icon_rows_key: "m_pSkillIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: false,
    },
    UserEquipIconTableSpec {
        table_key: "m_pGeneralItemTable",
        item_table: 27,
        item_rows_key: "m_pItemData",
        declared_item_field: "m_iItemNumber",
        item_subtable: 0,
        item_icon_field: "m_iIcon",
        icon_rows_key: "m_pItemIconData",
        icon_subtable: 2,
        require_icon_rows_when_table_present: false,
    },
];

pub(super) const VENDOR_ITEM_TABLES: [VendorItemTableSpec; 10] = [
    VendorItemTableSpec {
        table_key: "m_pWeaponItemTable",
        item_type: 0,
        item_table: 25,
        requires_level: true,
        is_general: false,
    },
    VendorItemTableSpec {
        table_key: "m_pShirtsItemTable",
        item_type: 1,
        item_table: 23,
        requires_level: true,
        is_general: false,
    },
    VendorItemTableSpec {
        table_key: "m_pPantsItemTable",
        item_type: 2,
        item_table: 22,
        requires_level: true,
        is_general: false,
    },
    VendorItemTableSpec {
        table_key: "m_pShoesItemTable",
        item_type: 3,
        item_table: 24,
        requires_level: true,
        is_general: false,
    },
    VendorItemTableSpec {
        table_key: "m_pHatItemTable",
        item_type: 4,
        item_table: 20,
        requires_level: true,
        is_general: false,
    },
    VendorItemTableSpec {
        table_key: "m_pGlassItemTable",
        item_type: 5,
        item_table: 19,
        requires_level: true,
        is_general: false,
    },
    VendorItemTableSpec {
        table_key: "m_pBackItemTable",
        item_type: 6,
        item_table: 17,
        requires_level: true,
        is_general: false,
    },
    VendorItemTableSpec {
        table_key: "m_pGeneralItemTable",
        item_type: 7,
        item_table: 27,
        requires_level: false,
        is_general: true,
    },
    VendorItemTableSpec {
        table_key: "m_pChestItemTable",
        item_type: 9,
        item_table: 28,
        requires_level: false,
        is_general: false,
    },
    VendorItemTableSpec {
        table_key: "m_pVehicleItemTable",
        item_type: 10,
        item_table: 26,
        requires_level: true,
        is_general: false,
    },
];

pub(super) const EXPECTED_WARPS: [ExpectedWarp; 3] = [
    ExpectedWarp {
        npc_type: 2694,
        warp_id: 253,
        required_task_id: 2251,
        target: TutorialWarpTarget {
            map_id: 0,
            x: 59_573,
            y: 74_545,
            z: -9_066,
        },
    },
    ExpectedWarp {
        npc_type: 2695,
        warp_id: 254,
        required_task_id: 2252,
        target: TutorialWarpTarget {
            map_id: 0,
            x: 59_612,
            y: 98_567,
            z: -13_300,
        },
    },
    ExpectedWarp {
        npc_type: 2696,
        warp_id: 255,
        required_task_id: 0,
        target: TutorialWarpTarget {
            map_id: 0,
            x: 90_765,
            y: 71_558,
            z: 1_043,
        },
    },
];

pub(super) const GUIDE_FIRST_WARP_ID: i32 = 76;

pub(super) const GUIDE_FIRST_WARP_NPC_TYPE: i32 = 1425;

pub(super) const GUIDE_FIRST_WARP_TARGET: TutorialWarpTarget = TutorialWarpTarget {
    map_id: 0,
    x: 373_330,
    y: 442_600,
    z: -5_700,
};
