use super::*;

impl WirePayload for BuddyAcceptRequest0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0] = self.accept_flag as u8;
        write_i32(&mut out, 4, self.buddy_id);
        write_i64(&mut out, 8, self.buddy_pc_uid);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            accept_flag: bytes[0] as i8,
            buddy_id: read_i32(bytes, 4),
            buddy_pc_uid: read_i64(bytes, 8),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuddyMakeSuccess0104 {
    pub request_id: i32,
    pub buddy_id: i32,
    pub buddy_pc_uid: i64,
}

impl WirePayload for BuddyMakeSuccess0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.request_id);
        write_i32(&mut out, 4, self.buddy_id);
        write_i64(&mut out, 8, self.buddy_pc_uid);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            request_id: read_i32(bytes, 0),
            buddy_id: read_i32(bytes, 4),
            buddy_pc_uid: read_i64(bytes, 8),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_ACCEPT_MAKE_BUDDY_SUCC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyAcceptSuccess0104 {
    pub buddy_slot: i8,
    pub buddy: BuddyBaseInfo0104,
}

impl WirePayload for BuddyAcceptSuccess0104 {
    const SIZE: usize = 76;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0] = self.buddy_slot as u8;
        out[4..76].copy_from_slice(&self.buddy.encode());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            buddy_slot: bytes[0] as i8,
            buddy: BuddyBaseInfo0104::decode(&bytes[4..76])?,
        })
    }
}

/// Protocol-0104
/// `sP_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC_TO_ACCEPTER`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyIncomingRequest0104 {
    pub request_id: i32,
    pub buddy_id: i32,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl WirePayload for BuddyIncomingRequest0104 {
    const SIZE: usize = 60;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.request_id);
        write_i32(&mut out, 4, self.buddy_id);
        write_utf16(&mut out, 8, &self.first_name);
        write_utf16(&mut out, 26, &self.last_name);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            request_id: read_i32(bytes, 0),
            buddy_id: read_i32(bytes, 4),
            first_name: read_utf16(bytes, 8),
            last_name: read_utf16(bytes, 26),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_BUDDY_WARP_OTHER_SHARD_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuddyWarpOtherShardSuccess0104 {
    pub buddy_pc_uid: i64,
    pub shard_num: i8,
    pub channel_num: i32,
}

impl WirePayload for BuddyWarpOtherShardSuccess0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i64(&mut out, 0, self.buddy_pc_uid);
        out[8] = self.shard_num as u8;
        write_i32(&mut out, 12, self.channel_num);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            buddy_pc_uid: read_i64(bytes, 0),
            shard_num: bytes[8] as i8,
            channel_num: read_i32(bytes, 12),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_PC_BUDDY_WARP_SAME_SHARD_SUCC`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuddyWarpSameShardSuccess0104 {
    pub unused: u8,
}

impl WirePayload for BuddyWarpSameShardSuccess0104 {
    const SIZE: usize = 1;

    fn encode(&self) -> Vec<u8> {
        vec![self.unused]
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self { unused: bytes[0] })
    }
}

/// Proven Buddy lifecycle packets, including the bounded variable buddy-list
/// response. Buddy FreeChat remains in its existing, separate ABI family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuddyLifecyclePacket0104 {
    MakeRequest(BuddyMakeRequest0104),
    FindNameRequest(BuddyFindNameRequest0104),
    FindNameAcceptRequest(BuddyFindNameAcceptRequest0104),
    AcceptRequest(BuddyAcceptRequest0104),
    StateRequest(BuddyStateRequest0104),
    SetBlockRequest(BuddySetBlockRequest0104),
    RemoveRequest(BuddyRemoveRequest0104),
    WarpRequest(BuddyWarpRequest0104),
    ListInfo(BuddyListInfo0104),
    ListFailure(BuddyListFailure0104),
    MakeSuccess(BuddyMakeSuccess0104),
    MakeFailure(BuddyMakeFailure0104),
    FindNameSuccess(BuddyFindNameSuccess0104),
    FindNameFailure(BuddyFindNameFailure0104),
    FindNameAcceptFailure(BuddyFindNameAcceptFailure0104),
    AcceptSuccess(BuddyAcceptSuccess0104),
    AcceptFailure(BuddyAcceptFailure0104),
    StateSuccess(BuddyStateSuccess0104),
    StateFailure(BuddyStateFailure0104),
    BlockSuccess(BuddyBlockSuccess0104),
    BlockFailure(BuddyBlockFailure0104),
    RemoveSuccess(BuddyRemoveSuccess0104),
    RemoveFailure(BuddyRemoveFailure0104),
    IncomingRequest(BuddyIncomingRequest0104),
    WarpFailure(BuddyWarpFailure0104),
    WarpOtherShardSuccess(BuddyWarpOtherShardSuccess0104),
    WarpSameShardSuccess(BuddyWarpSameShardSuccess0104),
}

/// Protocol-0104 `sP_CL2FE_REQ_SEND_BUDDY_FREECHAT_MESSAGE`.
///
/// The target is the buddy's persistent PC UID, while `buddy_slot` preserves
/// the exact signed slot index carried by the legacy client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyFreeChatRequest0104 {
    pub message: FixedUtf16<128>,
    pub emote_code: i32,
    pub buddy_pc_uid: i64,
    pub buddy_slot: i8,
}

impl WirePayload for BuddyFreeChatRequest0104 {
    const SIZE: usize = 272;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.message);
        write_i32(&mut out, 256, self.emote_code);
        write_i64(&mut out, 260, self.buddy_pc_uid);
        out[268] = self.buddy_slot as u8;
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            message: read_utf16(bytes, 0),
            emote_code: read_i32(bytes, 256),
            buddy_pc_uid: read_i64(bytes, 260),
            buddy_slot: bytes[268] as i8,
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_SEND_BUDDY_FREECHAT_MESSAGE_SUCC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyFreeChatSuccess0104 {
    pub from_pc_uid: i64,
    pub to_pc_uid: i64,
    pub message: FixedUtf16<128>,
    pub emote_code: i32,
}

impl WirePayload for BuddyFreeChatSuccess0104 {
    const SIZE: usize = 276;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i64(&mut out, 0, self.from_pc_uid);
        write_i64(&mut out, 8, self.to_pc_uid);
        write_utf16(&mut out, 16, &self.message);
        write_i32(&mut out, 272, self.emote_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            from_pc_uid: read_i64(bytes, 0),
            to_pc_uid: read_i64(bytes, 8),
            message: read_utf16(bytes, 16),
            emote_code: read_i32(bytes, 272),
        })
    }
}

/// The confirmed buddy FreeChat request/success family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuddyFreeChatPacket0104 {
    Request(BuddyFreeChatRequest0104),
    Success(BuddyFreeChatSuccess0104),
}

/// Buddy MenuChat preserves the FreeChat ABI while using its own request and
/// reply packet IDs.
pub type BuddyMenuChatRequest0104 = BuddyFreeChatRequest0104;

pub type BuddyMenuChatSuccess0104 = BuddyFreeChatSuccess0104;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuddyMenuChatPacket0104 {
    Request(BuddyMenuChatRequest0104),
    Success(BuddyMenuChatSuccess0104),
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_GROUP_LEAVE`.
///
/// The clean managed definition has one unused byte rather than a zero-sized
/// payload.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GroupLeaveRequest0104 {
    pub unused: u8,
}

impl WirePayload for GroupLeaveRequest0104 {
    const SIZE: usize = 1;

    fn encode(&self) -> Vec<u8> {
        vec![self.unused]
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self { unused: bytes[0] })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_SEND_ALL_GROUP_FREECHAT_MESSAGE`.
///
/// ALL GROUP has the same 260-byte fields as normal FreeChat and deliberately
/// carries no target player/group ID. The active group is server-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllGroupFreeChatRequest0104 {
    pub message: FixedUtf16<128>,
    pub emote_code: i32,
}

impl WirePayload for AllGroupFreeChatRequest0104 {
    const SIZE: usize = 260;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.message);
        write_i32(&mut out, 256, self.emote_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            message: read_utf16(bytes, 0),
            emote_code: read_i32(bytes, 256),
        })
    }
}

/// Protocol-0104 `sP_FE2CL_REP_SEND_ALL_GROUP_FREECHAT_MESSAGE_SUCC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllGroupFreeChatSuccess0104 {
    pub sender_pc_id: i32,
    pub message: FixedUtf16<128>,
    pub emote_code: i32,
}

impl WirePayload for AllGroupFreeChatSuccess0104 {
    const SIZE: usize = 264;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.sender_pc_id);
        write_utf16(&mut out, 4, &self.message);
        write_i32(&mut out, 260, self.emote_code);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            sender_pc_id: read_i32(bytes, 0),
            message: read_utf16(bytes, 4),
            emote_code: read_i32(bytes, 260),
        })
    }
}

/// The confirmed ALL GROUP FreeChat request/success family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllGroupFreeChatPacket0104 {
    Request(AllGroupFreeChatRequest0104),
    Success(AllGroupFreeChatSuccess0104),
}

/// ALL GROUP MenuChat preserves the FreeChat ABI while using its own request
/// and reply packet IDs.
pub type AllGroupMenuChatRequest0104 = AllGroupFreeChatRequest0104;

pub type AllGroupMenuChatSuccess0104 = AllGroupFreeChatSuccess0104;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllGroupMenuChatPacket0104 {
    Request(AllGroupMenuChatRequest0104),
    Success(AllGroupMenuChatSuccess0104),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginSuccess {
    pub character_count: i8,
    pub selected_slot: i8,
    pub payment_flag: i8,
    pub packing_byte: i8,
    pub server_time: u64,
    pub id: FixedUtf16<33>,
    pub open_beta_flag: i32,
}

impl WirePayload for LoginSuccess {
    const SIZE: usize = 84;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0u8; Self::SIZE];
        out[0] = self.character_count as u8;
        out[1] = self.selected_slot as u8;
        out[2] = self.payment_flag as u8;
        out[3] = self.packing_byte as u8;
        write_u64(&mut out, 4, self.server_time);
        write_utf16(&mut out, 12, &self.id);
        write_i32(&mut out, 80, self.open_beta_flag);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            character_count: bytes[0] as i8,
            selected_slot: bytes[1] as i8,
            payment_flag: bytes[2] as i8,
            packing_byte: bytes[3] as i8,
            server_time: read_u64(bytes, 4),
            id: read_utf16(bytes, 12),
            open_beta_flag: read_i32(bytes, 80),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginFailure {
    pub error_code: i32,
    pub id: FixedUtf16<33>,
}

impl WirePayload for LoginFailure {
    const SIZE: usize = 72;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0u8; Self::SIZE];
        write_i32(&mut out, 0, self.error_code);
        write_utf16(&mut out, 4, &self.id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            error_code: read_i32(bytes, 0),
            id: read_utf16(bytes, 4),
        })
    }
}

/// Lossless 204-byte `sP_LS2CL_REP_CHAR_INFO`.
///
/// The accessor offsets include its nested pack(4) `sPCStyle`; keeping the full blob preserves
/// equipment and appearance fields until they receive richer domain types.
#[derive(Clone, PartialEq, Eq)]
pub struct CharacterInfo0104(pub(super) [u8; 204]);

/// Exact `sPCStyle` fields needed to assemble the selected native player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterStyle0104 {
    pub name_check: i8,
    pub gender: i8,
    pub face_style: i8,
    pub hair_style: i8,
    pub hair_color: i8,
    pub skin_color: i8,
    pub eye_color: i8,
    pub height: i8,
    pub body: i8,
    pub class: i32,
    pub appearance_flag: i8,
    pub tutorial_flag: i8,
    pub payzone_flag: i8,
}

/// One exact pack(4) `sItemBase` from the login character equipment array.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EquippedItem0104 {
    pub item_type: i16,
    pub item_id: i16,
    pub option: i32,
    pub time_limit: i32,
}

/// Stable OpenFusion/0104 equipment array index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum CharacterEquipSlot0104 {
    Hand = 0,
    UpperBody = 1,
    LowerBody = 2,
    Foot = 3,
    Head = 4,
    Face = 5,
    Back = 6,
    ExtendedHand = 7,
    Vehicle = 8,
}

impl CharacterEquipSlot0104 {
    pub const ALL: [Self; CHARACTER_EQUIP_SLOT_COUNT_0104] = [
        Self::Hand,
        Self::UpperBody,
        Self::LowerBody,
        Self::Foot,
        Self::Head,
        Self::Face,
        Self::Back,
        Self::ExtendedHand,
        Self::Vehicle,
    ];
}

impl CharacterInfo0104 {
    pub const SIZE: usize = 204;
    pub(super) const EQUIPMENT_OFFSET: usize = 96;
    pub(super) const EQUIPPED_ITEM_SIZE: usize = 12;

    pub const fn zeroed() -> Self {
        Self([0; Self::SIZE])
    }

    pub const fn as_bytes(&self) -> &[u8; Self::SIZE] {
        &self.0
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8; Self::SIZE] {
        &mut self.0
    }

    pub fn slot(&self) -> i8 {
        self.0[0] as i8
    }

    pub fn level(&self) -> i16 {
        read_i16(&self.0, 2)
    }

    pub fn pc_uid(&self) -> i64 {
        read_i64(&self.0, 4)
    }

    pub fn first_name(&self) -> FixedUtf16<9> {
        read_utf16(&self.0, 14)
    }

    pub fn last_name(&self) -> FixedUtf16<17> {
        read_utf16(&self.0, 32)
    }

    pub fn position(&self) -> [i32; 3] {
        [
            read_i32(&self.0, 84),
            read_i32(&self.0, 88),
            read_i32(&self.0, 92),
        ]
    }

    pub fn style(&self) -> CharacterStyle0104 {
        CharacterStyle0104 {
            name_check: self.0[12] as i8,
            gender: self.0[66] as i8,
            face_style: self.0[67] as i8,
            hair_style: self.0[68] as i8,
            hair_color: self.0[69] as i8,
            skin_color: self.0[70] as i8,
            eye_color: self.0[71] as i8,
            height: self.0[72] as i8,
            body: self.0[73] as i8,
            class: read_i32(&self.0, 76),
            appearance_flag: self.0[80] as i8,
            tutorial_flag: self.0[81] as i8,
            payzone_flag: self.0[82] as i8,
        }
    }

    pub fn equipped_item(&self, slot: CharacterEquipSlot0104) -> EquippedItem0104 {
        let offset = Self::EQUIPMENT_OFFSET + slot as usize * Self::EQUIPPED_ITEM_SIZE;
        EquippedItem0104 {
            item_type: read_i16(&self.0, offset),
            item_id: read_i16(&self.0, offset + 2),
            option: read_i32(&self.0, offset + 4),
            time_limit: read_i32(&self.0, offset + 8),
        }
    }

    pub fn equipment(&self) -> [EquippedItem0104; CHARACTER_EQUIP_SLOT_COUNT_0104] {
        CharacterEquipSlot0104::ALL.map(|slot| self.equipped_item(slot))
    }
}

impl Default for CharacterInfo0104 {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl fmt::Debug for CharacterInfo0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CharacterInfo0104")
            .field("slot", &self.slot())
            .field("level", &self.level())
            .field("pc_uid", &self.pc_uid())
            .field("first_name", &self.first_name())
            .field("last_name", &self.last_name())
            .field("position", &self.position())
            .finish_non_exhaustive()
    }
}

impl WirePayload for CharacterInfo0104 {
    const SIZE: usize = 204;

    fn encode(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self(bytes.try_into().expect("payload size checked")))
    }
}

/// Exact protocol-0104 `sP_CL2LS_REQ_CHAR_DELETE`.
///
/// There is deliberately no password or confirmation-code field: OpenFusion authorizes this UID
/// against the account attached to the current login socket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterDeleteRequest0104 {
    pub pc_uid: i64,
}

impl WirePayload for CharacterDeleteRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        self.pc_uid.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_uid: read_i64(bytes, 0),
        })
    }
}

/// Exact protocol-0104 `sP_LS2CL_REP_CHAR_DELETE_SUCC` (`#pragma pack(1)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterDeleteSuccess0104 {
    pub slot: i8,
}

impl WirePayload for CharacterDeleteSuccess0104 {
    const SIZE: usize = 1;

    fn encode(&self) -> Vec<u8> {
        vec![self.slot as u8]
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            slot: bytes[0] as i8,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShardSelectSuccess {
    pub server_ip: [u8; 16],
    pub server_port: i32,
    pub enter_serial_key: i64,
}

impl ShardSelectSuccess {
    pub fn server_ip_string(&self) -> String {
        let length = self
            .server_ip
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.server_ip.len());
        String::from_utf8_lossy(&self.server_ip[..length]).into_owned()
    }
}

impl WirePayload for ShardSelectSuccess {
    const SIZE: usize = 28;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0u8; Self::SIZE];
        out[..16].copy_from_slice(&self.server_ip);
        write_i32(&mut out, 16, self.server_port);
        write_i64(&mut out, 20, self.enter_serial_key);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            server_ip: bytes[..16].try_into().expect("fixed range"),
            server_port: read_i32(bytes, 16),
            enter_serial_key: read_i64(bytes, 20),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_MOVE` (pack(4), 44 bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcMoveRequest0104 {
    pub client_time: u64,
    pub position: [i32; 3],
    /// Server-axis velocity: X, Y (client Z), Z (client Y).
    pub velocity: [f32; 3],
    pub angle: i32,
    pub key_value: u8,
    pub speed: i32,
}

impl WirePayload for PcMoveRequest0104 {
    const SIZE: usize = 44;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        for (index, value) in self.velocity.into_iter().enumerate() {
            write_f32(&mut out, 20 + index * 4, value);
        }
        write_i32(&mut out, 32, self.angle);
        out[36] = self.key_value;
        write_i32(&mut out, 40, self.speed);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            client_time: read_u64(bytes, 0),
            position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            velocity: [
                read_f32(bytes, 20),
                read_f32(bytes, 24),
                read_f32(bytes, 28),
            ],
            angle: read_i32(bytes, 32),
            key_value: bytes[36],
            speed: read_i32(bytes, 40),
        })
    }
}
