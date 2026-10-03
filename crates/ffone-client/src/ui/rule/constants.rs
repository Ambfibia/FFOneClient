use super::*;

pub const RULE_UI_SOURCE_BUILD: &str = "retrobution-20260613";

pub const RULE_UI_ROOT_FONT_EXTERNAL_FILE_ID: i64 = 1;

pub const RULE_UI_SCREEN_PIVOT_CENTER_VALUE: i32 = 4;

pub const RULE_UI_ROOT_INITIALLY_ACTIVE: bool = false;

pub const RULE_UI_MAIN_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";

pub const RULE_UI_TUTORIAL_SHA256: &str =
    "49A684FF4236848D0B882A5D725FFBE99D5CFB5D2090DC8705350E97DD3FD024";

pub const RULE_UI_CHARACTER_CREATION_SHA256: &str =
    "78785925E716027DE4BEC897C402C352BB411B7D627DE1783E6FBDA8BF34E59E";

pub const RULE_UI_ICONS_SHA256: &str =
    "A05602D6E96E2E74ECAD207F42E519605259434210DE8DA19E422B30D642E544";

pub const RULE_UI_PARITY_CAVEAT: &str = "The clean serialized root, component, skin, two RulesTable pages, semantic PNG assets, production GameFrame routing, key-first EN/RU copy, input gates, random-button sound contract, and exact 1264x681 geometry are represented. A normalized clean-primary GPU golden comparison remains acceptance work.";

pub const RULE_UI_VEHICLE_IMAGE_PATHS: [&str; 4] = [
    "ui/en/rule/vehicle-1.png",
    "ui/en/rule/vehicle-2.png",
    "ui/en/rule/vehicle-3.png",
    "ui/en/rule/vehicle-4.png",
];

pub const RULE_UI_COMBINE_IMAGE_PATHS: [&str; 4] = [
    "ui/en/rule/combine-1.png",
    "ui/en/rule/combine-2.png",
    "ui/en/rule/combine-3.png",
    "ui/en/rule/combine-4.png",
];

pub const RULE_UI_IMAGE_PATHS: [&str; 16] = [
    RULE_UI_PANEL_BACK_PATH,
    RULE_UI_RULE_BACK_PATH,
    RULE_UI_VEHICLE_IMAGE_PATHS[0],
    RULE_UI_VEHICLE_IMAGE_PATHS[1],
    RULE_UI_VEHICLE_IMAGE_PATHS[2],
    RULE_UI_VEHICLE_IMAGE_PATHS[3],
    RULE_UI_COMBINE_IMAGE_PATHS[0],
    RULE_UI_COMBINE_IMAGE_PATHS[1],
    RULE_UI_COMBINE_IMAGE_PATHS[2],
    RULE_UI_COMBINE_IMAGE_PATHS[3],
    RULE_UI_BACK_NORMAL_PATH,
    RULE_UI_BACK_HOVER_PATH,
    RULE_UI_NAV_NORMAL_PATH,
    RULE_UI_NAV_HOVER_PATH,
    RULE_UI_CLOSE_NORMAL_PATH,
    RULE_UI_CLOSE_HOVER_PATH,
];

pub const RULE_UI_JEFFE_12_FONT_SIZE: f32 = 10.0;

pub const RULE_UI_JEFFE_16_FONT_SIZE: f32 = 14.0;

pub const RULE_UI_CHALET_SMALL_FONT_SIZE: f32 = 12.0;

pub const RULE_UI_CYAN_TEXT_COLOR: [f32; 4] = [0.8, 1.0, 1.0, 1.0];

pub const RULE_UI_BLUE_TEXT_COLOR: [f32; 4] = [0.0, 0.2, 0.4, 1.0];

pub const RULE_UI_BACK_NORMAL_TEXT_COLOR: [f32; 4] = [0.898_039_2, 0.898_039_2, 0.898_039_2, 1.0];

pub const RULE_UI_DISABLED_ALPHA: f32 = 0.5;

pub const RULE_UI_BACK_LABEL_KEY: &str = "BACK";

pub const RULE_UI_PREVIOUS_LABEL_KEY: &str = "PREVIOUS";

pub const RULE_UI_NEXT_LABEL_KEY: &str = "NEXT";

pub const RULE_UI_VEHICLE_STRINGS: [&str; 7] = [
    "WHAT ARE VEHICLES?",
    "Welcome to Kevin's Hot Rod Rentals! ",
    "Who are we?",
    "Kevin's Hot Rod Rentals wants to put you in a new vehicle right now! Vehicles are a speedy form of transportation to help you get around the world faster.",
    "MORE ABOUT VEHICLES",
    "Vehicles are available for rental only, not for purchase. Detailed rental information is available on the vehicle's info card. When a vehicle's rental period expires, it disappears from your inventory.\n\nYou can equip a vehicle the same way you equip armor, accessories and weapons. Simply go to MY STUFF and select the vehicle you would like to equip, then click HOP ON. To mount and dismount your vehicle in the world, hit the \"V\" key.",
    "Remember: at Kevin's Hot Rod Rentals, all the vehicles go to eleven!",
];

pub const RULE_UI_COMBINING_STRINGS: [&str; 7] = [
    "WHAT IS COMBINING?",
    "Welcome to Croc Pot Catering Company!",
    "Who are we?",
    "The Croc Pot Catering Company creates delicious new combinations from your armor and weapons. It's simple: from MY STUFF, select the item with the STYLE that you want, then choose the item with the STATS you want. We'll throw them into our special Croc Pot and spit out a new item for you. Sound cool? It is! ",
    "RULES TO REMEMBER",
    "\n     * The items you wish to combine must be of the\n       same type. \n \n     * An item for the opposite gender cannot go in   \n       the STYLE slot.\n\n     * GUIDE items cannot go in the STATS slot.\n\nRemember, combinations are not 100% guaranteed.  If the combination fails, don't worry, you can always try again. But we'll still take your Taros!",
    "Thanks for Choosing Croc Pot Catering Company for Your Custom Item Needs!",
];

pub const RULE_UI_VEHICLE_TEXT_KEYS: [&str; 7] = [
    "ui.rule.vehicle.title",
    "ui.rule.vehicle.intro",
    "ui.rule.vehicle.who_title",
    "ui.rule.vehicle.description",
    "ui.rule.vehicle.more_title",
    "ui.rule.vehicle.details",
    "ui.rule.vehicle.closing",
];

pub const RULE_UI_COMBINING_TEXT_KEYS: [&str; 7] = [
    "ui.rule.combining.title",
    "ui.rule.combining.intro",
    "ui.rule.combining.who_title",
    "ui.rule.combining.description",
    "ui.rule.combining.rules_title",
    "ui.rule.combining.rules",
    "ui.rule.combining.closing",
];

pub const RULE_UI_PAGES: [RulePageSpec; 2] = [
    RulePageSpec {
        id: RulePageId::Vehicle,
        string_start: 1,
        image_start: 8,
        previous: None,
        next: None,
        strings: RULE_UI_VEHICLE_STRINGS,
        images: RULE_UI_VEHICLE_IMAGE_PATHS,
    },
    RulePageSpec {
        id: RulePageId::Combining,
        string_start: 12,
        image_start: 19,
        previous: None,
        next: None,
        strings: RULE_UI_COMBINING_STRINGS,
        images: RULE_UI_COMBINE_IMAGE_PATHS,
    },
];
