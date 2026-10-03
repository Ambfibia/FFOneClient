use super::*;

pub const CHARACTER_SELECTION_BACKGROUND_SPEED: f32 = 100.0;

pub const CHARACTER_SELECTION_PRIMARY_MAIN_UNITY3D_BYTES: u64 = 7_000_415;

pub const CHARACTER_SELECTION_PRIMARY_MAIN_UNITY3D_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const CHARACTER_SELECTION_PRIMARY_CREATION_RESOURCE_BYTES: u64 = 8_974_798;

pub const CHARACTER_SELECTION_PRIMARY_CREATION_RESOURCE_SHA256: &str =
    "78785925E716027DE4BEC897C402C352BB411B7D627DE1783E6FBDA8BF34E59E";

pub const CHARACTER_SELECTION_PRIMARY_PLAYER_RESOURCE_BYTES: u64 = 14_544_152;

pub const CHARACTER_SELECTION_PRIMARY_PLAYER_RESOURCE_SHA256: &str =
    "FE050AF079AF43458A79CB27284A243F532AD56BB79E3C526C44E313C1A72930";

pub const CHARACTER_SELECTION_GUI_DEPTH: i32 = 10;

pub const CHARACTER_SELECTION_JEFFE_14_FONT_SIZE: f32 = 12.0;

pub const CHARACTER_SELECTION_JEFFE_12_FONT_SIZE: f32 = 10.0;

pub const CHARACTER_SELECTION_JEFFE_16_FONT_SIZE: f32 = 14.0;

pub const CHARACTER_SELECTION_CHALET_SMALL_FONT_SIZE: f32 = 12.0;

pub const CHARACTER_SELECTION_CHALET_REGULAR_FONT_SIZE: f32 = 14.0;

pub const CHARACTER_SELECTION_STYLE_PADDING: [f32; 4] = [10.0, 6.0, 4.0, 6.0];

pub const CHARACTER_SELECTION_CANCEL_PADDING: [f32; 4] = [0.0, 0.0, 4.0, 7.0];

/// Every reached clean style serializes `m_ContentOffset.y = 0`. These remain
/// style-local so a future replacement-font correction cannot silently move
/// unrelated controls. Current EN/RU GPU bounds require no extra translation.
pub const CHARACTER_SELECTION_TRANSPARENT2_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_TRANSPARENT3_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_CHAR_NAME_UP_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_CHAR_NAME_DOWN_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_CHAR_LEVEL_UP_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_DELETE_TEXT_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_AVATAR_NAME_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_ENTER_GAME_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_CANCEL_Y_OFFSET: f32 = 0.0;

pub const CHARACTER_SELECTION_PREVIEW_ROTATION_SPEED_DEGREES: f32 = 100.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterSelectionImageDrawMode {
    Stretch,
    NineSlice5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacterSelectionImageSpec {
    pub role: &'static str,
    pub path: &'static str,
    pub source_asset: &'static str,
    pub source_path_id: i64,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub sha256: &'static str,
    pub draw_mode: CharacterSelectionImageDrawMode,
}

/// Exact reached images owned by `CnGuiCharSelection`. Paths 2..129 for the
/// rotating strip live in the clean CharacterCreation AssetBundle and are
/// pinned separately by `CharacterLocationBackground::source_path_ids`.
pub const CHARACTER_SELECTION_IMAGE_SPECS: [CharacterSelectionImageSpec; 27] = [
    selection_image_spec!(
        "chrome",
        CHARACTER_SELECTION_CHROME_PATH,
        "CustomAssetBundle-bd5f53480423447d7bcaed95cb2a96c8",
        CHARACTER_SELECTION_CHROME_SOURCE_PATH_ID,
        1024,
        768,
        268241,
        "D1F61D4B2048FCF11E4A334597589637F86068FB3F01BEBB7B9C91872541AEBE",
        Stretch
    ),
    selection_image_spec!(
        "slot-empty",
        CHARACTER_SELECTION_SLOT_EMPTY_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_SLOT_EMPTY_SOURCE_PATH_ID,
        413,
        67,
        1423,
        "635536C4792D1C115EA30C2E687F05085BF16D4B15337CC0E67C505E4105632D",
        NineSlice5
    ),
    selection_image_spec!(
        "slot-normal",
        CHARACTER_SELECTION_SLOT_NORMAL_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_SLOT_NORMAL_SOURCE_PATH_ID,
        413,
        67,
        1397,
        "D0F247EA0EEDC3D486F310493E2E9FFA7BDEDC3553B748599964DCBAC12224A4",
        NineSlice5
    ),
    selection_image_spec!(
        "slot-over",
        CHARACTER_SELECTION_SLOT_OVER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_SLOT_OVER_SOURCE_PATH_ID,
        413,
        67,
        1738,
        "DA89D7B146ECD44ACBEDE114A7033D7C57A5884CA933C4D476BA2ED3DF4F6F45",
        NineSlice5
    ),
    selection_image_spec!(
        "slot-subscription",
        CHARACTER_SELECTION_SLOT_LOCKED_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_SLOT_LOCKED_SOURCE_PATH_ID,
        413,
        68,
        1470,
        "C456AC8321A2C8902CA001BAC5F7FB88F1DC3A69DA05340A96122F8AAC1C0DD8",
        NineSlice5
    ),
    selection_image_spec!(
        "slot-lock",
        CHARACTER_SELECTION_LOCK_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_LOCK_SOURCE_PATH_ID,
        32,
        36,
        1381,
        "0FDBBE95948ABE835DDE9654FABC420933AC970078714C5FD0B62EC6CC750B4A",
        NineSlice5
    ),
    selection_image_spec!(
        "enter-normal",
        CHARACTER_SELECTION_ENTER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_ENTER_SOURCE_PATH_ID,
        325,
        57,
        9712,
        "C6DF29087409226EC67E3B6A9C35AD12670C219FE1AACDB073EA04AEA4573079",
        Stretch
    ),
    selection_image_spec!(
        "enter-over",
        CHARACTER_SELECTION_ENTER_OVER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_ENTER_OVER_SOURCE_PATH_ID,
        325,
        57,
        9226,
        "E17A93A8C23B6DC4FCC276C618263E680A0351FAE05A1E2DD30556CCE1D7685D",
        Stretch
    ),
    selection_image_spec!(
        "fullscreen-normal",
        CHARACTER_SELECTION_FULLSCREEN_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_FULLSCREEN_SOURCE_PATH_ID,
        37,
        31,
        1136,
        "EEA70205B517DF96FF012E31D1A23D540E3572FFDBA7BACA11B9F5B2D694B7EE",
        Stretch
    ),
    selection_image_spec!(
        "fullscreen-over",
        CHARACTER_SELECTION_FULLSCREEN_OVER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_FULLSCREEN_OVER_SOURCE_PATH_ID,
        37,
        31,
        1219,
        "79A6B140C4009CCDFC3F5CCB4515A1B742793EDBA8A062C2DA9E70AF42076446",
        Stretch
    ),
    selection_image_spec!(
        "windowed-normal",
        CHARACTER_SELECTION_WINDOWED_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_WINDOWED_SOURCE_PATH_ID,
        37,
        31,
        1221,
        "D258F23B1BF2B9D645809D305C4ADED8F983A065D65743F4016ADD7FD08ECE25",
        Stretch
    ),
    selection_image_spec!(
        "windowed-over",
        CHARACTER_SELECTION_WINDOWED_OVER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_WINDOWED_OVER_SOURCE_PATH_ID,
        37,
        31,
        1202,
        "188B59D3A7DBB38E247D054585C3029F4608AECFEB7088E07582AE33AA6A931D",
        Stretch
    ),
    selection_image_spec!(
        "music-normal",
        CHARACTER_SELECTION_MUSIC_TOGGLE_ON_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_MUSIC_NORMAL_SOURCE_PATH_ID,
        86,
        30,
        2944,
        "7DDEA3C569C14E4F1F88FEE928AFCFB15495FFF5A68408B49D67390D6A5548D7",
        Stretch
    ),
    selection_image_spec!(
        "music-on",
        CHARACTER_SELECTION_MUSIC_TOGGLE_OFF_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_MUSIC_ON_SOURCE_PATH_ID,
        86,
        30,
        2423,
        "E099361854C4F58DD08B6F2F8274B943F045A1878A9C4D27A1820910643EE0B3",
        Stretch
    ),
    selection_image_spec!(
        "red-normal",
        CHARACTER_SELECTION_RED_BUTTON_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_RED_NORMAL_SOURCE_PATH_ID,
        20,
        25,
        641,
        "71592F7FE1153EB8B35D381EB01A4414D53228F5A90BDABA1990A98D49507A5D",
        NineSlice5
    ),
    selection_image_spec!(
        "red-over",
        CHARACTER_SELECTION_RED_BUTTON_OVER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_RED_OVER_SOURCE_PATH_ID,
        20,
        25,
        705,
        "63FD2753C7F014396999A70F8769D430A7594A3F03481E88015DD93ED74E392E",
        NineSlice5
    ),
    selection_image_spec!(
        "blue-normal",
        CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_BLUE_NORMAL_SOURCE_PATH_ID,
        20,
        25,
        587,
        "6A841912FB35EB3C6EECEAF24176ECD3E1258157F96ED59B481B83F7C022BEF3",
        NineSlice5
    ),
    selection_image_spec!(
        "blue-over",
        CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_BLUE_OVER_SOURCE_PATH_ID,
        20,
        25,
        452,
        "E23A8BB2DBB1A90E785A21F776693E94BEB6AE542D55996231172F4EF3DD8C73",
        NineSlice5
    ),
    selection_image_spec!(
        "delete-backdrop",
        CHARACTER_SELECTION_DELETE_BACKDROP_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_DELETE_BACKDROP_SOURCE_PATH_ID,
        23,
        26,
        229,
        "82E552166253C5A4EB7707DAFC34BC9FAE92C80D7B751B766D0B3C0B1D8CEE9E",
        Stretch
    ),
    selection_image_spec!(
        "delete-window",
        CHARACTER_SELECTION_DELETE_WINDOW_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_DELETE_WINDOW_SOURCE_PATH_ID,
        524,
        162,
        5240,
        "14A24D9D4918E617F7CC2CEE88CFED3C8EFB84AAB69C7E6C280E3AE629839727",
        Stretch
    ),
    selection_image_spec!(
        "cancel-normal",
        CHARACTER_SELECTION_CANCEL_NORMAL_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_CANCEL_NORMAL_SOURCE_PATH_ID,
        112,
        26,
        540,
        "3284505C50A008F764460133E323935C4E9E70866A92063105757A801AC9F68C",
        NineSlice5
    ),
    selection_image_spec!(
        "rotate-left-normal",
        CHARACTER_SELECTION_ROTATE_LEFT_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_ROTATE_LEFT_SOURCE_PATH_ID,
        51,
        96,
        4694,
        "DAE18BA9AE79065647768CDED38486F4249642A6BA6E5D61E93FFF4200637E99",
        NineSlice5
    ),
    selection_image_spec!(
        "rotate-left-over",
        CHARACTER_SELECTION_ROTATE_LEFT_OVER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_ROTATE_LEFT_OVER_SOURCE_PATH_ID,
        51,
        93,
        3119,
        "44D2DF960C0DF98BF987735E9D1CABCCD7EB334AEB96FFB3861CE9179CB29F81",
        NineSlice5
    ),
    selection_image_spec!(
        "rotate-right-normal",
        CHARACTER_SELECTION_ROTATE_RIGHT_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_ROTATE_RIGHT_SOURCE_PATH_ID,
        55,
        95,
        4739,
        "BB73ED53609DF934C65A6F0E019037310DC2EF0A9C1B042F851C41A210C20440",
        NineSlice5
    ),
    selection_image_spec!(
        "rotate-right-over",
        CHARACTER_SELECTION_ROTATE_RIGHT_OVER_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_ROTATE_RIGHT_OVER_SOURCE_PATH_ID,
        51,
        97,
        3030,
        "BBDBA9210910A9CB6933FEE361B9FA2C0E28356DD2ED71AA4C84745BB304ABBD",
        NineSlice5
    ),
    selection_image_spec!(
        "disk-back",
        CHARACTER_SELECTION_DISK_BACK_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_DISK_BACK_SOURCE_PATH_ID,
        69,
        24,
        1702,
        "0D1DBA6895EDEAF1E24F0D4ADBEB12DC4DD17C857EA78EE1FE8BFFA4D0687B92",
        NineSlice5
    ),
    selection_image_spec!(
        "disk-front",
        CHARACTER_SELECTION_DISK_FRONT_PATH,
        CHARACTER_SELECTION_PRIMARY_ASSET_FILE,
        CHARACTER_SELECTION_DISK_FRONT_SOURCE_PATH_ID,
        69,
        23,
        2364,
        "071434E27EE1CF716F24B141866438567E6077145C55EA9F7FD2BBFB761CFB01",
        NineSlice5
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacterSelectionBackgroundSpec {
    pub source_path_id: i64,
    pub bytes: u64,
    pub sha256: &'static str,
}

/// Location order is `Future, Suburbs, Downtown, Wilds, Darklands`; frame
/// order is the exact clean `1..=10` `AssetLoader.Load` sequence.
pub const CHARACTER_SELECTION_BACKGROUND_SPECS: [[CharacterSelectionBackgroundSpec; 10]; 5] = [
    [
        selection_background_spec!(
            63,
            242749,
            "C1894DE1A9D2E483A0EC09D95744A07A16392F70319901590D05017EB4587BE1"
        ),
        selection_background_spec!(
            37,
            250274,
            "63F51B049F25991998BBD734409AC2A33226DE0E1AB952E3E473150839ACD028"
        ),
        selection_background_spec!(
            59,
            212210,
            "907081AB42F9DF871827C7191AA8311F63C2AAE0B122AE91D6CF39774708F07B"
        ),
        selection_background_spec!(
            30,
            300259,
            "D261C955E58F72D42A6064D2C9380867054C3BA6DA9A25B1B9C19C88F22565DD"
        ),
        selection_background_spec!(
            102,
            212147,
            "6944F62CECC5B9616A0A50F880AFD8BC52E45FA348D7B20495C4582B03019EBB"
        ),
        selection_background_spec!(
            118,
            345703,
            "C34EDDBC2347E2AB0AE1757E83943D9A9659B62B16CEE9CA89E262B5EF30C3C7"
        ),
        selection_background_spec!(
            73,
            248403,
            "EA956D5D4A2482ECC7FE1FC174144CA233A6C3391C202996777FF7059E509411"
        ),
        selection_background_spec!(
            117,
            275801,
            "173021DB5FB0151167288B7B3C54E16001FE208278E1A05D8718F1118C925EB7"
        ),
        selection_background_spec!(
            19,
            216573,
            "EAFC0C96AB96E14E21A436AB34034626D8271F2C7D04AEBFDDC699A7C29E9090"
        ),
        selection_background_spec!(
            38,
            269356,
            "A96C5E46EB9B700E0D17C8EA5AEA260557D5BB0174AB4A1D8862575DC228B61D"
        ),
    ],
    [
        selection_background_spec!(
            129,
            207012,
            "257F0CAA95DB0FE7F7DFBCFAA2CA60200F424F43F14FD251DC5E9CF601C07C17"
        ),
        selection_background_spec!(
            104,
            267316,
            "CC94156EB77E981C3AD18204199EE11403AA5C6ACAD9E2EB33DB24CC106342EE"
        ),
        selection_background_spec!(
            32,
            315332,
            "53494A0ACA95BB8F4B52028B9C6425C29D44A70E2CAF311AAC60BF059061C9C7"
        ),
        selection_background_spec!(
            3,
            229472,
            "9EB7D013468A8091DB60A7625F8075853566183E4130353CE0F8C09FDB96B234"
        ),
        selection_background_spec!(
            62,
            246054,
            "A205CB2A036783B2C2501EAB9BD6F639153C7331252A80310D6115604920969A"
        ),
        selection_background_spec!(
            92,
            240019,
            "85868B13F6470D4F27CD5D97496A0098615E4BFC7FA27BCC9BCF864E49FC8833"
        ),
        selection_background_spec!(
            79,
            212942,
            "7C8338EED4EF207A264259D30BFA72018FF1415BE11D3EA530CB8DA716174D6D"
        ),
        selection_background_spec!(
            110,
            277425,
            "923060FBCA58BFB3BFA934DB55AB98278E9CEAF906BE5DA1FBD6546757F2E938"
        ),
        selection_background_spec!(
            34,
            268606,
            "32BFDFC0FFB09E615A6D65E9A32F7709DC1A58C2988721AFBE0FD897EEFBE2D2"
        ),
        selection_background_spec!(
            5,
            226134,
            "EE42B27A144D0740C510D1189A568E2CB9F18BD85860AD1B3FA485BCB595F051"
        ),
    ],
    [
        selection_background_spec!(
            74,
            207818,
            "3B78E96332C344620DA514451419696174B47DD5FA26C82A5F066208AEF00549"
        ),
        selection_background_spec!(
            66,
            260277,
            "31B376620592748C82364E510755D8DFEAC67B38630893B07A942AEA847F73DD"
        ),
        selection_background_spec!(
            109,
            250883,
            "74AE15246EC8420719624EE5F1218A19D28850225E1DB55AF6614DA9314B91F7"
        ),
        selection_background_spec!(
            112,
            203126,
            "CA91E60147A71D767619B417E8F6F25D5C0F032B4C0D4EE809C906185666C829"
        ),
        selection_background_spec!(
            49,
            192481,
            "556F08D87926945E026BEFD87EA5E71393836364DFEB57AF658B678485CCF3B1"
        ),
        selection_background_spec!(
            99,
            233669,
            "BDF98E4943C8EEC71C6FDE35C501462E2B19D83FF217D6FD4D781D98DFE892AA"
        ),
        selection_background_spec!(
            80,
            143719,
            "690E0ADAD10C69B485FAEAF1E2E00142C5010126C2CA6992183028A69DD0B669"
        ),
        selection_background_spec!(
            36,
            195596,
            "A5BAC83729CDC39B1ADE2F312CAF0F739BE9637264BC421D85668A4264169038"
        ),
        selection_background_spec!(
            101,
            285094,
            "B58C546DF0FDABBE6C763E5DE4586ACE487ED79407BE6536FB233B7F0FEE8AD5"
        ),
        selection_background_spec!(
            100,
            193888,
            "48140B12D43F4231D5BADD3DE34245CFB0BB28EF1A9B04CA6A7A8D1533237AED"
        ),
    ],
    [
        selection_background_spec!(
            95,
            216677,
            "D85E611B9CCA2A5F558D5559FA2321B1513680E0D649513CF9EF6B73FAF82E09"
        ),
        selection_background_spec!(
            85,
            266314,
            "B41F3C6D588EE75E4EF4D7B5D151972E14FE2E64C2DA97A54E9E891A1F284EF9"
        ),
        selection_background_spec!(
            82,
            237834,
            "FA3FDAC5D3B58FE544EF96541A8BAE912E977C40D449F028064E6F716328CBF1"
        ),
        selection_background_spec!(
            58,
            224660,
            "D099F8BEF63C6CBBCB77AD1CC7039BB97C28C219A4A02CF9BA973F48F14F2026"
        ),
        selection_background_spec!(
            41,
            221687,
            "C83B68A3C2D5928600C1BE33B96E094595D0BDE3403FDCDE3B3C058530F902EE"
        ),
        selection_background_spec!(
            12,
            223315,
            "1267EEDC9F7EBAFE8C9B80B8BFCB7CCE058EB24A4E0EA6ABA101D77A939450C3"
        ),
        selection_background_spec!(
            55,
            251874,
            "1322F9D2FF4ECF97FBD4B816EAC7E4D0D7622B3F14ADD3C858F3AC2462DD2C08"
        ),
        selection_background_spec!(
            22,
            306133,
            "7BA312FCA8B1A8266AF0FBA87E050F1D8E98F45C047EB165EDB9D46077379689"
        ),
        selection_background_spec!(
            14,
            231884,
            "12390A5298A6F70E792B2FB4F974567ABE2E26955F65263F69777B1A7D402B3C"
        ),
        selection_background_spec!(
            67,
            191153,
            "27BC8940FD261AF63D6553CA95B4FC9E9083107EE341E4513AE9FE565AE0ABF4"
        ),
    ],
    [
        selection_background_spec!(
            51,
            294414,
            "14752E77D7621411F97E8A1D38D1BF4DFF53B3DD3C3ED8A90F1D90EA3AA5D841"
        ),
        selection_background_spec!(
            107,
            286970,
            "9123A41CE0F410C8CE4C9583488107C1D3D358DAFC501A21D86C1F5FF4309683"
        ),
        selection_background_spec!(
            2,
            315556,
            "2FE462F98F26D50751157880A90D55472BE5869C7C871D4D74083A599EE7E302"
        ),
        selection_background_spec!(
            48,
            159409,
            "13254E44FDB6B8E5BD9594E907AA11E92D7E5FF4BAABF1F3B01C4F2E35394054"
        ),
        selection_background_spec!(
            111,
            200283,
            "841CA4987754251A0B9FE4C79B23B3D519EEC2C2950138D47F9ACCB4FED4D4DF"
        ),
        selection_background_spec!(
            7,
            325597,
            "3CA867A152802C1ABA28B59655B7D6102E597E5D9A927DD94AD72BA4DF6E7547"
        ),
        selection_background_spec!(
            6,
            328919,
            "4CF015347F1B7A224F3A6B793B1AEC9125140294C247693E1A6C659DFD009E78"
        ),
        selection_background_spec!(
            106,
            331523,
            "3BB03C68B1EE1D5858AF6CBDAE71928FAE901798F6456F64D483C5C9AA7F70A0"
        ),
        selection_background_spec!(
            88,
            302168,
            "98FEE36E2C6ED4D170F85C22581C76D66BDC4913CB42B0E61D6D93CA4D7586C4"
        ),
        selection_background_spec!(
            16,
            300760,
            "B0B9E9F8F01C515A08C552F6CEEBC55E98AF18B4380CE359A0D56ECC598EEAF2"
        ),
    ],
];

/// Reserved top overlay range for blocking character UI.
///
/// The central preview and four portrait cameras occupy orders immediately
/// above the default legacy UI camera. Keeping modals sixteen orders later
/// prevents a portrait or future preview pass from drawing through them.
pub const CHARACTER_SELECTION_MODAL_CAMERA_ORDER: isize = GAMEPLAY_UI_CAMERA_ORDER + 16;

/// The four direct-to-window portrait cameras render after the base IMGUI
/// pass. `DiskFront` is the foreground half of the source portrait ring, so it
/// needs one later UI pass instead of sharing the base camera with `DiskBack`.
pub const CHARACTER_SELECTION_PORTRAIT_OVERLAY_CAMERA_ORDER: isize = GAMEPLAY_UI_CAMERA_ORDER + 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterSelectionFontRole {
    Jeffe,
    Chalet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterSelectionTextAnchor {
    MiddleLeft,
    MiddleCenter,
}

/// Exact text-bearing GUIStyles used by `CnGuiCharSelection`.
///
/// The clean `FusionFallCharCreation` skin contains many character-creation
/// controls, but this enum deliberately includes only styles reached by the
/// character-selection draw path.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum CharacterSelectionTextStyle {
    Transparent2,
    Transparent3,
    CharNameUp,
    CharNameDown,
    CharLevelUp,
    DeleteText,
    AvatarName,
    QuitButton,
    CreateButton,
    EnterGame,
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterSelectionTextStyleSpec {
    pub source_font_path_id: i64,
    pub font_role: CharacterSelectionFontRole,
    pub font_size: f32,
    pub line_height: f32,
    /// Left, right, top, bottom in clean IMGUI pixels.
    pub padding: [f32; 4],
    pub anchor: CharacterSelectionTextAnchor,
    pub normal_color: [f32; 4],
    pub word_wrap: bool,
    pub y_offset: f32,
}
