use super::*;

pub enum NetworkCommand {
    Login {
        login_address: String,
        username: String,
        password: String,
    },
    SelectCharacter {
        pc_uid: i64,
        location: CharacterEntryLocation0104,
    },
    CompleteTutorial {
        pc_uid: i64,
    },
    CheckCharacterName(CharacterNameCheckRequest0104),
    ReserveCharacterName {
        request: CharacterNameCheckRequest0104,
        slot: i8,
        gender: i8,
    },
    SaveCharacterName(CharacterNameSaveRequest0104),
    CreateCharacter(CharacterCreateRequest0104),
    DeleteCharacter {
        pc_uid: i64,
    },
    ChangeCharacterName(CharacterNameChangeRequest0104),
    ExitDuplicateSession {
        login_address: String,
        username: String,
        password: String,
    },
    RefreshCharacters,
    Move(PcMoveRequest0104),
    Stop(PcStopRequest0104),
    Jump(PcJumpRequest0104),
    CombatBegin(i32),
    CombatEnd(i32),
    AttackNpcs(ffone_protocol::PcAttackNpcsRequest0104),
    /// Clean PvP-mode hitscan (`bEnableAttackPC`) with mixed NPC/player targets.
    AttackChars(ffone_protocol::PcAttackCharsRequest0104),
    RocketStyleFire(PcRocketStyleFireRequest0104),
    GrenadeStyleFire(PcGrenadeStyleFireRequest0104),
    Regen(PcRegenRequest0104),
    ChangeMentor(PcChangeMentorRequest0104),
    UseNpcWarp(PcWarpUseNpcRequest0104),
    StopTask(PcTaskStopRequest0104),
    SwitchSpecialState(PcSpecialStateSwitchRequest0104),
    InteractWithNpc(NpcInteractionRequest0104),
    OpenBank(PcBankOpenRequest0104),
    CloseBank(PcBankCloseRequest0104),
    MoveItem(ItemMoveRequest0104),
    EquipNano(NanoEquipRequest0104),
    UnequipNano(NanoUnequipRequest0104),
    ActivateNano(NanoActiveRequest0104),
    UseNanoSkill(NanoSkillUseRequest0104),
    TuneNano(NanoTuneRequest0104),
    VehicleOff(PcVehicleOffRequest0104),
    VendorStart(VendorStartRequest0104),
    VendorTableUpdate(VendorTableUpdateRequest0104),
    VendorBuy(VendorItemBuyRequest0104),
    VendorSell(VendorItemSellRequest0104),
    VendorRestore(VendorItemRestoreBuyRequest0104),
    VendorBatteryBuy(VendorBatteryBuyRequest0104),
    DeleteInventoryItem(PcItemDeleteRequest0104),
    DisassembleInventoryItem(PcDisassembleItemRequest0104),
    RequestPresentNpcTypes(PresentNpcTypesRequest0104),
    RegisterQuickSlot(QuickSlotRegisterRequest0104),
    UseItem(ItemUseRequest0104),
    OpenChest(ItemChestOpenRequest0104),
    RequestBuddy(BuddyMakeRequest0104),
    RequestBuddyByName(BuddyFindNameRequest0104),
    AcceptBuddyByName(BuddyFindNameAcceptRequest0104),
    AcceptBuddy(BuddyAcceptRequest0104),
    RefreshBuddyState(BuddyStateRequest0104),
    BlockBuddy(BuddySetBlockRequest0104),
    RemoveBuddy(BuddyRemoveRequest0104),
    WarpToBuddy(BuddyWarpRequest0104),
    LeaveGroup(GroupLeaveRequest0104),
    SendFreeChat(FreeChatRequest0104),
    SetGmValue(GmSetValueRequest0104),
    SendBuddyFreeChat(BuddyFreeChatRequest0104),
    SendAllGroupFreeChat(AllGroupFreeChatRequest0104),
    /// Complete pinned-OpenFusion request escape hatch for feature-owned exact
    /// codecs. Construction proves that the shard registers the ID and that
    /// the body fits the 0104 packet buffer.
    SendRegisteredGameplay0104(RegisteredGameplayRequest0104),
    InviteEscortNpc(i32),
    KickEscortNpc(i32),
    EnvironmentDamage(bool),
    EnvironmentHeal(bool),
    ExitWorld,
    ReturnToCharacterSelection,
    Disconnect,
    Shutdown,
}

impl fmt::Debug for NetworkCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Login {
                login_address,
                username,
                password: _,
            } => f
                .debug_struct("Login")
                .field("login_address", login_address)
                .field("username", username)
                .field("password", &"<redacted>")
                .finish(),
            Self::SelectCharacter { pc_uid, location } => f
                .debug_struct("SelectCharacter")
                .field("pc_uid", pc_uid)
                .field("location", location)
                .finish(),
            Self::CompleteTutorial { pc_uid } => f
                .debug_struct("CompleteTutorial")
                .field("pc_uid", pc_uid)
                .finish(),
            Self::CheckCharacterName(request) => {
                f.debug_tuple("CheckCharacterName").field(request).finish()
            }
            Self::ReserveCharacterName {
                request,
                slot,
                gender,
            } => f
                .debug_struct("ReserveCharacterName")
                .field("request", request)
                .field("slot", slot)
                .field("gender", gender)
                .finish(),
            Self::SaveCharacterName(request) => {
                f.debug_tuple("SaveCharacterName").field(request).finish()
            }
            Self::CreateCharacter(request) => {
                f.debug_tuple("CreateCharacter").field(request).finish()
            }
            Self::DeleteCharacter { pc_uid } => f
                .debug_struct("DeleteCharacter")
                .field("pc_uid", pc_uid)
                .finish(),
            Self::ChangeCharacterName(request) => {
                f.debug_tuple("ChangeCharacterName").field(request).finish()
            }
            Self::ExitDuplicateSession {
                login_address,
                username,
                password: _,
            } => f
                .debug_struct("ExitDuplicateSession")
                .field("login_address", login_address)
                .field("username", username)
                .field("password", &"<redacted>")
                .finish(),
            Self::RefreshCharacters => f.write_str("RefreshCharacters"),
            Self::Move(request) => f.debug_tuple("Move").field(request).finish(),
            Self::Stop(request) => f.debug_tuple("Stop").field(request).finish(),
            Self::Jump(request) => f.debug_tuple("Jump").field(request).finish(),
            Self::CombatBegin(pc_id) => f.debug_tuple("CombatBegin").field(pc_id).finish(),
            Self::CombatEnd(pc_id) => f.debug_tuple("CombatEnd").field(pc_id).finish(),
            Self::AttackNpcs(request) => f.debug_tuple("AttackNpcs").field(request).finish(),
            Self::AttackChars(request) => f.debug_tuple("AttackChars").field(request).finish(),
            Self::RocketStyleFire(request) => {
                f.debug_tuple("RocketStyleFire").field(request).finish()
            }
            Self::GrenadeStyleFire(request) => {
                f.debug_tuple("GrenadeStyleFire").field(request).finish()
            }
            Self::Regen(request) => f.debug_tuple("Regen").field(request).finish(),
            Self::ChangeMentor(request) => f.debug_tuple("ChangeMentor").field(request).finish(),
            Self::UseNpcWarp(request) => f.debug_tuple("UseNpcWarp").field(request).finish(),
            Self::StopTask(request) => f.debug_tuple("StopTask").field(request).finish(),
            Self::SwitchSpecialState(request) => {
                f.debug_tuple("SwitchSpecialState").field(request).finish()
            }
            Self::InteractWithNpc(request) => {
                f.debug_tuple("InteractWithNpc").field(request).finish()
            }
            Self::OpenBank(request) => f.debug_tuple("OpenBank").field(request).finish(),
            Self::CloseBank(request) => f.debug_tuple("CloseBank").field(request).finish(),
            Self::MoveItem(request) => f.debug_tuple("MoveItem").field(request).finish(),
            Self::EquipNano(request) => f.debug_tuple("EquipNano").field(request).finish(),
            Self::UnequipNano(request) => f.debug_tuple("UnequipNano").field(request).finish(),
            Self::ActivateNano(request) => f.debug_tuple("ActivateNano").field(request).finish(),
            Self::UseNanoSkill(request) => f.debug_tuple("UseNanoSkill").field(request).finish(),
            Self::TuneNano(request) => f.debug_tuple("TuneNano").field(request).finish(),
            Self::VehicleOff(request) => f.debug_tuple("VehicleOff").field(request).finish(),
            Self::VendorStart(request) => f.debug_tuple("VendorStart").field(request).finish(),
            Self::VendorTableUpdate(request) => {
                f.debug_tuple("VendorTableUpdate").field(request).finish()
            }
            Self::VendorBuy(request) => f.debug_tuple("VendorBuy").field(request).finish(),
            Self::VendorSell(request) => f.debug_tuple("VendorSell").field(request).finish(),
            Self::VendorRestore(request) => f.debug_tuple("VendorRestore").field(request).finish(),
            Self::VendorBatteryBuy(request) => {
                f.debug_tuple("VendorBatteryBuy").field(request).finish()
            }
            Self::DeleteInventoryItem(request) => {
                f.debug_tuple("DeleteInventoryItem").field(request).finish()
            }
            Self::DisassembleInventoryItem(request) => f
                .debug_tuple("DisassembleInventoryItem")
                .field(request)
                .finish(),
            Self::RequestPresentNpcTypes(request) => f
                .debug_tuple("RequestPresentNpcTypes")
                .field(request)
                .finish(),
            Self::RegisterQuickSlot(request) => {
                f.debug_tuple("RegisterQuickSlot").field(request).finish()
            }
            Self::UseItem(request) => f.debug_tuple("UseItem").field(request).finish(),
            Self::OpenChest(request) => f.debug_tuple("OpenChest").field(request).finish(),
            Self::RequestBuddy(request) => f.debug_tuple("RequestBuddy").field(request).finish(),
            Self::RequestBuddyByName(request) => {
                f.debug_tuple("RequestBuddyByName").field(request).finish()
            }
            Self::AcceptBuddyByName(request) => {
                f.debug_tuple("AcceptBuddyByName").field(request).finish()
            }
            Self::AcceptBuddy(request) => f.debug_tuple("AcceptBuddy").field(request).finish(),
            Self::RefreshBuddyState(request) => {
                f.debug_tuple("RefreshBuddyState").field(request).finish()
            }
            Self::BlockBuddy(request) => f.debug_tuple("BlockBuddy").field(request).finish(),
            Self::RemoveBuddy(request) => f.debug_tuple("RemoveBuddy").field(request).finish(),
            Self::WarpToBuddy(request) => f.debug_tuple("WarpToBuddy").field(request).finish(),
            Self::LeaveGroup(request) => f.debug_tuple("LeaveGroup").field(request).finish(),
            Self::SendFreeChat(request) => f.debug_tuple("SendFreeChat").field(request).finish(),
            Self::SetGmValue(request) => f.debug_tuple("SetGmValue").field(request).finish(),
            Self::SendBuddyFreeChat(request) => {
                f.debug_tuple("SendBuddyFreeChat").field(request).finish()
            }
            Self::SendAllGroupFreeChat(request) => f
                .debug_tuple("SendAllGroupFreeChat")
                .field(request)
                .finish(),
            Self::SendRegisteredGameplay0104(request) => f
                .debug_struct("SendRegisteredGameplay0104")
                .field("packet_type", &request.packet_type())
                .field("packet_name", &request.registration().name)
                .field("payload_len", &request.payload().len())
                .finish(),
            Self::InviteEscortNpc(npc_id) => {
                f.debug_tuple("InviteEscortNpc").field(npc_id).finish()
            }
            Self::KickEscortNpc(npc_id) => f.debug_tuple("KickEscortNpc").field(npc_id).finish(),
            Self::EnvironmentDamage(enabled) => {
                f.debug_tuple("EnvironmentDamage").field(enabled).finish()
            }
            Self::EnvironmentHeal(enabled) => {
                f.debug_tuple("EnvironmentHeal").field(enabled).finish()
            }
            Self::ExitWorld => f.write_str("ExitWorld"),
            Self::ReturnToCharacterSelection => f.write_str("ReturnToCharacterSelection"),
            Self::Disconnect => f.write_str("Disconnect"),
            Self::Shutdown => f.write_str("Shutdown"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    Connecting,
    /// Lossless login entitlement consumed by NpcIconMode's Past Warp gate.
    LoginMetadata {
        payment_flag: i8,
    },
    Characters(Vec<CharacterSummary>),
    ReturnedToCharacterSelection(Vec<CharacterSummary>),
    CharacterNameChecked(CharacterNameCheckSuccess0104),
    CharacterNameSaved(CharacterNameSaveSuccess0104),
    CharacterCreated {
        slot: i8,
        response: CharacterCreateSuccess0104,
    },
    CharacterDeleted {
        pc_uid: i64,
        response: CharacterDeleteSuccess0104,
    },
    CharacterNameChanged(CharacterNameChangeSuccess0104),
    DuplicateSessionExitRequested,
    CharacterOperationRejected {
        stage: CharacterOperationStage,
        error_code: i32,
    },
    EnteringWorld {
        pc_uid: i64,
    },
    /// The exact tutorial-exit route could not reach `WorldReady`. This event is terminal for the
    /// client-local tutorial: its teardown has already begun and the login roster may be stale if
    /// `SAVE_CHAR_TUTOR` reached OpenFusion before a later selection/shard failure.
    TutorialExitFailed {
        pc_uid: i64,
        error: String,
    },
    WorldReady(WorldReady),
    /// Lossless non-heartbeat frame from the login server retained during gameplay.
    LoginFrame(DecodedFrame),
    Frame(DecodedFrame),
    /// A known fixed-layout packet whose body does not match the protocol-0104 ABI. The exact
    /// frame remains available for diagnostics, but is never exposed as a normal state-mutating
    /// login/gameplay frame.
    MalformedFrame0104 {
        stream: NetworkStream0104,
        frame: DecodedFrame,
        expected_payload_size: usize,
    },
    Error(String),
    Disconnected {
        reason: DisconnectReason0104,
    },
}

impl NetworkEvent {
    pub(super) fn incoming_frame_0104(stream: NetworkStream0104, frame: DecodedFrame) -> Self {
        if let Some(expected_payload_size) = fixed_payload_size(frame.packet_type)
            && frame.payload.len() != expected_payload_size
        {
            return Self::MalformedFrame0104 {
                stream,
                frame,
                expected_payload_size,
            };
        }

        match stream {
            NetworkStream0104::Login => Self::LoginFrame(frame),
            NetworkStream0104::Shard => Self::Frame(frame),
        }
    }

    /// Classify a retained gameplay frame without changing the lossless event
    /// surface. Fixed QuickSlot packets decode strictly; use
    /// [`Self::item_use_frame_0104`] for variable item-use responses.
    pub fn quick_slot_frame_0104(&self) -> Option<QuickSlotGameplayFrame0104> {
        match self {
            Self::Frame(frame) => Some(QuickSlotGameplayFrame0104::decode(frame.clone())),
            _ => None,
        }
    }

    /// Classify the complete item-use response/broadcast family while
    /// retaining the original lossless frame on both success and failure.
    pub fn item_use_frame_0104(&self) -> Option<ItemUseGameplayFrame0104> {
        match self {
            Self::Frame(frame) => Some(ItemUseGameplayFrame0104::decode(frame.clone())),
            _ => None,
        }
    }

    /// Classify authoritative fixed inventory/equipment packets while
    /// retaining the original lossless frame.
    pub fn inventory_frame_0104(&self) -> Option<InventoryGameplayFrame0104> {
        match self {
            Self::Frame(frame) => Some(InventoryGameplayFrame0104::decode(frame.clone())),
            _ => None,
        }
    }

    /// Strictly classify clean BankMode open/close replies while retaining the
    /// complete lossless frame for malformed and unrelated traffic.
    pub fn bank_frame_0104(&self) -> Option<BankGameplayFrame0104> {
        match self {
            Self::Frame(frame) => Some(BankGameplayFrame0104::decode(frame.clone())),
            _ => None,
        }
    }

    /// Classify the complete fixed VendorMode reply family while preserving
    /// the lossless transport event.
    pub fn vendor_frame_0104(&self) -> Option<VendorGameplayFrame0104> {
        match self {
            Self::Frame(frame) => Some(VendorGameplayFrame0104::decode(frame.clone())),
            _ => None,
        }
    }

    /// Strictly classify a clean Nano tune success/failure frame without
    /// changing the lossless event surface.
    pub fn nano_tune_frame_0104(&self) -> Option<NanoTuneGameplayFrame0104> {
        match self {
            Self::Frame(frame) => Some(NanoTuneGameplayFrame0104::decode(frame.clone())),
            _ => None,
        }
    }

    /// Decode and correlate a Nano tune reply against the caller's one pending
    /// request. Unrelated gameplay packets return `None`; known malformed or
    /// identity-mismatched packets fail closed with the complete raw frame.
    pub fn correlate_nano_tune_0104(
        &self,
        pending: NanoTunePending0104,
    ) -> Option<Result<CorrelatedNanoTuneReply0104, NanoTuneCorrelationError0104>> {
        let classified = self.nano_tune_frame_0104()?;
        match pending.correlate(classified) {
            Ok(Some(reply)) => Some(Ok(reply)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        }
    }

    /// Classify the Buddy lifecycle core while retaining the original frame.
    /// Variable buddy-list success packets decode only after strict
    /// count/range/length validation.
    pub fn buddy_lifecycle_frame_0104(&self) -> Option<BuddyLifecycleGameplayFrame0104> {
        match self {
            Self::Frame(frame) => Some(BuddyLifecycleGameplayFrame0104::decode(frame.clone())),
            _ => None,
        }
    }

    /// Classify the WorldMapMode present-NPC-types sync pair while retaining
    /// the exact original frame for decoded, malformed, and unrelated input.
    pub fn present_npc_types_frame_0104(&self) -> Option<PresentNpcTypesGameplayFrame0104> {
        match self {
            Self::Frame(frame) => Some(PresentNpcTypesGameplayFrame0104::decode(frame.clone())),
            _ => None,
        }
    }
}

#[cfg(test)]
pub(super) fn request_retained_shard_ticket(
    login_world: &mut LoginWorldHandle,
    pc_uid: i64,
    route: CharacterEntryRoute,
) -> Result<(CharacterStyle0104, ShardTicket), String> {
    let character = login_world
        .characters
        .iter_mut()
        .find(|character| character.pc_uid() == pc_uid)
        .ok_or_else(|| NetError::UnknownCharacter(pc_uid).to_string())?;

    if route == CharacterEntryRoute::TutorialCompletion {
        login_world
            .sender
            .send_tutorial_completion(&CharacterTutorialSaveRequest0104 {
                pc_uid,
                tutorial_flag: 1,
            })
            .map_err(|error| error.to_string())?;
        // The login server deliberately sends no SAVE_CHAR_TUTOR response. Mirror the accepted
        // fire-and-forget update locally so WorldReady enters the open-world scope immediately.
        character.as_bytes_mut()[81] = 1;
    }

    let login_style = character.style();
    login_world
        .sender
        .send_character_select(pc_uid)
        .map_err(|error| error.to_string())?;

    loop {
        match login_world
            .selection_events
            .recv_timeout(RETAINED_LOGIN_SELECTION_TIMEOUT)
        {
            Ok(RetainedLoginSelection::Frame(frame)) => {
                if frame.packet_type == packet::P_LS2CL_REP_CHAR_SELECT_SUCC {
                    // The clean login server acknowledges the character and
                    // waits for SHARD_SELECT; the shard ticket follows. OpenFusion
                    // never sends this frame.
                    login_world
                        .sender
                        .send_shard_select()
                        .map_err(|error| error.to_string())?;
                    continue;
                }
                return login_world
                    .sender
                    .shard_ticket_from_frame(pc_uid, &frame)
                    .map(|ticket| (login_style, ticket))
                    .map_err(|error| error.to_string());
            }
            Ok(RetainedLoginSelection::TransportFailed(error)) => return Err(error),
            Err(RecvTimeoutError::Timeout) => {
                return Err(format!(
                    "timed out waiting for retained login shard selection for character {pc_uid}"
                ));
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(
                    "retained login reader stopped before shard selection completed".to_owned(),
                );
            }
        }
    }
}

pub(super) fn scripted_entry_move_request(position: [i32; 3], angle: i32) -> PcMoveRequest0104 {
    PcMoveRequest0104 {
        client_time: 0,
        position,
        velocity: [0.0; 3],
        angle,
        key_value: 0,
        speed: 0,
    }
}

pub(super) fn registered_fixed_world_action_0104<P: WirePayload>(
    packet_type: u32,
    request: &P,
) -> Result<RegisteredGameplayRequest0104, String> {
    let expected = fixed_payload_size(packet_type).ok_or_else(|| {
        format!("packet {packet_type:#010x} has no typed fixed-size protocol-0104 ABI")
    })?;
    if expected != P::SIZE {
        return Err(format!(
            "packet {packet_type:#010x} expects {expected} bytes, but its request codec declares {}",
            P::SIZE
        ));
    }
    RegisteredGameplayRequest0104::new(packet_type, request.encode())
        .map_err(|error| error.to_string())
}

pub(super) fn registered_nano_skill_use_request_0104(
    request: &NanoSkillUseRequest0104,
) -> Result<RegisteredGameplayRequest0104, String> {
    let payload = request.encode().map_err(|error| error.to_string())?;
    RegisteredGameplayRequest0104::new(packet::P_CL2FE_REQ_NANO_SKILL_USE, payload)
        .map_err(|error| error.to_string())
}

pub(super) fn send_registered_world_action_0104(
    gameplay: Option<&GameplayHandle>,
    events: &SyncSender<NetworkEvent>,
    build_request: impl FnOnce() -> Result<RegisteredGameplayRequest0104, String>,
) {
    let Some(gameplay) = gameplay else {
        let _ = events.send(NetworkEvent::Error(
            "gameplay packet requested before entering the world".to_owned(),
        ));
        return;
    };
    let request = match build_request() {
        Ok(request) => request,
        Err(error) => {
            let _ = events.send(NetworkEvent::Error(error));
            return;
        }
    };
    if let Err(error) = gameplay.sender.send_registered_request(&request) {
        let _ = events.send(NetworkEvent::Error(error.to_string()));
    }
}
