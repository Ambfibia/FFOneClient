use super::*;

pub const OPTION_BLOCKED_SLOT_CAPACITY: usize = 50;

pub const OPTION_BLOCKED_VISIBLE_ROWS: usize = 18;

pub const OPTION_KEY_CAPTURE_TIMEOUT_SECONDS: f32 = 8.0;

/// Replacement-font sizes calibrated against the clean bitmap glyph bounds.
pub const OPTION_JEFFE_TAB_FONT_SIZE: f32 = 12.0;

pub const OPTION_JEFFE_TITLE_FONT_SIZE: f32 = 11.0;

pub const OPTION_CHALET_WHITE_LABEL_FONT_SIZE: f32 = 13.1;

pub const OPTION_CHALET_WHITE_LABEL_TOP_OFFSET: f32 = 4.0;

/// Bevy/Taffy cannot reproduce Unity's negative left RectOffset without
/// pushing the replacement glyphs outside the selected tab's left rail.
pub const OPTION_GRAPHICS_TAB_TEXT_INSET: f32 = 10.0;

pub const OPTION_FONT_CONTRACTS: [OptionFontContract; 3] = [
    OptionFontContract {
        runtime_path: OPTION_JEFFE_FONT_PATH,
        bytes: 32_776,
        sha256: "f8d41844ad2092d9998e51b8cbef5b65b3ce6db276c93949ececae227674c3e1",
    },
    OptionFontContract {
        runtime_path: OPTION_COMIC_FONT_PATH,
        bytes: 111_476,
        sha256: "873361465d994994762d0b9845c99fc7baa2a600442ea8db713a7dd19f8b0172",
    },
    OptionFontContract {
        runtime_path: OPTION_CHALET_FONT_PATH,
        bytes: 96_832,
        sha256: "6383bd9f81e56d61139884d8e42cb7b2146a11dde4efde55c8bff1e4c2c0bbe8",
    },
];

/// The reachable clean page draws only the first three rows of the source
/// client's 24-color table: 18 swatches, six columns by three rows.
pub const OPTION_CHAT_PALETTE_RGB: [(f32, f32, f32); 18] = [
    (1.0, 0.0, 0.0),
    (1.0, 0.6, 0.0),
    (0.5, 0.66, 0.0),
    (0.0, 0.39, 0.0),
    (0.0, 0.47, 0.47),
    (0.5, 0.0, 0.5),
    (1.0, 1.0, 0.0),
    (0.0, 0.0, 0.0),
    (1.0, 0.0, 1.0),
    (1.0, 0.78, 0.0),
    (1.0, 1.0, 0.0),
    (0.0, 1.0, 0.0),
    (0.0, 1.0, 1.0),
    (0.0, 0.7, 1.0),
    (0.56, 0.12, 0.35),
    (0.39, 0.39, 0.39),
    (1.0, 0.6, 0.8),
    (1.0, 0.8, 0.6),
];

/// A deterministic preview/runtime seed for the clean page's dynamic
/// `Screen.resolutions` dropdown. Adapters may still call `set_resolution`
/// with any host-provided mode.
pub const OPTION_RESOLUTION_CHOICES: [(u32, u32); 6] = [
    (800, 600),
    (1_024, 768),
    (1_280, 720),
    (1_280, 800),
    (1_366, 768),
    (1_920, 1_080),
];

pub const OPTION_DROPDOWN_ITEM_OVERFLOW_LEFT: f32 = 2.0;

pub const OPTION_DROPDOWN_ITEM_OVERFLOW_RIGHT: f32 = 3.0;

pub const OPTION_MOVEMENT_ACTIONS: [LegacyOptionAction; 8] = [
    LegacyOptionAction::Up,
    LegacyOptionAction::Down,
    LegacyOptionAction::RightTurn,
    LegacyOptionAction::LeftTurn,
    LegacyOptionAction::Right,
    LegacyOptionAction::Left,
    LegacyOptionAction::Jump,
    LegacyOptionAction::AutoRun,
];

pub const OPTION_INTERFACE_ACTIONS: [LegacyOptionAction; 11] = [
    LegacyOptionAction::NanoBook,
    LegacyOptionAction::Inventory,
    LegacyOptionAction::Journal,
    LegacyOptionAction::Email,
    LegacyOptionAction::WorldMap,
    LegacyOptionAction::Escape,
    LegacyOptionAction::Contacts,
    LegacyOptionAction::Help,
    LegacyOptionAction::Menu,
    LegacyOptionAction::Option,
    LegacyOptionAction::SendChat,
];

pub const OPTION_CAMERA_ACTIONS: [LegacyOptionAction; 7] = [
    LegacyOptionAction::CameraUp,
    LegacyOptionAction::CameraDown,
    LegacyOptionAction::CameraLeft,
    LegacyOptionAction::CameraRight,
    LegacyOptionAction::ZoomIn,
    LegacyOptionAction::ZoomOut,
    LegacyOptionAction::FreeCamera,
];

pub const OPTION_COMBAT_ACTIONS: [LegacyOptionAction; 13] = [
    LegacyOptionAction::Fire1,
    LegacyOptionAction::WeaponChange,
    LegacyOptionAction::Fire2,
    LegacyOptionAction::NanoCharge,
    LegacyOptionAction::Nano1,
    LegacyOptionAction::Nano2,
    LegacyOptionAction::Nano3,
    LegacyOptionAction::VehicleToggle,
    LegacyOptionAction::WeaponCharge,
    LegacyOptionAction::Skill1,
    LegacyOptionAction::Skill2,
    LegacyOptionAction::Skill3,
    LegacyOptionAction::None,
];
