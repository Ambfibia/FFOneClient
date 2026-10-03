use super::*;
#[test]
fn last_tab_roundtrips_and_invalid_preferences_are_ignored() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("assets/game");
    fs::create_dir_all(&root).unwrap();
    assert_eq!(read_tab(&root), None);
    for tab in ["xdt", "strings", "npc", "nano", "equipment"] {
        write_tab(&root, tab).unwrap();
        assert_eq!(read_tab(&root).as_deref(), Some(tab));
    }
    fs::write(preferences_path(&root), b"broken").unwrap();
    assert_eq!(read_tab(&root), None);
    write_tab(&root, "unknown").unwrap();
    assert_eq!(read_tab(&root), None);
}
