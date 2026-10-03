use super::*;

pub(super) fn character_select_rejected_0104(frame: &DecodedFrame) -> NetError {
    match LsCharSelectFailure0104::decode(&frame.payload) {
        Ok(failure) => NetError::CharacterSelectRejected {
            error_code: failure.error_code,
        },
        Err(error) => NetError::Payload(error),
    }
}
