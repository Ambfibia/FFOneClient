
pub const RACE_RANK_MODE_OBJECT_NAME: &str = "RaceRankMode";

pub const RACE_RANK_SERIALIZED_OBJECT_NAME: &str = "RaceRank";

pub const RACE_RANK_CAMERA_OBJECT_NAME: &str = "rankNPCCamera";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceRankSerializedStyleSource {
    pub clean_style: &'static str,
    pub texture_path_id: i64,
}

pub const RACE_RANK_SERIALIZED_STYLE_SOURCES: [RaceRankSerializedStyleSource; 25] = [
    RaceRankSerializedStyleSource {
        clean_style: "titleback",
        texture_path_id: 317,
    },
    RaceRankSerializedStyleSource {
        clean_style: "locationback",
        texture_path_id: 88,
    },
    RaceRankSerializedStyleSource {
        clean_style: "prevbut.normal",
        texture_path_id: 38,
    },
    RaceRankSerializedStyleSource {
        clean_style: "prevbut.hover",
        texture_path_id: 304,
    },
    RaceRankSerializedStyleSource {
        clean_style: "nextbut.normal",
        texture_path_id: 201,
    },
    RaceRankSerializedStyleSource {
        clean_style: "nextbut.hover",
        texture_path_id: 119,
    },
    RaceRankSerializedStyleSource {
        clean_style: "locationbar (serialized, unused)",
        texture_path_id: 675,
    },
    RaceRankSerializedStyleSource {
        clean_style: "locationselbar",
        texture_path_id: 284,
    },
    RaceRankSerializedStyleSource {
        clean_style: "tabbar",
        texture_path_id: 247,
    },
    RaceRankSerializedStyleSource {
        clean_style: "rightback",
        texture_path_id: 603,
    },
    RaceRankSerializedStyleSource {
        clean_style: "today",
        texture_path_id: 30,
    },
    RaceRankSerializedStyleSource {
        clean_style: "todaytab",
        texture_path_id: 251,
    },
    RaceRankSerializedStyleSource {
        clean_style: "ranktodaytabover",
        texture_path_id: 555,
    },
    RaceRankSerializedStyleSource {
        clean_style: "week",
        texture_path_id: 525,
    },
    RaceRankSerializedStyleSource {
        clean_style: "weektab",
        texture_path_id: 354,
    },
    RaceRankSerializedStyleSource {
        clean_style: "weektabover",
        texture_path_id: 72,
    },
    RaceRankSerializedStyleSource {
        clean_style: "myrankback",
        texture_path_id: 522,
    },
    RaceRankSerializedStyleSource {
        clean_style: "leftback",
        texture_path_id: 332,
    },
    RaceRankSerializedStyleSource {
        clean_style: "npcicon",
        texture_path_id: 375,
    },
    RaceRankSerializedStyleSource {
        clean_style: "built-in TextArea.normal",
        texture_path_id: 629,
    },
    RaceRankSerializedStyleSource {
        clean_style: "rankhighlight",
        texture_path_id: 576,
    },
    RaceRankSerializedStyleSource {
        clean_style: "tabbackbox",
        texture_path_id: 549,
    },
    RaceRankSerializedStyleSource {
        clean_style: "helpbut.normal",
        texture_path_id: 245,
    },
    RaceRankSerializedStyleSource {
        clean_style: "helpbut.hover",
        texture_path_id: 83,
    },
    RaceRankSerializedStyleSource {
        clean_style: "Inventory skin closebut.normal/hover",
        texture_path_id: 105,
    },
];
