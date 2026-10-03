use super::*;

pub(super) fn validate_body_len(body_len: usize) -> Result<(), FrameError> {
    if body_len < 4 {
        return Err(FrameError::InvalidBodyLength { declared: body_len });
    }
    if body_len > MAX_BODY_SIZE_0104 {
        return Err(FrameError::BodyTooLarge {
            declared: body_len,
            maximum: MAX_BODY_SIZE_0104,
        });
    }
    Ok(())
}

pub(super) fn validate_buddy_list_value(field: &'static str, value: i32) -> Result<usize, PayloadError> {
    let maximum = BUDDY_LIST_CAPACITY_0104 as i32;
    if !(0..=maximum).contains(&value) {
        return Err(PayloadError::ValueOutOfRange {
            field,
            value,
            minimum: 0,
            maximum,
        });
    }
    Ok(value as usize)
}

pub(super) fn validate_buddy_list_end(list_num: usize, buddy_count: usize) -> Result<(), PayloadError> {
    let end = list_num + buddy_count;
    if end > BUDDY_LIST_CAPACITY_0104 {
        return Err(PayloadError::ValueOutOfRange {
            field: "buddy list_num + count",
            value: end as i32,
            minimum: 0,
            maximum: BUDDY_LIST_CAPACITY_0104 as i32,
        });
    }
    Ok(())
}

/// Exact protocol-0104 `sP_CL2LS_REQ_CHECK_CHAR_NAME`.
///
/// OpenFusion echoes this name in `P_LS2CL_REP_CHECK_CHAR_NAME_SUCC`; its current server
/// implementation performs the authoritative availability/content check when the client sends
/// [`CharacterNameSaveRequest0104`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterNameCheckRequest0104 {
    pub first_name_code: i32,
    pub last_name_code: i32,
    pub middle_name_code: i32,
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl CharacterNameCheckRequest0104 {
    pub fn new(
        first_name: &str,
        last_name: &str,
        first_name_code: i32,
        middle_name_code: i32,
        last_name_code: i32,
    ) -> Result<Self, PayloadError> {
        Ok(Self {
            first_name_code,
            last_name_code,
            middle_name_code,
            first_name: FixedUtf16::from_str(first_name)?,
            last_name: FixedUtf16::from_str(last_name)?,
        })
    }
}

impl WirePayload for CharacterNameCheckRequest0104 {
    const SIZE: usize = 64;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.first_name_code);
        write_i32(&mut out, 4, self.last_name_code);
        write_i32(&mut out, 8, self.middle_name_code);
        write_utf16(&mut out, 12, &self.first_name);
        write_utf16(&mut out, 30, &self.last_name);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            first_name_code: read_i32(bytes, 0),
            last_name_code: read_i32(bytes, 4),
            middle_name_code: read_i32(bytes, 8),
            first_name: read_utf16(bytes, 12),
            last_name: read_utf16(bytes, 30),
        })
    }
}

/// Exact protocol-0104 `sP_LS2CL_REP_CHECK_CHAR_NAME_SUCC` (`#pragma pack(2)`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterNameCheckSuccess0104 {
    pub first_name: FixedUtf16<9>,
    pub last_name: FixedUtf16<17>,
}

impl WirePayload for CharacterNameCheckSuccess0104 {
    const SIZE: usize = 52;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_utf16(&mut out, 0, &self.first_name);
        write_utf16(&mut out, 18, &self.last_name);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            first_name: read_utf16(bytes, 0),
            last_name: read_utf16(bytes, 18),
        })
    }
}

pub(super) fn require_counted_header(
    payload: &[u8],
    header_size: usize,
) -> Result<(), CountedPayloadError0104> {
    if payload.len() < header_size {
        return Err(CountedPayloadError0104::MissingHeader {
            expected_at_least: header_size,
            actual: payload.len(),
        });
    }
    Ok(())
}

pub(super) fn require_size(bytes: &[u8], expected: usize) -> Result<(), PayloadError> {
    if bytes.len() != expected {
        return Err(PayloadError::WrongSize {
            expected,
            actual: bytes.len(),
        });
    }
    Ok(())
}

pub(super) fn require_prefix(bytes: &[u8], expected: usize) -> Result<(), PayloadError> {
    if bytes.len() < expected {
        return Err(PayloadError::WrongSize {
            expected,
            actual: bytes.len(),
        });
    }
    Ok(())
}
