use super::*;

/// Clean-primary ownership recovered from `main.unity3d` / `sharedassets0.assets`.
pub const CHARACTER_CREATION_PRIMARY_MAIN_UNITY3D_BYTES: u64 = 7_000_415;

pub const CHARACTER_CREATION_PRIMARY_MAIN_UNITY3D_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const CHARACTER_CREATION_PRIMARY_RESOURCE_FILE_BYTES: u64 = 8_974_798;

pub const CHARACTER_CREATION_PRIMARY_RESOURCE_FILE_SHA256: &str =
    "78785925E716027DE4BEC897C402C352BB411B7D627DE1783E6FBDA8BF34E59E";

pub const CHARACTER_CREATION_NAME_GUI_DEPTH: i32 = 8;

pub const CHARACTER_CREATION_APPEARANCE_GUI_DEPTH: i32 = 10;

pub const CHARACTER_CREATION_JEFFE_14_FONT_SIZE: f32 = 12.0;

pub const CHARACTER_CREATION_JEFFE_16_FONT_SIZE: f32 = 14.0;

pub const CHARACTER_CREATION_CHALET_SMALL_FONT_SIZE: f32 = 12.0;

pub const CHARACTER_CREATION_CHALET_REGULAR_FONT_SIZE: f32 = 14.0;

/// Every reached clean style serializes `m_ContentOffset = (0, 0)`. The
/// approved replacement fonts are therefore calibrated through their exact
/// source line spacing and style padding, with no speculative Y shift.
pub const CHARACTER_CREATION_TEXT_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_CREATION_TEXT_FIELD_SOURCE_FILE_ID: i64 = 1;

pub const CHARACTER_CREATION_PREVIEW_ROTATION_SPEED_DEGREES: f32 = 100.0;

pub const CHARACTER_CREATION_PREVIEW_MIN_DISTANCE: f32 = 1.0;

pub const CHARACTER_CREATION_PREVIEW_MAX_DISTANCE: f32 = 3.0;

/// UI pass for controls that occupy the same rectangle as the 3D preview.
pub const CHARACTER_CREATION_FOREGROUND_CAMERA_ORDER: isize = GAMEPLAY_UI_CAMERA_ORDER + 2;

pub const CHARACTER_CREATION_PREVIEW_ZOOM_DURATION_SECONDS: f32 = 5.0;

pub const CHARACTER_CREATION_PREVIEW_ZOOM_SPEED: f32 = (CHARACTER_CREATION_PREVIEW_MAX_DISTANCE
    - CHARACTER_CREATION_PREVIEW_MIN_DISTANCE)
    / CHARACTER_CREATION_PREVIEW_ZOOM_DURATION_SECONDS;

pub(super) const CCBG: &str = "ui/en/character/creation/layout/CCBG.png";

pub(super) const CC_CHARACTER_DISPLAY: &str = "ui/en/character/creation/layout/CCCharacterDisplayArea.png";

pub(super) const CC_RIGHT_BG: &str = "ui/en/character/creation/layout/CCRightBG.png";

pub(super) const CC_IN_12_BG: &str = "ui/en/character/creation/layout/CCLeftIn12BG.png";

pub(super) const CC_IN_3_BG: &str = "ui/en/character/creation/layout/CCLeftIn3BG.png";

pub(super) const CC_BODY_DISPLAY: &str = "ui/en/character/creation/body/CCBodyshapeDisplay.png";

pub(super) const CC_BODY_LEFT: &str = "ui/en/character/creation/body/CCBodyshapeLeftNormal.png";

pub(super) const CC_BODY_LEFT_OVER: &str = "ui/en/character/creation/body/CCBodyshapeLeftOver.png";

pub(super) const CC_BODY_RIGHT: &str = "ui/en/character/creation/body/CCBodyshapeRightNormal.png";

pub(super) const CC_BODY_RIGHT_OVER: &str = "ui/en/character/creation/body/CCBodyshapeRightOver.png";

pub(super) const CC_CHECKED: &str = "ui/en/character/creation/body/checkbox/CCCheckboxChecked.png";

pub(super) const CC_CHECKED_OVER: &str = "ui/en/character/creation/body/checkbox/CCCheckboxCkeckedOver.png";

pub(super) const CC_ROTATE_LEFT: &str = "ui/en/character/creation/camera/CCRotateLeftNormal.png";

pub(super) const CC_ROTATE_LEFT_OVER: &str = "ui/en/character/creation/camera/CCRotateLeftOver.png";

pub(super) const CC_ROTATE_RIGHT: &str = "ui/en/character/creation/camera/CCRotateRightNormal.png";

pub(super) const CC_ROTATE_RIGHT_OVER: &str = "ui/en/character/creation/camera/CCRoatateRightOver.png";

pub(super) const CC_ZOOM_IN: &str = "ui/en/character/creation/camera/CCZoomInNormal.png";

pub(super) const CC_ZOOM_IN_OVER: &str = "ui/en/character/creation/camera/CCZoomInOver.png";

pub(super) const CC_ZOOM_OUT: &str = "ui/en/character/creation/camera/CCZoomOutNormal.png";

pub(super) const CC_ZOOM_OUT_OVER: &str = "ui/en/character/creation/camera/CCZoomOutOver.png";

pub(super) const CC_CLOTHES_BG: &str = "ui/en/character/creation/clothes/CCClothesBG.png";

pub(super) const CC_COLOR_INSIDE: &str = "ui/en/character/creation/colors/CCColorInside.png";

pub(super) const CC_COLOR_OUTLINE: &str = "ui/en/character/creation/colors/CCColorOutline.png";

pub(super) const CC_NAME_BG: &str = "ui/en/character/creation/name/CCNameBG.png";

pub(super) const CC_NAME_DISPLAY: &str = "ui/en/character/creation/name/CCNameDisplay.png";

pub(super) const CC_NAME_SHADE: &str = "ui/en/character/creation/name/CCNameShade.png";

pub(super) const CC_NAME_TAB_1: &str = "ui/en/character/creation/name/CCNameTab1.png";

pub(super) const CC_NAME_TAB_2: &str = "ui/en/character/creation/name/CCNameTab2.png";

pub(super) const CC_FULLSCREEN: &str = "ui/en/character/selection/controls/CSFullScreenButton.png";

pub(super) const CC_WINDOWED: &str = "ui/en/character/selection/controls/CSFullToWindow.png";

pub(super) const CC_FULLSCREEN_OVER: &str = "ui/en/character/selection/controls/FullscreenModeButton_Over.png";

pub(super) const CC_WINDOWED_OVER: &str = "ui/en/character/selection/controls/WindowModeButton_Over.png";

/// Exact 53-entry source-proven Texture2D closure published below the native
/// `assets/game/ui/en/character/creation` semantic tree.
pub const CHARACTER_CREATION_IMAGE_SPECS: [CharacterCreationImageSpec; 53] = [
    image_spec!(
        "CharCreationBG",
        "ui/en/character/creation/background/CharCreationBG.png",
        1024,
        768
    ),
    image_spec!(
        "CCBodyshapeDisplay",
        "ui/en/character/creation/body/CCBodyshapeDisplay.png",
        116,
        22
    ),
    image_spec!(
        "CCBodyshapeLeftNormal",
        "ui/en/character/creation/body/CCBodyshapeLeftNormal.png",
        22,
        21
    ),
    image_spec!(
        "CCBodyshapeLeftOver",
        "ui/en/character/creation/body/CCBodyshapeLeftOver.png",
        22,
        21
    ),
    image_spec!(
        "CCBodyshapeRightNormal",
        "ui/en/character/creation/body/CCBodyshapeRightNormal.png",
        22,
        21
    ),
    image_spec!(
        "CCBodyshapeRightOver",
        "ui/en/character/creation/body/CCBodyshapeRightOver.png",
        22,
        21
    ),
    image_spec!(
        "CCCheckboxChecked",
        "ui/en/character/creation/body/checkbox/CCCheckboxChecked.png",
        24,
        23
    ),
    image_spec!(
        "CCCheckboxCkeckedOver",
        "ui/en/character/creation/body/checkbox/CCCheckboxCkeckedOver.png",
        24,
        23
    ),
    image_spec!(
        "CCCheckboxNormal",
        "ui/en/character/creation/body/checkbox/CCCheckboxNormal.png",
        24,
        23
    ),
    image_spec!(
        "CCCheckboxOver",
        "ui/en/character/creation/body/checkbox/CCCheckboxOver.png",
        24,
        23
    ),
    image_spec!(
        "CCRoatateRightOver",
        "ui/en/character/creation/camera/CCRoatateRightOver.png",
        86,
        156
    ),
    image_spec!(
        "CCRotateLeftNormal",
        "ui/en/character/creation/camera/CCRotateLeftNormal.png",
        86,
        156
    ),
    image_spec!(
        "CCRotateLeftOver",
        "ui/en/character/creation/camera/CCRotateLeftOver.png",
        86,
        156
    ),
    image_spec!(
        "CCRotateRightNormal",
        "ui/en/character/creation/camera/CCRotateRightNormal.png",
        86,
        156
    ),
    image_spec!(
        "CCZoomInNormal",
        "ui/en/character/creation/camera/CCZoomInNormal.png",
        37,
        38
    ),
    image_spec!(
        "CCZoomInOver",
        "ui/en/character/creation/camera/CCZoomInOver.png",
        37,
        38
    ),
    image_spec!(
        "CCZoomOutNormal",
        "ui/en/character/creation/camera/CCZoomOutNormal.png",
        37,
        38
    ),
    image_spec!(
        "CCZoomOutOver",
        "ui/en/character/creation/camera/CCZoomOutOver.png",
        37,
        38
    ),
    image_spec!(
        "CCClothesBG",
        "ui/en/character/creation/clothes/CCClothesBG.png",
        393,
        243
    ),
    image_spec!(
        "CCClothesLeftBottomButtonNormal",
        "ui/en/character/creation/clothes/CCClothesLeftBottomButtonNormal.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesLeftBottomButtonOver",
        "ui/en/character/creation/clothes/CCClothesLeftBottomButtonOver.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesLeftButtonNormal",
        "ui/en/character/creation/clothes/CCClothesLeftButtonNormal.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesLeftButtonOver",
        "ui/en/character/creation/clothes/CCClothesLeftButtonOver.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesLeftTopButtonNormal",
        "ui/en/character/creation/clothes/CCClothesLeftTopButtonNormal.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesLeftTopOver",
        "ui/en/character/creation/clothes/CCClothesLeftTopOver.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesRightBottomButtonNormal",
        "ui/en/character/creation/clothes/CCClothesRightBottomButtonNormal.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesRightBottomOver",
        "ui/en/character/creation/clothes/CCClothesRightBottomOver.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesRightButtonNormal",
        "ui/en/character/creation/clothes/CCClothesRightButtonNormal.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesRightButtonOver",
        "ui/en/character/creation/clothes/CCClothesRightButtonOver.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesRightTopButtonNormal",
        "ui/en/character/creation/clothes/CCClothesRightTopButtonNormal.png",
        22,
        81
    ),
    image_spec!(
        "CCClothesRightTopOver",
        "ui/en/character/creation/clothes/CCClothesRightTopOver.png",
        22,
        81
    ),
    image_spec!(
        "CCColorInside",
        "ui/en/character/creation/colors/CCColorInside.png",
        22,
        21
    ),
    image_spec!(
        "CCColorOutline",
        "ui/en/character/creation/colors/CCColorOutline.png",
        22,
        21
    ),
    image_spec!(
        "CCSelectedColor",
        "ui/en/character/creation/colors/CCSelectedColor.png",
        22,
        22
    ),
    image_spec!("CCBG", "ui/en/character/creation/layout/CCBG.png", 49, 653),
    image_spec!("CCBox", "ui/en/character/creation/layout/CCBox.png", 17, 25),
    image_spec!(
        "CCCharacterDisplayArea",
        "ui/en/character/creation/layout/CCCharacterDisplayArea.png",
        314,
        499
    ),
    image_spec!(
        "CCClassBG",
        "ui/en/character/creation/layout/CCClassBG.png",
        349,
        453
    ),
    image_spec!(
        "CCLeftIn12BG",
        "ui/en/character/creation/layout/CCLeftIn12BG.png",
        60,
        58
    ),
    image_spec!(
        "CCLeftIn3BG",
        "ui/en/character/creation/layout/CCLeftIn3BG.png",
        30,
        35
    ),
    image_spec!(
        "CCRightBG",
        "ui/en/character/creation/layout/CCRightBG.png",
        554,
        650
    ),
    image_spec!(
        "CCNameBG",
        "ui/en/character/creation/name/CCNameBG.png",
        27,
        364
    ),
    image_spec!(
        "CCNameDisplay",
        "ui/en/character/creation/name/CCNameDisplay.png",
        18,
        16
    ),
    image_spec!(
        "CCNameShade",
        "ui/en/character/creation/name/CCNameShade.png",
        37,
        157
    ),
    image_spec!(
        "CCNameTab1",
        "ui/en/character/creation/name/CCNameTab1.png",
        673,
        33
    ),
    image_spec!(
        "CCNameTab2",
        "ui/en/character/creation/name/CCNameTab2.png",
        673,
        31
    ),
    image_spec!(
        "CCNameTabButtonNormal",
        "ui/en/character/creation/name/CCNameTabButtonNormal.png",
        77,
        28
    ),
    image_spec!(
        "CCNameTabButtonOver",
        "ui/en/character/creation/name/CCNameTabButtonOver.png",
        76,
        30
    ),
    image_spec!(
        "CCScrollBG",
        "ui/en/character/creation/name/CCScrollBG.png",
        36,
        158
    ),
    image_spec!(
        "CCScrollDownOver",
        "ui/en/character/creation/name/CCScrollDownOver.png",
        148,
        15
    ),
    image_spec!(
        "CCScrollDownUp",
        "ui/en/character/creation/name/CCScrollDownUp.png",
        149,
        15
    ),
    image_spec!(
        "CCScrollUpNormal",
        "ui/en/character/creation/name/CCScrollUpNormal.png",
        149,
        15
    ),
    image_spec!(
        "CCScrollUpOver",
        "ui/en/character/creation/name/CCScrollUpOver.png",
        126,
        15
    ),
];

/// Source controls shared with character selection rather than duplicated in
/// the character-creation semantic folder. The four 20x25 button textures and
/// two hover textures are referenced by `FusionFallCharCreation` itself.
pub const CHARACTER_CREATION_SHARED_IMAGE_SPECS: [CharacterCreationImageSpec; 8] = [
    image_spec!(
        "CSFullScreenButton",
        "ui/en/character/selection/controls/CSFullScreenButton.png",
        37,
        31
    ),
    image_spec!(
        "CSFullToWindow",
        "ui/en/character/selection/controls/CSFullToWindow.png",
        37,
        31
    ),
    image_spec!(
        "FullscreenModeButton_Over",
        "ui/en/character/selection/controls/FullscreenModeButton_Over.png",
        37,
        31
    ),
    image_spec!(
        "WindowModeButton_Over",
        "ui/en/character/selection/controls/WindowModeButton_Over.png",
        37,
        31
    ),
    image_spec!(
        "blue_button_normal",
        "ui/en/character/selection/controls/blue_button_normal.png",
        20,
        25
    ),
    image_spec!(
        "blue_button_over",
        "ui/en/character/selection/controls/blue_button_over.png",
        20,
        25
    ),
    image_spec!(
        "red_button_normal",
        "ui/en/character/selection/controls/red_button_normal.png",
        20,
        25
    ),
    image_spec!(
        "red_button_over",
        "ui/en/character/selection/controls/red_button_over.png",
        20,
        25
    ),
];

/// Built-in `textField` normal background. Its external reference is
/// `fileID 1 / pathID 11024`; this already-published semantic copy is byte
/// identical and avoids introducing a duplicate runtime asset.
pub const CHARACTER_CREATION_ENGINE_IMAGE_SPECS: [CharacterCreationImageSpec; 1] = [image_spec!(
    "FusionFallCharCreation.textField.normal.background",
    "ui/en/launcher/login/ff-textfield-normal.png",
    5,
    25
)];

// Exact bsddata colors used by CnGuiCharCreation. Unity's serialized values
// are kept as their source fractions instead of sampled from screenshots.
pub const SKIN_COLORS: [Color; 12] = [
    Color::srgb(0.2265625, 0.15234375, 0.09765625),
    Color::srgb(0.35546875, 0.29296875, 0.16015625),
    Color::srgb(0.39453125, 0.30859375, 0.2265625),
    Color::srgb(0.59375, 0.40625, 0.328125),
    Color::srgb(0.54296875, 0.46484375, 0.3828125),
    Color::srgb(0.59375, 0.5078125, 0.32421875),
    Color::srgb(0.71875, 0.5078125, 0.40625),
    Color::srgb(0.7109375, 0.578125, 0.4296875),
    Color::srgb(0.8984375, 0.73046875, 0.5546875),
    Color::srgb(0.91015625, 0.8828125, 0.671875),
    Color::srgb(0.97265625, 0.83984375, 0.80078125),
    Color::srgb(0.828125, 0.859375, 0.87890625),
];

pub const HAIR_COLORS: [Color; 18] = [
    Color::srgb(0.99609375, 0.99609375, 0.99609375),
    Color::srgb(0.54296875, 0.56640625, 0.61328125),
    Color::srgb(0.055, 0.055, 0.055),
    Color::srgb(0.859375, 0.57421875, 0.609375),
    Color::srgb(0.578125, 0.2890625, 0.19140625),
    Color::srgb(0.84375, 0.2421875, 0.0390625),
    Color::srgb(0.73046875, 0.78125, 0.90625),
    Color::srgb(0.5390625, 0.078125, 0.625),
    Color::srgb(0.2578125, 0.0390625, 0.62109375),
    Color::srgb(0.43359375, 0.80859375, 0.703125),
    Color::srgb(0.05859375, 0.8203125, 0.0390625),
    Color::srgb(0.1015625, 0.48828125, 0.16796875),
    Color::srgb(0.9375, 0.9140625, 0.671875),
    Color::srgb(0.9375, 0.84765625, 0.41015625),
    Color::srgb(0.04296875, 0.60546875, 0.96484375),
    Color::srgb(0.9453125, 0.5234375, 0.03125),
    Color::srgb(0.48828125, 0.31640625, 0.265625),
    Color::srgb(0.3125, 0.18359375, 0.1640625),
];

pub const EYE_COLORS: [Color; 5] = [
    Color::srgb(0.78515625, 0.86328125, 35.0 / 128.0),
    Color::srgb(1.0 / 32.0, 41.0 / 128.0, 0.79296875),
    Color::srgb(0.05078125, 71.0 / 128.0, 0.15234375),
    Color::srgb(0.49609375, 9.0 / 32.0, 0.12890625),
    Color::BLACK,
];
