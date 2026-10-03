use super::*;
fn chunk(flags: i32, entries: &[[i32; 5]]) -> Vec<u8> {
    [flags, entries.len() as i32]
        .into_iter()
        .chain(entries.iter().flatten().copied())
        .flat_map(i32::to_le_bytes)
        .collect()
}
#[test]
fn snapshot_replaces_moved_and_removed_placements_atomically() {
    let mut map = NpcMapSnapshot::default();
    assert!(
        map.receive(&chunk(3, &[[1, 100, 10, 20, 30], [2, 100, 40, 50, 60]]))
            .unwrap()
    );
    assert!(!map.receive(&chunk(1, &[[1, 100, 70, 80, 90]])).unwrap());
    assert_eq!(map.entries.as_ref().unwrap().len(), 2);
    assert!(map.receive(&chunk(2, &[[3, 200, 1, 2, 3]])).unwrap());
    let entries = map.entries.unwrap();
    assert!(!entries.contains_key(&2));
    assert_eq!(entries[&1].position, [70, 80, 90]);
    assert_eq!(entries.len(), 2);
}
#[test]
fn malformed_or_duplicate_chunks_do_not_replace_published_map() {
    let mut map = NpcMapSnapshot::default();
    map.receive(&chunk(3, &[[1, 2, 3, 4, 5]])).unwrap();
    assert!(
        map.receive(&chunk(3, &[[2, 2, 0, 0, 0], [2, 2, 1, 1, 1]]))
            .is_err()
    );
    assert!(map.entries.as_ref().unwrap().contains_key(&1));
    assert!(map.receive(&chunk(2, &[])).is_err());
    assert!(NpcMapChunk::decode(&chunk(3, &[])[..7]).is_err());
    assert!(NpcMapChunk::decode(&chunk(4, &[])).is_err());
}
