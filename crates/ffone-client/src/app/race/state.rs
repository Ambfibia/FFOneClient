use super::*;

#[derive(Debug, Resource)]
pub(in super::super) struct RaceProductionRuntime {
    pub(in super::super) player: RacePlayerState,
    pub(in super::super) active_game_mode: Option<u8>,
    pub(in super::super) source_npc: Option<RaceNpcSession>,
    pub(in super::super) visible_ecom_icons: BTreeSet<i32>,
    pub(in super::super) ring_activation_requested: Option<bool>,
    pub(in super::super) course_bounds: Option<[i32; 4]>,
    pub(in super::super) race_ui_complete: bool,
    pub(in super::super) ring_ids: BTreeMap<([i32; 3], String), i32>,
    pub(in super::super) collected_rings: BTreeSet<i32>,
    pub(in super::super) pending_rings: BTreeSet<i32>,
    pub(in super::super) pending_npc_voice: Option<PendingRaceNpcVoice>,
    pub(in super::super) finished_voice_npc_id: Option<i32>,
    pub(in super::super) pending_ring_cues: usize,
    pub(in super::super) pending_system_messages: BTreeMap<u64, i32>,
    pub(in super::super) next_system_message_id: u64,
    pub(in super::super) voice_rng: u32,
    pub(in super::super) first_use_checks: Vec<i32>,
    pub(in super::super) last_message_box: Option<(i32, i32, &'static str)>,
    pub(in super::super) last_owned_packet: Option<u32>,
    pub(in super::super) rank_http_rejections: u64,
}

impl Default for RaceProductionRuntime {
    fn default() -> Self {
        Self {
            player: RacePlayerState::default(),
            active_game_mode: None,
            source_npc: None,
            visible_ecom_icons: BTreeSet::new(),
            ring_activation_requested: None,
            course_bounds: None,
            race_ui_complete: false,
            ring_ids: BTreeMap::new(),
            collected_rings: BTreeSet::new(),
            pending_rings: BTreeSet::new(),
            pending_npc_voice: None,
            finished_voice_npc_id: None,
            pending_ring_cues: 0,
            pending_system_messages: BTreeMap::new(),
            next_system_message_id: RACE_SYSTEM_MESSAGE_ID_BASE,
            voice_rng: 0x7261_6365,
            first_use_checks: Vec::new(),
            last_message_box: None,
            last_owned_packet: None,
            rank_http_rejections: 0,
        }
    }
}

impl RaceProductionRuntime {
    pub(in super::super) fn owns_final_voice_for(&self, npc_id: i32) -> bool {
        self.finished_voice_npc_id == Some(npc_id)
    }

    pub(in super::super) fn confirm_ring(&mut self, ring_id: i32, count: i32) -> bool {
        if !self.player.ring_race_active
            || !self.pending_rings.remove(&ring_id)
            || !self.collected_rings.insert(ring_id)
        {
            return false;
        }
        self.player.ring_count = count;
        self.pending_ring_cues += 1;
        true
    }

    pub(in super::super) fn reset(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub(in super::super) fn modal_active(&self) -> bool {
        matches!(
            self.active_game_mode,
            Some(RACE_MODE_GAME_MODE_ID) | Some(RACE_RANK_GAME_MODE_ID)
        )
    }

    #[must_use]
    pub(in super::super) fn camera_subtarget_npc_id(&self) -> Option<i32> {
        self.source_npc
            .as_ref()
            .filter(|source| source.camera_subtarget_active)
            .map(|source| source.runtime_npc_id)
    }

    pub(in super::super) fn begin_mode(
        &mut self,
        game_mode: u8,
        runtime_npc_id: i32,
        table_npc_id: i32,
        voice_owner: String,
    ) -> Result<(), &'static str> {
        if self.active_game_mode.is_some() {
            return Err("another race-owned game mode is already active");
        }
        self.active_game_mode = Some(game_mode);
        self.source_npc = Some(RaceNpcSession {
            runtime_npc_id,
            table_npc_id,
            voice_owner,
            camera_subtarget_active: true,
        });
        self.pending_npc_voice = None;
        self.finished_voice_npc_id = None;
        self.last_message_box = None;
        Ok(())
    }

    pub(in super::super) fn finish_mode(&mut self, game_mode: u8) {
        if self.active_game_mode == Some(game_mode) {
            self.active_game_mode = None;
            self.source_npc = None;
            self.pending_npc_voice = None;
        }
    }

    pub(in super::super) fn next_random(&mut self) -> u32 {
        self.voice_rng ^= self.voice_rng << 13;
        self.voice_rng ^= self.voice_rng >> 17;
        self.voice_rng ^= self.voice_rng << 5;
        self.voice_rng
    }

    pub(in super::super) fn next_voice_take(&mut self) -> usize {
        self.next_random() as usize % 3 + 1
    }

    pub(in super::super) fn next_button_sound_path(&mut self) -> &'static str {
        const PATHS: [&str; 5] = [
            "audio/sfx/ui/mouse_click01.ogg",
            "audio/sfx/ui/mouse_click02.ogg",
            "audio/sfx/ui/mouse_click03.ogg",
            "audio/sfx/ui/mouse_click04.ogg",
            "audio/sfx/ui/mouse_click05.ogg",
        ];
        PATHS[self.next_random() as usize % PATHS.len()]
    }

    pub(in super::super) fn queue_system_message(
        &mut self,
        content: &TutorialMissionContent,
        messages: &mut SystemMessageUiModel,
        legacy_message_id: i32,
    ) -> Result<u64, String> {
        let definition = content
            .system_message_definition(legacy_message_id)
            .ok_or_else(|| {
                format!("RaceMode SystemMessage {legacy_message_id} has no source TableData row")
            })?;
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < RACE_SYSTEM_MESSAGE_ID_BASE {
            self.next_system_message_id = RACE_SYSTEM_MESSAGE_ID_BASE;
        }
        self.pending_system_messages
            .insert(request_id, legacy_message_id);
        messages.push(SystemMessageRequest::new(
            request_id,
            definition.exact_text.clone(),
            definition.runtime_button_type,
        ));
        Ok(request_id)
    }
}

pub(in super::super) fn open_race_mode_from_npc(
    model: &mut RaceModeModel,
    catalog: &RaceRankCatalog,
    production: &mut RaceProductionRuntime,
    ecom_type: RaceEcomType,
    runtime_npc_id: i32,
    table_npc_id: i32,
    voice_owner: String,
    fusion_matter: i32,
    cursor_was_locked: bool,
) -> Result<(), String> {
    if production.active_game_mode.is_some() || model.phase() != RaceModePhase::Hidden {
        return Err("another RaceMode lifecycle is already active".to_owned());
    }
    let mut player = production.player;
    player.fusion_matter = fusion_matter;
    player.cursor_was_locked = cursor_was_locked;
    let current_ep_instance_exists = catalog.location_by_ep(player.current_ep_id).is_some();
    model
        .open(RaceModeOpenContext {
            ecom_type,
            npc: Some(RaceNpcContext {
                instance_id: runtime_npc_id,
                has_race_start_voice: !voice_owner.is_empty(),
            }),
            player,
            current_ep_instance_exists,
        })
        .map_err(|error| format!("RaceMode initialization rejected: {error:?}"))?;
    if let Err(error) = production.begin_mode(
        RACE_MODE_GAME_MODE_ID,
        runtime_npc_id,
        table_npc_id,
        voice_owner,
    ) {
        *model = RaceModeModel::default();
        return Err(format!("RaceMode lifecycle rejected: {error}"));
    }
    production.player = player;
    if ecom_type == RaceEcomType::Start && !player.ring_race_active {
        production.race_ui_complete = false;
    }
    Ok(())
}

pub(in super::super) fn consume_race_mode_ui_commands(
    mut model: ResMut<RaceModeModel>,
    mut commands: ResMut<RaceModeUiCommandOutbox>,
    mut status: ResMut<RuntimeStatus>,
) {
    while let Some(command) = commands.pop() {
        let accepted = match command {
            RaceModeUiCommand::Accept => model.accept(),
        };
        if !accepted {
            status.message = format!(
                "RaceMode input {command:?} rejected in phase {:?}",
                model.phase()
            );
        }
    }
}

pub(in super::super) fn abort_race_mode_transport(
    model: &mut RaceModeModel,
    production: &mut RaceProductionRuntime,
    cursors: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let restore_cursor = production.player.cursor_was_locked;
    *model = RaceModeModel::default();
    production.finish_mode(RACE_MODE_GAME_MODE_ID);
    set_race_cursor_locked(cursors, restore_cursor);
}
