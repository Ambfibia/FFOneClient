use super::*;

pub const RACE_START_ERROR_8_MESSAGE_ID: i32 = 150;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceModeOpenError {
    MissingStartEcomNpc,
    MissingEndEcomNpc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceModeReplyError {
    NoPendingRequest,
    StaleRequest {
        expected: u64,
        received: u64,
    },
    WrongReplyKind {
        expected: RaceRequestKind,
        received: RaceRequestKind,
    },
}
