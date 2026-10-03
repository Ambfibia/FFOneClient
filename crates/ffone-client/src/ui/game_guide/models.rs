use super::*;

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct GameGuideUiModel {
    pub visible: bool,
    pub selected_main_topic: usize,
    pub selected_sub_topic: usize,
    pub content_scroll_y: f32,
    pub ui_scale: f32,
}

impl Default for GameGuideUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            selected_main_topic: GAME_GUIDE_INITIAL_MAIN_TOPIC,
            selected_sub_topic: GAME_GUIDE_INITIAL_SUB_TOPIC,
            content_scroll_y: 0.0,
            ui_scale: 1.0,
        }
    }
}

impl GameGuideUiModel {
    /// Service help selects an absolute Help page through its FirstUse entry.
    /// The table is native content; unresolved entries leave the current page intact.
    pub fn open_first_use(&mut self, event_id: usize) -> bool {
        static ROUTES: std::sync::LazyLock<Vec<Option<(usize, usize)>>> =
            std::sync::LazyLock::new(|| {
                let root: Value = crate::xdt::from_slice(include_bytes!(
                    "../../../../../assets/game/data/tables/xdt.json"
                ))
                .expect("native table-set");
                let tables = &root["tables"][0]["value"];
                let help = &tables["m_pHelpTable"];
                tables["m_pFirstUseTable"]["m_pFirstUseData"]
                    .as_array()
                    .expect("FirstUse entries")
                    .iter()
                    .map(|entry| {
                        let main = entry["m_iHelpMain"].as_u64()? as usize;
                        let absolute = entry["m_iHelpSub"].as_u64()? as usize;
                        if main == 0 {
                            return None;
                        }
                        let topic = help["m_pHelpData"].get(main)?;
                        let start = topic["m_iTitleStartString"].as_u64()? as usize;
                        let end = topic["m_iSubEndString"].as_u64()? as usize;
                        let relative = absolute.checked_sub(start)?;
                        if absolute > end || help["m_pHelpPageData"].get(absolute).is_none() {
                            return None;
                        }
                        Some((main, relative))
                    })
                    .collect()
            });
        let Some(Some((main, sub))) = ROUTES.get(event_id) else {
            return false;
        };
        self.visible = true;
        self.selected_main_topic = *main;
        self.selected_sub_topic = *sub;
        self.content_scroll_y = 0.0;
        true
    }

    pub fn open_from_nanocom(&mut self) {
        self.visible = true;
        self.selected_main_topic = GAME_GUIDE_INITIAL_MAIN_TOPIC;
        self.selected_sub_topic = GAME_GUIDE_INITIAL_SUB_TOPIC;
        self.content_scroll_y = 0.0;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.content_scroll_y = 0.0;
    }

    #[must_use]
    pub const fn modal_active(&self) -> bool {
        self.visible
    }
}
