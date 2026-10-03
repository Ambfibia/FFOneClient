use super::*;

pub(super) const REPORT_SCHEMA: &str = "ffone.avatar-equipment-gpu-audit.v1";

pub(super) const MIN_CAPTURE_VISIBLE_PIXELS: u64 = 1_024;

pub(super) const MAX_TRANSIENT_RETRIES: u8 = 1;

pub(super) const ITEM_TIMEOUT: Duration = Duration::from_secs(20);

pub(super) const REPORT_CHECKPOINT_INTERVAL: usize = 25;
