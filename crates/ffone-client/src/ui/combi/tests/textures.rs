use super::*;

pub(super) fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert_eq!(&bytes[0..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&bytes[12..16], b"IHDR");
    (
        u32::from_be_bytes(bytes[16..20].try_into().expect("width")),
        u32::from_be_bytes(bytes[20..24].try_into().expect("height")),
    )
}
