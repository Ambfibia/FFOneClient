use super::*;

/// Protocol-0104 `sP_CL2FE_REQ_PC_STOP` (pack(4), 20 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcStopRequest0104 {
    pub client_time: u64,
    pub position: [i32; 3],
}

impl WirePayload for PcStopRequest0104 {
    const SIZE: usize = 20;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            client_time: read_u64(bytes, 0),
            position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_JUMP` (pack(4), 44 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcJumpRequest0104 {
    pub client_time: u64,
    pub position: [i32; 3],
    /// Server-axis velocity, scaled by 100 like the original client.
    pub velocity: [i32; 3],
    pub angle: i32,
    pub key_value: u8,
    pub speed: i32,
}

impl WirePayload for PcJumpRequest0104 {
    const SIZE: usize = 44;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        for (index, value) in self.velocity.into_iter().enumerate() {
            write_i32(&mut out, 20 + index * 4, value);
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
                read_i32(bytes, 20),
                read_i32(bytes, 24),
                read_i32(bytes, 28),
            ],
            angle: read_i32(bytes, 32),
            key_value: bytes[36],
            speed: read_i32(bytes, 40),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_JUMPPAD` (pack(4), 40 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcJumppadRequest0104 {
    pub client_time: u64,
    pub position: [i32; 3],
    /// Server-axis velocity, scaled by 100 like the clean client.
    pub velocity: [i32; 3],
    pub angle: i32,
    pub key_value: u8,
}

impl WirePayload for PcJumppadRequest0104 {
    const SIZE: usize = 40;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        for (index, value) in self.velocity.into_iter().enumerate() {
            write_i32(&mut out, 20 + index * 4, value);
        }
        write_i32(&mut out, 32, self.angle);
        out[36] = self.key_value;
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            client_time: read_u64(bytes, 0),
            position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            velocity: [
                read_i32(bytes, 20),
                read_i32(bytes, 24),
                read_i32(bytes, 28),
            ],
            angle: read_i32(bytes, 32),
            key_value: bytes[36],
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_LAUNCHER` (pack(4), 40 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcLauncherRequest0104 {
    pub client_time: u64,
    pub position: [i32; 3],
    /// Server-axis velocity, scaled by 100 like the clean client.
    pub velocity: [i32; 3],
    pub angle: i32,
    pub speed: i32,
}

impl WirePayload for PcLauncherRequest0104 {
    const SIZE: usize = 40;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        for (index, value) in self.velocity.into_iter().enumerate() {
            write_i32(&mut out, 20 + index * 4, value);
        }
        write_i32(&mut out, 32, self.angle);
        write_i32(&mut out, 36, self.speed);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            client_time: read_u64(bytes, 0),
            position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            velocity: [
                read_i32(bytes, 20),
                read_i32(bytes, 24),
                read_i32(bytes, 28),
            ],
            angle: read_i32(bytes, 32),
            speed: read_i32(bytes, 36),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_MOVEPLATFORM` (pack(4), 64 bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcMovePlatformRequest0104 {
    pub client_time: u64,
    pub local_position: [i32; 3],
    pub position: [i32; 3],
    pub velocity: [f32; 3],
    pub down: i32,
    pub platform_id: i32,
    pub angle: i32,
    pub key_value: u8,
    pub speed: i32,
}

impl WirePayload for PcMovePlatformRequest0104 {
    const SIZE: usize = 64;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.local_position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 20 + index * 4, value);
        }
        for (index, value) in self.velocity.into_iter().enumerate() {
            write_f32(&mut out, 32 + index * 4, value);
        }
        write_i32(&mut out, 44, self.down);
        write_i32(&mut out, 48, self.platform_id);
        write_i32(&mut out, 52, self.angle);
        out[56] = self.key_value;
        write_i32(&mut out, 60, self.speed);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            client_time: read_u64(bytes, 0),
            local_position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            position: [
                read_i32(bytes, 20),
                read_i32(bytes, 24),
                read_i32(bytes, 28),
            ],
            velocity: [
                read_f32(bytes, 32),
                read_f32(bytes, 36),
                read_f32(bytes, 40),
            ],
            down: read_i32(bytes, 44),
            platform_id: read_i32(bytes, 48),
            angle: read_i32(bytes, 52),
            key_value: bytes[56],
            speed: read_i32(bytes, 60),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_MOVETRANSPORTATION` (pack(4), 60 bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcMoveTransportationRequest0104 {
    pub client_time: u64,
    pub local_position: [i32; 3],
    pub position: [i32; 3],
    pub velocity: [f32; 3],
    pub transportation_id: i32,
    pub angle: i32,
    pub key_value: u8,
    pub speed: i32,
}

impl WirePayload for PcMoveTransportationRequest0104 {
    const SIZE: usize = 60;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.local_position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 20 + index * 4, value);
        }
        for (index, value) in self.velocity.into_iter().enumerate() {
            write_f32(&mut out, 32 + index * 4, value);
        }
        write_i32(&mut out, 44, self.transportation_id);
        write_i32(&mut out, 48, self.angle);
        out[52] = self.key_value;
        write_i32(&mut out, 56, self.speed);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            client_time: read_u64(bytes, 0),
            local_position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            position: [
                read_i32(bytes, 20),
                read_i32(bytes, 24),
                read_i32(bytes, 28),
            ],
            velocity: [
                read_f32(bytes, 32),
                read_f32(bytes, 36),
                read_f32(bytes, 40),
            ],
            transportation_id: read_i32(bytes, 44),
            angle: read_i32(bytes, 48),
            key_value: bytes[52],
            speed: read_i32(bytes, 56),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_SLOPE` (pack(4), 48 bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcSlopeRequest0104 {
    pub client_time: u64,
    pub position: [i32; 3],
    pub angle: i32,
    pub speed: i32,
    pub key_value: u8,
    pub velocity: [f32; 3],
    pub slope_id: i32,
}

impl WirePayload for PcSlopeRequest0104 {
    const SIZE: usize = 48;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        write_i32(&mut out, 20, self.angle);
        write_i32(&mut out, 24, self.speed);
        out[28] = self.key_value;
        for (index, value) in self.velocity.into_iter().enumerate() {
            write_f32(&mut out, 32 + index * 4, value);
        }
        write_i32(&mut out, 44, self.slope_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            client_time: read_u64(bytes, 0),
            position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            angle: read_i32(bytes, 20),
            speed: read_i32(bytes, 24),
            key_value: bytes[28],
            velocity: [
                read_f32(bytes, 32),
                read_f32(bytes, 36),
                read_f32(bytes, 40),
            ],
            slope_id: read_i32(bytes, 44),
        })
    }
}

/// Protocol-0104 `sP_CL2FE_REQ_PC_ZIPLINE` (pack(4), 76 bytes).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcZiplineRequest0104 {
    pub client_time: u64,
    pub start_position: [i32; 3],
    pub moved_distance: f32,
    pub maximum_distance: f32,
    pub dummy: f32,
    pub position: [i32; 3],
    pub velocity: [f32; 3],
    pub down: i32,
    pub roll_max: i32,
    pub roll: u8,
    pub angle: i32,
    pub speed: i32,
}

impl WirePayload for PcZiplineRequest0104 {
    const SIZE: usize = 76;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_u64(&mut out, 0, self.client_time);
        for (index, value) in self.start_position.into_iter().enumerate() {
            write_i32(&mut out, 8 + index * 4, value);
        }
        write_f32(&mut out, 20, self.moved_distance);
        write_f32(&mut out, 24, self.maximum_distance);
        write_f32(&mut out, 28, self.dummy);
        for (index, value) in self.position.into_iter().enumerate() {
            write_i32(&mut out, 32 + index * 4, value);
        }
        for (index, value) in self.velocity.into_iter().enumerate() {
            write_f32(&mut out, 44 + index * 4, value);
        }
        write_i32(&mut out, 56, self.down);
        write_i32(&mut out, 60, self.roll_max);
        out[64] = self.roll;
        write_i32(&mut out, 68, self.angle);
        write_i32(&mut out, 72, self.speed);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            client_time: read_u64(bytes, 0),
            start_position: [read_i32(bytes, 8), read_i32(bytes, 12), read_i32(bytes, 16)],
            moved_distance: read_f32(bytes, 20),
            maximum_distance: read_f32(bytes, 24),
            dummy: read_f32(bytes, 28),
            position: [
                read_i32(bytes, 32),
                read_i32(bytes, 36),
                read_i32(bytes, 40),
            ],
            velocity: [
                read_f32(bytes, 44),
                read_f32(bytes, 48),
                read_f32(bytes, 52),
            ],
            down: read_i32(bytes, 56),
            roll_max: read_i32(bytes, 60),
            roll: bytes[64],
            angle: read_i32(bytes, 68),
            speed: read_i32(bytes, 72),
        })
    }
}

/// Movement broadcast from OpenFusion to the other visible clients.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcMove0104 {
    pub movement: PcMoveRequest0104,
    pub pc_id: i32,
    pub server_time: u64,
}

impl WirePayload for PcMove0104 {
    const SIZE: usize = 56;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[..PcMoveRequest0104::SIZE].copy_from_slice(&self.movement.encode());
        write_i32(&mut out, 44, self.pc_id);
        write_u64(&mut out, 48, self.server_time);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            movement: PcMoveRequest0104::decode(&bytes[..PcMoveRequest0104::SIZE])?,
            pc_id: read_i32(bytes, 44),
            server_time: read_u64(bytes, 48),
        })
    }
}

/// Stop broadcast from OpenFusion to the other visible clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcStop0104 {
    pub movement: PcStopRequest0104,
    pub pc_id: i32,
    pub server_time: u64,
}

impl WirePayload for PcStop0104 {
    const SIZE: usize = 32;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[..PcStopRequest0104::SIZE].copy_from_slice(&self.movement.encode());
        write_i32(&mut out, 20, self.pc_id);
        write_u64(&mut out, 24, self.server_time);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            movement: PcStopRequest0104::decode(&bytes[..PcStopRequest0104::SIZE])?,
            pc_id: read_i32(bytes, 20),
            server_time: read_u64(bytes, 24),
        })
    }
}

/// Jump broadcast from OpenFusion to the other visible clients.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcJump0104 {
    pub movement: PcJumpRequest0104,
    pub pc_id: i32,
    pub server_time: u64,
}

impl WirePayload for PcJump0104 {
    const SIZE: usize = 56;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[..PcJumpRequest0104::SIZE].copy_from_slice(&self.movement.encode());
        write_i32(&mut out, 44, self.pc_id);
        write_u64(&mut out, 48, self.server_time);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            movement: PcJumpRequest0104::decode(&bytes[..PcJumpRequest0104::SIZE])?,
            pc_id: read_i32(bytes, 44),
            server_time: read_u64(bytes, 48),
        })
    }
}

/// Exact protocol-0104 `sRunningQuest` embedded in `sPCLoadData2CL`
/// (`#pragma pack(4)`, 52 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunningQuest0104 {
    pub task_id: i32,
    pub kill_npc_ids: [i32; 3],
    pub remaining_kill_counts: [i32; 3],
    pub needed_item_ids: [i32; 3],
    pub needed_item_counts: [i32; 3],
}

impl WirePayload for RunningQuest0104 {
    const SIZE: usize = 52;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.task_id);
        for (index, value) in self.kill_npc_ids.into_iter().enumerate() {
            write_i32(&mut out, 4 + index * 4, value);
        }
        for (index, value) in self.remaining_kill_counts.into_iter().enumerate() {
            write_i32(&mut out, 16 + index * 4, value);
        }
        for (index, value) in self.needed_item_ids.into_iter().enumerate() {
            write_i32(&mut out, 28 + index * 4, value);
        }
        for (index, value) in self.needed_item_counts.into_iter().enumerate() {
            write_i32(&mut out, 40 + index * 4, value);
        }
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            task_id: read_i32(bytes, 0),
            kill_npc_ids: std::array::from_fn(|index| read_i32(bytes, 4 + index * 4)),
            remaining_kill_counts: std::array::from_fn(|index| read_i32(bytes, 16 + index * 4)),
            needed_item_ids: std::array::from_fn(|index| read_i32(bytes, 28 + index * 4)),
            needed_item_counts: std::array::from_fn(|index| read_i32(bytes, 40 + index * 4)),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcEnterSuccess {
    pub id: i32,
    pub load: PcLoadData0104,
    pub server_time: u64,
}

impl WirePayload for PcEnterSuccess {
    const SIZE: usize = 2700;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0u8; Self::SIZE];
        write_i32(&mut out, 0, self.id);
        out[4..2692].copy_from_slice(self.load.as_bytes());
        write_u64(&mut out, 2692, self.server_time);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            id: read_i32(bytes, 0),
            load: PcLoadData0104::decode(&bytes[4..2692])?,
            server_time: read_u64(bytes, 2692),
        })
    }
}

/// Protocol-0104 `sPCStyle`, independent of any renderer or game engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcStyle0104 {
    pub pc_uid: i64,
    pub name_check: i8,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
    pub gender: i8,
    pub face_style: i8,
    pub hair_style: i8,
    pub hair_color: i8,
    pub skin_color: i8,
    pub eye_color: i8,
    pub height: i8,
    pub body: i8,
    pub class: i32,
}

impl PcStyle0104 {
    pub const SIZE: usize = 76;

    pub fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::decode_exact(bytes))
    }

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            pc_uid: read_i64(bytes, 0),
            name_check: bytes[8] as i8,
            first_name: read_utf16(bytes, 10),
            last_name: read_utf16(bytes, 28),
            gender: bytes[62] as i8,
            face_style: bytes[63] as i8,
            hair_style: bytes[64] as i8,
            hair_color: bytes[65] as i8,
            skin_color: bytes[66] as i8,
            eye_color: bytes[67] as i8,
            height: bytes[68] as i8,
            body: bytes[69] as i8,
            class: read_i32(bytes, 72),
        }
    }

    pub(super) fn encode_into(&self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        write_i64(bytes, 0, self.pc_uid);
        bytes[8] = self.name_check as u8;
        write_utf16(bytes, 10, &self.first_name);
        write_utf16(bytes, 28, &self.last_name);
        bytes[62] = self.gender as u8;
        bytes[63] = self.face_style as u8;
        bytes[64] = self.hair_style as u8;
        bytes[65] = self.hair_color as u8;
        bytes[66] = self.skin_color as u8;
        bytes[67] = self.eye_color as u8;
        bytes[68] = self.height as u8;
        bytes[69] = self.body as u8;
        write_i32(bytes, 72, self.class);
    }
}

/// Protocol-0104 `sPCStyle2` (`#pragma pack(1)`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PcStyle2Flags0104 {
    pub appearance_flag: i8,
    pub tutorial_flag: i8,
    pub payzone_flag: i8,
}

impl PcStyle2Flags0104 {
    pub const SIZE: usize = 3;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            appearance_flag: bytes[0] as i8,
            tutorial_flag: bytes[1] as i8,
            payzone_flag: bytes[2] as i8,
        }
    }

    pub(super) fn encode_into(self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        bytes[0] = self.appearance_flag as u8;
        bytes[1] = self.tutorial_flag as u8;
        bytes[2] = self.payzone_flag as u8;
    }
}

/// Protocol-0104 `sOnItem` (`#pragma pack(2)`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnItem0104 {
    pub hand_id: i16,
    pub upper_body_id: i16,
    pub lower_body_id: i16,
    pub foot_id: i16,
    pub head_id: i16,
    pub face_id: i16,
    pub back_id: i16,
}

impl OnItem0104 {
    pub const SIZE: usize = 14;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            hand_id: read_i16(bytes, 0),
            upper_body_id: read_i16(bytes, 2),
            lower_body_id: read_i16(bytes, 4),
            foot_id: read_i16(bytes, 6),
            head_id: read_i16(bytes, 8),
            face_id: read_i16(bytes, 10),
            back_id: read_i16(bytes, 12),
        }
    }

    pub(super) fn encode_into(self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        for (index, value) in [
            self.hand_id,
            self.upper_body_id,
            self.lower_body_id,
            self.foot_id,
            self.head_id,
            self.face_id,
            self.back_id,
        ]
        .into_iter()
        .enumerate()
        {
            let offset = index * 2;
            bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
    }
}

/// Protocol-0104 `sOnItem_Index` sent only while finalizing character creation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnItemIndex0104 {
    pub upper_body_index: i16,
    pub lower_body_index: i16,
    pub foot_index: i16,
    pub face_style_index: i16,
    pub hair_style_index: i16,
}

impl OnItemIndex0104 {
    pub const SIZE: usize = 10;

    pub(super) fn decode_exact(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            upper_body_index: read_i16(bytes, 0),
            lower_body_index: read_i16(bytes, 2),
            foot_index: read_i16(bytes, 4),
            face_style_index: read_i16(bytes, 6),
            hair_style_index: read_i16(bytes, 8),
        }
    }

    pub(super) fn encode_into(self, bytes: &mut [u8]) {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        for (index, value) in [
            self.upper_body_index,
            self.lower_body_index,
            self.foot_index,
            self.face_style_index,
            self.hair_style_index,
        ]
        .into_iter()
        .enumerate()
        {
            let offset = index * 2;
            bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
    }
}

/// Exact protocol-0104 `sP_CL2LS_REQ_CHAR_CREATE`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterCreateRequest0104 {
    pub style: PcStyle0104,
    pub equipped: OnItem0104,
    pub selected_indices: OnItemIndex0104,
}
