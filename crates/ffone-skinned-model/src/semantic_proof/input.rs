use super::*;

pub(super) fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(read_array::<4>(bytes, offset)?))
}

pub(super) fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> Result<[u8; N]> {
    bytes
        .get(offset..checked_add(offset, N, "binary read")?)
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| ModelError::Invalid("semantic proof binary read is out of range".into()))
}
