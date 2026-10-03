use super::*;

#[derive(Clone, Debug, Resource)]
pub struct NanoFreeTuningModel {
    pub(super) phase: NanoFreeTuningPhase,
    pub(super) phase_elapsed: f32,
    pub(super) player_id: Option<i32>,
    pub(super) killed_fusion: bool,
    pub(super) content: Option<NanoFreeTuningContent>,
    pub(super) world: Option<NanoFreeTuningWorldSnapshot>,
    pub(super) effect_ready: bool,
    pub(super) panel_visible: bool,
    pub(super) gui_enabled: bool,
    pub(super) selected_power: Option<usize>,
    pub(super) pending: Option<NanoFreeTuningPendingRequest>,
    pub(super) next_request_token: u64,
    pub(super) fault: Option<NanoFreeTuningFault>,
    pub(super) result_animation_duration: Option<f32>,
    pub(super) idle_roll_requested: bool,
    pub(super) intents: VecDeque<NanoFreeTuningIntent>,
}

impl Default for NanoFreeTuningModel {
    fn default() -> Self {
        Self {
            phase: NanoFreeTuningPhase::Closed,
            phase_elapsed: 0.0,
            player_id: None,
            killed_fusion: false,
            content: None,
            world: None,
            effect_ready: false,
            panel_visible: false,
            gui_enabled: true,
            selected_power: None,
            pending: None,
            next_request_token: 1,
            fault: None,
            result_animation_duration: None,
            idle_roll_requested: false,
            intents: VecDeque::new(),
        }
    }
}

impl NanoFreeTuningModel {
    pub fn open(
        &mut self,
        context: NanoFreeTuningOpenContext,
    ) -> Result<(), NanoFreeTuningTransitionError> {
        if self.phase != NanoFreeTuningPhase::Closed {
            return Err(NanoFreeTuningTransitionError::AlreadyOpen);
        }
        context
            .content
            .validate()
            .map_err(NanoFreeTuningTransitionError::InvalidContent)?;

        let nano_id = context.content.nano_id;
        self.phase = if context.killed_fusion {
            NanoFreeTuningPhase::EffectDelay
        } else {
            NanoFreeTuningPhase::CameraSettle
        };
        self.phase_elapsed = 0.0;
        self.player_id = Some(context.player_id);
        self.killed_fusion = context.killed_fusion;
        self.content = Some(context.content);
        self.world = Some(context.world);
        self.effect_ready = false;
        self.panel_visible = false;
        self.gui_enabled = true;
        self.selected_power = None;
        self.pending = None;
        self.fault = None;
        self.result_animation_duration = None;
        self.idle_roll_requested = false;
        self.intents.clear();

        self.intents.push_back(NanoFreeTuningIntent::Effect(
            NanoFreeTuningEffectIntent::Preload {
                effect_id: NANO_FREE_TUNING_EFFECT_ID,
            },
        ));
        if let Some(condition) = nano_free_tuning_entry_first_use_condition(nano_id) {
            self.intents
                .push_back(NanoFreeTuningIntent::FirstUseCheck(condition));
        }
        if let Some(fusion) = context.world.first_defeated_fusion {
            self.intents.push_back(NanoFreeTuningIntent::World(
                NanoFreeTuningWorldIntent::FacePlayerToward(fusion.position),
            ));
        }
        self.intents.push_back(NanoFreeTuningIntent::World(
            NanoFreeTuningWorldIntent::SpawnPreviewNano {
                nano_id,
                initial_position: Vec3::new(0.0, 0.0, -3_000.0),
            },
        ));
        if !context.killed_fusion {
            self.intents.push_back(NanoFreeTuningIntent::World(
                NanoFreeTuningWorldIntent::RevealPreviewNano {
                    position: self.reveal_position(context.world),
                    rotation: self.preview_rotation(context.world),
                },
            ));
        }
        self.intents.push_back(NanoFreeTuningIntent::Cinematic(
            NanoFreeTuningCinematicIntent::LookAt(self.camera_look_target()),
        ));
        self.intents
            .push_back(NanoFreeTuningIntent::SetUpsellUpdate(false));
        Ok(())
    }

    #[must_use]
    pub const fn phase(&self) -> NanoFreeTuningPhase {
        self.phase
    }

    #[must_use]
    pub const fn phase_elapsed(&self) -> f32 {
        self.phase_elapsed
    }

    #[must_use]
    pub const fn panel_visible(&self) -> bool {
        self.panel_visible
    }

    #[must_use]
    pub const fn gui_enabled(&self) -> bool {
        self.gui_enabled
    }

    #[must_use]
    pub const fn selected_power(&self) -> Option<usize> {
        self.selected_power
    }

    #[must_use]
    pub const fn pending_request(&self) -> Option<NanoFreeTuningPendingRequest> {
        self.pending
    }

    #[must_use]
    pub const fn fault(&self) -> Option<NanoFreeTuningFault> {
        self.fault
    }

    #[must_use]
    pub fn content(&self) -> Option<&NanoFreeTuningContent> {
        self.content.as_ref()
    }

    #[must_use]
    pub fn controls_enabled(&self) -> bool {
        self.phase == NanoFreeTuningPhase::PowerSelection
            && self.panel_visible
            && self.gui_enabled
            && self.pending.is_none()
            && self.fault.is_none()
    }

    #[must_use]
    pub fn intents(&self) -> &VecDeque<NanoFreeTuningIntent> {
        &self.intents
    }

    pub fn clear_intents(&mut self) {
        self.intents.clear();
    }

    pub fn drain_intents(
        &mut self,
    ) -> std::collections::vec_deque::Drain<'_, NanoFreeTuningIntent> {
        self.intents.drain(..)
    }

    pub fn mark_effect_ready(&mut self) -> Result<(), NanoFreeTuningTransitionError> {
        if self.phase == NanoFreeTuningPhase::Closed {
            return Err(NanoFreeTuningTransitionError::Closed);
        }
        self.effect_ready = true;
        Ok(())
    }

    pub fn advance(&mut self, delta_seconds: f32) -> Result<(), NanoFreeTuningTransitionError> {
        if self.phase == NanoFreeTuningPhase::Closed {
            return Err(NanoFreeTuningTransitionError::Closed);
        }
        if !delta_seconds.is_finite() || delta_seconds < 0.0 {
            return Err(NanoFreeTuningTransitionError::InvalidDelta);
        }
        self.phase_elapsed += delta_seconds;

        // Recovery extension: a dropped reply must never own input forever.
        // Do not retry or synthesize an authoritative result.
        if self.pending.is_some() && self.phase_elapsed > 15.0 {
            self.intents.push_back(NanoFreeTuningIntent::ProtocolFault(
                NanoFreeTuningProtocolFault::Timeout,
            ));
            return self.cancel();
        }
        if self.phase == NanoFreeTuningPhase::ResultSkill
            && self.result_animation_duration.is_none()
            && self.phase_elapsed > 15.0
        {
            return self.cancel();
        }

        match self.phase {
            NanoFreeTuningPhase::EffectDelay
                if self.phase_elapsed > NANO_FREE_TUNING_EFFECT_DELAY_SECONDS
                    && self.effect_ready =>
            {
                let world = self.world.expect("open mode retains world snapshot");
                let position = self.effect_position(world);
                self.intents.push_back(NanoFreeTuningIntent::Sound(
                    NanoFreeTuningSoundIntent::Play(NANO_FREE_TUNING_CREATION_SOUND),
                ));
                self.intents.push_back(NanoFreeTuningIntent::Effect(
                    NanoFreeTuningEffectIntent::Instantiate {
                        effect_id: NANO_FREE_TUNING_EFFECT_ID,
                        position,
                        rotation: self.anchor_rotation(world),
                    },
                ));
                self.intents.push_back(NanoFreeTuningIntent::Cinematic(
                    NanoFreeTuningCinematicIntent::ConfigureCreationCamera {
                        player_rotation: world.player.rotation,
                        rotation_x: 0.0,
                        distance: 5.0,
                    },
                ));
                self.phase = NanoFreeTuningPhase::ProjectileDelay;
                self.phase_elapsed = 0.0;
            }
            NanoFreeTuningPhase::ProjectileDelay
                if self.phase_elapsed > NANO_FREE_TUNING_PROJECTILE_DELAY_SECONDS =>
            {
                let world = self.world.expect("open mode retains world snapshot");
                self.intents.push_back(NanoFreeTuningIntent::World(
                    NanoFreeTuningWorldIntent::SpawnCreationBullets {
                        bullet_types: NANO_FREE_TUNING_BULLET_TYPES,
                        position: self.reveal_position(world),
                    },
                ));
                self.phase = NanoFreeTuningPhase::RevealDelay;
                self.phase_elapsed = 0.0;
            }
            NanoFreeTuningPhase::RevealDelay
                if self.phase_elapsed > NANO_FREE_TUNING_REVEAL_DELAY_SECONDS =>
            {
                let world = self.world.expect("open mode retains world snapshot");
                self.intents.push_back(NanoFreeTuningIntent::World(
                    NanoFreeTuningWorldIntent::RevealPreviewNano {
                        position: self.reveal_position(world),
                        rotation: self.preview_rotation(world),
                    },
                ));
                self.intents.push_back(NanoFreeTuningIntent::Cinematic(
                    NanoFreeTuningCinematicIntent::CallPreviewNano,
                ));
                self.phase = NanoFreeTuningPhase::CameraApproach;
                self.phase_elapsed = 0.0;
            }
            NanoFreeTuningPhase::CameraApproach => {
                self.intents.push_back(NanoFreeTuningIntent::Cinematic(
                    NanoFreeTuningCinematicIntent::ApproachFrame {
                        rotation_x_delta: -0.7,
                        // Clean subtracts the full phase elapsed value every frame.
                        distance_delta: self.phase_elapsed / 2.0,
                    },
                ));
                if self.phase_elapsed > NANO_FREE_TUNING_CAMERA_APPROACH_SECONDS {
                    self.phase = NanoFreeTuningPhase::CameraSettle;
                    // Clean does not reset fStartTimer when phase 3 becomes 4.
                    if self.phase_elapsed > NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS {
                        self.show_selection_panel();
                    }
                }
            }
            NanoFreeTuningPhase::CameraSettle
                if self.phase_elapsed > NANO_FREE_TUNING_CAMERA_SETTLE_SECONDS =>
            {
                self.show_selection_panel();
            }
            NanoFreeTuningPhase::PowerSelection
                if self.phase_elapsed > NANO_FREE_TUNING_IDLE_SECONDS
                    && self.pending.is_none()
                    && !self.idle_roll_requested =>
            {
                self.idle_roll_requested = true;
                self.phase_elapsed = 0.0;
                self.intents
                    .push_back(NanoFreeTuningIntent::RequestIdleRoll {
                        exclusive_max: NANO_FREE_TUNING_IDLE_RANDOM_EXCLUSIVE_MAX,
                    });
            }
            NanoFreeTuningPhase::ResultSkill => {
                if self
                    .result_animation_duration
                    .is_some_and(|duration| self.phase_elapsed > duration)
                {
                    self.phase = NanoFreeTuningPhase::ResultHide;
                    self.phase_elapsed = 0.0;
                    self.intents.push_back(NanoFreeTuningIntent::Cinematic(
                        NanoFreeTuningCinematicIntent::HidePreviewNano,
                    ));
                }
            }
            NanoFreeTuningPhase::ResultHide
                if self.phase_elapsed > NANO_FREE_TUNING_RESULT_HIDE_SECONDS =>
            {
                // Clean repeats the continuation event each frame while its
                // return value remains non-zero.
                self.gui_enabled = true;
                self.panel_visible = false;
                self.intents.push_back(NanoFreeTuningIntent::Ui(
                    NanoFreeTuningUiIntent::SetEnabled(true),
                ));
                self.intents
                    .push_back(NanoFreeTuningIntent::Ui(NanoFreeTuningUiIntent::Hide));
                self.intents
                    .push_back(NanoFreeTuningIntent::QueryAcquisitionContinuation);
            }
            _ => {}
        }
        Ok(())
    }

    pub fn apply_idle_roll(&mut self, roll: i32) -> Result<(), NanoFreeTuningTransitionError> {
        if !self.idle_roll_requested {
            return Err(NanoFreeTuningTransitionError::IdleRollNotRequested);
        }
        if !(0..NANO_FREE_TUNING_IDLE_RANDOM_EXCLUSIVE_MAX).contains(&roll) {
            return Err(NanoFreeTuningTransitionError::InvalidIdleRoll(roll));
        }
        self.idle_roll_requested = false;
        if roll == 1 {
            self.intents.push_back(NanoFreeTuningIntent::Cinematic(
                NanoFreeTuningCinematicIntent::PlayIdleHappy,
            ));
        }
        // `case 2: Sad()` exists in clean code but is unreachable because
        // Random.Range(0, 2) has an exclusive integer upper bound.
        Ok(())
    }

    pub fn select_power(
        &mut self,
        power_index: usize,
    ) -> Result<(), NanoFreeTuningTransitionError> {
        self.require_selection_phase()?;
        if self.pending.is_some() {
            return Err(NanoFreeTuningTransitionError::RequestAlreadyPending);
        }
        if !self.controls_enabled() {
            return Err(NanoFreeTuningTransitionError::ControlsDisabled);
        }
        if power_index >= NANO_FREE_TUNING_POWER_LAYOUTS.len() {
            return Err(NanoFreeTuningTransitionError::InvalidPowerIndex(
                power_index,
            ));
        }
        self.selected_power = Some(power_index);
        Ok(())
    }

    pub fn confirm_selection(&mut self) -> Result<u64, NanoFreeTuningTransitionError> {
        self.require_selection_phase()?;
        if self.pending.is_some() {
            return Err(NanoFreeTuningTransitionError::RequestAlreadyPending);
        }
        if !self.controls_enabled() {
            return Err(NanoFreeTuningTransitionError::ControlsDisabled);
        }
        let power_index = self
            .selected_power
            .ok_or(NanoFreeTuningTransitionError::MissingSelection)?;
        let content = self.content.as_ref().expect("open mode retains content");
        let power = content.powers.get(power_index).ok_or(
            NanoFreeTuningTransitionError::InvalidPowerIndex(power_index),
        )?;
        let nano_id = content.nano_id;
        let tune_id = power.tune_id;
        let skill_id = power.skill_id;
        let request_token = self.next_request_token;
        self.next_request_token = self.next_request_token.wrapping_add(1).max(1);
        let pending = NanoFreeTuningPendingRequest {
            request_token,
            power_index,
            nano_id,
            tune_id,
            skill_id,
        };
        self.pending = Some(pending);
        self.phase_elapsed = 0.0;
        self.gui_enabled = false;
        self.intents.push_back(NanoFreeTuningIntent::Sound(
            NanoFreeTuningSoundIntent::Play(NANO_FREE_TUNING_SELECT_SOUND),
        ));
        self.intents.push_back(NanoFreeTuningIntent::Ui(
            NanoFreeTuningUiIntent::SetEnabled(false),
        ));
        self.intents
            .push_back(NanoFreeTuningIntent::Wire(NanoTuneWireIntent {
                request_token,
                packet_id: NANO_TUNE_REQUEST_PACKET_ID,
                payload_size: NANO_TUNE_REQUEST_SIZE,
                body: NanoTuneRequest {
                    nano_id,
                    tune_id,
                    // Free tuning has no item costs; the fixed packet tail is zeroed.
                    needed_item_slots: [0; NANO_TUNE_ITEM_SLOT_COUNT],
                },
            }));
        Ok(request_token)
    }

    /// Exact clean button behavior: remember the displayed power and send its
    /// request in the same click.
    pub fn click_power(
        &mut self,
        power_index: usize,
    ) -> Result<u64, NanoFreeTuningTransitionError> {
        self.select_power(power_index)?;
        self.confirm_selection()
    }

    pub fn apply_reply(
        &mut self,
        envelope: NanoFreeTuningReplyEnvelope,
    ) -> Result<(), NanoFreeTuningTransitionError> {
        let Some(pending) = self.pending else {
            return self.fail_protocol(NanoFreeTuningProtocolFault::NoPendingRequest);
        };
        if envelope.request_token != pending.request_token {
            return self.fail_protocol(NanoFreeTuningProtocolFault::StaleRequestToken {
                expected: pending.request_token,
                actual: envelope.request_token,
            });
        }

        match envelope.packet_id {
            NANO_TUNE_SUCCESS_PACKET_ID => {
                if envelope.payload_size != NANO_TUNE_SUCCESS_SIZE {
                    return self.fail_protocol(NanoFreeTuningProtocolFault::PayloadSize {
                        packet_id: envelope.packet_id,
                        expected: NANO_TUNE_SUCCESS_SIZE,
                        actual: envelope.payload_size,
                    });
                }
                let NanoFreeTuningReplyBody::Success(success) = envelope.body else {
                    return self.fail_protocol(NanoFreeTuningProtocolFault::BodyKindMismatch {
                        packet_id: envelope.packet_id,
                    });
                };
                if success.nano_id != pending.nano_id {
                    return self.fail_protocol(NanoFreeTuningProtocolFault::NanoMismatch {
                        expected: pending.nano_id,
                        actual: success.nano_id,
                    });
                }
                if success.skill_id != pending.skill_id {
                    return self.fail_protocol(NanoFreeTuningProtocolFault::SkillMismatch {
                        expected: pending.skill_id,
                        actual: success.skill_id,
                    });
                }

                self.pending = None;
                self.fault = None;
                self.phase = NanoFreeTuningPhase::ResultSkill;
                self.phase_elapsed = 0.0;
                self.result_animation_duration = None;
                self.intents.push_back(NanoFreeTuningIntent::Cinematic(
                    NanoFreeTuningCinematicIntent::PlaySelectedSkill {
                        power_index: pending.power_index,
                    },
                ));
                self.intents
                    .push_back(NanoFreeTuningIntent::AuthoritativeCommit(success));
                self.intents.push_back(NanoFreeTuningIntent::DeleteEcomIcon(
                    NANO_FREE_TUNING_ECOM_ICON,
                ));
                Ok(())
            }
            NANO_TUNE_FAILURE_PACKET_ID => {
                if envelope.payload_size != NANO_TUNE_FAILURE_SIZE {
                    return self.fail_protocol(NanoFreeTuningProtocolFault::PayloadSize {
                        packet_id: envelope.packet_id,
                        expected: NANO_TUNE_FAILURE_SIZE,
                        actual: envelope.payload_size,
                    });
                }
                let NanoFreeTuningReplyBody::Failure(failure) = envelope.body else {
                    return self.fail_protocol(NanoFreeTuningProtocolFault::BodyKindMismatch {
                        packet_id: envelope.packet_id,
                    });
                };
                let player_id = self.player_id.expect("open mode retains player id");
                if failure.player_id != player_id {
                    return self.fail_protocol(NanoFreeTuningProtocolFault::PlayerMismatch {
                        expected: player_id,
                        actual: failure.player_id,
                    });
                }

                // Fail closed: preserve the panel, keep controls disabled, and
                // never synthesize the success-side Nano/inventory mutation.
                self.pending = None;
                self.fault = Some(NanoFreeTuningFault::ServerRejected {
                    player_id: failure.player_id,
                    error_code: failure.error_code,
                });
                self.gui_enabled = false;
                self.intents
                    .push_back(NanoFreeTuningIntent::AuthoritativeFailure(failure));
                Ok(())
            }
            packet_id => {
                self.fail_protocol(NanoFreeTuningProtocolFault::UnexpectedPacketId(packet_id))
            }
        }
    }

    pub fn set_result_animation_duration(
        &mut self,
        duration_seconds: f32,
    ) -> Result<(), NanoFreeTuningTransitionError> {
        if self.phase != NanoFreeTuningPhase::ResultSkill {
            return Err(NanoFreeTuningTransitionError::WrongPhase {
                expected: NanoFreeTuningPhase::ResultSkill,
                actual: self.phase,
            });
        }
        if !duration_seconds.is_finite() || duration_seconds < 0.0 {
            return Err(NanoFreeTuningTransitionError::InvalidAnimationDuration);
        }
        // A server reply may precede scene/animation readiness. Start the
        // result clock when playback is bound, not while it is still queued.
        if self.result_animation_duration.is_none() {
            self.phase_elapsed = 0.0;
        }
        self.result_animation_duration = Some(duration_seconds);
        Ok(())
    }

    pub fn reject_protocol_fault(
        &mut self,
        fault: NanoFreeTuningProtocolFault,
    ) -> Result<(), NanoFreeTuningTransitionError> {
        self.fail_protocol(fault)
    }

    pub fn resolve_acquisition_continuation(
        &mut self,
        legacy_return: i32,
    ) -> Result<(), NanoFreeTuningTransitionError> {
        if self.phase != NanoFreeTuningPhase::ResultHide {
            return Err(NanoFreeTuningTransitionError::WrongPhase {
                expected: NanoFreeTuningPhase::ResultHide,
                actual: self.phase,
            });
        }
        if legacy_return == 0 {
            self.cancel()?;
        }
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), NanoFreeTuningTransitionError> {
        if self.phase == NanoFreeTuningPhase::Closed {
            return Err(NanoFreeTuningTransitionError::Closed);
        }
        self.intents.push_back(NanoFreeTuningIntent::Cinematic(
            NanoFreeTuningCinematicIntent::EndSubTarget,
        ));
        self.intents.push_back(NanoFreeTuningIntent::Ui(
            NanoFreeTuningUiIntent::RestoreCapturedCursor,
        ));
        self.intents.push_back(NanoFreeTuningIntent::World(
            NanoFreeTuningWorldIntent::DestroyPreviewNano,
        ));
        self.intents.push_back(NanoFreeTuningIntent::ExitMode);
        self.phase = NanoFreeTuningPhase::Closed;
        self.phase_elapsed = 0.0;
        self.player_id = None;
        self.killed_fusion = false;
        self.content = None;
        self.world = None;
        self.effect_ready = false;
        self.panel_visible = false;
        self.gui_enabled = true;
        self.selected_power = None;
        self.pending = None;
        self.fault = None;
        self.result_animation_duration = None;
        self.idle_roll_requested = false;
        Ok(())
    }

    pub(super) fn require_selection_phase(&self) -> Result<(), NanoFreeTuningTransitionError> {
        if self.phase != NanoFreeTuningPhase::PowerSelection {
            return Err(NanoFreeTuningTransitionError::WrongPhase {
                expected: NanoFreeTuningPhase::PowerSelection,
                actual: self.phase,
            });
        }
        Ok(())
    }

    pub(super) fn fail_protocol<T>(
        &mut self,
        fault: NanoFreeTuningProtocolFault,
    ) -> Result<T, NanoFreeTuningTransitionError> {
        self.pending = None;
        self.gui_enabled = false;
        self.fault = Some(NanoFreeTuningFault::Protocol(fault));
        self.intents
            .push_back(NanoFreeTuningIntent::ProtocolFault(fault));
        Err(NanoFreeTuningTransitionError::Protocol(fault))
    }

    pub(super) fn show_selection_panel(&mut self) {
        let world = self.world.expect("open mode retains world snapshot");
        self.intents.push_back(NanoFreeTuningIntent::Cinematic(
            NanoFreeTuningCinematicIntent::SetSubTarget {
                preview_rotation: self.preview_rotation(world),
                player_rotation: world.player.rotation,
                distance: 2.0,
                height: 0.6,
            },
        ));
        self.intents.push_back(NanoFreeTuningIntent::Cinematic(
            NanoFreeTuningCinematicIntent::SetStandMotion,
        ));
        self.intents
            .push_back(NanoFreeTuningIntent::Ui(NanoFreeTuningUiIntent::Show));
        self.intents.push_back(NanoFreeTuningIntent::Ui(
            NanoFreeTuningUiIntent::CaptureAndUnlockCursor,
        ));
        if self
            .content
            .as_ref()
            .is_some_and(|content| content.nano_id == 2)
        {
            self.intents
                .push_back(NanoFreeTuningIntent::FirstUseCheck(23));
        }
        self.phase = NanoFreeTuningPhase::PowerSelection;
        self.phase_elapsed = 0.0;
        self.panel_visible = true;
        self.gui_enabled = true;
    }

    pub(super) fn camera_look_target(&self) -> Vec3 {
        let world = self.world.expect("open mode retains world snapshot");
        let mut target = world
            .first_defeated_fusion
            .map_or(world.player.position, |fusion| fusion.position);
        target.y += 1.0;
        target
    }

    pub(super) fn anchor_rotation(&self, world: NanoFreeTuningWorldSnapshot) -> Quat {
        world
            .first_defeated_fusion
            .map_or(world.player.rotation, |fusion| fusion.rotation)
    }

    pub(super) fn effect_position(&self, world: NanoFreeTuningWorldSnapshot) -> Vec3 {
        world.first_defeated_fusion.map_or_else(
            || world.player.position + world.player.rotation * Vec3::NEG_Z * 2.0,
            |fusion| fusion.position,
        )
    }

    pub(super) fn reveal_position(&self, world: NanoFreeTuningWorldSnapshot) -> Vec3 {
        let mut position = world.first_defeated_fusion.map_or_else(
            || world.player.position + world.player.rotation * Vec3::NEG_Z * 3.0,
            |fusion| fusion.position,
        );
        position.y += 3.0;
        position
    }

    pub(super) fn preview_rotation(&self, world: NanoFreeTuningWorldSnapshot) -> Quat {
        world.player.rotation * Quat::from_rotation_y(std::f32::consts::PI)
    }
}
