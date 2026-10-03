use crate::{assets::AssetLocator, tutorial_mission_content::TutorialMissionContent};
use std::{collections::BTreeSet, hint::black_box, path::Path, time::Instant};

fn content() -> TutorialMissionContent {
    TutorialMissionContent::open(
        &AssetLocator::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"))
            .unwrap(),
    )
    .unwrap()
}

#[test]
fn indexed_npc_tasks_match_the_complete_production_scan_in_order() {
    let content = content();
    let types: BTreeSet<_> = content
        .missions()
        .flat_map(|m| {
            [
                m.provenance.start_npc_type,
                m.provenance.terminator_npc_type,
            ]
        })
        .chain([-1, i32::MAX])
        .collect();
    for npc_type in types {
        let expected: Vec<_> = content
            .missions()
            .filter(|m| {
                m.provenance.start_npc_type == npc_type
                    || m.provenance.terminator_npc_type == npc_type
            })
            .map(|m| m.provenance.task_id)
            .collect();
        let actual: Vec<_> = content
            .missions_for_npc(npc_type)
            .map(|m| m.provenance.task_id)
            .collect();
        assert_eq!(actual, expected, "NPC {npc_type}");
    }
}

#[test]
#[ignore = "isolated CPU timing, run with --ignored --nocapture"]
fn production_npc_mission_lookup_benchmark() {
    let content = content();
    let types: Vec<_> = content
        .missions()
        .flat_map(|m| {
            [
                m.provenance.start_npc_type,
                m.provenance.terminator_npc_type,
            ]
        })
        .filter(|id| *id > 0)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .take(100)
        .collect();
    for indexed in [false, true] {
        let start = Instant::now();
        for _ in 0..1000 {
            for &npc in &types {
                if indexed {
                    for m in content.missions_for_npc(black_box(npc)) {
                        black_box(m.provenance.task_id);
                    }
                } else {
                    for m in content.missions() {
                        if m.provenance.start_npc_type == black_box(npc)
                            || m.provenance.terminator_npc_type == black_box(npc)
                        {
                            black_box(m.provenance.task_id);
                        }
                    }
                }
            }
        }
        println!(
            "npc_index={indexed} npcs={} missions={} ms_per_frame={:.4}",
            types.len(),
            content.missions().len(),
            start.elapsed().as_secs_f64()
        );
    }
}
