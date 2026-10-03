use super::*;

pub const SERVER_SELECTION_ROOT_PATH_ID: i64 = 1_295;

pub const SERVER_SELECTION_TRANSFORM_PATH_ID: i64 = 1_249;

pub const SERVER_SELECTION_MODE_COMPONENT_PATH_ID: i64 = 1_536;

pub const SERVER_SELECTION_GUI_COMPONENT_PATH_ID: i64 = 1_537;

pub const SERVER_SELECTION_MODE_SCRIPT_PATH_ID: i64 = 1_040;

pub const SERVER_SELECTION_GUI_SCRIPT_PATH_ID: i64 = 1_033;

pub const SERVER_SELECTION_SKIN_PATH_ID: i64 = 1_374;

pub const SERVER_SELECTION_UI_Z_INDEX: i32 = 10_000;

pub const SERVER_SELECTION_FONT_PATH: &str = "fonts/jeffe.otf";

pub const SERVER_SELECTION_SKIN_LEDGER_PATH: &str =
    "ui/en/gameplay/skins/retrobution-20260613.json";

pub const SERVER_SELECTION_JEFFE_16_SOURCE_PATH_ID: i64 = 1_012;

pub const SERVER_SELECTION_JEFFE_14_SOURCE_PATH_ID: i64 = 903;

pub const SERVER_SELECTION_BACKGROUND_PATH: &str =
    "ui/en/server-selection/login_screen_bg_16x10.png";

pub const SERVER_SELECTION_PANEL_PATH: &str = "ui/en/server-selection/SSBox.png";

pub const SERVER_SELECTION_INNER_PATH: &str = "ui/en/server-selection/SSBoxInside.png";

pub const SERVER_SELECTION_ROW_SELECTED_PATH: &str = "ui/en/server-selection/SSButtonDown.png";

pub const SERVER_SELECTION_RED_NORMAL_PATH: &str = "ui/en/server-selection/red_button_normal.png";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerSelectionAssetContract {
    pub path: &'static str,
    pub source_path_id: i64,
    pub source_name: &'static str,
    pub source_format: &'static str,
    pub source_width: u32,
    pub source_height: u32,
    pub source_payload_sha256: &'static str,
    pub png_size: u64,
    pub png_sha256: &'static str,
}

pub const SERVER_SELECTION_ASSET_CONTRACTS: [ServerSelectionAssetContract; 14] = [
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_BACKGROUND_PATH,
        source_path_id: 80,
        source_name: "login_screen_bg_16x10",
        source_format: "DXT1",
        source_width: 1_020,
        source_height: 638,
        source_payload_sha256: "7AF072D0F1B848984A18958F759F01F7DEB7C00F9993459EB8E73FD4A715DA39",
        png_size: 708_324,
        png_sha256: "EE32BAC1EB33BAA71E7412741AE1BEF42075BCDB9FE341952FB6E7E8E7BB2D5F",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_PANEL_PATH,
        source_path_id: 434,
        source_name: "SSBox",
        source_format: "DXT5",
        source_width: 375,
        source_height: 349,
        source_payload_sha256: "4296094BE2D90008F0F41C4FE828C4659C9825E2DA3FE5B6867DCDB544F2E8B6",
        png_size: 8_940,
        png_sha256: "92B4FC07FEBD1E710FFA1EB8D14B4235586E9958474A89771F5C959C7ED8CAF7",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_INNER_PATH,
        source_path_id: 328,
        source_name: "SSBoxInside",
        source_format: "DXT1",
        source_width: 293,
        source_height: 256,
        source_payload_sha256: "C958EAD985A8B06E6FBEB9DB5AA6C2846B92CA441685E34A7E03C04D546F599C",
        png_size: 2_499,
        png_sha256: "42430C4FAA73077A39C64B96C9BE065095A31FF4EA2C9E5973E1044D767DC4F7",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_ROW_HOVER_PATH,
        source_path_id: 367,
        source_name: "SSButtonOver",
        source_format: "DXT5",
        source_width: 248,
        source_height: 14,
        source_payload_sha256: "14662707B086874DEE4E94FC31E1B716A71A1B1A0758FE3349583598273E9B5F",
        png_size: 553,
        png_sha256: "9E8D7BCB3FB48E80E0352F873A5C611003705AC177634AEAD85433DEC19C41A9",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_ROW_SELECTED_PATH,
        source_path_id: 543,
        source_name: "SSButtonDown",
        source_format: "DXT5",
        source_width: 248,
        source_height: 14,
        source_payload_sha256: "C6C1C24253266E8A6949AC85F9A2DEB7C5E4C76F413B6A75046C6C0109C92AFB",
        png_size: 1_572,
        png_sha256: "0D901120D362A51C1B956AE90895D155A566412938659A03AB19A53AC92B18F3",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_BUTTON_NORMAL_PATH,
        source_path_id: 411,
        source_name: "ff-button-normal",
        source_format: "ARGB32",
        source_width: 11,
        source_height: 26,
        source_payload_sha256: "C53A9876A9CF384F382A3DB16415C5B74D14F6598D08085C65A41515CE646B8D",
        png_size: 629,
        png_sha256: "6205BA81A517B3861976BCDC68B622F83E04C6A005B416F4CECA9862156ACCA5",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_BUTTON_HOVER_PATH,
        source_path_id: 320,
        source_name: "ff-button-hover",
        source_format: "ARGB32",
        source_width: 11,
        source_height: 26,
        source_payload_sha256: "BC9709DC33EDE275AC503B2478661437DF6C9969A2467BA5C930DC10ECF2B2B5",
        png_size: 691,
        png_sha256: "180EADB13B03A8DC5F1A5CCEBEED9193ED932F87715DA6B424EFE20FE5A79BE8",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_BUTTON_ACTIVE_PATH,
        source_path_id: 567,
        source_name: "ff-button-active",
        source_format: "DXT5",
        source_width: 13,
        source_height: 31,
        source_payload_sha256: "4B7525782EE81E52EB95656AE52AC99B504FF175E6960E61DA1A17CE0901C7C8",
        png_size: 722,
        png_sha256: "AB17A0BD908F6C8F533CEFF18080C486B6AB2E4C4224F8291D8E359F04840847",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_RED_NORMAL_PATH,
        source_path_id: 282,
        source_name: "red_button_normal",
        source_format: "DXT5",
        source_width: 20,
        source_height: 25,
        source_payload_sha256: "4E58233EAC0ECCC81C88B51DDDDDFCFA6E2244F2F44B795FEBB439B03D0EDE66",
        png_size: 641,
        png_sha256: "71592F7FE1153EB8B35D381EB01A4414D53228F5A90BDABA1990A98D49507A5D",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_RED_HOVER_PATH,
        source_path_id: 190,
        source_name: "red_button_over",
        source_format: "DXT5",
        source_width: 20,
        source_height: 25,
        source_payload_sha256: "C9F6C0702BFEC07D90D878A0F67C1276FC736A20CC20BA1370BD3C6FD62B1C20",
        png_size: 705,
        png_sha256: "63FD2753C7F014396999A70F8769D430A7594A3F03481E88015DD93ED74E392E",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_SCROLL_TRACK_PATH,
        source_path_id: 374,
        source_name: "scroll_bar",
        source_format: "DXT5",
        source_width: 18,
        source_height: 32,
        source_payload_sha256: "672E04F8E195826928822E1A87B962BF72E461D8EE7222B7428B890989E9FBAA",
        png_size: 345,
        png_sha256: "9868058BDACCC651B3C0B6D1AA27D29587EA9484F453DEFCA0D20908463AB930",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_SCROLL_THUMB_PATH,
        source_path_id: 324,
        source_name: "scroll_Thumb",
        source_format: "DXT5",
        source_width: 13,
        source_height: 15,
        source_payload_sha256: "1571B2EB9FDB26EAED6DFA1E844ECC04A579F79035C4E55AE4FA2596257FFC44",
        png_size: 413,
        png_sha256: "5D9312B400EC1FED440A5C59097AF34E3D16352C9F0C5715E450AE50B4DF5B19",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_SCROLL_UP_PATH,
        source_path_id: 63,
        source_name: "scroll_up",
        source_format: "DXT5",
        source_width: 17,
        source_height: 12,
        source_payload_sha256: "DB2801BE4E08DC505B9C2D833AA76CE1EFD2B94C4C1AC90B45A7539388C97C88",
        png_size: 525,
        png_sha256: "4B1A86AD7CD3D5A885F1E0137769D0DE9F3CA2226572573BF541E14278F660EE",
    },
    ServerSelectionAssetContract {
        path: SERVER_SELECTION_SCROLL_DOWN_PATH,
        source_path_id: 415,
        source_name: "scroll_dn",
        source_format: "DXT5",
        source_width: 17,
        source_height: 12,
        source_payload_sha256: "FFC996316DBE48C96578F67DCC2804014EC18EB4B011A4EAA680902AC8612067",
        png_size: 499,
        png_sha256: "79403866BF80A0EECFEBEFC065653420E8EA8C1DD659B1A8A8B330B0AFDF8691",
    },
];
