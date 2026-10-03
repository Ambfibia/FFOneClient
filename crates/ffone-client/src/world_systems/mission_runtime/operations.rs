use super::*;

/// Resolve the server entity, never the table type, for the task's escort.
/// Both explicit starts and automatic chain transitions use this contract.
pub fn mission_escort_npc_id(
    task_id: i32,
    content: &TutorialMissionContent,
    npcs: impl IntoIterator<Item = (i32, i32)>,
) -> i32 {
    let Ok(task) = content.mission(task_id) else {
        return 0;
    };
    if task.provenance.escort_def_npc_type <= 0 {
        return 0;
    }
    npcs.into_iter()
        .find(|(npc_type, _)| *npc_type == task.provenance.escort_def_npc_type)
        .map_or(0, |(_, npc_id)| npc_id)
}

pub(super) fn repeat_flag_position(repeat_flag: i32) -> Option<(usize, u32)> {
    if repeat_flag <= 0 {
        return None;
    }
    let zero_based = repeat_flag.checked_sub(1)?;
    Some((
        usize::try_from(zero_based / 8).ok()?,
        (zero_based % 8) as u32,
    ))
}

pub(super) fn distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    let delta = [left[0] - right[0], left[1] - right[1], left[2] - right[2]];
    delta
        .into_iter()
        .map(|component| component * component)
        .sum::<f32>()
        .sqrt()
}

#[must_use]
pub fn quest_item_count(inventory: &[ItemBase0104], item_id: i32) -> i32 {
    let Ok(item_id) = i16::try_from(item_id) else {
        return 0;
    };
    inventory
        .iter()
        .find(|item| item.item_id == item_id)
        .map_or(0, |item| item.option)
}
