use super::*;

#[derive(Debug, Resource)]
pub struct NanocomMessageUiModel {
    pub(super) queue: VecDeque<QueuedNanocomMessage>,
    pub(super) reveal_remaining: f32,
    pub(super) expanded: bool,
    pub(super) scene_event_active: bool,
    pub(super) pending_head_removal: bool,
    pub(super) ui_scale_override: Option<f32>,
    pub(super) sounds: VecDeque<NanocomMessageSound>,
    pub(super) chat_echoes: VecDeque<NanocomChatEcho>,
    pub(super) next_passive_request_id: u64,
}

impl Default for NanocomMessageUiModel {
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
            reveal_remaining: 0.0,
            expanded: false,
            scene_event_active: false,
            pending_head_removal: false,
            ui_scale_override: None,
            sounds: VecDeque::new(),
            chat_echoes: VecDeque::new(),
            next_passive_request_id: NANOCOM_PASSIVE_REQUEST_ID_BASE,
        }
    }
}

impl NanocomMessageUiModel {
    /// Inserts an interactive message before the first passive message.
    /// Ordering remains FIFO inside both priority classes; the chat echo keeps
    /// receipt order, as the clean client writes it before queueing.
    pub fn enqueue(&mut self, request: NanocomMessageRequest) {
        self.push_chat_echo(&request);
        let was_empty = self.queue.is_empty();
        let insert_at = if request.kind.is_interactive() {
            self.queue
                .iter()
                .position(|queued| !queued.request.kind.is_interactive())
                .unwrap_or(self.queue.len())
        } else {
            self.queue.len()
        };
        let lifetime_seconds = request.lifetime_seconds;
        self.queue.insert(
            insert_at,
            QueuedNanocomMessage {
                request,
                remaining_seconds: lifetime_seconds,
            },
        );

        if was_empty || insert_at == 0 {
            self.reveal_remaining = 1.0;
            self.queue_head_audio();
        }
    }

    pub fn enqueue_buddy_invite(&mut self, request_id: u64, buddy_name: impl AsRef<str>) {
        self.enqueue(NanocomMessageRequest::buddy_invite(request_id, buddy_name));
    }

    pub fn enqueue_type_9(
        &mut self,
        request_id: u64,
        title: impl Into<String>,
        body: impl Into<String>,
        compact_icon_path: impl Into<String>,
    ) {
        self.enqueue(NanocomMessageRequest::type_9(
            request_id,
            title,
            body,
            compact_icon_path,
        ));
    }

    pub fn enqueue_type_9_numbuh_two(
        &mut self,
        request_id: u64,
        title: impl Into<String>,
        body: impl Into<String>,
    ) {
        self.enqueue(NanocomMessageRequest::type_9_numbuh_two(
            request_id, title, body,
        ));
    }

    /// Adds a source-authored NPC notice without making producers coordinate
    /// request IDs. The model owns a collision-free passive namespace.
    pub fn enqueue_type_9_localized(
        &mut self,
        title: LocalizedText,
        body: LocalizedText,
        compact_icon_path: impl Into<String>,
        voice_owner: Option<&str>,
    ) -> u64 {
        self.enqueue_type_9_localized_optional_icon(
            title,
            body,
            Some(compact_icon_path.into()),
            voice_owner,
        )
    }

    pub fn enqueue_type_9_localized_optional_icon(
        &mut self,
        title: LocalizedText,
        body: LocalizedText,
        compact_icon_path: Option<String>,
        voice_owner: Option<&str>,
    ) -> u64 {
        let request_id = self.next_passive_request_id;
        self.next_passive_request_id = self.next_passive_request_id.wrapping_add(1);
        if self.next_passive_request_id < NANOCOM_PASSIVE_REQUEST_ID_BASE {
            self.next_passive_request_id = NANOCOM_PASSIVE_REQUEST_ID_BASE;
        }
        self.enqueue(NanocomMessageRequest::type_9_localized_optional_icon(
            request_id,
            title,
            body,
            compact_icon_path,
            voice_owner,
        ));
        request_id
    }

    pub fn enqueue_nano_mission_localized(
        &mut self,
        body: LocalizedText,
        compact_icon_path: Option<String>,
        voice_owner: Option<&str>,
    ) -> u64 {
        let request_id = self.next_passive_request_id;
        self.next_passive_request_id = self.next_passive_request_id.wrapping_add(1);
        if self.next_passive_request_id < NANOCOM_PASSIVE_REQUEST_ID_BASE {
            self.next_passive_request_id = NANOCOM_PASSIVE_REQUEST_ID_BASE;
        }
        self.enqueue(NanocomMessageRequest::nano_mission_localized(
            request_id,
            body,
            compact_icon_path,
            voice_owner,
        ));
        request_id
    }

    #[must_use]
    pub fn active(&self) -> Option<&QueuedNanocomMessage> {
        self.queue.front()
    }

    #[must_use]
    pub fn queued(&self) -> &VecDeque<QueuedNanocomMessage> {
        &self.queue
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    #[must_use]
    pub fn reveal_parameter(&self) -> f32 {
        self.reveal_remaining.clamp(0.0, 1.0)
    }

    #[must_use]
    pub const fn expanded(&self) -> bool {
        self.expanded
    }

    #[must_use]
    pub const fn scene_event_active(&self) -> bool {
        self.scene_event_active
    }

    #[must_use]
    pub fn compact_visible(&self) -> bool {
        !self.scene_event_active
            && self
                .active()
                .is_some_and(|queued| queued.request.kind.is_reached_presentation())
    }

    #[must_use]
    pub fn expanded_visible(&self) -> bool {
        !self.scene_event_active
            && self.expanded
            && self
                .active()
                .is_some_and(|queued| queued.request.kind.has_reached_modal())
    }

    pub fn set_expanded(&mut self, expanded: bool) {
        self.expanded = expanded;
    }

    pub fn set_scene_event_active(&mut self, active: bool) {
        self.scene_event_active = active;
    }

    pub fn set_ui_scale(&mut self, scale: f32) {
        self.ui_scale_override = Some(valid_ui_scale(scale));
    }

    pub fn clear_ui_scale_override(&mut self) {
        self.ui_scale_override = None;
    }

    #[must_use]
    pub fn effective_ui_scale(&self, viewport_height: f32) -> f32 {
        self.ui_scale_override
            .unwrap_or_else(|| clean_nanocom_ui_scale(viewport_height))
    }

    /// The callback action is returned immediately. The legacy `bPopflag`
    /// behavior is preserved by deferring removal until the next `tick`.
    pub fn choose(&mut self, choice: NanocomMessageChoice) -> Option<NanocomMessageUiAction> {
        if self.scene_event_active || self.pending_head_removal {
            return None;
        }
        let active = self.active()?;
        if !active.request.kind.is_interactive() {
            return None;
        }
        let action = NanocomMessageUiAction {
            request_id: active.request.request_id,
            kind: active.request.kind,
            resolution: match choice {
                NanocomMessageChoice::Accept => NanocomMessageResolution::Accepted,
                NanocomMessageChoice::Decline => NanocomMessageResolution::Declined,
            },
        };
        self.pending_head_removal = true;
        self.sounds.push_back(match choice {
            NanocomMessageChoice::Accept => NanocomMessageSound::Yes,
            NanocomMessageChoice::Decline => NanocomMessageSound::No,
        });
        Some(action)
    }

    /// Advances only the current head. Scene events freeze reveal, lifetime,
    /// timeout, deferred removal, and presentation sounds.
    pub fn tick(&mut self, delta_seconds: f32) -> Option<NanocomMessageUiAction> {
        if self.scene_event_active || !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return None;
        }
        if self.queue.is_empty() {
            return None;
        }

        self.reveal_remaining =
            (self.reveal_remaining - delta_seconds / NANOCOM_REVEAL_SECONDS).max(0.0);

        if self.pending_head_removal {
            self.pending_head_removal = false;
            self.remove_head_and_reveal_next();
            return None;
        }

        let active = self.queue.front_mut().expect("queue was checked above");
        active.remaining_seconds -= delta_seconds;
        if active.remaining_seconds >= 0.0 {
            return None;
        }

        let action = NanocomMessageUiAction {
            request_id: active.request.request_id,
            kind: active.request.kind,
            resolution: NanocomMessageResolution::TimedOut,
        };
        self.remove_head_and_reveal_next();
        Some(action)
    }

    pub fn pop_sound(&mut self) -> Option<NanocomMessageSound> {
        self.sounds.pop_front()
    }

    #[must_use]
    pub fn has_chat_echo(&self) -> bool {
        !self.chat_echoes.is_empty()
    }

    pub fn pop_chat_echo(&mut self) -> Option<NanocomChatEcho> {
        self.chat_echoes.pop_front()
    }

    /// For a producer that already writes this message's chat line itself.
    pub fn discard_chat_echo(&mut self, request_id: u64) {
        if let Some(index) = self
            .chat_echoes
            .iter()
            .rposition(|echo| echo.request_id == request_id)
        {
            self.chat_echoes.remove(index);
        }
    }

    pub(super) fn push_chat_echo(&mut self, request: &NanocomMessageRequest) {
        let interactive = request.kind.is_interactive();
        // Interactive OutString is the invitation sentence, without the
        // compact panel's accept/decline countdown.
        let body = if interactive {
            request.expanded_body_localized()
        } else {
            request
                .localized_body
                .clone()
                .unwrap_or_else(|| nanocom_passthrough(&request.body))
        };
        if self.chat_echoes.len() == NANOCOM_CHAT_ECHO_LIMIT {
            self.chat_echoes.pop_front();
        }
        self.chat_echoes.push_back(NanocomChatEcho {
            request_id: request.request_id,
            interactive,
            title: request.compact_title_localized(),
            body,
        });
    }

    pub fn clear(&mut self) {
        *self = Self {
            ui_scale_override: self.ui_scale_override,
            ..default()
        };
    }

    pub(super) fn remove_head_and_reveal_next(&mut self) {
        self.queue.pop_front();
        if self.queue.is_empty() {
            self.reveal_remaining = 0.0;
            self.sounds.push_back(NanocomMessageSound::SlideOut);
        } else {
            self.reveal_remaining = 1.0;
            self.queue_head_audio();
        }
    }

    pub(super) fn queue_head_audio(&mut self) {
        self.sounds.push_back(NanocomMessageSound::SlideIn);
        let Some(request) = self.queue.front().map(|queued| &queued.request) else {
            return;
        };
        if let Some(true_name) = request.voice_true_name.clone() {
            self.sounds.push_back(NanocomMessageSound::Voice(true_name));
        }
        if request.kind == NanocomMessageKind::Nano {
            self.sounds
                .push_back(NanocomMessageSound::NanoCreationComplete);
        }
    }
}
