use super::*;

pub const COMBI_REQUEST_COSTUME_SLOT_OFFSET_0104: usize = 0;

pub const COMBI_REQUEST_STAT_SLOT_OFFSET_0104: usize = 4;

pub const COMBI_REQUEST_CASH_SLOT_1_OFFSET_0104: usize = 8;

pub const COMBI_REQUEST_CASH_SLOT_2_OFFSET_0104: usize = 12;

#[derive(Clone, Debug, PartialEq)]
pub enum CombiReplyError0104 {
    NotAwaiting {
        phase: CombiPhase0104,
    },
    NoPendingAttempt,
    MismatchedSuccessEnvelope {
        reply: CombiSuccessReply0104,
        expected_style_slot: usize,
        expected_stats_slot: usize,
    },
    MismatchedFailureEnvelope {
        reply: CombiFailureReply0104,
        expected_style_slot: usize,
        expected_stats_slot: usize,
    },
    SnapshotChangedWhileAwaiting,
    StalePresentationProjection,
    NegativeAuthoritativeTaros {
        taros_after: i32,
    },
    UnsupportedSuccessFlag {
        success_flag: i32,
    },
    UnexpectedSuccessItem {
        expected: ItemBase0104,
        actual: ItemBase0104,
    },
    UnexpectedFailureItem {
        expected: ItemBase0104,
        actual: ItemBase0104,
    },
    MissingSuccessPresentation,
}

impl fmt::Display for CombiReplyError0104 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "authoritative Combi reply rejected: {self:?}")
    }
}

impl Error for CombiReplyError0104 {}
