use super::*;

pub const NANO_TUNE_REQUEST_PACKET_ID: u32 = 318_767_120;

pub const NANO_TUNE_SUCCESS_PACKET_ID: u32 = 822_083_625;

pub const NANO_TUNE_FAILURE_PACKET_ID: u32 = 822_083_669;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NanoTuneWireIntent {
    pub request_token: u64,
    pub packet_id: u32,
    pub payload_size: usize,
    pub body: NanoTuneRequest,
}
