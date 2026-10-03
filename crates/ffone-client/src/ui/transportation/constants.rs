use super::*;

pub const TRANSPORTATION_ESCAPE_KEY_ID: u8 = 4;

pub const TRANSPORTATION_WARP_DELAY_SECONDS: f32 = 1.5;

pub const TRANSPORTATION_GAME_CONDITION_COOLDOWN: i32 = 14;

pub const TRANSPORTATION_WARP_EFFECT_ID: i32 = 394;

/// `icon_box` serializes padding 2 on all sides; the 64x64 GUI.Box content is
/// therefore the exact inner 60x60 Rect, independently of its 5 px border.
pub const TRANSPORTATION_ICON_CONTENT_PADDING: f32 = 2.0;

pub const TRANSPORTATION_UNREGISTERED_MESSAGE_ID: i32 = 161;

pub const TRANSPORTATION_INSUFFICIENT_TAROS_MESSAGE_ID: i32 = 116;

pub const TRANSPORTATION_FAILURE_MESSAGE_ID_8: i32 = 162;

pub const TRANSPORTATION_GENERIC_FAILURE_MESSAGE_ID: i32 = 163;

/// Clean `cnTrans` has no clickable or rendered zone tabs. Map-zone changes
/// are automatic, based on the player or selected item destination.
pub const TRANSPORTATION_HAS_ZONE_TABS: bool = false;

/// Type-1 draws a live 90×90 NPC camera. The production gameplay adapter binds
/// its render target to this shell's typed camera slot.
pub const TRANSPORTATION_TYPE1_CAMERA_BINDING_OWNED_EXTERNALLY: bool = true;

pub const TRANSPORTATION_TRANSPORT_ICON_PATHS: [&str; 33] = [
    "icons/transport/transport_00.png",
    "icons/transport/transport_01.png",
    "icons/transport/transport_02.png",
    "icons/transport/transport_03.png",
    "icons/transport/transport_04.png",
    "icons/transport/transport_05.png",
    "icons/transport/transport_06.png",
    "icons/transport/transport_07.png",
    "icons/transport/transport_08.png",
    "icons/transport/transport_09.png",
    "icons/transport/transport_10.png",
    "icons/transport/transport_11.png",
    "icons/transport/transport_12.png",
    "icons/transport/transport_13.png",
    "icons/transport/transport_14.png",
    "icons/transport/transport_15.png",
    "icons/transport/transport_16.png",
    "icons/transport/transport_17.png",
    "icons/transport/transport_18.png",
    "icons/transport/transport_19.png",
    "icons/transport/transport_20.png",
    "icons/transport/transport_21.png",
    "icons/transport/transport_22.png",
    "icons/transport/transport_23.png",
    "icons/transport/transport_24.png",
    "icons/transport/transport_25.png",
    "icons/transport/transport_26.png",
    "icons/transport/transport_27.png",
    "icons/transport/transport_28.png",
    "icons/transport/transport_29.png",
    "icons/transport/transport_30.png",
    "icons/transport/transport_31.png",
    "icons/transport/transport_32.png",
];

pub const RETROBUTION_TRANSPORTATION_MAIN_ARCHIVE_BYTES: u64 = 7_000_415;

pub const RETROBUTION_TRANSPORTATION_MAIN_ARCHIVE_SHA256: &str =
    "59788201962b6a1737b114486c361fe74eef69f507d1d125ca3171377eec602f";

pub const RETROBUTION_TRANSPORTATION_TABLE_ARCHIVE_BYTES: u64 = 784_963;

pub const RETROBUTION_TRANSPORTATION_TABLE_ARCHIVE_SHA256: &str =
    "6d4cea151152e2ab75b7d16590bda00fba172600a163e0b3a5318564df0d7e3b";

/// Vector replacement calibration for clean fixed-raster `JEFFE___14`.
/// The source Font has `m_LineSpacing = 13.710000038146973`; rendering the
/// approved Cyrillic-capable JEFFE outline at 12 px preserves its visible
/// glyph bounds without changing any clean Rect.
pub const TRANSPORTATION_JEFFE_14_FONT_SIZE: f32 = 12.0;

/// Vector replacement calibration for clean fixed-raster `JEFFE___16`.
/// The source Font has `m_LineSpacing = 16.45199966430664`.
pub const TRANSPORTATION_JEFFE_16_FONT_SIZE: f32 = 14.0;

/// The approved JEFFE outline shares the clean font's top-bearing at the
/// calibrated sizes. Keep each reached GUIStyle offset explicit so replacing
/// one source font cannot silently shift the other roles.
pub const TRANSPORTATION_BIGFONT14_REPLACEMENT_Y_OFFSET: f32 = 0.0;

pub const TRANSPORTATION_BIGFONT16_REPLACEMENT_Y_OFFSET: f32 = 0.0;

pub const TRANSPORTATION_RIGHT_TEXT_REPLACEMENT_Y_OFFSET: f32 = 0.0;

pub const RETROBUTION_TRANSPORT_JEFFE_FONT_BLAKE3: &str =
    "338fe408d9f83421ff470a8aa03180b9a160f01197ebdbf1c497c3fb643e3c6d";

pub const RETROBUTION_TRANSPORT_CHALET_FONT_BLAKE3: &str =
    "f38f29fbab9929d55f922ef60dacb0b587c5e5f8baaa06320f671e0f214b3662";

pub const RETROBUTION_TRANSPORT_TABLE_CANONICAL_SHA256: &str =
    "f3236dbb060c9ad2ceb44801aea3bddb6393f9f37cbf9e974c99018909a1957a";

pub const RETROBUTION_NATIVE_TABLE_SET_BYTES: u64 = 43_132_658;

pub const RETROBUTION_NATIVE_TABLE_SET_SHA256: &str =
    "0041ee74a52ffa76a6d34b533218399d136f178d04c9ffa00aad04e69a403a7c";

pub const RETROBUTION_TRANSPORTATION_WARP_LOCATION_COUNT: usize = 14;

pub const RETROBUTION_TRANSPORTATION_BROOM_LOCATION_COUNT: usize = 35;

pub const RETROBUTION_TRANSPORTATION_ICON_COUNT: usize = 34;

pub const RETROBUTION_WORLD_NAME_REGION_COUNT: usize = 96;

/// Exact row order from clean `TableData.resourceFile`, `worldname.asset`
/// (serialized object PathID 8). `ReceivePtDongName` returns the first match.
pub const RETROBUTION_WORLD_NAME_REGIONS: [TransportationRegionProof; 96] = [
    TransportationRegionProof::new(0, 0, 0, 0, "null", "null"),
    TransportationRegionProof::new(4096, 2560, 512, 512, "Candy Cove", "The Suburbs"),
    TransportationRegionProof::new(1024, 1536, 512, 512, "Townsville Park", "Downtown"),
    TransportationRegionProof::new(2560, 512, 512, 512, "Bravo Beach", "Downtown"),
    TransportationRegionProof::new(1536, 1024, 512, 512, "Orchid Bay", "Downtown"),
    TransportationRegionProof::new(1536, 512, 512, 512, "Orchid Bay", "Downtown"),
    TransportationRegionProof::new(2048, 512, 512, 512, "Orchid Bay", "Downtown"),
    TransportationRegionProof::new(1536, 1536, 512, 512, "Marquee Row", "Downtown"),
    TransportationRegionProof::new(512, 6144, 512, 512, "Green Maw", "The Darklands"),
    TransportationRegionProof::new(512, 5632, 512, 512, "Green Maw", "The Darklands"),
    TransportationRegionProof::new(1024, 5632, 512, 512, "Green Maw", "The Darklands"),
    TransportationRegionProof::new(3072, 4608, 512, 512, "Forsaken Valley", "The Darklands"),
    TransportationRegionProof::new(3584, 4608, 512, 512, "Forsaken Valley", "The Darklands"),
    TransportationRegionProof::new(3584, 5120, 512, 512, "Forsaken Valley", "The Darklands"),
    TransportationRegionProof::new(2560, 4608, 512, 512, "Dinosaur Pass", "The Darklands"),
    TransportationRegionProof::new(6144, 4096, 512, 512, "Nowhere", "The Wilds"),
    TransportationRegionProof::new(6656, 4096, 512, 512, "Nowhere", "The Wilds"),
    TransportationRegionProof::new(6656, 5120, 512, 512, "Area 51.5", "The Wilds"),
    TransportationRegionProof::new(6656, 4608, 512, 512, "Nowhere", "The Wilds"),
    TransportationRegionProof::new(4096, 7168, 512, 512, "Monkey Mountain", "The Wilds"),
    TransportationRegionProof::new(4608, 7168, 512, 512, "Monkey Foothills", "The Wilds"),
    TransportationRegionProof::new(5120, 7168, 512, 512, "Monkey Foothills", "The Wilds"),
    TransportationRegionProof::new(5120, 6656, 512, 512, "Monkey Foothills", "The Wilds"),
    TransportationRegionProof::new(2048, 2560, 512, 512, "City Point", "Downtown"),
    TransportationRegionProof::new(2560, 2560, 512, 512, "Galaxy Gardens", "Downtown"),
    TransportationRegionProof::new(2048, 1536, 512, 512, "Marquee Row", "Downtown"),
    TransportationRegionProof::new(2560, 1024, 512, 512, "Bravo Beach", "Downtown"),
    TransportationRegionProof::new(2048, 3072, 512, 512, "Endsville", "The Suburbs"),
    TransportationRegionProof::new(4608, 4608, 382, 85, "Prickly Pines", "The Wilds"),
    TransportationRegionProof::new(4608, 4690, 338, 201, "Prickly Pines", "The Wilds"),
    TransportationRegionProof::new(4608, 4890, 310, 741, "Prickly Pines", "The Wilds"),
    TransportationRegionProof::new(4991, 4608, 129, 85, "Camp Kidney", "The Wilds"),
    TransportationRegionProof::new(4947, 4690, 173, 201, "Camp Kidney", "The Wilds"),
    TransportationRegionProof::new(4919, 4890, 201, 741, "Camp Kidney", "The Wilds"),
    TransportationRegionProof::new(7680, 5632, 512, 512, "Lower Catacombs", "The Wilds"),
    TransportationRegionProof::new(7680, 5120, 512, 512, "Upper Catacombs", "The Wilds"),
    TransportationRegionProof::new(2560, 5120, 512, 512, "Dinosaur Pass", "The Darklands"),
    TransportationRegionProof::new(1536, 3072, 512, 512, "Habitat Homes", "The Suburbs"),
    TransportationRegionProof::new(1536, 2560, 512, 512, "City Hall", "Downtown"),
    TransportationRegionProof::new(2048, 2048, 512, 512, "City Station", "Downtown"),
    TransportationRegionProof::new(2560, 1536, 512, 512, "Morbucks Towers", "Downtown"),
    TransportationRegionProof::new(4096, 3584, 512, 512, "Peach Creek Commons", "The Suburbs"),
    TransportationRegionProof::new(4096, 3072, 512, 512, "Candy Cove", "The Suburbs"),
    TransportationRegionProof::new(3072, 2048, 512, 512, "Steam Alley", "Downtown"),
    TransportationRegionProof::new(2048, 6656, 512, 512, "Dark Glade", "The Darklands"),
    TransportationRegionProof::new(5632, 5120, 512, 512, "Devil's Bluff", "The Wilds"),
    TransportationRegionProof::new(5632, 4608, 512, 512, "Devil's Canyon", "The Wilds"),
    TransportationRegionProof::new(3072, 3072, 512, 512, "Genius Grove", "The Suburbs"),
    TransportationRegionProof::new(3072, 5120, 512, 512, "Forsaken Valley", "The Darklands"),
    TransportationRegionProof::new(2048, 6144, 512, 512, "Huntor's Crest", "The Darklands"),
    TransportationRegionProof::new(1024, 2048, 512, 512, "Townsville Park", "Downtown"),
    TransportationRegionProof::new(4608, 4096, 512, 512, "Wilson Way", "The Suburbs"),
    TransportationRegionProof::new(1024, 6144, 512, 512, "Green Maw", "The Darklands"),
    TransportationRegionProof::new(1536, 6144, 512, 512, "Hero's Hollow", "The Darklands"),
    TransportationRegionProof::new(1024, 6656, 512, 512, "Fuse's Lair", "The Darklands"),
    TransportationRegionProof::new(1536, 6656, 512, 512, "The Precipice", "The Darklands"),
    TransportationRegionProof::new(4608, 6144, 512, 512, "Forgotten Falls", "The Wilds"),
    TransportationRegionProof::new(2560, 3072, 512, 512, "Eternal Vistas", "The Suburbs"),
    TransportationRegionProof::new(2560, 3584, 512, 512, "Eternal Meadows", "The Suburbs"),
    TransportationRegionProof::new(5632, 4096, 512, 512, "Pimpleback Mountains", "The Wilds"),
    TransportationRegionProof::new(5632, 3584, 512, 512, "Haunted Ridge", "The Wilds"),
    TransportationRegionProof::new(4608, 3584, 512, 512, "Peach Creek Estates", "The Suburbs"),
    TransportationRegionProof::new(3584, 4096, 512, 512, "Sector V", "The Suburbs"),
    TransportationRegionProof::new(5120, 4608, 512, 512, "Leakey Lake", "The Wilds"),
    TransportationRegionProof::new(5120, 4096, 512, 512, "Acorn Flats", "The Wilds"),
    TransportationRegionProof::new(2560, 2048, 512, 512, "Mojo's Volcano", "Downtown"),
    TransportationRegionProof::new(4096, 6656, 512, 512, "Monkey Mountain", "The Wilds"),
    TransportationRegionProof::new(4608, 6656, 512, 512, "Monkey Foothills", "The Wilds"),
    TransportationRegionProof::new(6144, 4608, 512, 512, "Nowhere", "The Wilds"),
    TransportationRegionProof::new(6144, 5120, 512, 512, "Area 51.5", "The Wilds"),
    TransportationRegionProof::new(2048, 3584, 512, 512, "Nuclear Plant", "The Suburbs"),
    TransportationRegionProof::new(5632, 5632, 512, 512, "The Ruins", "The Wilds"),
    TransportationRegionProof::new(4608, 3072, 512, 512, "Goat's Junk Yard", "The Suburbs"),
    TransportationRegionProof::new(3072, 1536, 512, 512, "Offworld Plaza", "Downtown"),
    TransportationRegionProof::new(1536, 2048, 512, 512, "Townsville Center", "Downtown"),
    TransportationRegionProof::new(2560, 6144, 512, 512, "Firepits", "The Darklands"),
    TransportationRegionProof::new(2560, 5632, 512, 512, "Fireswamps", "The Darklands"),
    TransportationRegionProof::new(3072, 2560, 512, 512, "Tech Square", "Downtown"),
    TransportationRegionProof::new(3584, 3072, 512, 512, "Pokey Oaks South", "The Suburbs"),
    TransportationRegionProof::new(3584, 3584, 512, 512, "Pokey Oaks North", "The Suburbs"),
    TransportationRegionProof::new(5120, 5120, 512, 512, "Mount Blackhead", "The Wilds"),
    TransportationRegionProof::new(4608, 5632, 512, 512, "Really Twisted Forest", "The Wilds"),
    TransportationRegionProof::new(5120, 5632, 512, 512, "Twisted Forest", "The Wilds"),
    TransportationRegionProof::new(2048, 1024, 512, 512, "Orchid Bay", "Downtown"),
    TransportationRegionProof::new(7168, 3584, 512, 512, "Crystalline Caverns", "The Wilds"),
    TransportationRegionProof::new(6656, 1024, 512, 512, "Peach Creek Commons", "The Future"),
    TransportationRegionProof::new(6656, 512, 512, 512, "Candy Cove", "The Future"),
    TransportationRegionProof::new(5632, 512, 512, 512, "Genius Grove", "The Future"),
    TransportationRegionProof::new(7168, 1024, 512, 512, "Peach Creek Estates", "The Future"),
    TransportationRegionProof::new(6144, 1536, 512, 512, "Sector V", "The Future"),
    TransportationRegionProof::new(6144, 1024, 512, 512, "Pokey Oaks North", "The Future"),
    TransportationRegionProof::new(6144, 512, 512, 512, "Pokey Oaks South", "The Future"),
    TransportationRegionProof::new(7168, 512, 512, 512, "Goat's Junk Yard", "The Future"),
    TransportationRegionProof::new(512, 512, 512, 512, "Tech Square", "The Future"),
    TransportationRegionProof::new(
        -1,
        -1,
        0,
        0,
        "charactercreationassets/csdarklandsbg_",
        "Wayblue Sea",
    ),
    TransportationRegionProof::new(0, 4608, 512, 512, "unknown", "Wayblue Sea"),
];

pub(super) const TABLE_SET_SCHEMA: &str = "ffone.table-set.v1";

pub(super) const CONSOLIDATED_TABLE: &str = "npc_imports_consolidated";
