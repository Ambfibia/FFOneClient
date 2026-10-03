use super::*;

pub type Result<T> = std::result::Result<T, NetError>;

pub(super) struct LoginWorldWriter {
    pub(super) io: TcpFrameIo,
    pub(super) outbound_e_key: u64,
    pub(super) login_id: FixedUtf16<33>,
    pub(super) fe_key: u64,
}

/// Cloneable outbound half of the login-server connection retained during gameplay.
#[derive(Clone)]
pub struct LoginWorldSender {
    pub(super) inner: Arc<Mutex<LoginWorldWriter>>,
}

impl fmt::Debug for LoginWorldSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginWorldSender").finish_non_exhaustive()
    }
}

impl LoginWorldSender {
    /// Fire-and-forget the exact protocol-0104 tutorial completion request.
    ///
    /// OpenFusion deliberately sends no success response. Any non-heartbeat login frame remains
    /// observable through [`LoginWorldReceiver::read_next`].
    pub fn send_tutorial_completion(
        &self,
        request: &CharacterTutorialSaveRequest0104,
    ) -> Result<()> {
        self.with_writer(|writer| {
            writer.io.send_payload(
                packet::P_CL2LS_REQ_SAVE_CHAR_TUTOR,
                request,
                writer.outbound_e_key,
            )
        })
    }

    /// Send `CHAR_SELECT` on the retained authenticated login socket.
    ///
    /// The matching shard-selection reply is read by [`LoginWorldReceiver`].
    /// Keeping the write on this sender preserves the outbound sequence after
    /// heartbeats and `SAVE_CHAR_TUTOR`.
    pub fn send_character_select(&self, pc_uid: i64) -> Result<()> {
        self.with_writer(|writer| {
            writer.io.send_payload(
                packet::P_CL2LS_REQ_CHAR_SELECT,
                &CharacterSelectRequest { pc_uid },
                writer.outbound_e_key,
            )
        })
    }

    /// Answer the clean login server's `CHAR_SELECT_SUCC` with `SHARD_SELECT`
    /// on the retained socket. OpenFusion never sends `CHAR_SELECT_SUCC`, so
    /// this write only happens on the original login-server route.
    pub fn send_shard_select(&self) -> Result<()> {
        self.with_writer(|writer| {
            writer.io.send_payload(
                packet::P_CL2LS_REQ_SHARD_SELECT,
                &clean_shard_select_request_0104(),
                writer.outbound_e_key,
            )
        })
    }

    /// Convert the retained reader's exact shard-selection reply into the
    /// ticket required by [`ShardSession::connect`].
    pub fn shard_ticket_from_frame(
        &self,
        pc_uid: i64,
        frame: &DecodedFrame,
    ) -> Result<ShardTicket> {
        self.with_writer(|writer| {
            shard_ticket_from_login_frame(&writer.login_id, writer.fe_key, pc_uid, frame)
        })
    }

    pub fn shutdown(&self) -> Result<()> {
        self.with_writer(|writer| writer.io.shutdown())
    }

    pub(super) fn send_heartbeat(&self, frame: &DecodedFrame) -> Result<()> {
        let payload = heartbeat_payload(frame)?;
        self.with_writer(|writer| {
            writer.io.send_bytes(
                packet::P_CL2LS_REP_LIVE_CHECK,
                payload,
                writer.outbound_e_key,
            )
        })
    }

    pub(super) fn with_writer<T>(
        &self,
        operation: impl FnOnce(&mut LoginWorldWriter) -> Result<T>,
    ) -> Result<T> {
        let mut writer = self
            .inner
            .lock()
            .map_err(|_| NetError::LoginSenderPoisoned)?;
        operation(&mut writer)
    }
}

/// Blocking inbound half of the retained login-server connection.
pub struct LoginWorldReceiver {
    pub(super) io: TcpFrameIo,
    pub(super) inbound_e_key: u64,
    pub(super) sender: LoginWorldSender,
}

impl fmt::Debug for LoginWorldReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginWorldReceiver").finish_non_exhaustive()
    }
}

impl LoginWorldReceiver {
    /// Read one non-heartbeat login frame while answering every heartbeat on the shared sender.
    pub fn read_next(&mut self) -> Result<DecodedFrame> {
        loop {
            let frame = self.io.read_server_frame(self.inbound_e_key)?;
            if frame.packet_type == packet::P_LS2CL_REQ_LIVE_CHECK {
                self.sender.send_heartbeat(&frame)?;
                continue;
            }
            return Ok(frame);
        }
    }

    pub fn shutdown(&self) -> Result<()> {
        self.io.shutdown()
    }
}

/// Split retained login connection used after a character has selected a shard.
pub struct LoginWorldConnection {
    pub(super) sender: LoginWorldSender,
    pub(super) receiver: LoginWorldReceiver,
}

impl LoginWorldConnection {
    pub fn sender(&self) -> &LoginWorldSender {
        &self.sender
    }

    pub fn into_parts(self) -> (LoginWorldSender, LoginWorldReceiver) {
        (self.sender, self.receiver)
    }
}

#[derive(Clone)]
pub struct ShardTicket {
    pub(super) endpoint: SocketAddr,
    pub(super) login_id: FixedUtf16<33>,
    pub(super) pc_uid: i64,
    pub(super) enter_serial_key: i64,
    pub(super) fe_key: u64,
}

impl ShardTicket {
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }

    pub fn pc_uid(&self) -> i64 {
        self.pc_uid
    }
}

impl fmt::Debug for ShardTicket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShardTicket")
            .field("endpoint", &self.endpoint)
            .field("pc_uid", &self.pc_uid)
            .finish_non_exhaustive()
    }
}

pub(super) struct GameplayWriter {
    pub(super) io: TcpFrameIo,
    pub(super) outbound_e_key: u64,
}

/// Cloneable outbound half of a loaded OpenFusion shard connection.
#[derive(Clone)]
pub struct GameplaySender {
    pub(super) inner: Arc<Mutex<GameplayWriter>>,
}

impl fmt::Debug for GameplaySender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GameplaySender").finish_non_exhaustive()
    }
}

impl GameplaySender {
    /// Send a gameplay packet whose strict ABI is owned by a higher-level
    /// feature codec. This retains the shard session's exact encryption key
    /// and framing while avoiding a runtime dependency from `ffone-net` back
    /// into the presentation crate.
    pub fn send_registered_request(&self, request: &RegisteredGameplayRequest0104) -> Result<()> {
        self.with_writer(|writer| {
            writer.io.send_bytes(
                request.packet_type(),
                request.payload(),
                writer.outbound_e_key,
            )
        })
    }

    /// Request the protocol-0104 graceful shard exit for the active runtime PC.
    pub fn send_pc_exit(&self, pc_id: i32) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_EXIT, &PcExitRequest0104 { pc_id })
    }

    /// Send the exact protocol-0104 resurrection choice selected by the client.
    pub fn send_pc_regen(&self, request: &PcRegenRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_REGEN, request)
    }

    /// Request the exact protocol-0104 mentor change selected by Guide Mode.
    pub fn send_change_mentor(&self, request: &PcChangeMentorRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_CHANGE_MENTOR, request)
    }

    /// Use a runtime NPC entity as the source of a protocol-0104 warp.
    pub fn send_npc_warp(&self, request: &PcWarpUseNpcRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_WARP_USE_NPC, request)
    }

    /// Stop one active protocol-0104 task.
    pub fn send_task_stop(&self, request: &PcTaskStopRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_TASK_STOP, request)
    }

    /// Toggle one protocol-0104 PC special-state bit.
    pub fn send_special_state_switch(
        &self,
        request: &PcSpecialStateSwitchRequest0104,
    ) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH, request)
    }

    /// Mark a runtime NPC interaction as opened or closed.
    pub fn send_npc_interaction(&self, request: &NpcInteractionRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_NPC_INTERACTION, request)
    }

    /// Open clean protocol-0104 BankMode for the selected runtime NPC.
    pub fn send_bank_open(&self, request: &PcBankOpenRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_BANK_OPEN, request)
    }

    /// Send the dormant protocol-0104 bank-close request ABI. Clean
    /// Retrobution's normal `cnBank.BankOut` path does not call this method;
    /// it is retained for exact transport coverage of the published packet.
    pub fn send_bank_close(&self, request: &PcBankCloseRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_BANK_CLOSE, request)
    }

    /// Move one item between clean item-location slots. BankMode uses only
    /// locations 1 (general inventory) and 3 (bank).
    pub fn send_item_move(&self, request: &ItemMoveRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_ITEM_MOVE, request)
    }

    /// Send the exact clean-Retrobution Nano tuning request. The server reply
    /// is authoritative for the selected skill, Fusion Matter, and all ten
    /// inventory mutation records.
    pub fn send_nano_tune(&self, request: &NanoTuneRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_NANO_TUNE, request)
    }

    pub fn send_vendor_start(&self, request: &VendorStartRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_VENDOR_START, request)
    }

    pub fn send_vendor_table_update(&self, request: &VendorTableUpdateRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE, request)
    }

    pub fn send_vendor_item_buy(&self, request: &VendorItemBuyRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_BUY, request)
    }

    pub fn send_vendor_item_sell(&self, request: &VendorItemSellRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_SELL, request)
    }

    pub fn send_vendor_item_restore(
        &self,
        request: &VendorItemRestoreBuyRequest0104,
    ) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY, request)
    }

    pub fn send_vendor_battery_buy(&self, request: &VendorBatteryBuyRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY, request)
    }

    pub fn send_item_delete(&self, request: &PcItemDeleteRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_ITEM_DELETE, request)
    }

    pub fn send_disassemble_item(&self, request: &PcDisassembleItemRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_DISASSEMBLE_ITEM, request)
    }

    /// Synchronize the NPC type presence set used by Retrobution WorldMapMode.
    pub fn send_present_npc_types(&self, request: &PresentNpcTypesRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PRESENT_NPC_TYPES, request)
    }

    pub fn send_move(&self, request: &PcMoveRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_MOVE, request)
    }

    pub fn send_stop(&self, request: &PcStopRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_STOP, request)
    }

    pub fn send_jump(&self, request: &PcJumpRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_JUMP, request)
    }

    pub fn send_register_quick_slot(&self, request: &QuickSlotRegisterRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_REGIST_QUICK_SLOT, request)
    }

    pub fn send_item_use(&self, request: &ItemUseRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_ITEM_USE, request)
    }

    pub fn send_item_chest_open(&self, request: &ItemChestOpenRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_ITEM_CHEST_OPEN, request)
    }

    pub fn send_freechat(&self, request: &FreeChatRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_FREECHAT, request)
    }

    /// Send the clean client-side `/speed` GM value request. OpenFusion
    /// authorizes this packet independently using the account level.
    pub fn send_gm_set_value(&self, request: &GmSetValueRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_GM_REQ_PC_SET_VALUE, request)
    }

    pub fn send_all_group_freechat(&self, request: &AllGroupFreeChatRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE, request)
    }

    pub fn send_buddy_freechat(&self, request: &BuddyFreeChatRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE, request)
    }

    pub fn send_buddy_request(&self, request: &BuddyMakeRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_REQUEST_MAKE_BUDDY, request)
    }

    pub fn send_buddy_find_name(&self, request: &BuddyFindNameRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_FIND_NAME_MAKE_BUDDY, request)
    }

    pub fn send_buddy_find_name_accept(
        &self,
        request: &BuddyFindNameAcceptRequest0104,
    ) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_FIND_NAME_ACCEPT_BUDDY, request)
    }

    pub fn send_buddy_accept(&self, request: &BuddyAcceptRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_ACCEPT_MAKE_BUDDY, request)
    }

    pub fn send_buddy_state_refresh(&self, request: &BuddyStateRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_GET_BUDDY_STATE, request)
    }

    pub fn send_buddy_block(&self, request: &BuddySetBlockRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_SET_BUDDY_BLOCK, request)
    }

    pub fn send_buddy_remove(&self, request: &BuddyRemoveRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_REMOVE_BUDDY, request)
    }

    pub fn send_buddy_warp(&self, request: &BuddyWarpRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_BUDDY_WARP, request)
    }

    pub fn send_group_leave(&self, request: &GroupLeaveRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_GROUP_LEAVE, request)
    }

    /// Clean escort-group requests. These are distinct from the inventory of
    /// handlers registered by the stock OpenFusion shard.
    pub fn send_escort_invite(&self, npc_id: i32) -> Result<()> {
        self.send_payload(
            0x1300_0084,
            &ffone_protocol::wire_0104::NpcGroupInviteRequest0104 { npc_id },
        )
    }

    pub fn send_escort_kick(&self, npc_id: i32) -> Result<()> {
        self.send_payload(
            0x1300_0085,
            &ffone_protocol::wire_0104::NpcGroupKickRequest0104 { npc_id },
        )
    }

    /// Send the confirmed protocol-0104 hitscan request (at most three NPC IDs).
    pub fn send_attack_npcs(&self, request: &PcAttackNpcsRequest0104) -> Result<()> {
        let payload = request.encode().map_err(NpcCombatDecodeError0104::from)?;
        self.with_writer(|writer| {
            writer.io.send_bytes(
                packet::P_CL2FE_REQ_PC_ATTACK_NPCS,
                &payload,
                writer.outbound_e_key,
            )
        })
    }

    /// Send the clean PvP-mode hitscan request (`bEnableAttackPC`): mixed
    /// NPC/player targets as `(iID, eCT)` pairs.
    pub fn send_attack_chars(&self, request: &PcAttackCharsRequest0104) -> Result<()> {
        let payload = request.encode().map_err(NpcCombatDecodeError0104::from)?;
        self.with_writer(|writer| {
            writer.io.send_bytes(
                packet::P_CL2FE_REQ_PC_ATTACK_CHARS,
                &payload,
                writer.outbound_e_key,
            )
        })
    }

    pub fn send_rocket_style_fire(&self, request: &PcRocketStyleFireRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_ROCKET_STYLE_FIRE, request)
    }

    pub fn send_grenade_style_fire(&self, request: &PcGrenadeStyleFireRequest0104) -> Result<()> {
        self.send_payload(packet::P_CL2FE_REQ_PC_GRENADE_STYLE_FIRE, request)
    }

    pub fn send_combat_begin(&self, pc_id: i32) -> Result<()> {
        self.send_payload(
            packet::P_CL2FE_REQ_PC_COMBAT_BEGIN,
            &PcCombatStateRequest0104 { pc_id },
        )
    }

    pub fn send_combat_end(&self, pc_id: i32) -> Result<()> {
        self.send_payload(
            packet::P_CL2FE_REQ_PC_COMBAT_END,
            &PcCombatStateRequest0104 { pc_id },
        )
    }

    pub fn send_dot_damage(&self, enabled: bool) -> Result<()> {
        self.send_payload(
            packet::P_CL2FE_DOT_DAMAGE_ONOFF,
            &EnvironmentDotToggle0104 { enabled },
        )
    }

    pub fn send_dot_heal(&self, enabled: bool) -> Result<()> {
        self.send_payload(
            packet::P_CL2FE_DOT_HEAL_ONOFF,
            &EnvironmentDotToggle0104 { enabled },
        )
    }

    pub fn shutdown(&self) -> Result<()> {
        self.with_writer(|writer| writer.io.shutdown())
    }

    pub(super) fn send_heartbeat(&self, frame: &DecodedFrame) -> Result<()> {
        let payload = heartbeat_payload(frame)?;
        self.with_writer(|writer| {
            writer.io.send_bytes(
                packet::P_CL2FE_REP_LIVE_CHECK,
                payload,
                writer.outbound_e_key,
            )
        })
    }

    pub(super) fn send_payload<P: WirePayload>(&self, packet_type: u32, payload: &P) -> Result<()> {
        self.with_writer(|writer| {
            writer
                .io
                .send_payload(packet_type, payload, writer.outbound_e_key)
        })
    }

    pub(super) fn with_writer<T>(
        &self,
        operation: impl FnOnce(&mut GameplayWriter) -> Result<T>,
    ) -> Result<T> {
        let mut writer = self
            .inner
            .lock()
            .map_err(|_| NetError::GameplaySenderPoisoned)?;
        operation(&mut writer)
    }
}

/// Lossless engine-facing classification of the confirmed NPC lifecycle/basic-combat family.
///
/// The original frame is retained even when decoding succeeds. Unknown packets and malformed
/// known packets are deliberately not discarded, so later protocol slices and diagnostics can
/// consume the exact wire payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NpcCombatGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: NpcCombatPacket0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: NpcCombatDecodeError0104,
    },
    Passthrough(DecodedFrame),
}

impl NpcCombatGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_npc_combat_packet_0104(frame.packet_type, &frame.payload) {
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

/// Lossless engine-facing classification of fixed QuickSlot/item-use packets.
///
/// Variable item-use success and broadcast packets remain passthrough in this
/// fixed-layout view; [`ItemUseGameplayFrame0104`] provides their separate
/// `eST`-dependent typed closure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuickSlotGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: QuickSlotPacket0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Passthrough(DecodedFrame),
}

impl QuickSlotGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_quick_slot_packet_0104(frame.packet_type, &frame.payload) {
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

/// Lossless engine-facing classification of the complete protocol-0104
/// item-use response/broadcast family.
///
/// The fixed failure packet and every proven `eST`-dependent success or
/// broadcast tail decode to typed values. Unsupported positive-target skill
/// types and malformed lengths retain their complete original frame in
/// `Malformed`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemUseGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: ItemUsePacket0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: ItemUseDecodeError0104,
    },
    Passthrough(DecodedFrame),
}

impl ItemUseGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_item_use_packet_0104(frame.packet_type, &frame.payload) {
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

/// Lossless engine-facing classification of the fixed BankMode open/close
/// reply family. Exact known IDs reject every non-exact body while unrelated
/// frames remain available to the other gameplay classifiers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BankGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: PcBankReply0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Passthrough(DecodedFrame),
}

impl BankGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_pc_bank_reply_0104(frame.packet_type, &frame.payload) {
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

/// Lossless engine-facing classification of the fixed Nano tune reply pair.
/// Successful, malformed, and unrelated frames all retain their original
/// packet metadata and bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NanoTuneGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: NanoTunePacket0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Passthrough(DecodedFrame),
}

impl NanoTuneGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_nano_tune_packet_0104(frame.packet_type, &frame.payload) {
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

/// Lossless engine-facing classification of the fixed VendorMode reply
/// family. The exact frame survives both successful and malformed decoding.
#[derive(Debug, Clone, PartialEq)]
pub enum VendorGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: VendorPacket0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Passthrough(DecodedFrame),
}

impl VendorGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_vendor_packet_0104(frame.packet_type, &frame.payload) {
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

/// Lossless engine-facing classification of the fixed Buddy lifecycle core.
///
/// Variable buddy-list success frames are decoded only after strict count,
/// range, and exact-length validation. Buddy FreeChat is intentionally handled
/// by its existing, separate ABI family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuddyLifecycleGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: BuddyLifecyclePacket0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: PayloadError,
    },
    Passthrough(DecodedFrame),
}

impl BuddyLifecycleGameplayFrame0104 {
    pub fn decode(frame: DecodedFrame) -> Self {
        match decode_buddy_lifecycle_packet_0104(frame.packet_type, &frame.payload) {
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

/// Lossless classification of the WorldMapMode present-NPC-types sync pair.
///
/// Known frames decode only after exact fixed/count/length validation. Both
/// malformed known frames and unrelated frames retain the complete original
/// transport record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresentNpcTypesGameplayFrame0104 {
    Decoded {
        frame: DecodedFrame,
        packet: PresentNpcTypesPacket0104,
    },
    Malformed {
        frame: DecodedFrame,
        error: PresentNpcTypesDecodeError0104,
    },
    Passthrough(DecodedFrame),
}
