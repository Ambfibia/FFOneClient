use super::*;

pub const SKILL_BUFF_GAME_HUD_PATH_ID: i64 = 1_352;

pub const SKILL_BUFF_COMPONENT_PATH_ID: i64 = 1_566;

pub const SKILL_BUFF_SCRIPT_PATH_ID: i64 = 909;

pub const SKILL_BUFF_SKIN_PATH_ID: i64 = 1_372;

pub const SKILL_BUFF_BACK_PATH_ID: i64 = 166;

pub const SKILL_BUFF_FONT_PATH_ID: i64 = 1_018;

pub const SKILL_BUFF_BACK_PATH: &str = "ui/en/gameplay/nano/skill/buff_icon_back.png";

pub const SKILL_BUFF_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

pub const SKILL_BUFF_UI_Z_INDEX: i32 = 10;

/// Verified projection of `m_pSkillBuffData` plus native semantic icon routes.
#[derive(Clone, Debug, Default, Resource)]
pub struct SkillBuffUiCatalog {
    pub(super) ready: bool,
    pub(super) definitions: BTreeMap<i32, SkillBuffUiDefinition>,
}

impl SkillBuffUiCatalog {
    pub fn open(content: &TutorialMissionContent, locator: &AssetLocator) -> Result<Self, String> {
        Self::from_path_checker(content, |path| locator.require_file(path).map(|_| ()))
    }

    pub fn from_project_assets(
        content: &TutorialMissionContent,
        assets: &AssetLocator,
    ) -> Result<Self, String> {
        Self::open(content, assets)
    }

    pub(super) fn from_path_checker(
        content: &TutorialMissionContent,
        mut require: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, String> {
        let mut definitions = BTreeMap::new();
        for source in content.gameplay_skill_buffs() {
            let icon_path = format!("icons/skills/skillicon_{:02}.png", source.icon_number);
            let cash_icon_path =
                format!("icons/skills/skillicon_{:02}.png", source.cash_icon_number);
            require(&icon_path)?;
            require(&cash_icon_path)?;
            let definition = SkillBuffUiDefinition {
                buff_id: source.buff_id,
                icon_number: source.icon_number,
                icon_path,
                cash_icon_number: source.cash_icon_number,
                cash_icon_path,
            };
            if definitions.insert(source.buff_id, definition).is_some() {
                return Err(format!(
                    "duplicate native skill-buff definition {}",
                    source.buff_id
                ));
            }
        }
        for (_, buff_id) in SKILL_BUFF_DEBUFFS.into_iter().chain(SKILL_BUFF_BUFFS) {
            if !definitions.contains_key(&buff_id) {
                return Err(format!(
                    "Retrobution skill-buff row {buff_id} required by {SKILL_BUFF_LEGACY_CLASS} is missing"
                ));
            }
        }
        Ok(Self {
            ready: true,
            definitions,
        })
    }

    #[must_use]
    pub fn definition(&self, buff_id: i32) -> Option<&SkillBuffUiDefinition> {
        self.definitions.get(&buff_id)
    }

    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.ready
    }
}
