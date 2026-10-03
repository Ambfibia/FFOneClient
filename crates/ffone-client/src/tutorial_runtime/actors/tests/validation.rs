use super::*;

pub(super) fn audit_npc_action(
    action: NpcAction,
    origin: &str,
    live: &mut BTreeMap<i32, i32>,
    all_types: &BTreeMap<i32, BTreeSet<i32>>,
    catalog: &NetworkNpcVisualCatalog0104,
    requirements: &mut BTreeSet<(String, &'static str)>,
    blockers: &mut BTreeSet<String>,
) {
    let mut add_spawn = |spawn: NpcSpawn, live: &mut BTreeMap<i32, i32>| {
        live.insert(spawn.id, spawn.npc_type);
        if let Some(clip) = spawn.initial_animation {
            add_animation_requirement(
                spawn.id,
                clip,
                origin,
                live,
                all_types,
                catalog,
                requirements,
                blockers,
            );
        }
    };
    match action {
        NpcAction::Spawn(spawn) => add_spawn(spawn, live),
        NpcAction::SpawnBatch(spawns) => {
            for spawn in spawns {
                add_spawn(*spawn, live);
            }
        }
        NpcAction::Delete(id) => {
            live.remove(&id);
        }
        NpcAction::DeleteInclusive { first, last } => {
            for id in first..=last {
                live.remove(&id);
            }
        }
        NpcAction::Animation { id, clip, .. } => add_animation_requirement(
            id,
            clip,
            origin,
            live,
            all_types,
            catalog,
            requirements,
            blockers,
        ),
        NpcAction::AnimationInclusive {
            first, last, clip, ..
        } => {
            for id in first..=last {
                // Legacy inclusive helpers tolerate empty runtime slots.
                // BasicMove addresses 314..320 while only 314..317 are
                // created anywhere in the recovered tutorial corpus.
                if live.contains_key(&id) || all_types.contains_key(&id) {
                    add_animation_requirement(
                        id,
                        clip,
                        origin,
                        live,
                        all_types,
                        catalog,
                        requirements,
                        blockers,
                    );
                }
            }
        }
        NpcAction::Command {
            id,
            command: NpcCommand::ForceAnimation(clip),
        } => add_animation_requirement(
            id,
            clip,
            origin,
            live,
            all_types,
            catalog,
            requirements,
            blockers,
        ),
        _ => {}
    }
}
