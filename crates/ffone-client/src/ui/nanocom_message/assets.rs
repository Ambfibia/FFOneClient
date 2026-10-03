
pub const NANOCOM_GAME_OBJECT_PATH_ID: i64 = 1_352;

pub const NANOCOM_COMPONENT_PATH_ID: i64 = 1_562;

pub const NANOCOM_SCRIPT_PATH_ID: i64 = 1_112;

pub const NANOCOM_HUD_SKIN_PATH_ID: i64 = 1_372;

pub const NANOCOM_BUDDY_ICON_PATH: &str = "ui/en/gameplay/nanocom/messicon_buddy.png";

pub const NANOCOM_GROUP_ICON_PATH: &str = "ui/en/gameplay/system/messicon_group.png";

pub const NANOCOM_NUMBUH_TWO_ICON_PATH: &str = "ui/en/gameplay/nanocom/npcicon_87.png";

pub const NANOCOM_DIALOG_PATH: &str = "ui/en/gameplay/system/systemDialogBox.png";

pub const NANOCOM_MESSAGE_AREA_PATH: &str = "ui/en/gameplay/nanocom/messagearea.png";

pub const NANOCOM_JEFFE_FONT_PATH: &str = "fonts/jeffe.otf";

pub const NANOCOM_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const NANOCOM_SLIDE_IN_PATH: &str = "audio/sfx/ui/comm_slidein.ogg";

pub const NANOCOM_SLIDE_OUT_PATH: &str = "audio/sfx/ui/comm_slideout.ogg";

pub const NANOCOM_NANO_CREATION_COMPLETE_PATH: &str = "audio/sfx/nano/nano_creation_complete.ogg";

pub const NANOCOM_YES_PATH: &str = "audio/sfx/ui/yes_button.ogg";

pub const NANOCOM_NO_PATH: &str = "audio/sfx/ui/no_button.ogg";

pub const NANOCOM_JEFFE_14_FONT_PATH_ID: i64 = 903;

pub const NANOCOM_CHALET_SMALL_FONT_PATH_ID: i64 = 1_018;

pub const NANOCOM_MESSAGE_TITLE_FONT_PATH_ID: i64 = 934;

/// Clean `OnGUI` paints `RenderNanoMessage` before `RenderMenu` and
/// `RenderMinimap`; the compact panel must therefore remain behind the
/// resident HUD (most visibly where the minimap overlaps its right edge).
pub const NANOCOM_UI_Z_INDEX: i32 = -1;

pub const NANOCOM_EXPANDED_Z_INDEX: i32 = i32::MAX - 22;
