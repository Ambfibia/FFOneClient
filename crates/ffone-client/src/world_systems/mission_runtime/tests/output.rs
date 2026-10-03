use super::*;

pub(super) fn write_i32_at(load: &mut PcLoadData0104, offset: usize, value: i32) {
    load.as_bytes_mut()[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_i64_at(load: &mut PcLoadData0104, offset: usize, value: i64) {
    load.as_bytes_mut()[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

pub(super) fn write_running_quest(
    load: &mut PcLoadData0104,
    slot: usize,
    running: ffone_protocol::RunningQuest0104,
) {
    let offset =
        PcLoadData0104::RUNNING_QUESTS_OFFSET + slot * ffone_protocol::RunningQuest0104::SIZE;
    load.as_bytes_mut()[offset..offset + ffone_protocol::RunningQuest0104::SIZE]
        .copy_from_slice(&running.encode());
}
