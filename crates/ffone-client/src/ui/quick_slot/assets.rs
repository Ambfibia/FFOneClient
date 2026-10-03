use super::*;

pub const QUICK_SLOT_GAME_HUD_PATH_ID: i64 = 1_352;

pub const QUICK_SLOT_COMPONENT_PATH_ID: i64 = 1_568;

pub const QUICK_SLOT_SCRIPT_PATH_ID: i64 = 1_079;

pub const QUICK_SLOT_BACKGROUND_PATH_ID: i64 = 482;

pub const QUICK_SLOT_INVENTORY_SKIN_PATH_ID: i64 = 1_366;

pub const QUICK_SLOT_OCCUPIED_STYLE_PATH_ID: i64 = 50;

pub const QUICK_SLOT_EMPTY_STYLE_PATH_ID: i64 = 239;

pub const QUICK_SLOT_UI_Z_INDEX: i32 = 9;

pub const QUICK_SLOT_BACKGROUND_PATH: &str = "ui/en/gameplay/quick-slot/quickslot_main.png";

pub const QUICK_SLOT_OCCUPIED_STYLE_PATH: &str = "ui/en/gameplay/quick-slot/slotbox.png";

pub const QUICK_SLOT_EMPTY_STYLE_PATH: &str = "ui/en/gameplay/quick-slot/slotboxempty.png";

pub const QUICK_SLOT_COOLDOWN_PATH: &str = "ui/en/gameplay/quick-slot/boxalpha.png";

/// Converted clean-source texture routes retained as provenance evidence.
/// Byte identity stays in the SHA-256 fields instead of leaking into filenames.
pub const QUICK_SLOT_SOURCE_BACKGROUND_PATH: &str = "ui/en/gameplay/quick-slot/quickslot_main.png";

pub const QUICK_SLOT_SOURCE_OCCUPIED_STYLE_PATH: &str = "ui/en/gameplay/quick-slot/slotbox.png";

pub const QUICK_SLOT_SOURCE_EMPTY_STYLE_PATH: &str = "ui/en/gameplay/quick-slot/slotboxempty.png";

pub const QUICK_SLOT_SOURCE_COOLDOWN_PATH: &str = "ui/en/gameplay/quick-slot/boxalpha.png";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QuickSlotAssetRole {
    Background,
    OccupiedStyle,
    EmptyStyle,
    Cooldown,
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct QuickSlotUiAssetContract {
    pub(super) background: Option<String>,
    pub(super) occupied_style: Option<String>,
    pub(super) empty_style: Option<String>,
    pub(super) cooldown: Option<String>,
}

impl Default for QuickSlotUiAssetContract {
    fn default() -> Self {
        Self::try_from_semantic_paths(
            QUICK_SLOT_BACKGROUND_PATH,
            QUICK_SLOT_OCCUPIED_STYLE_PATH,
            QUICK_SLOT_EMPTY_STYLE_PATH,
            QUICK_SLOT_COOLDOWN_PATH,
        )
        .expect("built-in QuickSlot paths must remain semantic ui PNG routes")
    }
}

impl QuickSlotUiAssetContract {
    pub fn try_from_semantic_paths(
        background: impl Into<String>,
        occupied_style: impl Into<String>,
        empty_style: impl Into<String>,
        cooldown: impl Into<String>,
    ) -> Result<Self, QuickSlotAssetContractError> {
        let paths = [
            (QuickSlotAssetRole::Background, background.into()),
            (QuickSlotAssetRole::OccupiedStyle, occupied_style.into()),
            (QuickSlotAssetRole::EmptyStyle, empty_style.into()),
            (QuickSlotAssetRole::Cooldown, cooldown.into()),
        ];
        for (role, path) in &paths {
            if !is_semantic_ui_png_path(path) {
                return Err(QuickSlotAssetContractError {
                    role: *role,
                    path: path.clone(),
                });
            }
        }
        let [
            (_, background),
            (_, occupied_style),
            (_, empty_style),
            (_, cooldown),
        ] = paths;
        Ok(Self {
            background: Some(background),
            occupied_style: Some(occupied_style),
            empty_style: Some(empty_style),
            cooldown: Some(cooldown),
        })
    }

    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.resolved().is_some()
    }

    pub(super) fn resolved(&self) -> Option<[&str; 4]> {
        let paths = [
            self.background.as_deref()?,
            self.occupied_style.as_deref()?,
            self.empty_style.as_deref()?,
            self.cooldown.as_deref()?,
        ];
        paths
            .iter()
            .all(|path| is_semantic_ui_png_path(path))
            .then_some(paths)
    }
}
