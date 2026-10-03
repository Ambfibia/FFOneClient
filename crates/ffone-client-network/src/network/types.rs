use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CharacterEntryLocation0104 {
    /// Enter at the position and angle persisted by the shard.
    #[default]
    Saved,
    /// Enter the canonical Sector V overworld pose after completing the
    /// Future tutorial. This is explicit because FFOne runs the tutorial on
    /// the ordinary shard, whose temporary live pose may already have been
    /// flushed into both the player database and the retained login roster.
    TutorialCompletionSectorV,
    /// A sequence-owned entry pose that must be applied before the shard
    /// builds its initial chunk/NPC bootstrap.
    Scripted { position: [i32; 3], angle: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterSummary {
    pub slot: i8,
    pub level: i16,
    pub pc_uid: i64,
    pub first_name: String,
    pub last_name: String,
    pub position: [i32; 3],
    pub style: CharacterStyle0104,
    pub equipment: [EquippedItem0104; CHARACTER_EQUIP_SLOT_COUNT_0104],
}

#[derive(Debug, Clone)]
pub struct WorldReady {
    /// Stable account character identity selected on the login server.
    pub pc_uid: i64,
    /// Runtime entity identity assigned by the shard server.
    pub player_id: i32,
    /// Exact `PC_ENTER_SUCC.uiSvrTime` observed from the shard.
    pub server_time: u64,
    pub map_number: i32,
    pub hp: i32,
    pub position: [i32; 3],
    pub angle: i32,
    /// Exact `sPCStyle2` flags retained from the login character roster.
    ///
    /// OpenFusion intentionally leaves `PCLoadData2CL.PCStyle2` zeroed during
    /// shard entry because the legacy client keeps these flags from
    /// `P_LS2CL_REP_CHAR_INFO`. In particular, tutorial completion mutates the
    /// retained roster before `CHAR_SELECT`, so this is the authoritative
    /// tutorial/world-map discriminator for the new shard session.
    pub login_style: CharacterStyle0104,
    /// The complete protocol-0104 player load record. Keeping this lossless is required for
    /// native body/equipment/nano/inventory/quest assembly instead of a fixed visual fallback.
    pub load: PcLoadData0104,
    pub bootstrap: WorldBootstrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterOperationStage {
    NameCheck,
    NameSave,
    Appearance,
    Delete,
    Rename,
}

/// Client-local request context for correlating the clean Nano tune response.
/// `request_token` is deliberately absent from the legacy wire body; the
/// remaining fields are the exact identities that a reply must agree with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NanoTunePending0104 {
    pub request_token: u64,
    pub player_id: i32,
    pub nano_id: i16,
    pub skill_id: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NanoTuneCorrelationFault0104 {
    NanoId { expected: i16, actual: i16 },
    SkillId { expected: i16, actual: i16 },
    PlayerId { expected: i32, actual: i32 },
}

/// A known Nano tune frame that could not be safely committed. The complete
/// raw frame is retained for diagnostics and for lossless higher-level routing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NanoTuneCorrelationError0104 {
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Mismatch {
        frame: DecodedFrame,
        fault: NanoTuneCorrelationFault0104,
    },
}

/// A decoded and identity-checked Nano tune reply. Its server-provided Fusion
/// Matter and inventory records remain authoritative and unmodified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorrelatedNanoTuneReply0104 {
    pub request_token: u64,
    pub frame: DecodedFrame,
    pub packet: NanoTunePacket0104,
}

impl CorrelatedNanoTuneReply0104 {
    pub const fn packet_id(&self) -> u32 {
        self.frame.packet_type
    }

    pub fn payload_size(&self) -> usize {
        self.frame.payload.len()
    }
}

impl NanoTunePending0104 {
    pub fn correlate(
        self,
        classified: NanoTuneGameplayFrame0104,
    ) -> Result<Option<CorrelatedNanoTuneReply0104>, NanoTuneCorrelationError0104> {
        let (frame, packet) = match classified {
            NanoTuneGameplayFrame0104::Decoded { frame, packet } => (frame, packet),
            NanoTuneGameplayFrame0104::Malformed { frame, error } => {
                return Err(NanoTuneCorrelationError0104::Malformed { frame, error });
            }
            NanoTuneGameplayFrame0104::Passthrough(_) => return Ok(None),
        };

        let fault = match packet {
            NanoTunePacket0104::Success(reply) if reply.nano_id != self.nano_id => {
                Some(NanoTuneCorrelationFault0104::NanoId {
                    expected: self.nano_id,
                    actual: reply.nano_id,
                })
            }
            NanoTunePacket0104::Success(reply) if reply.skill_id != self.skill_id => {
                Some(NanoTuneCorrelationFault0104::SkillId {
                    expected: self.skill_id,
                    actual: reply.skill_id,
                })
            }
            NanoTunePacket0104::Failure(reply) if reply.pc_id != self.player_id => {
                Some(NanoTuneCorrelationFault0104::PlayerId {
                    expected: self.player_id,
                    actual: reply.pc_id,
                })
            }
            _ => None,
        };
        if let Some(fault) = fault {
            return Err(NanoTuneCorrelationError0104::Mismatch { frame, fault });
        }

        Ok(Some(CorrelatedNanoTuneReply0104 {
            request_token: self.request_token,
            frame,
            packet,
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkStream0104 {
    Login,
    Shard,
}

impl fmt::Display for NetworkStream0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Login => formatter.write_str("login"),
            Self::Shard => formatter.write_str("shard"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectReason0104 {
    Requested,
    LoginTransportFailed(String),
    ShardTransportFailed(String),
}

impl fmt::Display for DisconnectReason0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Requested => formatter.write_str("disconnect requested by the client"),
            Self::LoginTransportFailed(error) => {
                write!(formatter, "login transport failed: {error}")
            }
            Self::ShardTransportFailed(error) => {
                write!(formatter, "shard transport failed: {error}")
            }
        }
    }
}

#[derive(Resource)]
pub struct NetworkBridge {
    pub(super) commands: SyncSender<NetworkCommand>,
    pub(super) events: Mutex<Receiver<NetworkEvent>>,
}

impl NetworkBridge {
    pub const COMMAND_QUEUE_CAPACITY: usize = 256;
    pub const EVENT_QUEUE_CAPACITY: usize = 2048;

    pub fn start() -> Self {
        let (command_tx, command_rx) = mpsc::sync_channel(Self::COMMAND_QUEUE_CAPACITY);
        let (event_tx, event_rx) = mpsc::sync_channel(Self::EVENT_QUEUE_CAPACITY);
        thread::Builder::new()
            .name("ffone-network".to_owned())
            .spawn(move || network_worker(command_rx, event_tx))
            .expect("spawn FFOne network worker");
        Self {
            commands: command_tx,
            events: Mutex::new(event_rx),
        }
    }

    pub fn send(&self, command: NetworkCommand) -> Result<(), String> {
        self.commands
            .try_send(command)
            .map_err(|error| match error {
                TrySendError::Full(_) => format!(
                    "network command queue is full (capacity {})",
                    Self::COMMAND_QUEUE_CAPACITY
                ),
                TrySendError::Disconnected(_) => "network worker has stopped".to_owned(),
            })
    }

    pub fn drain(&self) -> Vec<NetworkEvent> {
        let receiver = self.events.lock().expect("network event lock poisoned");
        receiver.try_iter().collect()
    }
}

impl Drop for NetworkBridge {
    fn drop(&mut self) {
        let _ = self.commands.try_send(NetworkCommand::Shutdown);
    }
}

pub(super) struct GameplayHandle {
    pub(super) sender: GameplaySender,
    pub(super) player_id: i32,
    pub(super) shutting_down: Arc<AtomicBool>,
    pub(super) reader_finished: Arc<GameplayReaderFinished>,
}

impl GameplayHandle {
    pub(super) fn begin_graceful_exit(&self) -> ffone_net::Result<()> {
        // The shard normally writes REP_PC_EXIT_SUCC and then closes the
        // socket. Mark the expected EOF before sending, while leaving the
        // reader alive so it can still publish that final success frame.
        self.shutting_down.store(true, Ordering::Release);
        self.sender.send_pc_exit(self.player_id)
    }

    pub(super) fn stop(self) {
        self.shutting_down.store(true, Ordering::Release);
        let _ = self.sender.shutdown();
    }

    pub(super) fn wait_for_reader(&self, timeout: Duration) -> bool {
        self.reader_finished.wait(timeout)
    }
}

#[derive(Default)]
pub(super) struct GameplayReaderFinished {
    pub(super) finished: Mutex<bool>,
    pub(super) wake: Condvar,
}

impl GameplayReaderFinished {
    pub(super) fn finish(&self) {
        if let Ok(mut finished) = self.finished.lock() {
            *finished = true;
            self.wake.notify_all();
        }
    }

    pub(super) fn wait(&self, timeout: Duration) -> bool {
        let Ok(finished) = self.finished.lock() else {
            return false;
        };
        if *finished {
            return true;
        }
        self.wake
            .wait_timeout_while(finished, timeout, |finished| !*finished)
            .map(|(finished, _)| *finished)
            .unwrap_or(false)
    }
}

#[cfg(test)]
pub(super) struct LoginWorldHandle {
    pub(super) sender: LoginWorldSender,
    pub(super) shutting_down: Arc<AtomicBool>,
    pub(super) selection_events: Receiver<RetainedLoginSelection>,
    pub(super) characters: Vec<CharacterInfo0104>,
}

#[cfg(test)]
impl LoginWorldHandle {
    pub(super) fn stop(self) {
        self.shutting_down.store(true, Ordering::Release);
        let _ = self.sender.shutdown();
    }
}
