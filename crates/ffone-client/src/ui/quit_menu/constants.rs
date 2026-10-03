
pub const QUIT_MENU_ROOT_FONT_CAVEAT: &str = "QuitMenuSkin.m_Font resolves through external \
fileID 1/pathID 10102, but clean cnQuit.OnGUI emits no text through it. Reachable Button and \
CancelButton styles both explicitly use JEFFE___14 pathID 903; the external root font is unused \
serialized UI and is intentionally not published.";

pub const QUIT_MENU_FONT_SIZE: f32 = 12.0;

/// All reachable QuitMenu GUIStyles serialize `m_ContentOffset.y = 0`.
pub const QUIT_MENU_TEXT_Y_OFFSET: f32 = 0.0;

pub const QUIT_MENU_CANCEL_NORMAL_TEXT_COLOR: [f32; 4] =
    [0.983_703_4, 0.983_703_4, 0.983_703_4, 1.0];

pub const QUIT_MENU_CANCEL_ACTIVE_TEXT_COLOR: [f32; 4] = [1.0; 4];
