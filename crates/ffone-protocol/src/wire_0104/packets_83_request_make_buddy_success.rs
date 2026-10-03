// Generated wire layouts; do not edit by hand.
use super::*;
/// `sP_FE2CL_REP_REQUEST_MAKE_BUDDY_SUCC` (`#pragma pack(4)`, 16 bytes, packet ID `0x83000065`).
///
/// Generated packet mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestMakeBuddySuccess0104 {
    /// `iRequestID` at offset 0.
    pub request_id: i32,
    /// `iBuddyID` at offset 4.
    pub buddy_id: i32,
    /// `iBuddyPCUID` at offset 8.
    pub buddy_pcuid: i64,
}

impl RequestMakeBuddySuccess0104 {
    pub const SIZE: usize = 16;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.request_id.to_le_bytes());
        write_prim(out, 4, &self.buddy_id.to_le_bytes());
        write_prim(out, 8, &self.buddy_pcuid.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            request_id: i32::from_le_bytes(read_array(bytes, 0)),
            buddy_id: i32::from_le_bytes(read_array(bytes, 4)),
            buddy_pcuid: i64::from_le_bytes(read_array(bytes, 8)),
        }
    }
}

impl WirePayload for RequestMakeBuddySuccess0104 {
    const SIZE: usize = Self::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        self.write_into(&mut out);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self::read_from(bytes))
    }
}
