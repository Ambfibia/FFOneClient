use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct UpsellUiModel {
    pub(super) visible: bool,
    pub(super) active_mode: Option<UpsellUiMode>,
    pub(super) level: Option<u8>,
    pub(super) news_page_paths: Vec<String>,
    pub(super) news_page: usize,
    pub(super) page_alpha: f32,
    pub(super) system_popup_open: bool,
    pub(super) help_open: bool,
    pub(super) escape_mode_allows_exit: bool,
    pub(super) nanocom_state: i32,
    pub(super) ui_scale_override: Option<f32>,
}

impl Default for UpsellUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            active_mode: None,
            level: None,
            news_page_paths: Vec::new(),
            news_page: 0,
            page_alpha: 0.0,
            system_popup_open: false,
            help_open: false,
            escape_mode_allows_exit: false,
            nanocom_state: 0,
            ui_scale_override: None,
        }
    }
}

impl UpsellUiModel {
    /// Clean `ReceiveInit`: mode `2` for an existing camera outside the paid
    /// zone, otherwise mode `1`. It deliberately starts with no news image.
    pub fn receive_init(&mut self, level: i32, is_pay_zone: bool) -> Result<(), UpsellUiError> {
        let level = valid_level(level)?;
        self.visible = true;
        self.active_mode = Some(if is_pay_zone {
            UpsellUiMode::NewsPayZone
        } else {
            UpsellUiMode::NewsFreeZone
        });
        self.level = Some(level);
        self.news_page_paths.clear();
        self.news_page = 0;
        self.page_alpha = 0.0;
        Ok(())
    }

    /// Explicit preservation API for the source-retained but clean-unreached
    /// mode `0`; it must never be called as a `ReceiveInit` fallback.
    pub fn open_retained_upgrade(&mut self, level: i32) -> Result<(), UpsellUiError> {
        let level = valid_level(level)?;
        self.visible = true;
        self.active_mode = Some(UpsellUiMode::Upgrade);
        self.level = Some(level);
        self.news_page_paths.clear();
        self.news_page = 0;
        self.page_alpha = 1.0;
        Ok(())
    }

    /// Inject locally owned news image paths after the source-equivalent
    /// asynchronous provider resolves. Empty is valid and leaves the news
    /// dialog blank. Invalid paths preserve the previous page set.
    pub fn set_news_page_paths(&mut self, paths: Vec<String>) -> Result<(), UpsellUiError> {
        if !self.active_mode.is_some_and(UpsellUiMode::is_news) {
            return Err(UpsellUiError::NotNewsMode);
        }
        if let Some(index) = paths.iter().position(|path| !valid_news_page_path(path)) {
            return Err(UpsellUiError::InvalidNewsPagePath { index });
        }
        self.news_page_paths = paths;
        self.news_page = 0;
        self.page_alpha = 0.0;
        Ok(())
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.active_mode = None;
        self.level = None;
        self.news_page_paths.clear();
        self.news_page = 0;
        self.page_alpha = 0.0;
    }

    pub fn set_external_modes(&mut self, system_popup_open: bool, help_open: bool) {
        self.system_popup_open = system_popup_open;
        self.help_open = help_open;
    }

    /// Supplies the two clean event-bus results queried only when Escape is
    /// pressed: event `2/24` must return `1`, and event `11/13[0]` must be `0`.
    pub fn set_escape_context(&mut self, mode_exit_allowed: bool, nanocom_state: i32) {
        self.escape_mode_allows_exit = mode_exit_allowed;
        self.nanocom_state = nanocom_state;
    }

    pub fn set_ui_scale(&mut self, ui_scale: f32) {
        self.ui_scale_override = Some(valid_ui_scale(ui_scale));
    }

    pub fn clear_ui_scale_override(&mut self) {
        self.ui_scale_override = None;
    }

    #[must_use]
    pub const fn visible(&self) -> bool {
        self.visible
    }

    #[must_use]
    pub const fn active_mode(&self) -> Option<UpsellUiMode> {
        self.active_mode
    }

    #[must_use]
    pub const fn level(&self) -> Option<u8> {
        self.level
    }

    #[must_use]
    pub const fn news_page(&self) -> usize {
        self.news_page
    }

    #[must_use]
    pub fn news_page_count(&self) -> usize {
        self.news_page_paths.len()
    }

    #[must_use]
    pub fn current_news_page_path(&self) -> Option<&str> {
        self.news_page_paths.get(self.news_page).map(String::as_str)
    }

    #[must_use]
    pub const fn page_alpha(&self) -> f32 {
        self.page_alpha
    }

    #[must_use]
    pub const fn controls_enabled(&self) -> bool {
        self.visible && !self.system_popup_open && !self.help_open
    }

    #[must_use]
    pub const fn escape_dismiss_enabled(&self) -> bool {
        self.visible
            && self.escape_mode_allows_exit
            && !self.system_popup_open
            && self.nanocom_state == 0
    }

    #[must_use]
    pub const fn input_boundary(&self) -> UpsellInputBoundary {
        UpsellInputBoundary {
            blocks_lower_ui: self.visible,
            blocks_gameplay_input: self.visible,
            requires_pointer: self.visible,
            mouse_controls_enabled: self.controls_enabled(),
            // `GameFrame.bHelp` disables IMGUI controls, but `cnUpsell.Update`
            // does not directly include it in the Escape condition.
            escape_dismiss_enabled: self.escape_dismiss_enabled(),
        }
    }

    #[must_use]
    pub const fn ui_scale_override(&self) -> Option<f32> {
        self.ui_scale_override
    }

    #[must_use]
    pub fn effective_ui_scale(&self, viewport_height: f32) -> f32 {
        self.ui_scale_override
            .unwrap_or_else(|| clean_upsell_ui_scale(viewport_height))
    }

    pub fn advance_page_fade(&mut self, delta_seconds: f32) {
        if !self.visible
            || !self.active_mode.is_some_and(UpsellUiMode::is_news)
            || self.current_news_page_path().is_none()
            || !delta_seconds.is_finite()
            || delta_seconds <= 0.0
        {
            return;
        }
        self.page_alpha = (self.page_alpha + UPSELL_PAGE_FADE_PER_SECOND * delta_seconds).min(1.0);
    }
}
