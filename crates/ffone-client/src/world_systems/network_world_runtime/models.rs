use super::*;

pub(super) fn select_registry_model(candidates: &[RegistryModel0104]) -> Option<&RegistryModel0104> {
    match candidates {
        [candidate] => Some(candidate),
        candidates => {
            let npc_candidates = candidates
                .iter()
                .filter(|candidate| candidate.category == "npc")
                .collect::<Vec<_>>();
            let [candidate] = npc_candidates.as_slice() else {
                return None;
            };
            Some(*candidate)
        }
    }
}
