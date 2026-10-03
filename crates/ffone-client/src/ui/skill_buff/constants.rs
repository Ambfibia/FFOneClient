
/// Clean `TableData` time-buff ID 17 condition bit.
pub const SKILL_BUFF_INFECTION_FLAG: u32 = 65_536;

pub const SKILL_BUFF_SOURCE_BUILD: &str = "retrobution-20260613";

pub const SKILL_BUFF_LEGACY_CLASS: &str = "CnGuiSkillBuffIcon";

pub const SKILL_BUFF_ICON_SIZE: f32 = 26.0;

pub const SKILL_BUFF_LOCAL_CENTER_X: f32 = 117.0;

pub const SKILL_BUFF_LOCAL_Y: f32 = 72.0;

pub const SKILL_BUFF_CASH_X: f32 = 630.0;

pub const SKILL_BUFF_CASH_Y: f32 = 40.0;

pub const SKILL_BUFF_TARGET_CENTER_OFFSET_X: f32 = 65.0;

pub const SKILL_BUFF_TARGET_Y: f32 = 40.0;

pub const SKILL_BUFF_MAX_ICONS: usize = 22;

pub const SKILL_BUFF_MAX_ID: usize = 25;

pub const SKILL_BUFF_FONT_SIZE: f32 = 12.0;

/// The approved Chalet replacement needs no extra baseline shift for the
/// reached HUD `label` style. Keep this per-style value explicit rather than
/// inheriting a global replacement-font adjustment.
pub const SKILL_BUFF_FONT_Y_OFFSET: f32 = 0.0;

pub const SKILL_BUFF_LABEL_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];

pub const SKILL_BUFF_BUFFS: [(u32, i32); 16] = [
    (1, 1),
    (4, 3),
    (8, 4),
    (16, 5),
    (32, 6),
    (64, 7),
    (4_096, 13),
    (8_192, 14),
    (16_384, 15),
    (32_768, 16),
    (131_072, 18),
    (524_288, 20),
    (8_388_608, 24),
    (1_048_576, 21),
    (2_097_152, 22),
    (4_194_304, 23),
];

pub const SKILL_BUFF_DEBUFFS: [(u32, i32); 6] = [
    (128, 8),
    (256, 9),
    (512, 10),
    (1_024, 11),
    (65_536, 17),
    (262_144, 19),
];

pub(super) const STIM_FLAGS: [u32; 3] = [1_048_576, 2_097_152, 4_194_304];
