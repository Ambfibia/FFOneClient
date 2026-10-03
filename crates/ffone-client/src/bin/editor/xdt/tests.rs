use super::*;
use serde_json::json;

#[test]
fn mission_graph_distinguishes_mission_requirements_from_task_transitions_and_rejects_new_cycles() {
    let before=vec![json!({"m_iHTaskID":101,"m_iHMissionID":1,"m_iCSTReqMission":[0,0],"m_iSUOutgoingTask":102}),
        json!({"m_iHTaskID":102,"m_iHMissionID":1,"m_iCSTReqMission":[0,0]}),
        json!({"m_iHTaskID":201,"m_iHMissionID":2,"m_iCSTReqMission":[1,0]})];
    let missions=mission_graph::edges(&before,false);assert_eq!((missions[0].from,missions[0].to),(1,2));
    let stages=mission_graph::edges(&before,true);assert_eq!((stages[0].from,stages[0].to),(101,102));
    let mut next=before.clone();next[0]["m_iCSTReqMission"]=json!([2,0]);assert!(mission_graph::validate(&before,&next).is_err());
    assert!(mission_graph::validate(&next,&next).is_ok()); // Existing accepted cycles remain inspectable.
    next[0]["m_iCSTReqMission"]=json!([1,0]);assert!(mission_graph::validate(&before,&next).is_err());
    next[0]["m_iCSTReqMission"]=json!([0,0]);assert!(mission_graph::validate(&before,&next).is_ok());
    assert_eq!(relations::rule("native/m_pMissionTable/m_pMissionData",mission_graph::REQUIRE).unwrap().1,Some("m_iHMissionID"));
}
#[test]
fn graph_prerequisite_picker_and_removal_preserve_slots_and_undo() {
    let (_dir,mut e)=production_editor("m_pMissionTable/m_pMissionData");
    let row=e.row.unwrap();let before=e.document.clone();
    let group=e.rows()[row]["m_iHMissionID"].clone();
    let target=e.rows().iter().position(|r| {
        if r["m_iHMissionID"]==group || !r["m_iHMissionID"].as_i64().is_some_and(|id|id>0) {return false;}
        let mut next=e.rows().to_vec();next[row][mission_graph::REQUIRE][0]=r["m_iHMissionID"].clone();
        mission_graph::validate(e.rows(),&next).is_ok()
    }).unwrap();
    e.graph_add(mission_graph::REQUIRE).unwrap();let slot=e.array_slot.unwrap();
    let candidates=e.reference_candidates(e.table,mission_graph::REQUIRE);
    let ids:BTreeSet<_>=candidates.iter().map(|r|e.rows()[*r]["m_iHMissionID"].as_i64().unwrap()).collect();
    assert_eq!(ids.len(),candidates.len());assert!(!ids.contains(&group.as_i64().unwrap()));
    e.pick_reference(mission_graph::REQUIRE.into(),e.table,target).unwrap();
    assert_eq!(e.rows()[row][mission_graph::REQUIRE][slot],e.rows()[target]["m_iHMissionID"]);
    assert!(e.reference_candidates(e.table,mission_graph::REQUIRE).iter().all(|r|e.rows()[*r]["m_iHMissionID"]!=e.rows()[target]["m_iHMissionID"]));
    e.graph_remove(row,mission_graph::REQUIRE,slot).unwrap();assert_eq!(e.document,before);
    e.undo(false);assert_ne!(e.document,before);e.undo(false);assert_eq!(e.document,before);
}
#[test]
fn hnpc_assignment_preserves_owner_identity_and_cancelled_creation_is_lossless() {
    let (_dir,mut e)=production_editor("m_pNpcTable/m_pNpcData");let before=e.document.clone();let row=e.row.unwrap();
    e.apply_hnpc(123).unwrap();assert_eq!(e.rows()[row]["m_iHNpcNum"],123);assert_eq!(e.rows()[row]["m_iHNpc"],1);
    assert_eq!(e.rows()[row]["m_iNpcNumber"],before.pointer(&e.tables[e.table].pointer).unwrap()[row]["m_iNpcNumber"]);
    e.undo(false);assert_eq!(e.document,before);
    e.start_draft(false).unwrap();e.apply_hnpc(124).unwrap();assert_eq!(e.draft.as_ref().unwrap().value["m_iHNpcNum"],124);
    assert_eq!(e.document,before);e.draft=None;assert_eq!(e.document,before);
}

#[test]
fn authoring_blocks_are_lossless_and_optional_zero_lists_stay_available() {
    let table = "native/m_pMissionTable/m_pMissionData";
    let value = json!({"m_iHTaskID":7,"m_iHMissionID":9,"m_iHMissionName":0,
        "m_iHCurrentObjective":1,"m_iCSTReqMission":[0,0,0],"m_iCSUEnemyID":[0,17,0],
        "m_iFOutgoingTask":0,"future_field":{"preserve":[1,2]}});
    let fields: Vec<_> = value.as_object().unwrap().keys().cloned().collect();
    let basic = authoring::blocks(table, &fields, &value, false);
    assert!(
        !basic
            .iter()
            .find(|b| b.key == "conditions")
            .unwrap()
            .configured
    );
    assert!(
        basic
            .iter()
            .find(|b| b.key == "objective")
            .unwrap()
            .configured
    );
    assert!(
        basic
            .iter()
            .find(|b| b.key == "failure")
            .unwrap()
            .fields
            .contains(&"m_iFOutgoingTask".into())
    );
    assert!(
        !basic
            .iter()
            .any(|b| b.fields.contains(&"future_field".into()))
    );
    let all: BTreeSet<_> = authoring::blocks(table, &fields, &value, true)
        .into_iter()
        .flat_map(|b| b.fields)
        .collect();
    assert_eq!(all, fields.into_iter().collect());
    assert_eq!(value["future_field"], json!({"preserve":[1,2]}));
}

fn shared_text_editor() -> (tempfile::TempDir, XdtEditor) {
    let (dir, mut e) = editor(json!([]));
    e.document["tables"][0]["value"] = json!({"m_pNpcTable": {
        "m_pNpcData":[{"m_iNpcNumber":17,"m_iNpcName":0,"future":true},
            {"m_iNpcNumber":18,"m_iNpcName":0}],
        "m_pNpcStringData":[{"m_strName":"Dexter","m_strComment":"Приветствие",
            "future":{"payload":[1,2]}}]
    }});
    e.base = e.document.clone();
    fs::write(&e.path, e.document.to_string()).unwrap();
    e.discover();
    e.select_table(
        e.tables
            .iter()
            .position(|t| t.label.ends_with("/m_pNpcData"))
            .unwrap(),
    );
    e.row = Some(0);
    e.rebuild_links();
    (dir, e)
}

#[test]
fn new_mission_allocates_group_identity_and_new_stage_keeps_existing_group_and_name() {
    let (_dir, mut e) = production_editor("m_pMissionTable/m_pMissionData");
    let before = e.document.clone();
    let group = e.rows()[1]["m_iHMissionID"].clone();
    let name = e.rows()[1]["m_iHMissionName"].clone();
    let max_group = e
        .rows()
        .iter()
        .filter_map(|r| r["m_iHMissionID"].as_i64())
        .max()
        .unwrap();
    e.mission_filter = group.as_i64();
    e.refresh();
    assert!(
        e.filtered
            .iter()
            .all(|r| e.rows()[*r]["m_iHMissionID"] == group)
    );
    e.start_draft(true).unwrap();
    assert_eq!(e.draft.as_ref().unwrap().value["m_iHMissionID"], group);
    assert_eq!(e.draft.as_ref().unwrap().value["m_iHMissionName"], name);
    assert!(e.draft.as_ref().unwrap().name.is_none());
    e.draft = None;
    e.start_draft(false).unwrap();
    assert_eq!(
        e.draft.as_ref().unwrap().value["m_iHMissionID"],
        max_group + 1
    );
    e.begin(Focus::NewName);
    e.replace("Новая миссия");
    assert!(e.apply());
    e.draft.as_mut().unwrap().value[mission_graph::REQUIRE][0]=Value::from(max_group+1);
    assert!(e.create_draft().is_err());assert_eq!(e.document,before);
    e.draft.as_mut().unwrap().value[mission_graph::REQUIRE][0]=Value::from(0);
    e.create_draft().unwrap();
    assert_eq!(e.mission_filter, Some(max_group + 1));
    assert_eq!(e.filtered, vec![e.rows().len() - 1]);
    assert_eq!(e.record_name(e.table, e.row.unwrap()), "Новая миссия");
    e.undo(false);
    assert_eq!(e.document, before);
    assert_eq!(e.mission_filter, None);
}

#[test]
fn private_text_copy_preserves_metadata_and_other_owners_and_undoes_atomically() {
    let (_dir, mut e) = shared_text_editor();
    let owner_table = e.table;
    let original = e.document.clone();
    let target = e.text_target("m_iNpcName").unwrap();
    assert_eq!(target.users, 2);
    assert!(e.copy_text("missing").is_err());
    assert_eq!(e.document, original);
    e.copy_text("m_iNpcName").unwrap();
    assert_eq!(e.rows().len(), 2);
    assert_eq!(e.rows()[0], e.rows()[1]);
    assert_eq!(e.rows()[1]["future"], json!({"payload":[1,2]}));
    assert_eq!(e.undo.len(), 1);
    assert_eq!(
        e.document.pointer(&e.tables[owner_table].pointer).unwrap()[0]["m_iNpcName"],
        1
    );
    assert_eq!(
        e.document.pointer(&e.tables[owner_table].pointer).unwrap()[1]["m_iNpcName"],
        0
    );
    e.focus = None;
    e.undo(false);
    assert_eq!(e.document, original);
    e.undo(true);
    e.save().unwrap();
    assert_eq!(read(&e.path).unwrap(), e.document);
    e.select_table(owner_table);
    e.row = Some(0);
    e.edit_shared_text("m_iNpcName").unwrap();
    e.replace("Новая версия");
    assert!(e.apply());
    assert_eq!(e.rows()[0]["m_strName"], "Dexter");
    assert_eq!(e.rows()[1]["m_strName"], "Новая версия");
}

#[test]
fn shared_text_edit_keeps_links_and_changes_all_owners_without_extra_rows() {
    let (_dir, mut e) = shared_text_editor();
    let owner = e.table;
    e.edit_shared_text("m_iNpcName").unwrap();
    e.replace("Общее имя");
    assert!(e.apply());
    assert_eq!(e.rows().len(), 1);
    assert_eq!(e.record_name(owner, 0), "Общее имя");
    assert_eq!(e.record_name(owner, 1), "Общее имя");
}

#[test]
fn list_element_edit_and_reference_pick_preserve_shape_and_unselected_slots() {
    let (_dir, mut e) = editor(json!([]));
    e.document["tables"][0]["value"] = json!({
        "m_pMissionTable":{"m_pMissionData":[{"m_iHTaskID":8,"m_iCSUEnemyID":[0,17,0],"counts":[1,2,3]}]},
        "m_pNpcTable":{"m_pNpcData":[{"m_iNpcNumber":17},{"m_iNpcNumber":18}]}
    });
    e.discover();
    e.select_table(
        e.tables
            .iter()
            .position(|t| t.label.ends_with("/m_pMissionData"))
            .unwrap(),
    );
    e.row = Some(0);
    let table = e.table;
    e.begin_element("counts".into(), 1).unwrap();
    e.replace("bad");
    assert!(!e.apply());
    assert_eq!(e.rows()[0]["counts"], json!([1, 2, 3]));
    e.edit = "12".into();
    assert!(e.apply());
    assert_eq!(e.rows()[0]["counts"], json!([1, 12, 3]));
    e.undo(false);
    assert_eq!(e.rows()[0]["counts"], json!([1, 2, 3]));
    e.begin_element("m_iCSUEnemyID".into(), 1).unwrap();
    let (target, _) = e.reference_target("m_iCSUEnemyID").unwrap();
    e.begin(Focus::ReferenceSearch);
    assert_eq!(e.array_slot, Some(1));
    e.pick_reference("m_iCSUEnemyID".into(), target, 1).unwrap();
    assert_eq!(e.rows()[0]["m_iCSUEnemyID"], json!([0, 18, 0]));
    e.begin_element("m_iCSUEnemyID".into(), 1).unwrap();
    e.clear_reference("m_iCSUEnemyID").unwrap();
    assert_eq!(e.rows()[0]["m_iCSUEnemyID"], json!([0, 0, 0]));
    e.select_table(table);
    e.row = Some(0);
    e.start_draft(false).unwrap();
    e.draft.as_mut().unwrap().required.insert("counts".into());
    assert!(
        e.authoring_fields(&e.draft.as_ref().unwrap().value)
            .contains(&"counts".into())
    );
    e.begin_element("counts".into(), 0).unwrap();
    e.replace("9");
    assert!(e.apply());
    assert_eq!(e.draft.as_ref().unwrap().value["counts"], json!([9, 2, 3]));
    assert_eq!(e.rows()[0]["counts"], json!([1, 2, 3]));
}

fn production_editor(table: &str) -> (tempfile::TempDir, XdtEditor) {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir_all(directory.path().join("data/tables")).unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    fs::copy(
        root.join("data/tables/xdt.json"),
        directory.path().join("data/tables/xdt.json"),
    )
    .unwrap();
    let editor =
        XdtEditor::open(directory.path().to_owned()).with_selection(Some(table), Some(1), "");
    (directory, editor)
}

#[test]
fn creation_form_allocates_identity_and_inserts_only_on_explicit_create() {
    for table in [
        "m_pNpcTable/m_pNpcData",
        "m_pNanoTable/m_pNanoData",
        "m_pWeaponItemTable/m_pItemData",
        "m_pMissionTable/m_pMissionData",
    ] {
        let (_dir, mut e) = production_editor(table);
        let before = e.rows().to_vec();
        let id = schema::identity(&e.tables[e.table].label).unwrap();
        let max = before.iter().filter_map(|r| r[id].as_i64()).max().unwrap();
        e.start_draft(false).unwrap();
        assert_eq!(e.draft.as_ref().unwrap().value[id], json!(max + 1));
        assert_eq!(e.rows(), before);
        assert!(!e.save_on_close());
        e.begin(Focus::NewName);
        e.replace("New authored record / Новая запись");
        assert!(e.apply());
        e.create_draft().unwrap();
        assert_eq!(e.rows().len(), before.len() + 1);
        assert_eq!(&e.rows()[..before.len()], before);
        e.undo(false);
        assert_eq!(e.rows(), before);
    }
}

#[test]
fn inline_name_and_owner_are_one_atomic_undo_step_and_cancel_is_lossless() {
    let (_dir, mut e) = production_editor("m_pNpcTable/m_pNpcData");
    let before = e.document.clone();
    e.start_draft(false).unwrap();
    let name = e.draft.as_ref().unwrap().name.as_ref().unwrap();
    let target = name.table;
    let text_count = e
        .document
        .pointer(&e.tables[target].pointer)
        .unwrap()
        .as_array()
        .unwrap()
        .len();
    assert!(e.create_draft().is_err());
    assert_eq!(e.document, before);
    e.begin(Focus::NewName);
    e.replace("Новый персонаж");
    assert!(e.apply());
    e.create_draft().unwrap();
    assert_eq!(e.record_name(e.table, e.rows().len() - 1), "Новый персонаж");
    let texts = e
        .document
        .pointer(&e.tables[target].pointer)
        .unwrap()
        .as_array()
        .unwrap();
    assert_eq!(texts.len(), text_count + 1);
    assert_eq!(texts[text_count]["m_strComment"], "");
    e.undo(false);
    assert_eq!(e.document, before);
    e.undo(true);
    assert_eq!(e.record_name(e.table, e.rows().len() - 1), "Новый персонаж");
    e.save().unwrap();
    assert_eq!(read(&e.path).unwrap(), e.document);
    e.start_draft(false).unwrap();
    e.draft = None;
    assert!(!e.dirty());
}

#[test]
fn required_form_rejects_missing_names_invalid_attributes_and_stamina() {
    let (_dir, mut e) = production_editor("m_pNanoTable/m_pNanoData");
    e.start_draft(false).unwrap();
    let draft = e.draft.as_mut().unwrap();
    draft.value["m_iNanoName"] = json!(999999);
    draft.value["m_iStyle"] = json!(9);
    draft.value["m_iNanoBattery1"] = json!(0);
    let errors = e.draft_errors();
    for field in ["m_iNanoName", "m_iStyle", "m_iNanoBattery1"] {
        assert!(errors.iter().any(|(f, _)| f == field));
    }
    assert!(e.create_draft().is_err());
    assert!(e.draft.is_some());
    assert!(!e.dirty());
}

#[test]
fn reference_picker_uses_index_or_declared_id_and_preserves_shared_data() {
    let (_dir, mut e) = production_editor("m_pNpcTable/m_pNpcData");
    e.start_draft(false).unwrap();
    let (t, _) = e.reference_target("m_iNpcName").unwrap();
    let texts = e.document.pointer(&e.tables[t].pointer).unwrap().clone();
    e.pick_reference("m_iNpcName".into(), t, 2).unwrap();
    assert_eq!(e.draft.as_ref().unwrap().value["m_iNpcName"], json!(2));
    assert_eq!(e.document.pointer(&e.tables[t].pointer).unwrap(), &texts);
    let (_dir, mut e) = production_editor("m_pMissionTable/m_pMissionData");
    e.start_draft(false).unwrap();
    let (t, id) = e.reference_target("m_iHNPCID").unwrap();
    assert_eq!(id, Some("m_iNpcNumber"));
    let expected = e.document.pointer(&e.tables[t].pointer).unwrap()[5]["m_iNpcNumber"].clone();
    e.pick_reference("m_iHNPCID".into(), t, 5).unwrap();
    assert_eq!(e.draft.as_ref().unwrap().value["m_iHNPCID"], expected);
}

#[test]
fn readable_name_search_finds_entities_from_linked_text_rows() {
    let (_dir, mut e) = production_editor("m_pNpcTable/m_pNpcData");
    let name = e.record_name(e.table, 1);
    assert!(!name.starts_with("ID "));
    e.search = name;
    e.refresh();
    assert!(e.filtered.contains(&1));
    assert!(
        e.value_label(1, "m_iNpcName")
            .contains(&e.record_name(e.table, 1))
    );
}

#[test]
fn new_text_requires_name_but_allows_empty_optional_comments() {
    let (_dir, mut e) = production_editor("m_pNpcTable/m_pNpcStringData");
    e.row = None;
    let count = e.rows().len();
    e.start_draft(false).unwrap();
    assert!(
        e.draft_errors()
            .iter()
            .any(|(field, _)| field == "m_strName")
    );
    e.begin(Focus::Draft("m_strName".into()));
    e.replace("New NPC / Новый NPC");
    assert!(e.apply());
    assert!(e.draft_errors().is_empty());
    e.create_draft().unwrap();
    assert_eq!(e.rows().len(), count + 1);
    assert_eq!(e.rows()[count]["m_strName"], "New NPC / Новый NPC");
    assert_eq!(e.rows()[count]["m_strComment"], "");
}

fn document(rows: Value) -> Value {
    json!({"schema":"ffone.table-set.v1", "metadata":{"keep":true},
        "tables":[{"name":"test", "key":"accepted-key", "value":{"m_pRows":rows}}]})
}
fn editor(rows: Value) -> (tempfile::TempDir, XdtEditor) {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir_all(directory.path().join("data/tables")).unwrap();
    fs::write(
        directory.path().join("data/tables/xdt.json"),
        document(rows).to_string(),
    )
    .unwrap();
    let editor = XdtEditor::open(directory.path().to_owned());
    (directory, editor)
}
#[test]
fn lossless_edit_add_undo_and_atomic_save() {
    let (_dir, mut e) = editor(json!([{"m_iID":1,"name":"EN / РУ","nested":{"values":[1,2]}}]));
    e.row = Some(0);
    e.begin(Focus::Cell("name".into()));
    e.replace("Привет\nworld");
    assert!(e.apply());
    e.row = Some(1);
    e.focus = Some(Focus::Row);
    e.edit = json!({"m_iID":2,"name":"new","nested":{"values":[3]}}).to_string();
    assert!(e.apply());
    assert_eq!(e.rows().len(), 2);
    e.undo(false);
    assert_eq!(e.rows().len(), 1);
    e.undo(true);
    assert_eq!(e.rows().len(), 2);
    e.save().unwrap();
    let disk = read(&e.path).unwrap();
    let published: Value = serde_json::from_slice(&fs::read(&e.path).unwrap()).unwrap();
    assert!(published.get("tables").is_none());
    assert_eq!(published["m_pRows"].as_array().unwrap().len(), 2);
    assert_eq!(published["_ffone"]["tables"][0]["key"], "accepted-key");
    assert_eq!(disk["metadata"], json!({"keep":true}));
    assert_eq!(disk["tables"][0]["key"], "accepted-key");
    assert_eq!(
        disk["tables"][0]["value"]["m_pRows"][0]["nested"],
        json!({"values":[1,2]})
    );
    assert!(!e.dirty());
    e.undo(false);
    assert!(e.dirty());
    assert_eq!(e.rows().len(), 1);
    e.undo(true);
    assert!(!e.dirty());
    assert_eq!(e.rows().len(), 2);
}
#[test]
fn conflicting_save_leaves_disk_and_draft_intact() {
    let (_dir, mut e) = editor(json!([{"m_iID":1,"name":"base"}]));
    e.row = Some(0);
    e.begin(Focus::Cell("name".into()));
    e.replace("draft");
    assert!(e.apply());
    let disk = document(json!([{"m_iID":1,"name":"external"}]));
    fs::write(&e.path, disk.to_string()).unwrap();
    assert!(e.save().unwrap_err().contains("conflict"));
    assert_eq!(read(&e.path).unwrap(), disk);
    assert_eq!(e.rows()[0]["name"], "draft");
    assert!(e.dirty());
}
#[test]
fn unrelated_external_fields_merge_without_loss() {
    let base = document(json!([{"m_iID":1,"a":0,"b":0}]));
    let draft = document(json!([{"m_iID":1,"a":7,"b":0}]));
    let mut disk = document(json!([{"m_iID":1,"a":0,"b":9}]));
    disk["extra"] = json!(["retain"]);
    let merged = merge_document(&base, &draft, &disk).unwrap();
    assert_eq!(
        merged["tables"][0]["value"]["m_pRows"][0],
        json!({"m_iID":1,"a":7,"b":9})
    );
    assert_eq!(merged["extra"], json!(["retain"]));
}
#[test]
fn external_row_reordering_cannot_attach_a_draft_to_another_id() {
    let base = document(json!([{"m_iID":1,"name":"one"},{"m_iID":2,"name":"two"}]));
    let draft = document(json!([{"m_iID":1,"name":"edited"},{"m_iID":2,"name":"two"}]));
    let disk = document(json!([{"m_iID":2,"name":"two"},{"m_iID":1,"name":"one"}]));
    assert!(
        merge_document(&base, &draft, &disk)
            .unwrap_err()
            .contains("identities/order")
    );
}
#[test]
fn invalid_types_duplicates_and_invalid_json_keep_active_draft() {
    let (_dir, mut e) = editor(json!([{"m_iID":1,"x":4},{"m_iID":2,"x":4}]));
    e.row = Some(1);
    e.begin(Focus::Cell("m_iID".into()));
    e.replace("1");
    assert!(!e.apply());
    assert_eq!(e.rows()[1]["m_iID"], 2);
    assert!(e.focus.is_some());
    e.edit = "1.5".into();
    assert!(!e.apply());
    e.focus = Some(Focus::Row);
    e.edit = "{".into();
    assert!(!e.apply());
}
#[test]
fn csv_round_trip_retains_unicode_arrays_null_and_absent_fields() {
    let rows = json!([{"m_iID":1,"text":"Привет, \"world\"\nline","nested":[1,{"x":true}],"null":null},
        {"m_iID":2,"text":"","nested":[],"optional":"value"}]);
    let rows = rows.as_array().unwrap();
    let csv = exchange::csv_export(rows).unwrap();
    assert_eq!(exchange::csv_import(&csv, rows).unwrap(), *rows);
    assert!(exchange::csv_import("m_iID,m_iID\n1,1", rows).is_err());
    assert!(
        exchange::csv_import("m_iID,text,nested,null,optional\n1,broken,[],null,", rows).is_err()
    );
}
#[test]
fn explicit_index_routes_and_incoming_mission_references() {
    let mut v = document(json!([]));
    v["tables"][0]["value"] = json!({
        "m_pNpcTable": {
            "m_pNpcData":[{"m_iNpcNumber":17,"m_iIcon1":1,"m_iNpcName":0}],
            "m_pNpcIconData":[{"m_iIconNumber":100},{"m_iIconNumber":900}],
            "m_pNpcStringData":[{"m_strName":"Dexter"}]
        },
        "m_pMissionTable":{"m_pMissionData":[{"m_iHNPCID":17,"m_iHTaskID":21}]}
    });
    let (_dir, mut e) = editor(json!([]));
    e.document = v.clone();
    e.base = v.clone();
    e.discover();
    let npc = e
        .tables
        .iter()
        .position(|t| t.label.ends_with("m_pNpcTable/m_pNpcData"))
        .unwrap();
    e.select_table(npc);
    e.row = Some(0);
    e.rebuild_links();
    assert!(e.links.iter().any(|l| l.confirmed
        && !l.incoming
        && l.row == 1
        && e.tables[l.table].label.ends_with("m_pNpcIconData")));
    assert!(
        e.links
            .iter()
            .any(|l| l.confirmed && l.incoming && l.field == "m_iHNPCID")
    );
    let mut broken = v.clone();
    broken["tables"][0]["value"]["m_pMissionTable"]["m_pMissionData"][0]["m_iHNPCID"] = json!(99);
    assert!(relations::validate_changes(&v, &broken).is_err());
}
#[test]
fn numeric_sort_and_filtered_navigation_use_original_row_indices() {
    let (_dir, mut e) = editor(
        json!([{"m_iID":10,"name":"ten"},{"m_iID":2,"name":"two"},{"m_iID":100,"name":"hundred"}]),
    );
    e.sort = Some(("m_iID".into(), false));
    e.refresh();
    assert_eq!(e.filtered, vec![1, 0, 2]);
    e.search = "two".into();
    e.refresh();
    assert_eq!(e.filtered, vec![1]);
}

#[test]
fn identity_and_index_sort_toggle_both_directions_without_rewriting_rows() {
    let (_dir, mut e) = editor(json!([
        {"m_iID":100,"name":"match A"}, {"m_iID":2,"name":"match B"},
        {"m_iID":10,"name":"other"}, {"m_iID":7,"name":"match C"}
    ]));
    let before = e.document.clone();
    e.row = Some(1);
    e.search = "match".into();
    e.toggle_sort(SortKey::Field("m_iID".into()));
    assert_eq!(e.filtered, vec![1,3,0]);
    e.toggle_sort(SortKey::Field("m_iID".into()));
    assert_eq!(e.filtered, vec![0,3,1]);
    e.toggle_sort(SortKey::Index);
    assert_eq!(e.filtered, vec![0,1,3]);
    e.toggle_sort(SortKey::Index);
    assert_eq!(e.filtered, vec![3,1,0]);
    assert_eq!(e.row, Some(1));
    assert_eq!(e.document, before);
    assert_eq!(sorting::identity("test/m_pSkillTable/m_pSkillData", &["m_iSkillNumber".into()]), SortKey::Field("m_iSkillNumber".into()));
    assert_eq!(sorting::identity("test/m_pMissionTable/m_pJournalData", &["m_iMissionSummary".into()]), SortKey::Index);
    assert_eq!(sorting::identity("test/m_pNpcTable/m_pNpcStringData", &["m_iExtraNumber".into()]), SortKey::Index);
}
#[test]
fn deleting_indexed_rows_cannot_silently_retarget_a_reference() {
    let mut base = document(json!([]));
    base["tables"][0]["value"] = json!({"m_pNpcTable":{
        "m_pNpcData":[{"m_iNpcNumber":1,"m_iIcon1":1}],
        "m_pNpcIconData":[{"m_iIconNumber":10},{"m_iIconNumber":20},{"m_iIconNumber":30}]
    }});
    let mut draft = base.clone();
    draft["tables"][0]["value"]["m_pNpcTable"]["m_pNpcIconData"]
        .as_array_mut()
        .unwrap()
        .remove(0);
    assert!(
        relations::validate_changes(&base, &draft)
            .unwrap_err()
            .contains("shifts")
    );
    draft["tables"][0]["value"]["m_pNpcTable"]["m_pNpcData"][0]["m_iIcon1"] = json!(0);
    assert!(relations::validate_changes(&base, &draft).is_ok());
    let mut reordered = base.clone();
    reordered["tables"][0]["value"]["m_pNpcTable"]["m_pNpcIconData"]
        .as_array_mut()
        .unwrap()
        .swap(1, 2);
    assert!(
        relations::validate_changes(&base, &reordered)
            .unwrap_err()
            .contains("shifts")
    );
    reordered["tables"][0]["value"]["m_pNpcTable"]["m_pNpcData"][0]["m_iIcon1"] = json!(2);
    assert!(relations::validate_changes(&base, &reordered).is_ok());
}
#[test]
fn production_table_discovery_and_all_table_csv_round_trips() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let e = XdtEditor::open(root);
    assert!(e.status.is_empty(), "{}", e.status);
    assert!(e.tables.len() > 100);
    for table in &e.tables {
        let rows = e
            .document
            .pointer(&table.pointer)
            .unwrap()
            .as_array()
            .unwrap();
        validate_rows(rows, rows).unwrap_or_else(|err| panic!("{}: {err}", table.label));
        let csv = exchange::csv_export(rows).unwrap();
        assert_eq!(
            exchange::csv_import(&csv, rows).unwrap_or_else(|err| panic!("{}: {err}", table.label)),
            *rows,
            "{}",
            table.label
        );
    }
}
#[test]
fn semantic_string_id_candidates_preserve_exact_unicode_identity() {
    let (_dir, mut e) = editor(json!([{"id":"npc/Декстер"}]));
    e.document["tables"][0]["value"]["m_pRelated"] = json!([
        {"id":"npc/Декстер"},{"id":"npc/декстер"}
    ]);
    e.discover();
    e.row = Some(0);
    e.rebuild_links();
    assert_eq!(e.links.len(), 1);
    assert_eq!(e.links[0].row, 0);
    assert!(!e.links[0].confirmed);
}
#[test]
fn repeated_number_columns_do_not_create_spurious_id_fanout() {
    let (_dir, mut e) = editor(json!([
        {"m_iNpcNumber":1,"m_iTargetNumber":1},{"m_iNpcNumber":2,"m_iTargetNumber":1}
    ]));
    e.document["tables"][0]["value"]["m_pRelated"] = json!([
        {"m_iTargetNumber":1},{"m_iTargetNumber":1}
    ]);
    e.discover();
    e.row = Some(0);
    e.rebuild_links();
    assert!(e.links.is_empty());
}
#[test]
fn tab_navigation_skips_absent_optional_cells_in_both_directions() {
    let (_dir, mut e) = editor(json!([{"m_iID":1},{"m_iID":2,"optional":"present"}]));
    e.row = Some(0);
    e.begin(Focus::Cell("m_iID".into()));
    e.tab_cell(false, None);
    assert_eq!(e.row, Some(1));
    assert_eq!(e.focus, Some(Focus::Cell("m_iID".into())));
    e.tab_cell(true, None);
    assert_eq!(e.row, Some(0));
    assert_eq!(e.focus, Some(Focus::Cell("m_iID".into())));
    assert!(!e.dirty());
}
