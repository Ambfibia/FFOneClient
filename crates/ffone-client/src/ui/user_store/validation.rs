use super::*;

pub(super) fn require_exact(
    packet_id: u32,
    payload: &[u8],
    expected: usize,
) -> Result<(), UserStoreCodecError0104> {
    if payload.len() == expected {
        Ok(())
    } else {
        Err(UserStoreCodecError0104::ExactSize {
            packet_id,
            expected,
            actual: payload.len(),
        })
    }
}

pub(super) fn user_store_error_text(code: i32) -> LocalizedText {
    LocalizedText::new("ui.user_store.error", "STORE ERROR {code}")
        .with_arg("code", code.to_string())
}
