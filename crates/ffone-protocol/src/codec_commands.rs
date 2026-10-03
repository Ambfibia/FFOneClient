use super::*;

#[derive(Clone, PartialEq, Eq)]
pub struct LoginRequest {
    pub id: FixedUtf16<33>,
    pub password: FixedUtf16<33>,
    pub client_version_a: i32,
    pub client_version_b: i32,
    pub client_version_c: i32,
    /// 1 = UTF-16 credentials, 2 = byte cookie fields.
    pub login_type: i32,
    pub cookie_teg_id: [u8; 64],
    pub cookie_auth_id: [u8; 255],
}

impl fmt::Debug for LoginRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoginRequest")
            .field("id", &self.id)
            .field("password", &"<redacted>")
            .field("client_version_a", &self.client_version_a)
            .field("client_version_b", &self.client_version_b)
            .field("client_version_c", &self.client_version_c)
            .field("login_type", &self.login_type)
            .finish_non_exhaustive()
    }
}

impl LoginRequest {
    pub fn password_login(
        id: &str,
        password: &str,
        client_version_a: i32,
        client_version_b: i32,
        client_version_c: i32,
    ) -> Result<Self, PayloadError> {
        Ok(Self {
            id: FixedUtf16::from_str(id)?,
            password: FixedUtf16::from_str(password)?,
            client_version_a,
            client_version_b,
            client_version_c,
            login_type: 1,
            cookie_teg_id: [0; 64],
            cookie_auth_id: [0; 255],
        })
    }
}

impl WirePayload for LoginRequest {
    const SIZE: usize = 468;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0u8; Self::SIZE];
        write_utf16(&mut out, 0, &self.id);
        write_utf16(&mut out, 66, &self.password);
        write_i32(&mut out, 132, self.client_version_a);
        write_i32(&mut out, 136, self.client_version_b);
        write_i32(&mut out, 140, self.client_version_c);
        write_i32(&mut out, 144, self.login_type);
        out[148..212].copy_from_slice(&self.cookie_teg_id);
        out[212..467].copy_from_slice(&self.cookie_auth_id);
        // Byte 467 is C tail padding.
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            id: read_utf16(bytes, 0),
            password: read_utf16(bytes, 66),
            client_version_a: read_i32(bytes, 132),
            client_version_b: read_i32(bytes, 136),
            client_version_c: read_i32(bytes, 140),
            login_type: read_i32(bytes, 144),
            cookie_teg_id: bytes[148..212].try_into().expect("fixed range"),
            cookie_auth_id: bytes[212..467].try_into().expect("fixed range"),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterSelectRequest {
    pub pc_uid: i64,
}

impl WirePayload for CharacterSelectRequest {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PcEnterRequest {
    pub id: FixedUtf16<33>,
    pub temporary_value: i32,
    pub enter_serial_key: i64,
}

impl WirePayload for PcEnterRequest {
    const SIZE: usize = 80;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0u8; Self::SIZE];
        write_utf16(&mut out, 0, &self.id);
        // Bytes 66..68 are pack(4) padding.
        write_i32(&mut out, 68, self.temporary_value);
        write_i64(&mut out, 72, self.enter_serial_key);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            id: read_utf16(bytes, 0),
            temporary_value: read_i32(bytes, 68),
            enter_serial_key: read_i64(bytes, 72),
        })
    }
}
