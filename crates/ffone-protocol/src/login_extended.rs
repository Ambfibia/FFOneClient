use crate::{FixedUtf16, PayloadError, WirePayload};

fn require_size(bytes: &[u8], expected: usize) -> Result<(), PayloadError> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(PayloadError::WrongSize {
            expected,
            actual: bytes.len(),
        })
    }
}

fn write_utf16<const N: usize>(out: &mut [u8], offset: usize, value: &FixedUtf16<N>) {
    for (index, unit) in value.as_units().iter().enumerate() {
        let start = offset + index * 2;
        out[start..start + 2].copy_from_slice(&unit.to_le_bytes());
    }
}

fn read_utf16<const N: usize>(bytes: &[u8], offset: usize) -> FixedUtf16<N> {
    FixedUtf16::from_units(std::array::from_fn(|index| {
        let start = offset + index * 2;
        u16::from_le_bytes([bytes[start], bytes[start + 1]])
    }))
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("four bytes"))
}

fn read_i64(bytes: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("eight bytes"))
}

/// `sP_CL2LS_REQ_PC_EXIT_DUPLICATE` (`#pragma pack(2)`, 132 bytes).
///
/// The password remains an explicit fixed buffer so callers can construct the
/// packet without ever storing a plaintext `String` in session state.
#[derive(Clone, PartialEq, Eq)]
pub struct DuplicateExitRequest0104 {
    pub id: FixedUtf16<33>,
    pub password: FixedUtf16<33>,
}

impl std::fmt::Debug for DuplicateExitRequest0104 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DuplicateExitRequest0104")
            .field("id", &self.id)
            .field("password", &"<redacted>")
            .finish()
    }
}

impl WirePayload for DuplicateExitRequest0104 {
    const SIZE: usize = 132;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.id);
        write_utf16(&mut out, 66, &self.password);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            id: read_utf16(bytes, 0),
            password: read_utf16(bytes, 66),
        })
    }
}

/// Declared login-server duplicate-exit reply (`#pragma pack(4)`, 4 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DuplicateExitReply0104 {
    pub error_code: i32,
}

impl WirePayload for DuplicateExitReply0104 {
    const SIZE: usize = 4;

    fn encode(&self) -> Vec<u8> {
        self.error_code.to_le_bytes().to_vec()
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            error_code: read_i32(bytes, 0),
        })
    }
}

/// `sP_CL2LS_REQ_CHANGE_CHAR_NAME` (`#pragma pack(4)`, 76 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterNameChangeRequest0104 {
    pub pc_uid: i64,
    pub slot: i8,
    pub gender: i8,
    pub first_name_code: i32,
    pub last_name_code: i32,
    pub middle_name_code: i32,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl WirePayload for CharacterNameChangeRequest0104 {
    const SIZE: usize = 76;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..8].copy_from_slice(&self.pc_uid.to_le_bytes());
        out[8] = self.slot as u8;
        out[9] = self.gender as u8;
        // 10..12 is pack(4) padding.
        out[12..16].copy_from_slice(&self.first_name_code.to_le_bytes());
        out[16..20].copy_from_slice(&self.last_name_code.to_le_bytes());
        out[20..24].copy_from_slice(&self.middle_name_code.to_le_bytes());
        write_utf16(&mut out, 24, &self.first_name);
        write_utf16(&mut out, 42, &self.last_name);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_uid: read_i64(bytes, 0),
            slot: bytes[8] as i8,
            gender: bytes[9] as i8,
            first_name_code: read_i32(bytes, 12),
            last_name_code: read_i32(bytes, 16),
            middle_name_code: read_i32(bytes, 20),
            first_name: read_utf16(bytes, 24),
            last_name: read_utf16(bytes, 42),
        })
    }
}

/// `sP_LS2CL_REP_CHANGE_CHAR_NAME_SUCC` (`#pragma pack(4)`, 64 bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterNameChangeSuccess0104 {
    pub pc_uid: i64,
    pub slot: i8,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl WirePayload for CharacterNameChangeSuccess0104 {
    const SIZE: usize = 64;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..8].copy_from_slice(&self.pc_uid.to_le_bytes());
        out[8] = self.slot as u8;
        // 9..10 is the one-byte alignment gap before UTF-16.
        write_utf16(&mut out, 10, &self.first_name);
        write_utf16(&mut out, 28, &self.last_name);
        // 62..64 is pack(4) tail padding.
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_uid: read_i64(bytes, 0),
            slot: bytes[8] as i8,
            first_name: read_utf16(bytes, 10),
            last_name: read_utf16(bytes, 28),
        })
    }
}

/// `sP_LS2CL_REP_CHANGE_CHAR_NAME_FAIL` (`#pragma pack(4)`, 16 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterNameChangeFailure0104 {
    pub pc_uid: i64,
    pub slot: i8,
    pub error_code: i32,
}

impl WirePayload for CharacterNameChangeFailure0104 {
    const SIZE: usize = 16;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        out[0..8].copy_from_slice(&self.pc_uid.to_le_bytes());
        out[8] = self.slot as u8;
        // 9..12 is pack(4) padding.
        out[12..16].copy_from_slice(&self.error_code.to_le_bytes());
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            pc_uid: read_i64(bytes, 0),
            slot: bytes[8] as i8,
            error_code: read_i32(bytes, 12),
        })
    }
}

#[cfg(test)]
mod tests;
