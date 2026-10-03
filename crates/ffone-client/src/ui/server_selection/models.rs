use super::*;

#[derive(Clone, Debug, Resource)]
pub struct ServerSelectionUiModel {
    pub phase: ServerSelectionPhase,
    pub server_expanded: bool,
    pub selected_server: i32,
    pub selected_shard: i32,
    pub server_status: i32,
    pub shard_statuses: [u8; SERVER_SELECTION_SHARD_ARRAY_LEN],
    pub scroll_y: f32,
    pub scroll_view_height: f32,
    pub refresh_timer: f32,
    pub init: Option<ServerSelectionInit>,
}

impl Default for ServerSelectionUiModel {
    fn default() -> Self {
        Self {
            phase: ServerSelectionPhase::Hidden,
            server_expanded: false,
            selected_server: 0,
            selected_shard: 0,
            // `Awake` creates two records (dummy index 0 plus server index 1)
            // and assigns both status 1 before any packet arrives.
            server_status: ServerSelectionPopulation::Empty as i32,
            shard_statuses: [0; SERVER_SELECTION_SHARD_ARRAY_LEN],
            scroll_y: 0.0,
            // Serialized value is zero; Awake adds 1*402 before the first
            // `OnGUI`, then each pass replaces it with 30 or 492.
            scroll_view_height: SERVER_SELECTION_INITIAL_SCROLL_HEIGHT,
            refresh_timer: 0.0,
            init: None,
        }
    }
}

impl ServerSelectionUiModel {
    pub fn open(&mut self, init: ServerSelectionInit, outbox: &mut ServerSelectionUiOutbox) {
        self.phase = ServerSelectionPhase::Visible;
        self.init = Some(init);
        self.refresh_timer = 0.0;
        // Clean `InitServerSelectionMode` does not reset toggle, selection, or
        // scroll state; those fields survive another entry on the same object.
        self.request_shard_list(outbox);
    }

    pub fn hide(&mut self) {
        self.phase = ServerSelectionPhase::Hidden;
    }

    #[must_use]
    pub const fn visible(&self) -> bool {
        matches!(
            self.phase,
            ServerSelectionPhase::Visible | ServerSelectionPhase::AwaitingQuitGate
        )
    }

    #[must_use]
    pub const fn input_boundary(&self) -> ServerSelectionInputBoundary {
        let visible = self.visible();
        ServerSelectionInputBoundary {
            blocks_lower_ui: visible,
            blocks_gameplay_input: visible,
            requires_pointer: visible,
            // Neither clean source class checks Escape.
            escape_enabled: false,
        }
    }

    pub fn tick(&mut self, delta_seconds: f32, outbox: &mut ServerSelectionUiOutbox) {
        if self.phase != ServerSelectionPhase::Visible
            || !delta_seconds.is_finite()
            || delta_seconds <= 0.0
        {
            return;
        }
        self.refresh_timer += delta_seconds;
        // Preserve the source's strict `> 60f` and reset-to-zero behavior;
        // it neither fires at exactly 60 nor catches up multiple intervals.
        if self.refresh_timer > SERVER_SELECTION_REFRESH_SECONDS {
            self.request_shard_list(outbox);
            self.refresh_timer = 0.0;
        }
    }

    pub(super) fn request_shard_list(&self, outbox: &mut ServerSelectionUiOutbox) {
        outbox.push(ServerSelectionUiEffect::RequestShardList {
            packet_id: SERVER_SELECTION_REQ_SHARD_LIST_PACKET_ID,
            packet_size: SERVER_SELECTION_REQ_SHARD_LIST_PACKET_SIZE,
        });
    }

    pub fn receive_shard_list(
        &mut self,
        packet_id: u32,
        payload: [u8; SERVER_SELECTION_SHARD_ARRAY_LEN],
    ) -> bool {
        if packet_id != SERVER_SELECTION_REP_SHARD_LIST_PACKET_ID || !self.visible() {
            return false;
        }
        // `ReceivePacket` loops 1..26 and intentionally leaves slot zero as
        // the prior value.
        self.shard_statuses[1..SERVER_SELECTION_SHARD_ARRAY_LEN]
            .copy_from_slice(&payload[1..SERVER_SELECTION_SHARD_ARRAY_LEN]);
        true
    }

    pub fn toggle_server(&mut self) {
        if self.phase != ServerSelectionPhase::Visible {
            return;
        }
        self.server_expanded = !self.server_expanded;
        self.complete_gui_pass();
    }

    pub fn select_shard(&mut self, server: i32, shard: i32) -> bool {
        if self.phase != ServerSelectionPhase::Visible
            || server != 1
            || !(i32::from(SERVER_SELECTION_FIRST_SHARD)..=i32::from(SERVER_SELECTION_LAST_SHARD))
                .contains(&shard)
        {
            return false;
        }
        // Source does not gate selection on CLOSED status.
        self.selected_server = server;
        self.selected_shard = shard;
        true
    }

    pub fn set_selected_shard_from_legacy_event(&mut self, shard: i32) {
        // `ReceiveSetSelectedServer` is an unchecked assignment.
        self.selected_shard = shard;
    }

    #[must_use]
    pub const fn selected_shard_for_legacy_event(&self) -> i32 {
        self.selected_shard
    }

    pub fn connect(&mut self, outbox: &mut ServerSelectionUiOutbox) -> bool {
        if self.phase != ServerSelectionPhase::Visible {
            return false;
        }
        if self.selected_server == 0 || self.selected_shard == 0 {
            outbox.push(ServerSelectionUiEffect::SystemMessage {
                message_id: SERVER_SELECTION_STATUS_MESSAGE_ID,
            });
            return false;
        }
        let Some(init) = self.init.clone() else {
            return false;
        };

        if init.login.character_count > 0 {
            outbox.push(ServerSelectionUiEffect::SetGameMode(2));
            outbox.push(ServerSelectionUiEffect::InitCharacterSelection {
                login: init.login.clone(),
                account_id: init.login.account_id.clone(),
                shard: self.selected_shard,
                zero: 0,
                auto_login: init.auto_login,
                warp_shard: init.warp_shard,
            });
        } else {
            outbox.push(ServerSelectionUiEffect::InitCharacterCreation {
                mode: 2,
                login: init.login.clone(),
                account_id: init.login.account_id.clone(),
                shard: self.selected_shard,
                zero: 0,
                inverted_auto_login: !init.auto_login,
                warp_shard: false,
            });
            outbox.push(ServerSelectionUiEffect::ApplySoundOptions);
            outbox.push(ServerSelectionUiEffect::SetGameMode(21));
            outbox.push(ServerSelectionUiEffect::InitDexterNameCreate {
                character_id: 0,
                event_scene_name: "NameCreate",
                slot_number: 1,
            });
            outbox.push(ServerSelectionUiEffect::LegacyEvent {
                manager: 9,
                function: 6,
                element_function: None,
            });
        }
        outbox.push(ServerSelectionUiEffect::SendServerSelect {
            server_number: self.selected_server as i8,
            packet_id: SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_ID,
            packet_size: SERVER_SELECTION_REQ_SERVER_SELECT_PACKET_SIZE,
        });
        self.phase = ServerSelectionPhase::Transitioning;
        true
    }

    pub fn open_web(&self, source: ServerSelectionWebSource, outbox: &mut ServerSelectionUiOutbox) {
        if self.phase == ServerSelectionPhase::Visible {
            outbox.push(ServerSelectionUiEffect::OpenUrl {
                source,
                url: SERVER_SELECTION_URL,
            });
        }
    }

    pub fn request_quit(&mut self, outbox: &mut ServerSelectionUiOutbox) -> bool {
        if self.phase != ServerSelectionPhase::Visible {
            return false;
        }
        self.phase = ServerSelectionPhase::AwaitingQuitGate;
        outbox.push(ServerSelectionUiEffect::RequestQuitGate {
            manager: 3,
            function: 1,
        });
        true
    }

    pub fn resolve_quit_gate(
        &mut self,
        legacy_return: i32,
        is_editor: bool,
        outbox: &mut ServerSelectionUiOutbox,
    ) -> bool {
        if self.phase != ServerSelectionPhase::AwaitingQuitGate {
            return false;
        }
        if legacy_return != 0 {
            self.phase = ServerSelectionPhase::Visible;
            return false;
        }
        outbox.push(ServerSelectionUiEffect::SetGameMode(1));
        outbox.push(ServerSelectionUiEffect::LegacyEvent {
            manager: 2,
            function: 3,
            element_function: Some(8),
        });
        if !is_editor {
            outbox.push(ServerSelectionUiEffect::SetSystemFocusOut(true));
            outbox.push(ServerSelectionUiEffect::LegacyEvent {
                manager: 9,
                function: 6,
                element_function: None,
            });
            outbox.push(ServerSelectionUiEffect::SkipWorldUpdate(true));
            outbox.push(ServerSelectionUiEffect::SetSaveResolution(false));
            outbox.push(ServerSelectionUiEffect::ExternalCallHomePage);
        }
        self.phase = ServerSelectionPhase::Transitioning;
        true
    }

    #[must_use]
    pub const fn press_escape(&self) -> bool {
        // Explicit no-op: there is no Input.GetKeyDown(Escape) branch.
        false
    }

    pub fn complete_gui_pass(&mut self) {
        self.scroll_view_height = if self.server_expanded {
            SERVER_SELECTION_EXPANDED_SCROLL_HEIGHT
        } else {
            SERVER_SELECTION_COLLAPSED_SCROLL_HEIGHT
        };
        self.scroll_y = self.scroll_y.min(self.maximum_scroll()).max(0.0);
    }

    #[must_use]
    pub fn maximum_scroll(&self) -> f32 {
        (self.scroll_view_height - SERVER_SELECTION_VIEWPORT_RECT.height).max(0.0)
    }

    pub fn scroll_by(&mut self, delta_pixels: f32) {
        if self.phase != ServerSelectionPhase::Visible || !delta_pixels.is_finite() {
            return;
        }
        self.scroll_y = (self.scroll_y + delta_pixels).clamp(0.0, self.maximum_scroll());
    }

    #[must_use]
    pub fn server_population(&self) -> Option<ServerSelectionPopulation> {
        ServerSelectionPopulation::from_raw(self.server_status)
    }

    #[must_use]
    pub fn shard_population(&self, shard: u8) -> Option<ServerSelectionPopulation> {
        self.shard_statuses
            .get(usize::from(shard))
            .and_then(|raw| ServerSelectionPopulation::from_raw(i32::from(*raw)))
    }
}
