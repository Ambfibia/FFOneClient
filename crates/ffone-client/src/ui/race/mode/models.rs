use super::*;

/// Pure authoritative projection of `cnRaceMode` plus the `GameFrame` race
/// cases that run immediately before it. It owns no socket or world objects.
#[derive(Debug, Resource)]
pub struct RaceModeModel {
    pub(super) phase: RaceModePhase,
    pub(super) ecom_type: Option<RaceEcomType>,
    pub(super) player: RacePlayerState,
    pub(super) npc: Option<RaceNpcContext>,
    pub(super) result: RaceResult,
    pub(super) current_ep_instance_exists: bool,
    pub(super) pending: Option<PendingRaceRequest>,
    pub(super) next_request_id: u64,
    pub(super) outputs: VecDeque<RaceModeOutput>,
}

impl Default for RaceModeModel {
    fn default() -> Self {
        Self {
            phase: RaceModePhase::Hidden,
            ecom_type: None,
            player: RacePlayerState::default(),
            npc: None,
            result: RaceResult::default(),
            current_ep_instance_exists: false,
            pending: None,
            next_request_id: 1,
            outputs: VecDeque::new(),
        }
    }
}

impl RaceModeModel {
    #[must_use]
    pub const fn phase(&self) -> RaceModePhase {
        self.phase
    }

    #[must_use]
    pub const fn ecom_type(&self) -> Option<RaceEcomType> {
        self.ecom_type
    }

    #[must_use]
    pub const fn player(&self) -> &RacePlayerState {
        &self.player
    }

    #[must_use]
    pub const fn result(&self) -> &RaceResult {
        &self.result
    }

    #[must_use]
    pub const fn current_ep_instance_exists(&self) -> bool {
        self.current_ep_instance_exists
    }

    #[must_use]
    pub fn controls_enabled(&self, system_popup_active: bool) -> bool {
        self.phase == RaceModePhase::Result && !system_popup_active
    }

    #[must_use]
    pub const fn pending_request_id(&self) -> Option<u64> {
        match self.pending {
            Some(pending) => Some(pending.request_id),
            None => None,
        }
    }

    #[must_use]
    pub fn output_len(&self) -> usize {
        self.outputs.len()
    }

    pub fn pop_output(&mut self) -> Option<RaceModeOutput> {
        self.outputs.pop_front()
    }

    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }

    pub fn open(&mut self, context: RaceModeOpenContext) -> Result<(), RaceModeOpenError> {
        let request = match context.ecom_type {
            RaceEcomType::Start if !context.player.ring_race_active => {
                let npc = context.npc.ok_or(RaceModeOpenError::MissingStartEcomNpc)?;
                Some((
                    RaceRequestKind::Start,
                    RaceRequestIntent::Start {
                        request_id: self.next_request_id,
                        i_start_ecom_id: npc.instance_id,
                        i_ep_race_mode: RACE_RECORD_MODE,
                        i_ep_ticket_item_slot_num: RACE_TICKET_SLOT,
                    },
                ))
            }
            RaceEcomType::Start => Some((
                RaceRequestKind::Cancel,
                RaceRequestIntent::Cancel {
                    request_id: self.next_request_id,
                    i_start_ecom_id: context.npc.map_or(0, |npc| npc.instance_id),
                },
            )),
            RaceEcomType::End if context.player.ring_race_active => {
                let npc = context.npc.ok_or(RaceModeOpenError::MissingEndEcomNpc)?;
                Some((
                    RaceRequestKind::End,
                    RaceRequestIntent::End {
                        request_id: self.next_request_id,
                        i_end_ecom_id: npc.instance_id,
                        i_ep_ticket_item_slot_num: RACE_TICKET_SLOT,
                    },
                ))
            }
            _ => None,
        };

        self.phase = RaceModePhase::Hidden;
        self.ecom_type = Some(context.ecom_type);
        self.player = context.player;
        self.npc = context.npc;
        self.result = RaceResult::default();
        self.current_ep_instance_exists = context.current_ep_instance_exists;
        self.pending = None;
        self.outputs.clear();
        self.outputs
            .push_back(RaceModeOutput::Effect(RaceModeEffect::SetCursorLocked(
                false,
            )));

        match context.ecom_type {
            RaceEcomType::Start => {
                let (kind, intent) = request.expect("start request was validated above");
                self.queue_request(kind, intent);
            }
            RaceEcomType::End if context.player.ring_race_active => {
                let (kind, intent) = request.expect("end request was validated above");
                self.queue_request(kind, intent);
            }
            RaceEcomType::End => self.finish_and_exit(),
            RaceEcomType::Fail => {
                self.phase = RaceModePhase::FailSystemMessage;
                self.outputs
                    .push_back(RaceModeOutput::Effect(RaceModeEffect::SystemMessage {
                        message_id: RACE_FAIL_SYSTEM_MESSAGE_ID,
                        key: RACE_FAIL_SYSTEM_MESSAGE_KEY,
                    }));
            }
            RaceEcomType::Rank => {
                self.phase = RaceModePhase::DormantRank;
            }
        }
        Ok(())
    }

    pub fn apply_reply(
        &mut self,
        envelope: RaceReplyEnvelope,
        reply_received_local_time: f32,
    ) -> Result<(), RaceModeReplyError> {
        let Some(pending) = self.pending else {
            return Err(RaceModeReplyError::NoPendingRequest);
        };
        if pending.request_id != envelope.request_id {
            return Err(RaceModeReplyError::StaleRequest {
                expected: pending.request_id,
                received: envelope.request_id,
            });
        }
        let received = envelope.reply.kind();
        if pending.kind != received {
            return Err(RaceModeReplyError::WrongReplyKind {
                expected: pending.kind,
                received,
            });
        }

        self.pending = None;
        match envelope.reply {
            RaceModeReply::StartSuccess {
                start_tick: _,
                limit_time,
            } => {
                // Clean GameFrame ignores the server start tick and uses the
                // local Unity Time.time at packet receipt.
                self.player.ring_race_active = true;
                self.player.instance_race_mode = RACE_RECORD_MODE;
                self.player.ring_count = 0;
                self.player.local_start_time = reply_received_local_time;
                self.player.race_limit_time = limit_time;
                self.effect(RaceModeEffect::ActivateRings);
                self.finish_start_reply();
            }
            RaceModeReply::StartFailure { error_code } => {
                self.player.ring_race_active = false;
                self.player.ring_count = 0;
                self.player.race_limit_time = 0;
                self.effect(RaceModeEffect::DeactivateRings);
                if error_code == 8 {
                    self.effect(RaceModeEffect::SystemMessage {
                        message_id: RACE_START_ERROR_8_MESSAGE_ID,
                        key: "",
                    });
                }
                self.finish_and_exit();
            }
            RaceModeReply::CancelSuccess { temporary: _ } => {
                self.player.ring_race_active = false;
                self.player.ring_count = 0;
                self.player.race_limit_time = 0;
                self.effect(RaceModeEffect::DeactivateRings);
                self.finish_cancel_reply();
            }
            RaceModeReply::CancelFailure { error_code: _ } => {
                // RustyFusion rejects cancel when it no longer owns this run.
                self.player.ring_race_active = false;
                self.player.ring_count = 0;
                self.player.race_limit_time = 0;
                self.effect(RaceModeEffect::DeactivateRings);
                self.finish_and_exit();
            }
            RaceModeReply::EndSuccess(reply) => {
                self.player.ring_race_active = false;
                self.player.ring_count = 0;
                self.player.race_limit_time = 0;
                self.effect(RaceModeEffect::DeactivateRings);
                self.result = reply.into();
                self.effect(RaceModeEffect::Ecom(RaceEcomOperation::Delete(
                    RACE_END_ECOM_ICON,
                )));
                self.effect(RaceModeEffect::Ecom(RaceEcomOperation::Make(
                    RACE_START_ECOM_ICON,
                )));
                self.effect(RaceModeEffect::PlaySound(RaceSound::RaceFinish));

                if self.player.fatigue_level != reply.fatigue_level && reply.fatigue_level == 2 {
                    self.effect(RaceModeEffect::MessageBox {
                        message_id: RACE_FATIGUE_MESSAGE_BOX_ID,
                        box_type: RACE_FATIGUE_MESSAGE_BOX_TYPE,
                        copy: RACE_LOW_ENERGY_WARNING,
                    });
                }
                if self.player.fatigue > 0 && reply.fatigue == 0 {
                    self.effect(RaceModeEffect::MessageBox {
                        message_id: RACE_FATIGUE_MESSAGE_BOX_ID,
                        box_type: RACE_FATIGUE_MESSAGE_BOX_TYPE,
                        copy: RACE_EMPTY_ENERGY_WARNING,
                    });
                }
                self.player.fatigue_level = reply.fatigue_level;
                self.player.fatigue = reply.fatigue;
                self.player.fusion_matter = reply.fusion_matter;
                if reply.fatigue_level == 2 {
                    self.effect(RaceModeEffect::CheckFirstUseCondition(
                        RACE_FIRST_USE_LOW_ENERGY_CONDITION,
                    ));
                }
                self.player.top_record = RaceTopRecord {
                    rank: reply.top_rank,
                    rings: reply.top_ring_count,
                    score: reply.top_score,
                    time_seconds: reply.top_time_seconds,
                };
                if reply.reward_item.was_granted() {
                    self.effect(RaceModeEffect::ReceiveRewardItem {
                        inventory_location: reply.reward_item.e_il,
                        inventory_slot: reply.reward_item.slot,
                        item: reply.reward_item,
                    });
                    self.effect(RaceModeEffect::RefreshInventory);
                    self.effect(RaceModeEffect::CheckFirstUseCondition(
                        RACE_FIRST_USE_REWARD_CONDITION,
                    ));
                }
                self.phase = RaceModePhase::Result;
            }
            RaceModeReply::EndFailure { error_code: _ } => {
                self.player.ring_race_active = false;
                self.player.ring_count = 0;
                self.player.race_limit_time = 0;
                self.effect(RaceModeEffect::DeactivateRings);
                self.finish_and_exit();
            }
        }
        Ok(())
    }

    pub fn accept(&mut self) -> bool {
        if self.phase != RaceModePhase::Result {
            return false;
        }
        self.effect(RaceModeEffect::PlaySound(RaceSound::Button));
        self.finish_and_exit();
        true
    }

    /// Models the inventory-manager gate used by Escape (`event 2/24`).
    pub fn escape(&mut self, system_popup_active: bool, inventory_gate_returned_one: bool) -> bool {
        if self.phase.is_busy() || system_popup_active || !inventory_gate_returned_one {
            return false;
        }
        self.finish_and_exit();
        true
    }

    pub(super) fn queue_request(&mut self, kind: RaceRequestKind, intent: RaceRequestIntent) {
        let request_id = intent.request_id();
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.pending = Some(PendingRaceRequest { request_id, kind });
        self.phase = match kind {
            RaceRequestKind::Start => RaceModePhase::AwaitingStart,
            RaceRequestKind::End => RaceModePhase::AwaitingEnd,
            RaceRequestKind::Cancel => RaceModePhase::AwaitingCancel,
        };
        self.outputs.push_back(RaceModeOutput::Request(intent));
    }

    pub(super) fn finish_start_reply(&mut self) {
        self.effect(RaceModeEffect::PlaySound(RaceSound::RaceStart));
        self.effect(RaceModeEffect::Ecom(RaceEcomOperation::Delete(
            RACE_START_ECOM_ICON,
        )));
        self.effect(RaceModeEffect::Ecom(RaceEcomOperation::Make(
            RACE_END_ECOM_ICON,
        )));
        if let Some(npc) = self.npc.filter(|npc| npc.has_race_start_voice) {
            self.effect(RaceModeEffect::PlayNpcRaceStartVoice {
                npc_instance_id: npc.instance_id,
            });
        }
        self.finish_and_exit();
    }

    pub(super) fn finish_cancel_reply(&mut self) {
        self.effect(RaceModeEffect::Ecom(RaceEcomOperation::Delete(
            RACE_END_ECOM_ICON,
        )));
        self.effect(RaceModeEffect::Ecom(RaceEcomOperation::Make(
            RACE_START_ECOM_ICON,
        )));
        self.finish_and_exit();
    }

    pub(super) fn finish_and_exit(&mut self) {
        self.effect(RaceModeEffect::EndCameraSubTarget);
        self.effect(RaceModeEffect::SetCursorLocked(
            self.player.cursor_was_locked,
        ));
        self.effect(RaceModeEffect::ExitMode);
        self.phase = RaceModePhase::Hidden;
        self.pending = None;
    }

    pub(super) fn effect(&mut self, effect: RaceModeEffect) {
        self.outputs.push_back(RaceModeOutput::Effect(effect));
    }
}
