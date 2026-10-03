use super::*;

/// Exact protocol-0104 `sP_CL2LS_REQ_SAVE_CHAR_NAME`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterNameSaveRequest0104 {
    pub slot: i8,
    pub gender: i8,
    pub first_name_code: i32,
    pub last_name_code: i32,
    pub middle_name_code: i32,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl CharacterNameSaveRequest0104 {
    pub fn from_check(
        slot: i8,
        gender: i8,
        request: &CharacterNameCheckRequest0104,
        checked: &CharacterNameCheckSuccess0104,
    ) -> Self {
        Self {
            slot,
            gender,
            first_name_code: request.first_name_code,
            last_name_code: request.last_name_code,
            middle_name_code: request.middle_name_code,
            first_name: checked.first_name.clone(),
            last_name: checked.last_name.clone(),
        }
    }
}

impl WirePayload for CharacterNameSaveRequest0104 {
    const SIZE: usize = 68;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0] = self.slot as u8;
        out[1] = self.gender as u8;
        // Bytes 2..4 are the C pack(4) alignment before iFNCode.
        write_i32(&mut out, 4, self.first_name_code);
        write_i32(&mut out, 8, self.last_name_code);
        write_i32(&mut out, 12, self.middle_name_code);
        write_utf16(&mut out, 16, &self.first_name);
        write_utf16(&mut out, 34, &self.last_name);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            slot: bytes[0] as i8,
            gender: bytes[1] as i8,
            first_name_code: read_i32(bytes, 4),
            last_name_code: read_i32(bytes, 8),
            middle_name_code: read_i32(bytes, 12),
            first_name: read_utf16(bytes, 16),
            last_name: read_utf16(bytes, 34),
        })
    }
}

/// Exact protocol-0104 `sP_LS2CL_REP_SAVE_CHAR_NAME_SUCC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterNameSaveSuccess0104 {
    pub pc_uid: i64,
    pub slot: i8,
    pub gender: i8,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl WirePayload for CharacterNameSaveSuccess0104 {
    const SIZE: usize = 64;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i64(&mut out, 0, self.pc_uid);
        out[8] = self.slot as u8;
        out[9] = self.gender as u8;
        write_utf16(&mut out, 10, &self.first_name);
        write_utf16(&mut out, 28, &self.last_name);
        // Bytes 62..64 are C tail padding.
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_uid: read_i64(bytes, 0),
            slot: bytes[8] as i8,
            gender: bytes[9] as i8,
            first_name: read_utf16(bytes, 10),
            last_name: read_utf16(bytes, 28),
        })
    }
}

/// Exact protocol-0104 `sP_CL2LS_REQ_SAVE_CHAR_TUTOR` (`#pragma pack(4)`).
///
/// The three trailing bytes are C-ABI tail padding and are always emitted as zeroes. OpenFusion
/// consumes this request without sending a success response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterTutorialSaveRequest0104 {
    pub pc_uid: i64,
    pub tutorial_flag: i8,
}

impl WirePayload for CharacterTutorialSaveRequest0104 {
    const SIZE: usize = 12;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0u8; Self::SIZE];
        write_i64(&mut out, 0, self.pc_uid);
        out[8] = self.tutorial_flag as u8;
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_uid: read_i64(bytes, 0),
            tutorial_flag: bytes[8] as i8,
        })
    }
}

pub(super) fn write_utf16<const N: usize>(bytes: &mut [u8], offset: usize, value: &FixedUtf16<N>) {
    for (index, unit) in value.as_units().iter().enumerate() {
        let start = offset + index * 2;
        bytes[start..start + 2].copy_from_slice(&unit.to_le_bytes());
    }
}

pub(super) fn write_i32(bytes: &mut [u8], offset: usize, value: i32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_i64(bytes: &mut [u8], offset: usize, value: i64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_f32(bytes: &mut [u8], offset: usize, value: f32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
