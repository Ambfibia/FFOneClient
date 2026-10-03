use super::*;

pub(super) const EMAIL_ITEM_BASE_SIZE: usize = 12;

pub(super) const EMAIL_OUTGOING_ITEM_SIZE: usize = 16;

pub(super) const EMAIL_SUMMARY_SIZE: usize = 204;

pub(super) const EMAIL_SUMMARY_FIRST_OFFSET: usize = 4;

pub(super) const EMAIL_SUBJECT_UNITS: usize = 32;

pub(super) const EMAIL_CONTENT_UNITS: usize = 512;

pub(super) const EMAIL_FIRST_NAME_UNITS: usize = 9;

pub(super) const EMAIL_LAST_NAME_UNITS: usize = 17;

/// Clean buddy storage is a fixed fifty-slot array. Projection preserves slot
/// order and removes only empty/blocked entries before this boundary.
pub const EMAIL_BUDDY_MAX_COUNT_0104: usize = 50;

const _: [(); EMAIL_INVENTORY_SLOT_COUNT] = [(); INVENTORY_SLOT_COUNT_0104];

const _: [(); EMAIL_BUDDY_MAX_COUNT_0104] = [(); BUDDY_MAX_SLOTS];
