use super::*;

pub(super) fn character_entry_pose(
    shard_position: [i32; 3],
    shard_angle: i32,
    location: CharacterEntryLocation0104,
) -> ([i32; 3], i32, Option<PcMoveRequest0104>) {
    match location {
        CharacterEntryLocation0104::Saved => (shard_position, shard_angle, None),
        CharacterEntryLocation0104::TutorialCompletionSectorV => {
            // Do not reuse the shard or login-roster pose here. A compatible
            // server can persist FFOne's temporary tutorial MOVE before the
            // replacement shard connection is ready, contaminating both.
            let position = TUTORIAL_COMPLETION_SECTOR_V_POSITION_0104;
            let angle = TUTORIAL_COMPLETION_SECTOR_V_ANGLE_0104;
            let request = scripted_entry_move_request(position, angle);
            (position, angle, Some(request))
        }
        CharacterEntryLocation0104::Scripted { position, angle } => {
            // The clean tutorial synthesized PC_ENTER_SUCC with its fixed
            // StartPos. In the shared-shard runtime, replay that sequence-owned
            // pose through the ordinary movement protocol instead.
            let request = scripted_entry_move_request(position, angle);
            (position, angle, Some(request))
        }
    }
}
