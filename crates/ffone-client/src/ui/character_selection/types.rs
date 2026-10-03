use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CharacterLocationBackground {
    #[default]
    Future,
    Suburbs,
    Downtown,
    Wilds,
    Darklands,
}

impl CharacterLocationBackground {
    pub const ALL: [Self; 5] = [
        Self::Future,
        Self::Suburbs,
        Self::Downtown,
        Self::Wilds,
        Self::Darklands,
    ];

    /// Exact Texture2D PathIDs in the clean
    /// `CharacterCreation.resourceFile` AssetBundle. `LoadAssets` addresses
    /// the same objects by `CharacterCreationAssets/<name>.png`.
    pub const fn source_path_ids(self) -> [i64; CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT] {
        match self {
            Self::Future => [63, 37, 59, 30, 102, 118, 73, 117, 19, 38],
            Self::Suburbs => [129, 104, 32, 3, 62, 92, 79, 110, 34, 5],
            Self::Downtown => [74, 66, 109, 112, 49, 99, 80, 36, 101, 100],
            Self::Wilds => [95, 85, 82, 58, 41, 12, 55, 22, 14, 67],
            Self::Darklands => [51, 107, 2, 48, 111, 7, 6, 106, 88, 16],
        }
    }

    pub const fn folder(self) -> &'static str {
        match self {
            Self::Future => "future",
            Self::Suburbs => "suburbs",
            Self::Downtown => "downtown",
            Self::Wilds => "wilds",
            Self::Darklands => "darklands",
        }
    }

    pub const fn file_prefix(self) -> &'static str {
        match self {
            Self::Future => "CSFutureBG",
            Self::Suburbs => "CSSuburbsBG",
            Self::Downtown => "CSDowntownBG",
            Self::Wilds => "CSWildsBG",
            Self::Darklands => "CSDarklandsBG",
        }
    }

    pub fn asset_path(self, frame: usize) -> String {
        assert!((1..=CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT).contains(&frame));
        format!(
            "ui/en/character/selection/backgrounds/{}/{}_{}.png",
            self.folder(),
            self.file_prefix(),
            frame
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccupiedCharacterSlotUi {
    pub pc_uid: i64,
    pub display_name: String,
    pub level: i16,
    pub district: String,
    pub zone: String,
    pub background: CharacterLocationBackground,
}

impl OccupiedCharacterSlotUi {
    /// Compatibility projection used by shell/runtime diagnostics. The clean
    /// draw path below additionally depends on hover/selection state.
    pub fn location_label(&self) -> String {
        match (self.district.is_empty(), self.zone.is_empty()) {
            (true, true) => String::new(),
            (false, true) => self.district.clone(),
            (true, false) => self.zone.clone(),
            (false, false) => format!("{} - {}", self.district, self.zone),
        }
    }

    pub fn location_label_for_state(&self, selected_or_hovered: bool) -> String {
        match (self.district.is_empty(), self.zone.is_empty()) {
            (true, true) => String::new(),
            (false, true) => self.district.clone(),
            (true, false) => self.zone.clone(),
            (false, false) => {
                // Keep creation placeholder spacing; real locations always use a hyphen.
                let character_creation = self.district == "CHARACTER CREATION";
                let separator = if character_creation && selected_or_hovered {
                    "  "
                } else {
                    " - "
                };
                format!("{}{separator}{}", self.district, self.zone)
            }
        }
    }

    pub fn location_localized(&self, selected_or_hovered: bool) -> LocalizedText {
        self.location_localized_in(selected_or_hovered, None, None)
    }

    /// `ReceivePtDongName` stores English TableData `worldname` text. Resolve
    /// each part through its semantic location key before the clean upper-case
    /// projection, so the selected text language owns the whole label.
    pub fn location_localized_in(
        &self,
        selected_or_hovered: bool,
        localization: Option<&Localization>,
        language: Option<&Language>,
    ) -> LocalizedText {
        let part = |source: &str| character_selection_location_part(source, localization, language);
        match (self.district.is_empty(), self.zone.is_empty()) {
            (true, true) => {
                LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "")
            }
            (false, true) => LocalizedText::new("ui.content.passthrough", "{text}")
                .with_arg("text", part(&self.district)),
            (true, false) => LocalizedText::new("ui.content.passthrough", "{text}")
                .with_arg("text", part(&self.zone)),
            (false, false) => {
                let character_creation = self.district == "CHARACTER CREATION";
                let (key, fallback) = if character_creation && selected_or_hovered {
                    ("ui.character_select.location.spaced", "{district}  {zone}")
                } else {
                    ("ui.character_select.location.hyphen", "{district} - {zone}")
                };
                LocalizedText::new(key, fallback)
                    .with_arg("district", part(&self.district))
                    .with_arg("zone", part(&self.zone))
            }
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum CharacterSlotUi {
    #[default]
    Empty,
    Occupied(OccupiedCharacterSlotUi),
    SubscriptionLocked,
    SubscriptionLockedOccupied(OccupiedCharacterSlotUi),
}

impl CharacterSlotUi {
    pub const fn occupied(&self) -> Option<&OccupiedCharacterSlotUi> {
        match self {
            Self::Occupied(character) | Self::SubscriptionLockedOccupied(character) => {
                Some(character)
            }
            Self::Empty | Self::SubscriptionLocked => None,
        }
    }

    pub const fn subscription_locked(&self) -> bool {
        matches!(
            self,
            Self::SubscriptionLocked | Self::SubscriptionLockedOccupied(_)
        )
    }
}
