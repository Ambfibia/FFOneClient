use super::*;

/// Exact protocol-0104 `sP_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VendorTableUpdateRequest0104 {
    pub npc_id: i32,
    pub vendor_id: i32,
}

impl WirePayload for VendorTableUpdateRequest0104 {
    const SIZE: usize = 8;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        write_i32(&mut out, 0, self.npc_id);
        write_i32(&mut out, 4, self.vendor_id);
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            npc_id: read_i32(bytes, 0),
            vendor_id: read_i32(bytes, 4),
        })
    }
}

/// Exact protocol-0104 `sP_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VendorTableUpdateSuccess0104 {
    pub items: [ItemVendor0104; VENDOR_TABLE_ITEM_COUNT_0104],
}

impl WirePayload for VendorTableUpdateSuccess0104 {
    const SIZE: usize = VENDOR_TABLE_ITEM_COUNT_0104 * ItemVendor0104::SIZE;

    fn encode(&self) -> Vec<u8> {
        let mut out = vec![0; Self::SIZE];
        for (index, item) in self.items.iter().copied().enumerate() {
            let start = index * ItemVendor0104::SIZE;
            item.encode_into(&mut out[start..start + ItemVendor0104::SIZE]);
        }
        out
    }

    fn decode(bytes: &[u8]) -> Result<Self, PayloadError> {
        require_size(bytes, Self::SIZE)?;
        Ok(Self {
            items: std::array::from_fn(|index| {
                let start = index * ItemVendor0104::SIZE;
                ItemVendor0104::decode_exact(&bytes[start..start + ItemVendor0104::SIZE])
            }),
        })
    }
}
