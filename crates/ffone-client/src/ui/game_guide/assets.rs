use super::*;

pub const GAME_GUIDE_BACKDROP_PATH: &str = "ui/en/shared/panelback.png";

pub const GAME_GUIDE_BACKGROUND_PATH: &str = "ui/en/character/creation/help/background.png";

pub const GAME_GUIDE_CLOSE_PATH: &str = "ui/en/shared/close_normal.png";

pub const GAME_GUIDE_CLOSE_OVER_PATH: &str = "ui/en/shared/close_over.png";

pub const GAME_GUIDE_TITLE_BAR_PATH: &str = "ui/en/gameplay/game-guide/controls/title.png";

pub const GAME_GUIDE_JEFFE_FONT_PATH: &str = "fonts/jeffe.otf";

pub const GAME_GUIDE_CHALET_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";

#[derive(Resource, Clone, Debug)]
pub(super) struct GameGuideCatalog {
    pub(super) topics: Vec<HelpRange>,
    pub(super) pages: Vec<HelpRange>,
    pub(super) content: Vec<HelpContent>,
    pub(super) help_names: Vec<String>,
    pub(super) help_comments: Vec<String>,
    pub(super) page_names: Vec<String>,
    pub(super) page_comments: Vec<String>,
}

impl FromWorld for GameGuideCatalog {
    fn from_world(_: &mut World) -> Self {
        let root: Value = crate::xdt::from_slice(include_bytes!(
            "../../../../../assets/game/data/tables/xdt.json"
        ))
        .expect("checked native table-set JSON");
        let help = &root["tables"][0]["value"]["m_pHelpTable"];
        let topics = help["m_pHelpData"]
            .as_array()
            .expect("m_pHelpData")
            .iter()
            .map(|value| HelpRange {
                start: value["m_iTitleStartString"].as_u64().unwrap_or(0) as usize,
                end: value["m_iSubEndString"].as_u64().unwrap_or(0) as usize,
            })
            .collect();
        let pages = help["m_pHelpPageData"]
            .as_array()
            .expect("m_pHelpPageData")
            .iter()
            .map(|value| HelpRange {
                start: value["m_iStartContent"].as_u64().unwrap_or(0) as usize,
                end: value["m_iEndContent"].as_u64().unwrap_or(0) as usize,
            })
            .collect();
        let content = help["m_pHelpPageDescData"]
            .as_array()
            .expect("m_pHelpPageDescData")
            .iter()
            .map(|value| HelpContent {
                kind: value["m_iType"].as_u64().unwrap_or(0) as u8,
                size: value["m_iSize"].as_u64().unwrap_or(0) as u8,
                string_id: value["m_iString"].as_u64().unwrap_or(0) as usize,
            })
            .collect();
        let strings = help["m_pHelpString"].as_array().expect("m_pHelpString");
        let help_names = strings
            .iter()
            .map(|value| value["m_strName"].as_str().unwrap_or_default().to_owned())
            .collect();
        let help_comments = strings
            .iter()
            .map(|value| {
                value["m_strComment"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect();
        let page_strings = help["m_pHelpPageString"]
            .as_array()
            .expect("m_pHelpPageString");
        let page_names = page_strings
            .iter()
            .map(|value| value["m_strName"].as_str().unwrap_or_default().to_owned())
            .collect();
        let page_comments = page_strings
            .iter()
            .map(|value| {
                value["m_strComment"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect();
        Self {
            topics,
            pages,
            content,
            help_names,
            help_comments,
            page_names,
            page_comments,
        }
    }
}

pub(super) fn screenshot_asset_path(filename: &str) -> Option<String> {
    let trimmed = filename.trim();
    let stem = trimmed
        .strip_suffix(".jpg")
        .or_else(|| trimmed.strip_suffix(".JPG"))?;
    if stem.eq_ignore_ascii_case("lteminfo") {
        return None;
    }
    let stem = stem
        .chars()
        .flat_map(char::to_lowercase)
        .map(|ch| if ch.is_whitespace() { '-' } else { ch })
        .collect::<String>();
    Some(format!("ui/en/gameplay/game-guide/screenshots/{stem}.png"))
}
