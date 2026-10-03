// Generated wire layouts; do not edit by hand.
use super::*;
/// `sSkillResult_Stamina_Self` (`#pragma pack(4)`, 24 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillResultStaminaSelf0104 {
    /// `eCT` at offset 0.
    pub e_ct: i32,
    /// `iID` at offset 4.
    pub id: i32,
    /// `iReduceHP` at offset 8.
    pub reduce_hp: i32,
    /// `iHP` at offset 12.
    pub hp: i32,
    /// `iHealNanoStamina` at offset 16.
    pub heal_nano_stamina: i16,
    /// `Nano` at offset 18.
    pub nano: Nano0104,
}

impl SkillResultStaminaSelf0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_ct.to_le_bytes());
        write_prim(out, 4, &self.id.to_le_bytes());
        write_prim(out, 8, &self.reduce_hp.to_le_bytes());
        write_prim(out, 12, &self.hp.to_le_bytes());
        write_prim(out, 16, &self.heal_nano_stamina.to_le_bytes());
        self.nano.write_into(&mut out[18..24]);
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_ct: i32::from_le_bytes(read_array(bytes, 0)),
            id: i32::from_le_bytes(read_array(bytes, 4)),
            reduce_hp: i32::from_le_bytes(read_array(bytes, 8)),
            hp: i32::from_le_bytes(read_array(bytes, 12)),
            heal_nano_stamina: i16::from_le_bytes(read_array(bytes, 16)),
            nano: Nano0104::read_from(&bytes[18..24]),
        }
    }
}

impl WirePayload for SkillResultStaminaSelf0104 {
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

/// `sTimeBuff` (`#pragma pack(4)`, 28 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeBuff0104 {
    /// `iTimeLimit` at offset 0.
    pub time_limit: u64,
    /// `iTimeDuration` at offset 8.
    pub time_duration: u64,
    /// `iTimeRepeat` at offset 16.
    pub time_repeat: i32,
    /// `iValue` at offset 20.
    pub value: i32,
    /// `iConfirmNum` at offset 24.
    pub confirm_num: i32,
}

impl TimeBuff0104 {
    pub const SIZE: usize = 28;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.time_limit.to_le_bytes());
        write_prim(out, 8, &self.time_duration.to_le_bytes());
        write_prim(out, 16, &self.time_repeat.to_le_bytes());
        write_prim(out, 20, &self.value.to_le_bytes());
        write_prim(out, 24, &self.confirm_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            time_limit: u64::from_le_bytes(read_array(bytes, 0)),
            time_duration: u64::from_le_bytes(read_array(bytes, 8)),
            time_repeat: i32::from_le_bytes(read_array(bytes, 16)),
            value: i32::from_le_bytes(read_array(bytes, 20)),
            confirm_num: i32::from_le_bytes(read_array(bytes, 24)),
        }
    }
}

impl WirePayload for TimeBuff0104 {
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

/// `sTimeBuff_Svr` (`#pragma pack(4)`, 32 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeBuffSvr0104 {
    /// `iTimeLimit` at offset 0.
    pub time_limit: u64,
    /// `iTimeDuration` at offset 8.
    pub time_duration: u64,
    /// `iTimeRepeat` at offset 16.
    pub time_repeat: i32,
    /// `iValue` at offset 20.
    pub value: i32,
    /// `iConfirmNum` at offset 24.
    pub confirm_num: i32,
    /// `iTimeFlow` at offset 28.
    pub time_flow: i16,
}

impl TimeBuffSvr0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.time_limit.to_le_bytes());
        write_prim(out, 8, &self.time_duration.to_le_bytes());
        write_prim(out, 16, &self.time_repeat.to_le_bytes());
        write_prim(out, 20, &self.value.to_le_bytes());
        write_prim(out, 24, &self.confirm_num.to_le_bytes());
        write_prim(out, 28, &self.time_flow.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            time_limit: u64::from_le_bytes(read_array(bytes, 0)),
            time_duration: u64::from_le_bytes(read_array(bytes, 8)),
            time_repeat: i32::from_le_bytes(read_array(bytes, 16)),
            value: i32::from_le_bytes(read_array(bytes, 20)),
            confirm_num: i32::from_le_bytes(read_array(bytes, 24)),
            time_flow: i16::from_le_bytes(read_array(bytes, 28)),
        }
    }
}

impl WirePayload for TimeBuffSvr0104 {
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

/// `sTimeLimitItemDeleteInfo2CL` (`#pragma pack(4)`, 8 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeLimitItemDeleteInfo0104 {
    /// `eIL` at offset 0.
    pub e_il: i32,
    /// `iSlotNum` at offset 4.
    pub slot_num: i32,
}

impl TimeLimitItemDeleteInfo0104 {
    pub const SIZE: usize = 8;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_il.to_le_bytes());
        write_prim(out, 4, &self.slot_num.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_il: i32::from_le_bytes(read_array(bytes, 0)),
            slot_num: i32::from_le_bytes(read_array(bytes, 4)),
        }
    }
}

impl WirePayload for TimeLimitItemDeleteInfo0104 {
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

/// `sTransportationAppearanceData` (`#pragma pack(4)`, 24 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportationAppearanceData0104 {
    /// `eTT` at offset 0.
    pub e_tt: i32,
    /// `iT_ID` at offset 4.
    pub t_id: i32,
    /// `iT_Type` at offset 8.
    pub t_type: i32,
    /// `iX` at offset 12.
    pub x: i32,
    /// `iY` at offset 16.
    pub y: i32,
    /// `iZ` at offset 20.
    pub z: i32,
}

impl TransportationAppearanceData0104 {
    pub const SIZE: usize = 24;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.e_tt.to_le_bytes());
        write_prim(out, 4, &self.t_id.to_le_bytes());
        write_prim(out, 8, &self.t_type.to_le_bytes());
        write_prim(out, 12, &self.x.to_le_bytes());
        write_prim(out, 16, &self.y.to_le_bytes());
        write_prim(out, 20, &self.z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            e_tt: i32::from_le_bytes(read_array(bytes, 0)),
            t_id: i32::from_le_bytes(read_array(bytes, 4)),
            t_type: i32::from_le_bytes(read_array(bytes, 8)),
            x: i32::from_le_bytes(read_array(bytes, 12)),
            y: i32::from_le_bytes(read_array(bytes, 16)),
            z: i32::from_le_bytes(read_array(bytes, 20)),
        }
    }
}

impl WirePayload for TransportationAppearanceData0104 {
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

/// `sTransportationLoadData` (`#pragma pack(4)`, 32 bytes).
///
/// Generated record mirror of the clean Retrobution marshaled layout.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportationLoadData0104 {
    /// `iAISvrID` at offset 0.
    pub ai_svr_id: i32,
    /// `eTT` at offset 4.
    pub e_tt: i32,
    /// `iT_Type` at offset 8.
    pub t_type: i32,
    /// `iMapType` at offset 12.
    pub map_type: i32,
    /// `iMapNum` at offset 16.
    pub map_num: i32,
    /// `iX` at offset 20.
    pub x: i32,
    /// `iY` at offset 24.
    pub y: i32,
    /// `iZ` at offset 28.
    pub z: i32,
}

impl TransportationLoadData0104 {
    pub const SIZE: usize = 32;

    /// Write the exact layout into `out`, which must be `SIZE` bytes.
    pub fn write_into(&self, out: &mut [u8]) {
        debug_assert_eq!(out.len(), Self::SIZE);
        write_prim(out, 0, &self.ai_svr_id.to_le_bytes());
        write_prim(out, 4, &self.e_tt.to_le_bytes());
        write_prim(out, 8, &self.t_type.to_le_bytes());
        write_prim(out, 12, &self.map_type.to_le_bytes());
        write_prim(out, 16, &self.map_num.to_le_bytes());
        write_prim(out, 20, &self.x.to_le_bytes());
        write_prim(out, 24, &self.y.to_le_bytes());
        write_prim(out, 28, &self.z.to_le_bytes());
    }

    /// Read the exact layout from `bytes`, which must be `SIZE` bytes.
    pub fn read_from(bytes: &[u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::SIZE);
        Self {
            ai_svr_id: i32::from_le_bytes(read_array(bytes, 0)),
            e_tt: i32::from_le_bytes(read_array(bytes, 4)),
            t_type: i32::from_le_bytes(read_array(bytes, 8)),
            map_type: i32::from_le_bytes(read_array(bytes, 12)),
            map_num: i32::from_le_bytes(read_array(bytes, 16)),
            x: i32::from_le_bytes(read_array(bytes, 20)),
            y: i32::from_le_bytes(read_array(bytes, 24)),
            z: i32::from_le_bytes(read_array(bytes, 28)),
        }
    }
}

impl WirePayload for TransportationLoadData0104 {
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
