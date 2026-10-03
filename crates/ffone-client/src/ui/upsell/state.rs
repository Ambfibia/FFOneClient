use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum UpsellUiMode {
    Upgrade = 0,
    NewsPayZone = 1,
    NewsFreeZone = 2,
}

impl UpsellUiMode {
    #[must_use]
    pub const fn source_index(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn from_source_index(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Upgrade),
            1 => Some(Self::NewsPayZone),
            2 => Some(Self::NewsFreeZone),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_news(self) -> bool {
        matches!(self, Self::NewsPayZone | Self::NewsFreeZone)
    }

    #[must_use]
    pub const fn dialog_rect(self) -> UpsellUiRect {
        match self {
            Self::Upgrade => UPSELL_UPGRADE_DIALOG_RECT,
            Self::NewsPayZone | Self::NewsFreeZone => UPSELL_NEWS_DIALOG_RECT,
        }
    }
}
