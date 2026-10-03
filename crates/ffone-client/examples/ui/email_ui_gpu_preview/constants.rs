use super::*;

pub(super) const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);

pub(super) const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);

pub(super) const MIN_WINDOW_VISIBLE_PIXELS: usize = 180_000;

pub(super) const MIN_CYAN_PIXELS: usize = 4_000;

pub(super) const ICON_GENERAL: &str = "icons/items/general/generalitemicon_00.png";

pub(super) const ICON_WEAPON: &str = "icons/items/weapons/wpnicon_01.png";

pub(super) const ICON_COSMETIC: &str = "icons/items/cosmetics/cosicon_00.png";

pub(super) const PREVIEW_ICON_PATHS: [&str; 3] = [ICON_GENERAL, ICON_WEAPON, ICON_COSMETIC];
