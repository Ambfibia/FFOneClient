use super::*;

pub const LOGIN_SOURCE_BUILD: &str = "retrobution-20260613";

pub const LOGIN_SOURCE_MAIN_ARCHIVE: &str = "main.unity3d";

pub const LOGIN_SOURCE_MAIN_ARCHIVE_BYTES: u64 = 7_000_415;

pub const LOGIN_SOURCE_MAIN_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const LOGIN_SOURCE_CREATION_ARCHIVE: &str = "CharacterCreation.resourceFile";

pub const LOGIN_SOURCE_CREATION_ARCHIVE_BYTES: u64 = 8_974_798;

pub const LOGIN_SOURCE_CREATION_ARCHIVE_SHA256: &str =
    "78785925E716027DE4BEC897C402C352BB411B7D627DE1783E6FBDA8BF34E59E";

pub const LOGIN_COMMUNITY_URL: &str = "http://www.forums.fusionfalluniverse.com";

pub const LOGIN_REGISTRATION_INSTRUCTIONS: &str = "To Register:\nChoose an Username and Password into the appropriate boxes and press Log In. Make sure to remember these as there is no account recovery!";

pub const LOGIN_IMAGE_SPECS: [LoginImageSpec; 7] = [
    LoginImageSpec {
        role: "loaded background",
        path: LOGIN_BACKGROUND_PATH,
        source_asset: LOGIN_SOURCE_CREATION_ASSET,
        source_path_id: LOGIN_LOADED_BACKGROUND_TEXTURE_PATH_ID,
        width: 1_920,
        height: 1_080,
        bytes: 3_080_509,
        sha256: "0CD6F24D9412914EDE3369228581BAE52F37E653E1CA71D9E6B763D8A549C3EF",
    },
    LoginImageSpec {
        role: "fallback background",
        path: LOGIN_FALLBACK_BACKGROUND_PATH,
        source_asset: LOGIN_SOURCE_SERIALIZED_FILE,
        source_path_id: LOGIN_FALLBACK_BACKGROUND_PATH_ID,
        width: 1_920,
        height: 1_012,
        bytes: 2_660_857,
        sha256: "75D7029A98B05EFF4452E27A0F744562527C4D9029A2DC4F557C72FD974556A2",
    },
    LoginImageSpec {
        role: "window panel",
        path: LOGIN_PANEL_PATH,
        source_asset: LOGIN_SOURCE_SERIALIZED_FILE,
        source_path_id: LOGIN_PANEL_TEXTURE_PATH_ID,
        width: 375,
        height: 244,
        bytes: 33_665,
        sha256: "870C8DEC908E13D152856DA8D07A105FEA16B4A4A160883745A8EF2E564F798F",
    },
    LoginImageSpec {
        role: "button normal",
        path: LOGIN_BUTTON_PATH,
        source_asset: LOGIN_SOURCE_SERIALIZED_FILE,
        source_path_id: LOGIN_BUTTON_NORMAL_TEXTURE_PATH_ID,
        width: 11,
        height: 26,
        bytes: 629,
        sha256: "6205BA81A517B3861976BCDC68B622F83E04C6A005B416F4CECA9862156ACCA5",
    },
    LoginImageSpec {
        role: "button hover",
        path: LOGIN_BUTTON_OVER_PATH,
        source_asset: LOGIN_SOURCE_SERIALIZED_FILE,
        source_path_id: LOGIN_BUTTON_HOVER_TEXTURE_PATH_ID,
        width: 11,
        height: 26,
        bytes: 691,
        sha256: "180EADB13B03A8DC5F1A5CCEBEED9193ED932F87715DA6B424EFE20FE5A79BE8",
    },
    LoginImageSpec {
        role: "button active",
        path: LOGIN_BUTTON_ACTIVE_PATH,
        source_asset: LOGIN_SOURCE_SERIALIZED_FILE,
        source_path_id: LOGIN_BUTTON_ACTIVE_TEXTURE_PATH_ID,
        width: 13,
        height: 31,
        bytes: 722,
        sha256: "AB17A0BD908F6C8F533CEFF18080C486B6AB2E4C4224F8291D8E359F04840847",
    },
    LoginImageSpec {
        role: "text field",
        path: LOGIN_TEXT_FIELD_PATH,
        source_asset: LOGIN_SOURCE_SERIALIZED_FILE,
        source_path_id: LOGIN_TEXT_FIELD_TEXTURE_PATH_ID,
        width: 5,
        height: 25,
        bytes: 217,
        sha256: "F5A79B4FDA32A41147172A2EF4CFCCBF62BEED87B01F7A31DC4ACF9E82CD1556",
    },
];

pub const LOGIN_REPLACEMENT_FONT_SPECS: [LoginReplacementFontSpec; 2] = [
    LoginReplacementFontSpec {
        role: "JEFFE replacement",
        path: LOGIN_FONT_PATH,
        source_font_path_id: LOGIN_JEFFE_SOURCE_FONT_PATH_ID,
        bytes: 32_776,
        sha256: "F8D41844AD2092D9998E51B8CBEF5B65B3CE6DB276C93949ECECAE227674C3E1",
    },
    LoginReplacementFontSpec {
        role: "Chalet replacement",
        path: LOGIN_TEXT_FIELD_FONT_PATH,
        source_font_path_id: LOGIN_CHALET_SOURCE_FONT_PATH_ID,
        bytes: 96_832,
        sha256: "6383BD9F81E56D61139884D8E42CB7B2146A11DDE4EFDE55C8BFF1E4C2C0BBE8",
    },
];

pub const LOGIN_FALLBACK_BACKGROUND_SIZE: Vec2 = Vec2::new(1_920.0, 1_012.0);

pub const LOGIN_BACKGROUND_SIZE: Vec2 = Vec2::new(1_920.0, 1_080.0);

pub const LOGIN_PANEL_SIZE: Vec2 = Vec2::new(370.0, 275.0);

pub const LOGIN_JEFFE_FONT_SIZE: f32 = 14.0;

pub const LOGIN_CHALET_FONT_SIZE: f32 = 14.0;

// All reached clean styles serialize contentOffset=(0, 0). Deterministic
// JEFFE/Chalet EN+RU captures keep their glyph and line boxes inside the
// source controls without an additional replacement-font baseline shift.
pub const LOGIN_LABEL_Y_OFFSET: f32 = 0.0;

pub const LOGIN_TEXT_FIELD_Y_OFFSET: f32 = 0.0;

pub const LOGIN_LABEL_PADDING: UiRect = UiRect {
    left: Val::Px(0.0),
    right: Val::Px(0.0),
    top: Val::Px(3.0),
    bottom: Val::Px(3.0),
};

pub const LOGIN_TEXT_FIELD_PADDING: UiRect = UiRect {
    left: Val::Px(5.0),
    right: Val::Px(5.0),
    top: Val::Px(5.0),
    bottom: Val::Px(5.0),
};
