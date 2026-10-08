use super::*;

fn edit_timer(e: &mut XdtEditor, timer: i64) {
    e.begin(Focus::Cell("m_iCSUCheckTimer".into()));
    e.edit = timer.to_string();
    assert!(e.apply());
}

#[test]
fn saving_restores_table_and_translations_without_touching_game_or_server() {
    let (dir, mut e) = fixture();
    let root = dir.path().join("assets/game");
    let server = dir.path().join("remote/tabledata");
    fs::create_dir_all(&server).unwrap();
    fs::write(server.join("xdt.json"), e.document.to_string()).unwrap();
    e.select_server_folder(server.parent().unwrap()).unwrap();
    let paths = [e.path.clone(), server.join("xdt.json"), root.join("localization/en.json"), root.join("localization/ru.json")];
    let before = paths.each_ref().map(|path| fs::read(path).unwrap());
    e.start_quick("m_iHCurrentObjective").unwrap();
    e.edit = "Найти маяк {count}".into();
    assert!(e.apply());
    e.begin(Focus::Locale(0));
    e.edit = "Find beacon {count}".into();
    e.create_draft().unwrap();
    edit_timer(&mut e, 75);
    e.save().unwrap();
    assert!(!e.dirty());
    assert!(e.unpublished());
    assert!(!e.reload_models);
    for (path, bytes) in paths.iter().zip(&before) { assert_eq!(&fs::read(path).unwrap(), bytes); }
    let mut loaded = XdtEditor::open(root).with_selection(Some("m_pMissionTable/m_pMissionData"), Some(0), "");
    assert_eq!(loaded.document, e.document);
    assert_eq!(loaded.workspace.locale_drafts, e.workspace.locale_drafts);
    assert!(!loaded.dirty());
    assert!(loaded.unpublished());
    assert!(loaded.status.contains("restored"));
    loaded.rewrite().unwrap();
    assert_eq!(read(&loaded.path).unwrap(), loaded.document);
    assert_eq!(fs::read(&paths[0]).unwrap(), fs::read(&paths[1]).unwrap());
    let ru: Value = serde_json::from_slice(&fs::read(&paths[3]).unwrap()).unwrap();
    assert_eq!(ru["entries"]["content.mission.task.10.objective"], "Найти маяк {count}");
    assert!(!loaded.unpublished());
    assert!(!loaded.dirty());
    assert!(loaded.reload_models);
    let reopened = XdtEditor::open(dir.path().join("assets/game"));
    assert!(!reopened.unpublished());
    assert_eq!(reopened.document, loaded.document);
}

#[test]
fn publication_error_keeps_invalid_work_saved_for_correction() {
    let (dir, mut e) = fixture();
    let before = fs::read(&e.path).unwrap();
    let pointer = e.tables[e.table].pointer.clone();
    e.document.pointer_mut(&pointer).unwrap()[0]["m_iSTItemDropRate"] = json!([5, 0, 0]);
    e.save().unwrap();
    assert!(e.rewrite().unwrap_err().contains("mission_drop_item"));
    assert_eq!(fs::read(&e.path).unwrap(), before);
    let loaded = XdtEditor::open(dir.path().join("assets/game"));
    assert_eq!(loaded.document, e.document);
    assert!(loaded.unpublished());
    assert!(!loaded.dirty());
}

#[test]
fn closing_saves_the_focused_field_without_publishing() {
    let (dir, mut e) = fixture();
    let before = fs::read(&e.path).unwrap();
    e.begin(Focus::Cell("m_iCSUCheckTimer".into()));
    e.edit = "85".into();
    assert!(e.save_on_close());
    assert_eq!(fs::read(&e.path).unwrap(), before);
    let loaded = XdtEditor::open(dir.path().join("assets/game"))
        .with_selection(Some("m_pMissionTable/m_pMissionData"), Some(0), "");
    assert_eq!(loaded.rows()[0]["m_iCSUCheckTimer"], 85);
}

#[test]
fn restart_preserves_client_and_server_conflict_baselines() {
    let (dir, mut e) = fixture();
    let root = dir.path().join("assets/game");
    let server = dir.path().join("remote/tabledata");
    fs::create_dir_all(&server).unwrap();
    fs::write(server.join("xdt.json"), e.document.to_string()).unwrap();
    e.select_server_folder(server.parent().unwrap()).unwrap();
    edit_timer(&mut e, 90);
    e.save().unwrap();
    let mut external = e.base.clone();
    let pointer = e.tables[e.table].pointer.clone();
    external.pointer_mut(&pointer).unwrap()[0]["m_iCSUCheckTimer"] = json!(95);
    fs::write(&e.path, external.to_string()).unwrap();
    let mut loaded = XdtEditor::open(root.clone());
    assert!(loaded.rewrite().unwrap_err().contains("conflict"));
    assert_eq!(read(&e.path).unwrap(), external);
    fs::write(&e.path, e.base.to_string()).unwrap();
    fs::write(server.join("xdt.json"), "{\"external\":true}").unwrap();
    let mut loaded = XdtEditor::open(root);
    assert!(loaded.rewrite().unwrap_err().contains("Server TableData changed externally"));
    assert_eq!(read(&e.path).unwrap(), e.base);
    assert_eq!(fs::read(server.join("xdt.json")).unwrap(), b"{\"external\":true}");
    assert_eq!(loaded.document, e.document);
}

#[test]
fn failed_work_save_keeps_unsaved_state_and_game_unchanged() {
    let (_dir, mut e) = fixture();
    let before = fs::read(&e.path).unwrap();
    edit_timer(&mut e, 45);
    fs::create_dir_all(e.work_path().unwrap()).unwrap();
    assert!(e.save().is_err());
    assert!(e.dirty());
    assert!(!e.save_on_close());
    assert_eq!(fs::read(&e.path).unwrap(), before);
}

#[test]
fn undo_and_reload_track_saved_work_and_discard_pending_locales() {
    let (dir, mut e) = fixture();
    edit_timer(&mut e, 45);
    e.workspace.locale_drafts.insert("pending".into(), [Some("Draft".into()), Some("Черновик".into())]);
    e.save().unwrap();
    assert!(!e.dirty());
    e.undo(false);
    assert!(e.dirty());
    e.undo(true);
    assert!(!e.dirty());
    e.reload_game().unwrap();
    assert!(!e.dirty());
    assert!(!e.unpublished());
    assert!(e.workspace.locale_drafts.is_empty());
    let loaded = XdtEditor::open(dir.path().join("assets/game"));
    assert!(!loaded.unpublished());
    assert!(loaded.workspace.locale_drafts.is_empty());
}

fn with_existing_destination(dir: &tempfile::TempDir, e: &mut XdtEditor) -> PathBuf {
    let npc_table = e.tables.iter().find(|table| table.label.ends_with("/m_pNpcTable/m_pNpcData")).unwrap();
    let npcs = e.document.pointer_mut(&npc_table.pointer).unwrap().as_array_mut().unwrap();
    let mut npc = npcs[0].clone();
    npc["m_iNpcNumber"] = json!(3492);
    npcs.push(npc);
    e.base = e.document.clone();
    fs::write(&e.path, e.document.to_string()).unwrap();
    e.reindex();
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/data/missions/client-npc-waypoints.json");
    let mut catalog: Value = serde_json::from_slice(&fs::read(source).unwrap()).unwrap();
    let rows = catalog["rows"].as_array_mut().unwrap();
    rows.truncate(ffone_client::world_mission_indicators::CLIENT_NPC_WAYPOINT_CATALOG_ROW_COUNT);
    rows.push(json!({"npcType":3492,"clientPosition":[3948.74,-56.9,3737.58]}));
    let path = dir.path().join("assets/game/data/missions/client-npc-waypoints.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, catalog.to_string()).unwrap();
    let server = dir.path().join("remote/tabledata");
    fs::create_dir_all(&server).unwrap();
    fs::write(server.join("xdt.json"), e.document.to_string()).unwrap();
    e.select_server_folder(&server).unwrap();
    path
}

#[test]
fn existing_npc_3492_coordinates_do_not_block_unrelated_publication() {
    for has_placements_file in [false, true] {
        let (dir, mut e) = fixture();
        let waypoint = with_existing_destination(&dir, &mut e);
        if has_placements_file {
            fs::write(dir.path().join("remote/tabledata/NPCs.json"), r#"{"NPCs":{}}"#).unwrap();
        }
        let before = fs::read(&waypoint).unwrap();
        // Both an unused destination and an unchanged mission reference must survive.
        let pointer = e.tables[e.table].pointer.clone();
        e.document.pointer_mut(&pointer).unwrap()[0]["m_iHNPCID"] = json!(3492);
        e.base = e.document.clone();
        fs::write(&e.path, e.base.to_string()).unwrap();
        edit_timer(&mut e, 75);
        assert!(e.prepare_waypoints(&e.document).unwrap().is_none());
        e.rewrite().unwrap();
        assert_eq!(fs::read(&waypoint).unwrap(), before);
        assert_eq!(read(&e.path).unwrap(), e.document);
        assert_eq!(e.rows()[0]["m_iCSUCheckTimer"], 75);
        assert!(!e.unpublished());
    }
}

#[test]
fn newly_assigned_npc_3492_still_requires_server_placement_but_work_can_be_saved() {
    let (dir, mut e) = fixture();
    let waypoint = with_existing_destination(&dir, &mut e);
    let before = fs::read(&e.path).unwrap();
    let before_waypoint = fs::read(&waypoint).unwrap();
    let pointer = e.tables[e.table].pointer.clone();
    e.document.pointer_mut(&pointer).unwrap()[0]["m_iHNPCID"] = json!(3492);
    e.save().unwrap();
    let error = e.rewrite().unwrap_err();
    assert!(error.contains("Mission NPC has no world placement: 3492"), "{error}");
    assert!(error.contains("mission ID: 1, stage ID: 10"));
    assert_eq!(fs::read(&e.path).unwrap(), before);
    assert_eq!(fs::read(&waypoint).unwrap(), before_waypoint);
    let restored = XdtEditor::open(dir.path().join("assets/game"));
    assert_eq!(restored.document, e.document);
    assert!(!restored.dirty());
    assert!(restored.unpublished());
}
