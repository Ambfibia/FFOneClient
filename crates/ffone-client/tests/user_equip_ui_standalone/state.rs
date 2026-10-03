use super::*;

pub(super) fn inventory_keys_in_source(source: &str) -> BTreeSet<&str> {
    let prefix = "\"ui.inventory.";
    let mut keys = BTreeSet::new();
    let mut remainder = source;
    while let Some(start) = remainder.find(prefix) {
        remainder = &remainder[start + 1..];
        let end = remainder
            .find('"')
            .expect("quoted ui.inventory key must be terminated");
        let key = &remainder[..end];
        // Rust format strings in regression assertions are not static UI keys.
        if !key.contains(['{', '}']) {
            keys.insert(key);
        }
        remainder = &remainder[end + 1..];
    }
    keys
}
