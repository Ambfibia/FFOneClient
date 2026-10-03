use super::*;

pub(super) fn refresh_editable_metadata(root: &Path, graph: &mut AssetGraph) -> Result<(), String> {
    for entry in &mut graph.files {
        if !is_editable_asset(&entry.path) {
            continue;
        }
        let metadata = regular_metadata(&root.join(native_path(&entry.path)))?;
        entry.bytes = metadata.len();
        entry.modified_nanos = modified_nanos(&metadata)?;
        entry.blake3 = None;
    }
    graph.payload_bytes = graph.files.iter().try_fold(0_u64, |total, file| {
        total
            .checked_add(file.bytes)
            .ok_or_else(|| "asset payload byte count overflow".to_owned())
    })?;
    graph.groups.clear();
    for file in &graph.files {
        graph
            .groups
            .entry(file.group.clone())
            .or_default()
            .push(file.clone());
    }
    Ok(())
}
