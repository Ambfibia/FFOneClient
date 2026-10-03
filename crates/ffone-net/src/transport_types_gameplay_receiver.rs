use super::*;

impl PresentNpcTypesGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_present_npc_types_packet_0104(frame.packet_type, &frame.payload) {
            Ok(Some(packet)) => Self::Decoded { frame, packet },
            Ok(None) => Self::Passthrough(frame),
            Err(error) => Self::Malformed { frame, error },
        }
    }

    pub fn raw_frame(&self) -> &DecodedFrame {
        match self {
            Self::Decoded { frame, .. } | Self::Malformed { frame, .. } => frame,
            Self::Passthrough(frame) => frame,
        }
    }
}

/// Blocking inbound half of a loaded OpenFusion shard connection.
pub struct GameplayReceiver {
    pub(super) io: TcpFrameIo,
    pub(super) inbound_fe_key: u64,
    pub(super) sender: GameplaySender,
}

impl fmt::Debug for GameplayReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameplayReceiver").finish_non_exhaustive()
    }
}

impl GameplayReceiver {
    /// Read one gameplay packet. Live checks are answered on the shared outbound encoder and are
    /// not surfaced to the engine.
    pub fn read_next(&mut self) -> Result<DecodedFrame> {
        loop {
            let frame = self.io.read_server_frame(self.inbound_fe_key)?;
            if frame.packet_type == packet::P_FE2CL_REQ_LIVE_CHECK {
                self.sender.send_heartbeat(&frame)?;
                continue;
            }
            return Ok(frame);
        }
    }

    pub fn read_next_npc_combat(&mut self) -> Result<NpcCombatGameplayFrame0104> {
        Ok(NpcCombatGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn read_next_quick_slot(&mut self) -> Result<QuickSlotGameplayFrame0104> {
        Ok(QuickSlotGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn read_next_item_use(&mut self) -> Result<ItemUseGameplayFrame0104> {
        Ok(ItemUseGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn read_next_inventory(&mut self) -> Result<InventoryGameplayFrame0104> {
        Ok(InventoryGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn read_next_bank(&mut self) -> Result<BankGameplayFrame0104> {
        Ok(BankGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn read_next_nano_tune(&mut self) -> Result<NanoTuneGameplayFrame0104> {
        Ok(NanoTuneGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn read_next_vendor(&mut self) -> Result<VendorGameplayFrame0104> {
        Ok(VendorGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn read_next_buddy_lifecycle(&mut self) -> Result<BuddyLifecycleGameplayFrame0104> {
        Ok(BuddyLifecycleGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn read_next_present_npc_types(&mut self) -> Result<PresentNpcTypesGameplayFrame0104> {
        Ok(PresentNpcTypesGameplayFrame0104::decode(self.read_next()?))
    }

    pub fn shutdown(&self) -> Result<()> {
        self.io.shutdown()
    }
}

/// Engine-facing split gameplay connection after loading complete.
#[derive(Debug)]
pub struct GameplayConnection {
    pub(super) player_id: i32,
    pub(super) load: PcLoadData0104,
    pub(super) sender: GameplaySender,
    pub(super) receiver: GameplayReceiver,
}

impl GameplayConnection {
    pub fn player_id(&self) -> i32 {
        self.player_id
    }

    pub fn load_data(&self) -> &PcLoadData0104 {
        &self.load
    }

    pub fn sender(&self) -> &GameplaySender {
        &self.sender
    }

    pub fn into_parts(self) -> (GameplaySender, GameplayReceiver) {
        (self.sender, self.receiver)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadingComplete {
    pub response: PcLoadingCompleteSuccess,
    /// Frames sent by chunk registration before loading-complete success.
    pub prelude: Vec<DecodedFrame>,
}

impl LoadingComplete {
    /// Decode the initial entity buckets without performing any I/O.
    ///
    /// Packet order and every raw frame are retained. Recognized but malformed AROUND packets are
    /// represented explicitly so one bad bucket cannot hide later packets from diagnostics or
    /// recovery code.
    pub fn into_world_bootstrap(self) -> WorldBootstrap {
        WorldBootstrap::from_loading_complete(self)
    }
}

/// Engine-neutral, loss-aware view of the packets received before loading completed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldBootstrap {
    pub response: PcLoadingCompleteSuccess,
    /// Same order as [`LoadingComplete::prelude`].
    pub packets: Vec<WorldBootstrapPacket>,
}

impl WorldBootstrap {
    /// Pure protocol conversion; all blocking socket I/O remains in [`ShardSession`].
    pub fn from_loading_complete(loading: LoadingComplete) -> Self {
        Self {
            response: loading.response,
            packets: loading
                .prelude
                .into_iter()
                .map(WorldBootstrapPacket::decode)
                .collect(),
        }
    }

    pub fn has_decode_errors(&self) -> bool {
        self.packets
            .iter()
            .any(WorldBootstrapPacket::is_malformed_initial_entities)
    }

    /// Return malformed bucket diagnostics together with their original ordered packet indices.
    pub fn decode_errors(
        &self,
    ) -> impl Iterator<Item = (usize, &DecodedFrame, &AroundDecodeError)> {
        self.packets
            .iter()
            .enumerate()
            .filter_map(|(packet_index, packet)| match packet {
                WorldBootstrapPacket::MalformedInitialEntities { frame, error } => {
                    Some((packet_index, frame, error))
                }
                _ => None,
            })
    }
}

impl From<LoadingComplete> for WorldBootstrap {
    fn from(loading: LoadingComplete) -> Self {
        Self::from_loading_complete(loading)
    }
}
