use super::*;

#[derive(Debug, Resource)]
pub struct RaceRankModel {
    pub(super) phase: RaceRankPhase,
    pub(super) pcuid: i32,
    pub(super) endpoint: String,
    pub(super) npc_name: String,
    pub(super) npc_target_instance_id: i32,
    pub(super) current_page: usize,
    pub(super) selected_index: Option<usize>,
    pub(super) period: RaceRankPeriod,
    pub(super) scores: RaceRankScores,
    pub(super) pending_request_id: Option<u64>,
    pub(super) next_request_id: u64,
    pub(super) window_scroll: f32,
    pub(super) left_scroll_sample: f32,
    pub(super) right_scroll_sample: f32,
    pub(super) outputs: VecDeque<RaceRankOutput>,
    pub(super) last_parse_error: Option<RaceRankParseError>,
}

impl Default for RaceRankModel {
    fn default() -> Self {
        Self {
            phase: RaceRankPhase::Hidden,
            pcuid: 0,
            endpoint: RACE_RANK_LEGACY_FALLBACK_URL.to_owned(),
            npc_name: String::new(),
            npc_target_instance_id: 0,
            current_page: 0,
            selected_index: None,
            period: RaceRankPeriod::Today,
            scores: RaceRankScores::default(),
            pending_request_id: None,
            next_request_id: 1,
            // Deliberately not reset by open: the clean component only slides
            // on its first lifetime opening.
            window_scroll: 0.0,
            left_scroll_sample: 0.0,
            right_scroll_sample: 0.0,
            outputs: VecDeque::new(),
            last_parse_error: None,
        }
    }
}

impl RaceRankModel {
    #[must_use]
    pub const fn phase(&self) -> RaceRankPhase {
        self.phase
    }

    #[must_use]
    pub const fn current_page(&self) -> usize {
        self.current_page
    }

    #[must_use]
    pub const fn selected_index(&self) -> Option<usize> {
        self.selected_index
    }

    #[must_use]
    pub const fn period(&self) -> RaceRankPeriod {
        self.period
    }

    #[must_use]
    pub const fn scores(&self) -> &RaceRankScores {
        &self.scores
    }

    #[must_use]
    pub const fn window_scroll(&self) -> f32 {
        self.window_scroll
    }

    #[must_use]
    pub const fn left_scroll_sample(&self) -> f32 {
        self.left_scroll_sample
    }

    #[must_use]
    pub const fn right_scroll_sample(&self) -> f32 {
        self.right_scroll_sample
    }

    #[must_use]
    pub const fn last_parse_error(&self) -> Option<&RaceRankParseError> {
        self.last_parse_error.as_ref()
    }

    #[must_use]
    pub const fn pending_request_id(&self) -> Option<u64> {
        self.pending_request_id
    }

    #[must_use]
    pub fn npc_name(&self) -> &str {
        &self.npc_name
    }

    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    #[must_use]
    pub fn selected_location<'a>(
        &self,
        catalog: &'a RaceRankCatalog,
    ) -> Option<&'a RaceRankLocation> {
        self.selected_index
            .and_then(|index| catalog.locations().get(index))
    }

    #[must_use]
    pub fn visible_locations<'a>(&self, catalog: &'a RaceRankCatalog) -> &'a [RaceRankLocation] {
        let start = (self.current_page * RACE_RANK_PAGE_SIZE).min(catalog.locations().len());
        let end = (start + RACE_RANK_PAGE_SIZE).min(catalog.locations().len());
        &catalog.locations()[start..end]
    }

    #[must_use]
    pub fn controls_enabled(&self, system_popup_active: bool) -> bool {
        self.phase == RaceRankPhase::Browsing && !system_popup_active
    }

    #[must_use]
    pub fn page_copy(&self, location_count: usize) -> String {
        let first = self.current_page * RACE_RANK_PAGE_SIZE + 1;
        let last = (first + RACE_RANK_PAGE_SIZE - 1).min(location_count);
        format!("{first} - {last} of {location_count} Locations")
    }

    #[must_use]
    pub fn page_localized(&self, location_count: usize) -> LocalizedText {
        let first = self.current_page * RACE_RANK_PAGE_SIZE + 1;
        let last = (first + RACE_RANK_PAGE_SIZE - 1).min(location_count);
        LocalizedText::new(RACE_RANK_PAGE_KEY, "{first} - {last} of {total} Locations")
            .with_arg("first", first.to_string())
            .with_arg("last", last.to_string())
            .with_arg("total", location_count.to_string())
    }

    #[must_use]
    pub fn highlighted_top_index(&self) -> Option<usize> {
        let period = self.period as usize;
        let pcuid = self.scores.personal[period].as_ref()?.pcuid;
        self.scores.top[period]
            .iter()
            .take_while(|score| score.is_some())
            .position(|score| score.as_ref().is_some_and(|score| score.pcuid == pcuid))
    }

    pub fn open(
        &mut self,
        catalog: &RaceRankCatalog,
        context: RaceRankOpenContext,
    ) -> Result<(), RaceRankOpenError> {
        if catalog.locations().is_empty() {
            return Err(RaceRankOpenError::EmptyCatalog);
        }
        let Some((selected_index, location)) = catalog.location_by_ep(context.current_ep_id) else {
            return Err(RaceRankOpenError::CurrentEpNotFound(context.current_ep_id));
        };
        self.phase = RaceRankPhase::Loading;
        self.pcuid = context.pcuid;
        self.endpoint = if context.rank_url.is_empty() {
            RACE_RANK_LEGACY_FALLBACK_URL.to_owned()
        } else {
            context.rank_url
        };
        self.npc_name = context.npc_name;
        self.npc_target_instance_id = context.npc_target_instance_id;
        self.current_page = selected_index / RACE_RANK_PAGE_SIZE;
        self.selected_index = Some(selected_index);
        self.period = RaceRankPeriod::Today;
        self.scores.clear_all();
        self.pending_request_id = None;
        self.outputs.clear();
        self.last_parse_error = None;
        self.outputs
            .push_back(RaceRankOutput::Effect(RaceRankEffect::BindNpcCamera {
                target_instance_id: self.npc_target_instance_id,
            }));
        self.queue_fetch(location.ep_id);
        Ok(())
    }

    pub fn select_visible_slot(
        &mut self,
        catalog: &RaceRankCatalog,
        one_based_slot: usize,
    ) -> Result<(), RaceRankTransitionError> {
        if !self.phase.visible() {
            return Err(RaceRankTransitionError::NotVisible);
        }
        if self.phase == RaceRankPhase::Loading {
            return Err(RaceRankTransitionError::Loading);
        }
        if !(1..=RACE_RANK_PAGE_SIZE).contains(&one_based_slot) {
            return Err(RaceRankTransitionError::SlotOutOfRange(one_based_slot));
        }
        let index = self.current_page * RACE_RANK_PAGE_SIZE + one_based_slot - 1;
        let Some(location) = catalog.locations().get(index) else {
            return Err(RaceRankTransitionError::EmptySlot(one_based_slot));
        };
        self.selected_index = Some(index);
        self.last_parse_error = None;
        // Clean leaves old score rows visible but disabled until the request
        // completes and clears them.
        self.queue_fetch(location.ep_id);
        Ok(())
    }

    pub fn previous_page(&mut self) -> Result<(), RaceRankTransitionError> {
        self.page_delta(-1)
    }

    pub fn next_page(&mut self, location_count: usize) -> Result<(), RaceRankTransitionError> {
        if !self.phase.visible() {
            return Err(RaceRankTransitionError::NotVisible);
        }
        if self.phase == RaceRankPhase::Loading {
            return Err(RaceRankTransitionError::Loading);
        }
        // Exact clean bug: inclusive `iEpSize / 5`, leaving an empty page
        // when the count is divisible by five.
        let max_page = location_count / RACE_RANK_PAGE_SIZE;
        self.current_page = (self.current_page + 1).min(max_page);
        self.clear_selection_for_page();
        Ok(())
    }

    pub(super) fn page_delta(&mut self, delta: isize) -> Result<(), RaceRankTransitionError> {
        if !self.phase.visible() {
            return Err(RaceRankTransitionError::NotVisible);
        }
        if self.phase == RaceRankPhase::Loading {
            return Err(RaceRankTransitionError::Loading);
        }
        self.current_page = self.current_page.saturating_add_signed(delta);
        self.clear_selection_for_page();
        Ok(())
    }

    pub(super) fn clear_selection_for_page(&mut self) {
        self.selected_index = None;
        self.scores.clear_page_visible_heads();
        self.last_parse_error = None;
    }

    pub fn select_period(&mut self, period: RaceRankPeriod) -> Result<(), RaceRankTransitionError> {
        if !self.phase.visible() {
            return Err(RaceRankTransitionError::NotVisible);
        }
        if self.phase == RaceRankPhase::Loading {
            return Err(RaceRankTransitionError::Loading);
        }
        self.period = period;
        Ok(())
    }

    pub fn complete_fetch(
        &mut self,
        request_id: u64,
        body: &str,
    ) -> Result<(), RaceRankCompletionError> {
        let Some(expected) = self.pending_request_id else {
            return Err(RaceRankCompletionError::NoPendingRequest);
        };
        if expected != request_id {
            return Err(RaceRankCompletionError::StaleRequest {
                expected,
                received: request_id,
            });
        }
        self.pending_request_id = None;
        // Clean clears all rows at HTTP completion before checking SUCCESS.
        self.scores.clear_all();
        self.phase = RaceRankPhase::Browsing;
        match parse_clean_rank_response(body) {
            Ok(scores) => {
                self.scores = scores;
                self.last_parse_error = None;
                Ok(())
            }
            Err(error) => {
                self.last_parse_error = Some(error.clone());
                Err(RaceRankCompletionError::Parse(error))
            }
        }
    }

    /// Exact `ScrollWindow` behavior: the same scalar advances once for the
    /// left group and again for the right group during one GUI pass.
    pub fn advance_slide(&mut self, delta_seconds: f32) {
        if !self.phase.visible() || !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }
        if self.window_scroll < 1.0 {
            self.window_scroll = (self.window_scroll + delta_seconds).min(1.0);
        }
        self.left_scroll_sample = self.window_scroll;
        if self.window_scroll < 1.0 {
            self.window_scroll = (self.window_scroll + delta_seconds).min(1.0);
        }
        self.right_scroll_sample = self.window_scroll;
    }

    pub fn close(&mut self) -> bool {
        if !self.phase.visible() {
            return false;
        }
        self.pending_request_id = None;
        self.phase = RaceRankPhase::Hidden;
        self.outputs
            .push_back(RaceRankOutput::Effect(RaceRankEffect::ReleaseNpcCamera));
        self.outputs
            .push_back(RaceRankOutput::Effect(RaceRankEffect::FreeLegacyAssets));
        self.outputs
            .push_back(RaceRankOutput::Effect(RaceRankEffect::ExitMode));
        true
    }

    pub fn pop_output(&mut self) -> Option<RaceRankOutput> {
        self.outputs.pop_front()
    }

    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }

    pub(super) fn queue_fetch(&mut self, ep_id: i32) {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.pending_request_id = Some(request_id);
        self.phase = RaceRankPhase::Loading;
        self.outputs
            .push_back(RaceRankOutput::Http(RaceRankHttpIntent {
                request_id,
                url: self.endpoint.clone(),
                pcuid: self.pcuid,
                ep_id,
            }));
    }
}
