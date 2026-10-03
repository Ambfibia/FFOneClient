use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct SkillBuffUiModel {
    pub visible: bool,
    pub ui_scale: f32,
    pub local_character_id: Option<i32>,
    pub local_condition_bit_flag: u32,
    pub cash_condition_bit_flag: u32,
    pub target: Option<SkillBuffTargetUi>,
    pub nano_styles: [Option<u8>; 3],
    pub(super) target_condition_cache: BTreeMap<(i32, i32), u32>,
    pub(super) cash_server_duration_ms: [u64; SKILL_BUFF_MAX_ID + 1],
    pub(super) cash_remaining_ms: [u64; SKILL_BUFF_MAX_ID + 1],
    pub(super) cash_tick_elapsed_seconds: f32,
}

impl Default for SkillBuffUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            ui_scale: 1.0,
            local_character_id: None,
            local_condition_bit_flag: 0,
            cash_condition_bit_flag: 0,
            target: None,
            nano_styles: [None; 3],
            target_condition_cache: BTreeMap::new(),
            cash_server_duration_ms: [0; SKILL_BUFF_MAX_ID + 1],
            cash_remaining_ms: [0; SKILL_BUFF_MAX_ID + 1],
            cash_tick_elapsed_seconds: 0.0,
        }
    }
}

impl SkillBuffUiModel {
    /// Status buff 10 is stun; buff 11 is sleep and wins their ordered projection.
    pub fn local_control_condition(&self) -> i32 {
        if self.local_condition_bit_flag & 0x400 != 0 {
            6
        } else if self.local_condition_bit_flag & 0x200 != 0 {
            5
        } else {
            0
        }
    }

    pub fn reveals_mobs(&self) -> bool {
        self.local_condition_bit_flag & 0x1000 != 0
    }
    pub fn reveals_shinies(&self) -> bool {
        self.local_condition_bit_flag & 0x2000 != 0
    }

    pub fn seed_from_pc_load(&mut self, local_character_id: i32, load: &PcLoadData0104) {
        self.local_character_id = Some(local_character_id);
        self.local_condition_bit_flag = load.condition_bit_flag() as u32;
    }

    pub fn apply_packet(&mut self, packet: SkillBuffPacket0104) {
        match packet {
            SkillBuffPacket0104::Pc(update) => self.apply_pc_update(update),
            SkillBuffPacket0104::Cash(update) => self.apply_cash_update(update),
            SkillBuffPacket0104::Timeout(timeout) => self.apply_timeout(timeout),
        }
    }

    pub fn apply_pc_update(&mut self, update: PcBuffUpdate0104) {
        // Clean membership follows the authoritative bit mask regardless of
        // Add/Del/Change. Its TimeBuff fields affect status mechanics, not
        // this icon row.
        self.local_condition_bit_flag = update.condition_bit_flag as u32;
    }

    pub fn apply_cash_update(&mut self, update: PcCashBuffUpdate0104) {
        self.cash_condition_bit_flag = update.condition_bit_flag as u32;
        if let Ok(buff_id) = usize::try_from(update.buff_id)
            && let Some(server_duration) = self.cash_server_duration_ms.get_mut(buff_id)
        {
            match update.update_kind {
                1 | 3 => *server_duration = update.time_buff.time_duration,
                2 => *server_duration = 0,
                _ => {}
            }
        }
        // UpdateCashIconList discards arrCashIconTime and rebuilds every
        // active displayed entry from Status.arrSkillCashBuffTime's retained
        // server duration. Thus an update for one buff restores every other
        // active countdown to its last server-supplied value.
        self.cash_remaining_ms.fill(0);
        for buff_id in cash_buff_ids(self.cash_condition_bit_flag) {
            let Ok(buff_id) = usize::try_from(buff_id) else {
                continue;
            };
            if let (Some(displayed), Some(server_duration)) = (
                self.cash_remaining_ms.get_mut(buff_id),
                self.cash_server_duration_ms.get(buff_id),
            ) {
                *displayed = *server_duration;
            }
        }
        // The same rebuild assigns fStartTime=Time.time, so every cash update
        // restarts the one-second tick window even when eTBU is unknown.
        self.cash_tick_elapsed_seconds = 0.0;
    }

    pub fn apply_timeout(&mut self, timeout: CharTimeBuffTimeout0104) {
        // This mask also owns world effects and radar visibility. A timeout is
        // authoritative even when the local character is not the selected target.
        if timeout.character_type == 1 && self.local_character_id == Some(timeout.character_id) {
            self.local_condition_bit_flag = timeout.condition_bit_flag as u32;
        }
        let key = (timeout.character_type, timeout.character_id);
        self.target_condition_cache
            .insert(key, timeout.condition_bit_flag as u32);
        if self.target.is_some_and(|target| {
            target.character_type == timeout.character_type
                && target.character_id == timeout.character_id
        }) {
            self.target = Some(SkillBuffTargetUi {
                character_type: timeout.character_type,
                character_id: timeout.character_id,
                condition_bit_flag: timeout.condition_bit_flag as u32,
            });
        }
    }

    pub fn apply_dot_damage_tick(&mut self, tick: TimeBuffDotDamageTick0104) {
        // Status.BuffTimeTick mutates infection condition membership only on
        // the protected branch. Its own-avatar path then returns without
        // emitting the local CnGuiSkillBuffIcon refresh, so the local row must
        // continue to wait for PC_BUFF_UPDATE.
        if !tick.protected {
            return;
        }
        let owner_character_type = if tick.character_type == 4 {
            // OpenFusion's NPC/mob status identity is normalized to the same
            // UI target type used by its TIME_OUT compatibility path.
            2
        } else {
            tick.character_type
        };
        self.target_condition_cache.insert(
            (owner_character_type, tick.character_id),
            tick.condition_bit_flag as u32,
        );
        if self.target.is_some_and(|target| {
            target.character_type == owner_character_type
                && target.character_id == tick.character_id
        }) {
            self.target = Some(SkillBuffTargetUi {
                character_type: owner_character_type,
                character_id: tick.character_id,
                condition_bit_flag: tick.condition_bit_flag as u32,
            });
        }
    }

    /// Selects the clean status owner without discarding condition packets
    /// received while that owner was off-target.
    ///
    /// `appearance_changed` is meaningful only while the same target remains
    /// selected. A newly selected target first consumes its retained status
    /// cache because its appearance component can still carry an older mask
    /// than a timeout/tick packet already received in this session.
    pub fn select_target_from_appearance(
        &mut self,
        character_type: i32,
        character_id: i32,
        appearance_condition_bit_flag: u32,
        appearance_changed: bool,
    ) {
        let key = (character_type, character_id);
        let identity_unchanged = self.target.is_some_and(|target| {
            target.character_type == character_type && target.character_id == character_id
        });
        if identity_unchanged && appearance_changed {
            self.target_condition_cache
                .insert(key, appearance_condition_bit_flag);
        }
        let condition_bit_flag = *self
            .target_condition_cache
            .entry(key)
            .or_insert(appearance_condition_bit_flag);
        self.target = Some(SkillBuffTargetUi {
            character_type,
            character_id,
            condition_bit_flag,
        });
    }

    pub fn clear_target_selection(&mut self) {
        self.target = None;
    }

    /// Status ownership follows live network entities. Retaining only those
    /// keys prevents a recycled NPC ID from inheriting a despawned owner's
    /// condition bits while preserving off-target updates for living owners.
    pub fn retain_target_conditions(&mut self, live_targets: &BTreeSet<(i32, i32)>) {
        self.target_condition_cache
            .retain(|key, _| live_targets.contains(key));
    }

    pub fn set_cash_remaining_ms(&mut self, buff_id: i32, remaining_ms: u64) {
        if let Ok(buff_id) = usize::try_from(buff_id)
            && let (Some(server_duration), Some(remaining)) = (
                self.cash_server_duration_ms.get_mut(buff_id),
                self.cash_remaining_ms.get_mut(buff_id),
            )
        {
            *server_duration = remaining_ms;
            *remaining = remaining_ms;
        }
    }

    #[must_use]
    pub fn cash_remaining_ms(&self, buff_id: i32) -> u64 {
        usize::try_from(buff_id)
            .ok()
            .and_then(|buff_id| self.cash_remaining_ms.get(buff_id))
            .copied()
            .unwrap_or(0)
    }

    pub fn advance_cash_timer(&mut self, delta_seconds: f32) {
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return;
        }
        self.cash_tick_elapsed_seconds += delta_seconds;
        if self.cash_tick_elapsed_seconds < 1.0 {
            return;
        }
        for buff_id in cash_buff_ids(self.cash_condition_bit_flag) {
            let Ok(buff_id) = usize::try_from(buff_id) else {
                continue;
            };
            let Some(remaining) = self.cash_remaining_ms.get_mut(buff_id) else {
                continue;
            };
            // The clean C# build performs unchecked ulong subtraction and
            // only one subtraction per Update, even after a long frame.
            *remaining = remaining.wrapping_sub(1_000);
        }
        self.cash_tick_elapsed_seconds = 0.0;
    }
}
