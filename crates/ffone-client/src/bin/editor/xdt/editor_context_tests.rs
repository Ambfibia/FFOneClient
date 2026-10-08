use super::*;
#[test]
fn type_six_mail_keeps_its_own_message_and_speaker_with_one_undo(){
    let (_dir,mut e)=fixture();
    let pointer=e.tables[e.table].pointer.clone();
    let task=&mut e.document.pointer_mut(&pointer).unwrap()[0];
    task["m_iSTMessageType"]=json!(6);task["m_iSTMessageTextID"]=json!(1);task["m_iSTMessageSendNPC"]=json!(17);
    e.discover();e.row=Some(0);let before=e.document.clone();
    e.edit_email(0,"m_iSTMessageTextID",None).unwrap();
    e.begin(Focus::Locale(0));e.edit="Separate mail".into();e.apply();
    e.begin(Focus::Locale(1));e.edit="Отдельное письмо".into();e.create_draft().unwrap();
    assert_eq!(e.rows()[0]["m_iSTMessageTextID"],1);
    let email=e.rows()[0]["m_iSTEmailTextID"].as_u64().unwrap();assert!(email>1);
    let events=mission_preview::events(&e.rows()[0]);
    assert!(events.iter().any(|event|event.channel==mission_preview::Channel::Email&&event.field=="m_iSTEmailTextID"));
    assert!(events.iter().any(|event|event.channel==mission_preview::Channel::Message&&event.field=="m_iSTMessageTextID"));
    assert_eq!(e.undo.len(),1);e.undo(false);assert_eq!(e.document,before);
}
#[test]
fn editing_journal_description_updates_both_locales_and_semantic_aliases(){
    let (dir,mut e)=fixture();
    let table=&mut e.document["tables"][0]["value"]["m_pMissionTable"];
    table["m_pJournalData"]=json!([{}, {"m_iMissionSummary":1,"m_iDetaileMissionDesc":1,"m_iTaskSummary":1,"m_iDetailedTaskDesc":1,"m_iMissionCompleteSummary":1,"m_iDetaileMissionCompleteSummary":1}]);
    table["m_pMissionData"][0]["m_iSTJournalIDAdd"]=json!(1);
    e.discover();e.select_table(e.tables.iter().position(|t|t.label.ends_with("/m_pMissionData")).unwrap());e.row=Some(0);
    e.base=e.document.clone();fs::write(&e.path,e.document.to_string()).unwrap();
    for (locale,text) in [("en","Saved description"),("ru","Сохранённое описание")] {
        let path=dir.path().join(format!("assets/game/localization/{locale}.json"));
        fs::write(path,json!({"entries":{"unrelated":"preserve","content.mission.task.10.task_description":text,"content.tabledata.mission.mission_string.1.str_name_string":"Stale table copy"}}).to_string()).unwrap();
    }
    e.start_journal_edit("m_iSTJournalIDAdd").unwrap();
    e.start_mission_text_edit("m_iDetailedTaskDesc",true).unwrap();
    assert_eq!(e.workspace.quick_text.as_ref().unwrap().values,["Saved description","Сохранённое описание"]);
    e.begin(Focus::Locale(0));e.edit="New task description".into();e.apply();
    e.begin(Focus::Locale(1));e.edit="Новое описание этапа".into();e.create_draft().unwrap();
    e.create_draft().unwrap();
    e.undo(false);assert_eq!(e.document,e.base);e.undo(true);
    e.rewrite().unwrap();
    for (locale,expected) in [("en","New task description"),("ru","Новое описание этапа")] {
        let data:Value=serde_json::from_slice(&fs::read(dir.path().join(format!("assets/game/localization/{locale}.json"))).unwrap()).unwrap();
        assert_eq!(data["entries"]["content.mission.task.10.task_description"],expected);
        assert_eq!(data["entries"]["content.tabledata.mission.mission_string.1.str_name_string"],expected);
    }
}
#[test]
fn rewriting_equal_xdt_preserves_client_bytes_and_copies_them_to_server(){
    let (dir,mut e)=fixture();
    let server=dir.path().join("server");fs::create_dir_all(&server).unwrap();
    let raw=ffone_client::xdt::into_server_document(e.document.clone()).unwrap();
    let bytes=serde_json::to_vec(&raw).unwrap();fs::write(&e.path,&bytes).unwrap();
    fs::write(server.join("xdt.json"),serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    e.publication=Some(mission_publish::Publication::open(server.join("xdt.json")).unwrap());
    e.rewrite().unwrap();
    assert_eq!(fs::read(&e.path).unwrap(),bytes);assert_eq!(fs::read(server.join("xdt.json")).unwrap(),bytes);
    assert!(!e.path.parent().unwrap().join(".ffone-backups/xdt.json").exists());
}
#[test]
fn mission_and_npc_string_filters_include_items_names_and_speakers(){
    let (_dir,mut e)=fixture();let pointer=e.tables[e.table].pointer.clone();
    e.document.pointer_mut(&pointer).unwrap()[0]["m_iCSUItemID"]=json!([1,0,0]);
    e.document.pointer_mut(&pointer).unwrap()[0]["m_iSTDialogBubble"]=json!(1);
    e.document.pointer_mut(&pointer).unwrap()[0]["m_iSTDialogBubbleNPCID"]=json!(17);e.discover();
    let keys=BTreeMap::from([
        ("content.tabledata.mission.mission_string.1.str_name_string".into(),"Talk".into()),
        ("content.npc.17.name".into(),"Dexter".into()),
        ("content.quest_item.1.name".into(),"Beacon".into()),
        ("unrelated".into(),"Other".into()),
    ]);
    for (mode,id) in [(1,1),(2,17)] {let linked=e.related_string_keys(mode,id,&keys);assert_eq!(linked.len(),3);assert!(!linked.contains("unrelated"));}
    assert_eq!(e.string_speakers(&keys)["content.tabledata.mission.mission_string.1.str_name_string"],17);
    assert!(e.matches_search(e.table,0,"секретный маяк"));
}
#[test]
fn copying_mob_level_keeps_model_identity_and_voice(){
    let (_dir,mut e)=fixture();let table=e.tables.iter().position(|t|t.label.ends_with("/m_pNpcData")).unwrap();e.select_table(table);e.row=Some(0);
    let pointer=e.tables[table].pointer.clone();let mut source=e.rows()[0].clone();source["m_iNpcNumber"]=json!(999);source["m_iMesh"]=json!(88);source["m_iComment"]=json!(55);source["m_iNpcLevel"]=json!(30);source["m_iHP"]=json!(10000);source["m_iProtection"]=json!(500);
    e.document.pointer_mut(&pointer).unwrap().as_array_mut().unwrap().push(source);e.discover();e.row=Some(0);e.start_draft(true).unwrap();
    let before=e.draft.as_ref().unwrap().value.clone();e.copy_mob_stats(1).unwrap();let result=&e.draft.as_ref().unwrap().value;
    for field in ["m_iNpcNumber","m_iNpcName","m_iMesh","m_iComment"]{assert_eq!(result[field],before[field]);}
    assert_eq!(result["m_iHP"],10000);assert_eq!(result["m_iNpcLevel"],30);assert_eq!(result["m_iProtection"],500);
}
