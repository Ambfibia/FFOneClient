use super::*;

pub(super) const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);

pub(super) const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);

pub(super) const MIN_VISIBLE_PIXELS: usize = 100_000;

pub(super) const MIN_MISSING_CHECKER_PIXELS: usize = 600;

pub(super) const EXPECTED_ELEMENT_COUNT: usize = 1_074;

pub(super) const EXPECTED_TEXT_COUNT: usize = 274;

pub(super) const EXPECTED_VISIBLE_TEXT_COUNT: usize = 28;

pub(super) const ICON_GENERAL_00: &str = "icons/items/general/generalitemicon_00.png";

pub(super) const ICON_GENERAL_01: &str = "icons/items/general/generalitemicon_01.png";

pub(super) const ICON_WEAPON_01: &str = "icons/items/weapons/wpnicon_01.png";

pub(super) const ICON_WEAPON_02: &str = "icons/items/weapons/wpnicon_02.png";

pub(super) const ICON_COSMETIC_00: &str = "icons/items/cosmetics/cosicon_00.png";

pub(super) const ICON_COSMETIC_01: &str = "icons/items/cosmetics/cosicon_01.png";

pub(super) const ICON_COSMETIC_02: &str = "icons/items/cosmetics/cosicon_02.png";

pub(super) const ICON_COSMETIC_03: &str = "icons/items/cosmetics/cosicon_03.png";

pub(super) const ICON_COSMETIC_04: &str = "icons/items/cosmetics/cosicon_04.png";

pub(super) const ICON_COSMETIC_05: &str = "icons/items/cosmetics/cosicon_05.png";

pub(super) const ICON_VEHICLE_00: &str = "icons/items/vehicles/vehicle_00.png";

pub(super) const PREVIEW_ICON_PATHS: [&str; 11] = [
    ICON_GENERAL_00,
    ICON_GENERAL_01,
    ICON_WEAPON_01,
    ICON_WEAPON_02,
    ICON_COSMETIC_00,
    ICON_COSMETIC_01,
    ICON_COSMETIC_02,
    ICON_COSMETIC_03,
    ICON_COSMETIC_04,
    ICON_COSMETIC_05,
    ICON_VEHICLE_00,
];
