use super::*;

pub(super) fn resolve_requirement_type(
    id: i32,
    live: &BTreeMap<i32, i32>,
    all_types: &BTreeMap<i32, BTreeSet<i32>>,
) -> Result<i32, String> {
    if let Some(npc_type) = live.get(&id) {
        return Ok(*npc_type);
    }
    let candidates = all_types.get(&id).cloned().unwrap_or_default();
    if candidates.len() == 1 {
        return Ok(*candidates.first().unwrap());
    }
    Err(format!(
        "actor id {id} has unresolved npc types {candidates:?}"
    ))
}
