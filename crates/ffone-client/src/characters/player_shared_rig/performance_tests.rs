use super::*;
use bevy::ecs::system::SystemState;

#[test]
fn query_index_preserves_source_order_nested_ownership_and_rebuilds_after_removal() {
    let mut world = World::new();
    let root = world.spawn_empty().id();
    let nested = world.spawn(ChildOf(root)).id();
    let first = world.spawn(ChildOf(nested)).id();
    let second = world.spawn(ChildOf(root)).id();
    let outside = world.spawn_empty().id();
    let mut index = RigQueryIndex::default();
    let mut scratch = RigDescendantScratch::default();
    for order in [
        vec![outside, second, first, nested],
        vec![first, nested, outside, second],
    ] {
        index.rebuild(order.iter().copied());
        let mut state: SystemState<(Query<&Children>, Query<&ChildOf>)> =
            SystemState::new(&mut world);
        let (children, parents) = state.get(&world).unwrap();
        for owner in [root, nested] {
            scratch.collect(owner, &children);
            let expected: Vec<_> = order
                .iter()
                .copied()
                .filter(|entity| is_descendant_of(*entity, owner, &parents))
                .collect();
            let actual: Vec<_> = scratch
                .ordered_matches(&index)
                .iter()
                .map(|rank| order[*rank])
                .collect();
            assert_eq!(actual, expected);
        }
    }
    world.entity_mut(first).insert(ChildOf(outside));
    world.despawn(second);
    let replacement = world.spawn(ChildOf(root)).id();
    let order = [first, replacement, nested];
    index.rebuild(order.iter().copied());
    let mut state: SystemState<Query<&Children>> = SystemState::new(&mut world);
    scratch.collect(root, &state.get(&world).unwrap());
    let actual: Vec<_> = scratch
        .ordered_matches(&index)
        .iter()
        .map(|rank| order[*rank])
        .collect();
    assert_eq!(actual, vec![replacement, nested]);
    assert!(index.rank(second).is_none());
    index.rebuild(std::iter::empty());
    assert!(scratch.ordered_matches(&index).is_empty());
    index.epoch = u64::MAX;
    index.rebuild([replacement].into_iter());
    assert_eq!(index.rank(replacement), Some(0));
    assert!(index.rank(first).is_none());
}

#[test]
#[ignore = "opt-in CPU benchmark; assert order/count parity, not timing thresholds"]
fn rig_query_index_benchmark() {
    let mut world = World::new();
    let mut roots = Vec::new();
    for _ in 0..52 {
        let root = world.spawn_empty().id();
        roots.push(root);
        for _ in 0..80 {
            world.spawn(ChildOf(root));
        }
        for _ in 0..6 {
            world.spawn((
                ChildOf(root),
                LegacyMaterialRendererOrder { renderer_index: 0 },
            ));
        }
    }
    for _ in 0..6000 {
        world.spawn(LegacyMaterialRendererOrder { renderer_index: 0 });
    }
    let mut state: SystemState<(
        Query<&Children>,
        Query<Entity, With<LegacyMaterialRendererOrder>>,
    )> = SystemState::new(&mut world);
    let (children, sources) = state.get(&world).unwrap();
    let mut scratch = RigDescendantScratch::default();
    let mut index = RigQueryIndex::default();
    let mut counts = Vec::new();
    for indexed in [false, true] {
        let start = std::time::Instant::now();
        let mut count = 0;
        for _ in 0..200 {
            if indexed {
                index.rebuild(sources.iter());
            }
            for root in &roots {
                scratch.collect_for_query(*root, &children, indexed);
                count += if indexed {
                    scratch.ordered_matches(&index).len()
                } else {
                    sources
                        .iter()
                        .filter(|entity| scratch.contains(*entity))
                        .count()
                };
            }
        }
        counts.push(count);
        println!(
            "rig index={indexed} frames=200 roots=52 sources=6312 elapsed_ms={} count={}",
            start.elapsed().as_secs_f64() * 1000.0,
            std::hint::black_box(count)
        );
    }
    assert_eq!(counts, [62400, 62400]);
}
