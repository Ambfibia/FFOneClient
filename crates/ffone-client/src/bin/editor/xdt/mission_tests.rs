use super::*;
use serde_json::json;

fn fixture() -> (tempfile::TempDir,XdtEditor) {
    let dir=tempfile::tempdir().unwrap();
    let root=dir.path().join("assets/game");
    let mut mission=json!({});
    for field in schema::required("test/m_pMissionTable/m_pMissionData") {mission[*field]=json!(0);}
    for (field,value) in [
        ("m_iHTaskID",json!(10)),("m_iHMissionID",json!(1)),("m_iHMissionType",json!(3)),("m_iHTaskType",json!(1)),
        ("m_iHNPCID",json!(17)),("m_iHTerminatorNPCID",json!(17)),("m_iHCurrentObjective",json!(1)),
        ("m_iCSTReqMission",json!([0,0])),("m_iCSUEnemyID",json!([0,0,0])),("m_iCSUNumToKill",json!([0,0,0])),
        ("m_iCSUItemID",json!([0,0,0])),("m_iCSUItemNumNeeded",json!([0,0,0])),("m_iCSUCheckTimer",json!(30)),
    ] {mission[field]=value;}
    let mut second=mission.clone();second["m_iHTaskID"]=json!(11);
    let document=json!({"schema":"ffone.table-set.v1","tables":[{"name":"test","value":{
        "m_pMissionTable":{"m_pMissionData":[mission,second],
            "m_pMissionStringData":[{"m_pstrNameString":"Mission","extra":"keep"},{"m_pstrNameString":"Talk to Dexter","description":"секретный маяк"}],
            "m_pRewardData":[{"m_iMissionRewardID":1,"m_iCash":100,"m_iFusionMatter":20}]},
        "m_pNpcTable":{"m_pNpcData":[{"m_iNpcNumber":17,"m_iNpcName":0,"m_iHP":100,"m_iNpcLevel":1,"m_iNpcStyle":0,"m_iNpcType":0,"m_iRadius":1,"m_iHeight":1,"m_iSightRange":1}],
            "m_pNpcStringData":[{"m_strName":"Dexter","m_strComment":"секретный исследователь"}]},
        "m_pQuestItemTable":{"m_pItemData":[{"m_iItemNumber":1,"m_iItemName":0,"m_iIcon":0}],
            "m_pItemStringData":[{"m_strName":"Beacon"}],"m_pItemIconData":[{"m_iIconNumber":1}]}
    }}]});
    fs::create_dir_all(root.join("data/tables")).unwrap();
    fs::write(root.join("data/tables/xdt.json"),document.to_string()).unwrap();
    fs::create_dir_all(root.join("localization")).unwrap();
    for locale in ["en","ru"]{fs::write(root.join(format!("localization/{locale}.json")),json!({"entries":{"unrelated":"preserve"}}).to_string()).unwrap();}
    let e=XdtEditor::open(root).with_selection(Some("m_pMissionTable/m_pMissionData"),Some(0),"");
    (dir,e)
}

#[test]
fn right_mouse_input_pans_on_drag_and_opens_or_dismisses_the_menu_on_click() {
    let (_dir, mut editor) = fixture();
    editor.workspace.view_request=None;
    let before=editor.document.clone();
    let mut app=App::new();
    app.insert_resource(EditorState {
        reveal_selection:false,kind:CatalogKind::Npc,selected:0,search:String::new(),search_focused:false,
        strings_open:false,xdt_open:true,viewer_tabs:BTreeMap::new(),details_open:false,
        equipment_female:false,equipment_category:None,animation_page:0,clip_index:0,
        pose_mode:EditorPoseMode::Default,paused:true,looping:true,speed:1.,turntable:false,playback_revision:0,
    }).insert_resource(editor).init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<ButtonInput<KeyCode>>().init_resource::<Time>()
        .add_message::<MouseWheel>().add_message::<bevy::window::WindowEvent>()
        .add_systems(Update,mission_pointer::pointer);
    let window=app.world_mut().spawn(Window::default()).id();
    app.world_mut().spawn((mission_canvas::Hit::Canvas,
        ComputedNode {size:Vec2::new(1000.,700.),inverse_scale_factor:1.,..default()},
        UiGlobalTransform::default()));
    let step=|app:&mut App, position:Vec2, press:Option<MouseButton>, release:Option<MouseButton>| {
        app.world_mut().get_mut::<Window>(window).unwrap().set_cursor_position(Some(position));
        let mut mouse=app.world_mut().resource_mut::<ButtonInput<MouseButton>>();mouse.clear();
        if let Some(button)=press{mouse.press(button);}if let Some(button)=release{mouse.release(button);}
        drop(mouse);app.update();
    };
    step(&mut app,Vec2::new(100.,100.),Some(MouseButton::Right),None);
    assert!(app.world().resource::<XdtEditor>().workspace.context.is_none());
    step(&mut app,Vec2::new(180.,140.),None,None);
    step(&mut app,Vec2::new(200.,150.),None,Some(MouseButton::Right));
    let e=app.world().resource::<XdtEditor>();
    assert_eq!(e.workspace.pan,Vec2::new(100.,50.));assert!(e.workspace.context.is_none());
    assert_eq!(e.document,before);assert!(e.workspace.positions.is_empty());
    step(&mut app,Vec2::new(210.,155.),Some(MouseButton::Right),None);
    step(&mut app,Vec2::new(211.,156.),None,Some(MouseButton::Right));
    assert_eq!(app.world().resource::<XdtEditor>().workspace.context.as_ref().unwrap().0,Vec2::new(210.,155.));
    app.world_mut().spawn((mission_context::Panel {anchor:Vec2::ZERO,interactive:true},
        ComputedNode {size:Vec2::splat(30.),inverse_scale_factor:1.,..default()},UiGlobalTransform::default()));
    step(&mut app,Vec2::new(250.,200.),Some(MouseButton::Left),None);
    let e=app.world().resource::<XdtEditor>();
    assert!(e.workspace.context.is_none());assert_eq!(e.workspace.pan,Vec2::new(100.,50.));
    assert!(e.workspace.marquee.is_none());
}

#[test]
fn stage_preview_uses_filled_objectives_orders_events_and_keeps_cards_apart() {
    use mission_preview::{Channel, Phase};
    let (_dir, mut e)=fixture();
    let pointer=e.tables[e.table].pointer.clone();
    let row=&mut e.document.pointer_mut(&pointer).unwrap()[0];
    row["m_iHTaskType"]=json!(5);
    row["m_iCSUEnemyID"]=json!([17,0,0]);
    row["m_iCSUItemID"]=json!([1,0,0]);row["m_iCSUItemNumNeeded"]=json!([3,0,0]);
    row["m_iSTItemID"]=json!([0,1,0]);row["m_iSTItemDropRate"]=json!([0,25,0]);
    row["m_iSTJournalIDAdd"]=json!(1);row["m_iHJournalNPCID"]=json!(17);
    row["m_iSTMessageTextID"]=json!(1);row["m_iSTMessageSendNPC"]=json!(18);
    row["m_iSTDialogBubble"]=json!(1);row["m_iSTDialogBubbleNPCID"]=json!(19);
    row["m_iSUMessagetextID"]=json!(1);row["m_iSUMessageSendNPC"]=json!(20);
    row["m_iSUDialogBubble"]=json!(1);row["m_iSUDialogBubbleNPCID"]=json!(21);
    row["m_iSUOutgoingTask"]=json!(11);
    let goals=mission_preview::goals(row);
    assert_eq!(goals.len(),1);assert_eq!(goals[0].kind,"collect");
    assert_eq!(goals[0].count,Some(3));assert_eq!(goals[0].drop_rate,Some(25));assert_eq!(goals[0].sources,vec![17]);
    let events=mission_preview::events(row);
    assert_eq!(events.iter().map(|v|(v.phase,v.channel,v.npc)).collect::<Vec<_>>(),vec![
        (Phase::Start,Channel::Journal,17),(Phase::Start,Channel::Message,18),(Phase::Start,Channel::Bubble,19),
        (Phase::Complete,Channel::Message,20),(Phase::Complete,Channel::Bubble,21)]);
    let before=e.document.clone();
    let scene=mission_canvas::scene(&e);
    let first=scene.nodes.iter().find(|n|n.row==Some(0)).unwrap();
    let next=scene.nodes.iter().find(|n|n.row==Some(1)).unwrap();
    assert_eq!(first.size().x,mission_preview::WIDTH);
    assert!(first.size().y>600.);assert!(next.position.y>first.position.y+first.size().y);
    let expanded_height = first.size().y;
    e.workspace.collapsed_stages.insert(10);
    let collapsed = mission_canvas::scene(&e);
    let folded = collapsed.nodes.iter().find(|n|n.row==Some(0)).unwrap();
    let after_fold = collapsed.nodes.iter().find(|n|n.row==Some(1)).unwrap();
    assert_eq!(folded.size(),Vec2::new(mission_preview::WIDTH,152.));
    assert!(after_fold.position.y>folded.position.y+folded.size().y);
    assert_eq!(scene.edges.iter().map(|v|(v.from,v.to)).collect::<Vec<_>>(),
        collapsed.edges.iter().map(|v|(v.from,v.to)).collect::<Vec<_>>());
    e.save_layout().unwrap();
    let root=e.path.parent().unwrap().parent().unwrap().parent().unwrap();
    assert!(mission_workspace::Workspace::load(root).collapsed_stages.contains(&10));
    e.workspace.collapsed_stages.remove(&10);
    assert_eq!(mission_canvas::scene(&e).nodes.iter().find(|n|n.row==Some(0)).unwrap().size().y,expanded_height);
    assert_eq!(e.document,before);
    let long="Длинная реплика без потери символов. ".repeat(8);
    let short=mission_preview::excerpt(&long,104);
    assert_eq!(short.chars().count(),104);assert!(short.ends_with('…'));
}

#[test]
fn bubble_text_and_nested_journal_edits_preserve_ids_and_cancel_or_undo_together() {
    let (_dir,mut e)=fixture();
    let zero=Value::Object(mission_journal::TEXT_FIELDS.iter().map(|f|((*f).into(),json!(0))).collect());
    let mut entry=zero.clone();entry["m_iDetailedTaskDesc"]=json!(1);
    e.document["tables"][0]["value"]["m_pMissionTable"]["m_pJournalData"]=json!([zero,entry]);
    let pointer=e.tables[e.table].pointer.clone();
    e.document.pointer_mut(&pointer).unwrap()[0]["m_iSTJournalIDAdd"]=json!(1);
    for field in ["m_iSTDialogBubble","m_iSUDialogBubble","m_iFDialogBubble"] {
        e.document.pointer_mut(&pointer).unwrap()[0][field]=json!(0);
    }
    e.discover();let owner=e.tables.iter().position(|t|t.label.ends_with("/m_pMissionData")).unwrap();
    e.select_table(owner);e.row=Some(0);
    let before=e.document.clone();
    for field in ["m_iSTDialogBubble","m_iSUDialogBubble","m_iFDialogBubble"] {
        let (table,id)=e.reference_target(field).unwrap();
        assert!(e.tables[table].label.ends_with("/m_pMissionStringData"));assert_eq!(id,None);
    }
    e.start_mission_text_edit("m_iSTDialogBubble",true).unwrap();
    assert!(e.workspace.quick_existing.is_none(),"Zero is disabled, never a shared editable text");
    e.begin(Focus::Locale(1));e.edit="Новая реплика".into();e.create_draft().unwrap();
    assert_eq!(e.rows()[0]["m_iSTDialogBubble"],2);
    assert_eq!(e.document["tables"][0]["value"]["m_pMissionTable"]["m_pMissionStringData"][0],before["tables"][0]["value"]["m_pMissionTable"]["m_pMissionStringData"][0]);
    e.undo(false);assert_eq!(e.document,before);
    e.start_journal_edit("m_iSTJournalIDAdd").unwrap();
    e.start_mission_text_edit("m_iDetailedTaskDesc",false).unwrap();
    e.begin(Focus::Locale(1));e.edit="Изменённый дневник".into();e.create_draft().unwrap();
    e.cancel_quick();assert_eq!(e.document,before);
    e.start_journal_edit("m_iSTJournalIDAdd").unwrap();
    e.start_mission_text_edit("m_iDetailedTaskDesc",false).unwrap();
    e.begin(Focus::Locale(1));e.edit="Изменённый дневник".into();e.create_draft().unwrap();
    e.create_draft().unwrap();
    assert_eq!(e.rows()[0]["m_iSTJournalIDAdd"],1);
    assert_eq!(e.document["tables"][0]["value"]["m_pMissionTable"]["m_pJournalData"][1]["m_iDetailedTaskDesc"],2);
    assert_eq!(e.undo.len(),1);e.undo(false);assert_eq!(e.document,before);
}

#[test]
fn stage_appends_after_selection_inherits_identity_and_preserves_the_tail() {
    let (_dir, mut e) = fixture();
    let pointer = e.tables[e.table].pointer.clone();
    let rows = e.document.pointer_mut(&pointer).unwrap().as_array_mut().unwrap();
    rows[0]["m_iHDifficultyType"] = json!(2);
    rows[0]["m_iHMissionType"] = json!(2);
    rows[0]["m_iHTaskType"] = json!(5);
    rows[0]["m_iHJournalNPCID"] = json!(17);
    rows[0]["m_iSUOutgoingTask"] = json!(11);
    e.reindex();e.refresh();
    let before = e.document.clone();
    let r = e.append_mission(false, false, "unused", "New objective").unwrap();
    assert_eq!(e.rows()[0]["m_iSUOutgoingTask"], 12);
    assert_eq!(e.rows()[r]["m_iSUOutgoingTask"], 11);
    for field in ["m_iHMissionID", "m_iHMissionName", "m_iHDifficultyType", "m_iHMissionType", "m_iHTaskType", "m_iHJournalNPCID"] {
        assert_eq!(e.rows()[r][field], e.rows()[0][field], "{field}");
    }
    assert_eq!(e.rows()[r]["m_iCSUEnemyID"], json!([0,0,0]));
    assert_eq!(mission_graph::ordered_stages(e.rows(), &[1,r,0]), vec![0,r,1]);
    assert_eq!(mission_dependencies::first_rows(e.rows()).get(&1), Some(&0));
    e.undo(false);assert_eq!(e.document, before);
    e.undo(true);assert_eq!(e.rows()[0]["m_iSUOutgoingTask"],12);
    e.row=Some(1);
    let tail=e.append_mission(false,false,"unused","Last stage").unwrap();
    assert_eq!(e.rows()[1]["m_iSUOutgoingTask"],e.rows()[tail]["m_iHTaskID"]);
    assert_eq!(e.rows()[tail]["m_iSUOutgoingTask"],0);
}

#[test]
fn waypoint_picker_and_placeholder_keep_the_template_and_other_npcs_intact() {
    let (_dir,mut e)=fixture();
    let npcs=&mut e.document["tables"][0]["value"]["m_pNpcTable"];
    let mut template=npcs["m_pNpcData"][0].clone();
    template["m_iNpcNumber"]=json!(1401);template["m_iNpcName"]=json!(1);
    template["m_iMesh"]=json!(256);template["m_iNpcType"]=json!(111);
    template["m_iHP"]=json!(597);template["future"]=json!({"keep":[1,2]});
    npcs["m_pNpcData"].as_array_mut().unwrap().push(template.clone());
    npcs["m_pNpcStringData"].as_array_mut().unwrap().push(json!({"m_strName":"Location A256"}));
    e.reindex();e.refresh();
    let owner=e.table;let before=e.document.clone();
    let (npc_table,id)=e.reference_target("m_iSTGrantWayPoint").unwrap();
    assert_eq!(id,Some("m_iNpcNumber"));
    e.reference_search="Dexter".into();
    assert_eq!(e.reference_candidates(npc_table,"m_iSTGrantWayPoint"),vec![0]);
    e.reference_search="1401".into();
    assert_eq!(e.reference_candidates(npc_table,"m_iSTGrantWayPoint")[0],1);
    e.start_quick_kind("m_iSTGrantWayPoint",true).unwrap();
    assert!(e.draft.as_ref().unwrap().placeholder);
    e.begin(Focus::NewName);e.edit="Empty marker".into();e.create_draft().unwrap();
    assert_eq!(e.table,owner);assert_eq!(e.rows()[0]["m_iSTGrantWayPoint"],1402);
    let made=&e.document.pointer(&e.tables[npc_table].pointer).unwrap()[2];
    for (field,value) in template.as_object().unwrap() {
        if !matches!(field.as_str(),"m_iNpcNumber"|"m_iNpcName") {assert_eq!(&made[field],value,"{field}");}
    }
    assert_eq!(e.document.pointer(&e.tables[npc_table].pointer).unwrap()[1],template);
    e.undo(false);assert_eq!(e.document,before);
}

#[test]
fn journal_picker_creates_nested_localized_text_and_saves_the_index_atomically() {
    let (_dir,mut e)=fixture();
    let zero=Value::Object(mission_journal::TEXT_FIELDS.iter().map(|f|((*f).into(),json!(0))).collect());
    let mut entry=zero.clone();entry["m_iDetailedTaskDesc"]=json!(1);
    e.document["tables"][0]["value"]["m_pMissionTable"]["m_pJournalData"]=json!([zero,entry]);
    e.discover();let owner=e.tables.iter().position(|t|t.label.ends_with("/m_pMissionData")).unwrap();
    e.select_table(owner);e.row=Some(0);
    let before=e.document.clone();e.base=before.clone();fs::write(&e.path,before.to_string()).unwrap();
    for field in ["m_iSTJournalIDAdd","m_iSUJournaliDAdd","m_iFJournalIDAdd"] {
        assert!(e.reference_target(field).is_some());
    }
    let (journal,identity)=e.reference_target("m_iSTJournalIDAdd").unwrap();
    assert_eq!(identity,None);
    e.reference_search="Dexter".into();assert_eq!(e.reference_candidates(journal,"m_iSTJournalIDAdd"),vec![1]);
    e.reference_search="1".into();assert_eq!(e.reference_candidates(journal,"m_iSTJournalIDAdd")[0],1);
    e.start_quick("m_iSTJournalIDAdd").unwrap();
    e.start_quick("m_iDetailedTaskDesc").unwrap();
    e.begin(Focus::Locale(0));e.edit="Visit the empty marker".into();assert!(e.apply());
    e.begin(Focus::Locale(1));e.edit="Посети пустышку".into();e.create_draft().unwrap();
    assert_eq!(e.table,journal);
    e.create_draft().unwrap();assert_eq!(e.table,owner);assert_eq!(e.rows()[0]["m_iSTJournalIDAdd"],2);
    assert_eq!(e.undo.len(),1);
    e.reference_search="пустышку".into();assert_eq!(e.reference_candidates(journal,"m_iSTJournalIDAdd"),vec![2]);
    e.undo(false);assert_eq!(e.document,before);
    e.undo(true);e.save().unwrap();
    let disk:Value=serde_json::from_slice(&fs::read(&e.path).unwrap()).unwrap();
    assert_eq!(disk["m_pMissionTable"]["m_pMissionData"][0]["m_iSTJournalIDAdd"],2);
    assert_eq!(disk["m_pMissionTable"]["m_pJournalData"][2]["m_iDetailedTaskDesc"],2);
}

#[test]
fn new_mission_is_neutral_and_undo_preserves_all_existing_content() {
    let (_dir,mut e)=fixture();let before=e.document.clone();
    let r=e.append_mission(true,false,"Новая миссия","Завершить разговор").unwrap();
    assert_eq!(e.rows()[r]["m_iHMissionID"],json!(2));
    assert_eq!(e.rows()[r]["m_iHTaskID"],json!(12));
    assert_eq!(e.rows()[r]["m_iCSUCheckTimer"],json!(0));
    assert_eq!(e.rows()[r]["m_iHNPCID"],json!(0));
    assert_eq!(e.rows()[r]["m_iHTaskType"],json!(1));
    assert_eq!(e.record_name(e.table,r),"Новая миссия");
    assert_eq!(mission_canvas::stage_title(&e,e.table,r),"Завершить разговор");
    assert!(e.authoring_blocks(&e.rows()[r]).iter().all(|b|!e.block_open(b)));
    e.undo(false);assert_eq!(e.document,before);
    assert!(mission_canvas::scene(&e).nodes.is_empty());
    e.undo(true);assert_eq!(e.rows()[r]["m_iHTaskID"],json!(12));
}

#[test]
fn typed_codes_are_checked_for_cells_whole_rows_and_saving() {
    let (_dir,mut e)=fixture();
    for (field,invalid) in [("m_iHTaskType",0),("m_iHMissionType",4),("m_iHDifficultyType",3)] {
        let before=e.document.clone();let mut rows=e.rows().to_vec();rows[0][field]=json!(invalid);
        assert!(e.change(rows.clone()).is_err());assert_eq!(e.document,before);
        e.begin(Focus::Row);e.edit=rows[0].to_string();assert!(!e.apply());
        e.focus=None;e.picker_field=None;
        let pointer=e.tables[e.table].pointer.clone();
        *e.document.pointer_mut(&pointer).unwrap()=json!(rows);
        assert!(e.save().is_err());e.document=before;
    }
}

#[test]
fn existing_unknown_code_is_preserved_when_editing_another_record() {
    let (_dir,mut e)=fixture();let pointer=e.tables[e.table].pointer.clone();
    e.document.pointer_mut(&pointer).unwrap()[0]["m_iHTaskType"]=json!(99);
    e.base=e.document.clone();fs::write(&e.path,e.base.to_string()).unwrap();
    let mut rows=e.rows().to_vec();rows[1]["m_iHDifficultyType"]=json!(2);
    e.change(rows).unwrap();e.save().unwrap();assert_eq!(e.rows()[0]["m_iHTaskType"],json!(99));
}

#[test]
fn creating_linked_text_is_one_history_entry_and_cancel_is_lossless() {
    let (_dir,mut e)=fixture();let before=e.document.clone();let owner=e.table;
    e.start_quick("m_iHCurrentObjective").unwrap();e.cancel_quick();assert_eq!(e.document,before);assert_eq!(e.table,owner);
    e.start_quick("m_iHCurrentObjective").unwrap();
    e.begin(Focus::Draft("m_pstrNameString".into()));e.edit="Новый текст".into();
    e.create_draft().unwrap();assert_eq!(e.table,owner);assert_eq!(e.undo.len(),1);
    assert_eq!(mission_canvas::stage_title(&e,owner,0),"Новый текст");
    assert_eq!(mission_canvas::stage_title(&e,owner,1),"Talk to Dexter");
    e.undo(false);assert_eq!(e.document,before);
    e.undo(true);assert_eq!(mission_canvas::stage_title(&e,owner,0),"Новый текст");
}

#[test]
fn quest_item_creation_preserves_other_slots_and_target_domain() {
    let (_dir,mut e)=fixture();let before=e.document.clone();let owner=e.table;
    e.begin_element("m_iCSUItemID".into(),1).unwrap();e.start_quick("m_iCSUItemID").unwrap();
    assert!(e.tables[e.table].label.contains("m_pQuestItemTable"));
    e.begin(Focus::NewName);e.edit="New beacon".into();e.create_draft().unwrap();
    assert_eq!(e.table,owner);assert_eq!(e.rows()[0]["m_iCSUItemID"],json!([0,2,0]));
    assert_eq!(e.rows()[1]["m_iCSUItemID"],json!([0,0,0]));
    e.undo(false);assert_eq!(e.document,before);
}

#[test]
fn stage_insertion_and_retargeting_keep_correct_identifiers_and_undo() {
    let (_dir,mut e)=fixture();e.connect_stage(0,"m_iSUOutgoingTask",1).unwrap();let before=e.document.clone();
    e.workspace.pending_link=Some((0,"m_iSUOutgoingTask".into()));
    e.workspace.selected_edge=Some((0,"m_iSUOutgoingTask".into(),0));
    let r=e.append_mission(false,false,"unused","Middle stage").unwrap();
    assert_eq!(e.rows()[0]["m_iSUOutgoingTask"],e.rows()[r]["m_iHTaskID"]);
    assert_eq!(e.rows()[r]["m_iSUOutgoingTask"],json!(11));
    assert_eq!(e.rows()[r]["m_iHMissionID"],json!(1));
    assert_eq!(mission_graph::ordered_stages(e.rows(),&[0,1,r]),vec![0,r,1]);
    assert_eq!(mission_canvas::scene(&e).nodes.iter().filter_map(|n|n.row).collect::<Vec<_>>(),vec![0,r,1]);
    assert_eq!(e.rows()[1]["m_iHTaskID"],json!(11));
    e.undo(false);assert_eq!(e.document,before);
    assert_eq!(e.row,Some(0));
    assert_eq!(mission_canvas::scene(&e).nodes.len(),2);
    assert!(e.connect_stage(1,"m_iSUOutgoingTask",0).is_err());
    assert!(e.connect_stage(1,"m_iFOutgoingTask",0).is_ok());
}

#[test]
fn description_search_and_reference_search_share_the_same_index() {
    let (_dir,mut e)=fixture();
    assert!(e.matches_search(e.table,0,"СЕКРЕТНЫЙ маяк"));
    let (npc,_)=e.reference_target("m_iHNPCID").unwrap();
    e.reference_search="исследователь".into();assert_eq!(e.reference_candidates(npc,"m_iHNPCID"),vec![0]);
    e.reference_search="17".into();assert_eq!(e.reference_candidates(npc,"m_iHNPCID"),vec![0]);
}

#[test]
fn canvas_positions_do_not_change_game_data_or_initial_stage() {
    let (_dir,mut e)=fixture();let before=e.document.clone();
    e.workspace.positions.insert("task:10".into(),[600.,800.]);
    let scene=mission_canvas::scene(&e);
    assert_eq!(scene.nodes.iter().find(|n|n.id==10).unwrap().position,Vec2::new(600.,800.));
    assert_eq!(e.document,before);assert_eq!(e.rows()[0]["m_iHTaskID"],json!(10));
    e.workspace.positions.insert("task:999".into(),[100.,200.]);
    e.workspace.positions.insert("mission:1".into(),[50.,70.]);
    let layout=e.workspace.positions.clone();
    e.arrange_mission_canvas().unwrap();
    assert!(!e.workspace.positions.contains_key("task:10"));
    assert_eq!(e.workspace.positions["task:999"],[100.,200.]);
    assert_eq!(e.workspace.positions["mission:1"],[50.,70.]);
    e.undo(false);assert_eq!(e.workspace.positions,layout);assert_eq!(e.document,before);
}

#[test]
fn localized_creation_save_undo_save_redo_keeps_aliases_in_step_with_data() {
    let (dir,mut e)=fixture();let before=e.document.clone();
    e.start_quick("m_iHCurrentObjective").unwrap();
    e.edit="Найти маяк {count}".into();assert!(e.apply());
    e.begin(Focus::Locale(0));e.edit="Find beacon {count}".into();
    e.create_draft().unwrap();assert_eq!(e.undo.len(),1);
    assert!(e.matches_search(e.table,0,"найти маяк"));
    e.save().unwrap();assert!(!e.dirty());
    let path=dir.path().join("assets/game/localization/ru.json");
    let saved:Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["entries"]["content.mission.task.10.objective"],json!("Найти маяк {count}"));
    e.undo(false);assert_eq!(e.document,before);e.save().unwrap();
    let saved:Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert!(saved["entries"].get("content.mission.task.10.objective").is_none());
    assert_eq!(saved["entries"]["unrelated"],json!("preserve"));
    e.undo(true);e.save().unwrap();
    let saved:Value=serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["entries"]["content.mission.task.10.objective"],json!("Найти маяк {count}"));
}

#[test]
fn localized_placeholder_and_external_conflicts_fail_before_table_changes() {
    let (dir,mut e)=fixture();let before=e.document.clone();
    e.start_quick("m_iHCurrentObjective").unwrap();
    e.edit="Найти {count}".into();assert!(e.apply());
    e.begin(Focus::Locale(0));e.edit="Find {wrong}".into();
    assert!(e.create_draft().is_err());assert_eq!(e.document,before);assert!(e.undo.is_empty());
    e.begin(Focus::Locale(0));e.edit="Find {count}".into();e.create_draft().unwrap();
    fs::write(dir.path().join("assets/game/localization/ru.json"),json!({"entries":{"content.mission.task.10.objective":"External change"}}).to_string()).unwrap();
    assert!(e.save().unwrap_err().contains("Localization conflict"));
    let disk:Value=serde_json::from_slice(&fs::read(&e.path).unwrap()).unwrap();
    assert_eq!(disk,before);assert!(e.dirty());
}

#[test]
fn arrows_persist_task_ids_and_mission_ids_in_their_own_domains() {
    let (dir, mut e) = fixture();
    let b = e.append_mission(true, false, "B", "B start").unwrap();
    let c = e.append_mission(true, false, "C", "C start").unwrap();
    e.connect_canvas(0, "m_iSUOutgoingTask", 1).unwrap();
    e.connect_canvas(0, mission_graph::REQUIRE, b).unwrap();
    e.connect_canvas(0, mission_graph::REQUIRE, c).unwrap();
    assert_eq!(e.rows()[0]["m_iSUOutgoingTask"], json!(11));
    assert_eq!(e.rows()[b][mission_graph::REQUIRE], json!([1, 0]));
    assert_eq!(e.rows()[c][mission_graph::REQUIRE], json!([1, 0]));
    let connected = e.document.clone();
    e.save().unwrap();
    let published: Value = serde_json::from_slice(&fs::read(&e.path).unwrap()).unwrap();
    assert_eq!(published["m_pMissionTable"]["m_pMissionData"][b][mission_graph::REQUIRE], json!([1, 0]));
    assert!(published.get("tables").is_none());
    let mut reopened = XdtEditor::open(dir.path().join("assets/game"))
        .with_selection(Some("m_pMissionTable/m_pMissionData"), Some(0), "");
    assert_eq!(reopened.document, connected);
    let scene = mission_canvas::scene(&reopened);
    assert_eq!(scene.nodes.iter().filter(|n| n.compact).count(), 2);
    assert_eq!(scene.edges.iter().filter(|edge| edge.link.field == mission_graph::REQUIRE).count(), 2);
    reopened.connect_stage(1, "m_iSUOutgoingTask", 0).unwrap_err();
    assert_eq!(reopened.document, connected);
}

#[test]
fn dependencies_retarget_atomically_and_reject_full_slots_or_cycles() {
    let (_dir, mut e) = fixture();
    let b = e.append_mission(true, false, "B", "B start").unwrap();
    let c = e.append_mission(true, false, "C", "C start").unwrap();
    let d = e.append_mission(true, false, "D", "D start").unwrap();
    e.connect_missions(0, b, None).unwrap();
    e.connect_missions(c, b, None).unwrap();
    let before = e.document.clone();
    assert!(e.connect_missions(d, b, None).unwrap_err().contains("occupied"));
    assert!(e.connect_missions(b, 0, None).unwrap_err().contains("cycle"));
    assert_eq!(e.document, before);
    e.connect_missions(0, b, None).unwrap(); // Repeated gestures never duplicate links.
    assert_eq!(e.document, before);
    e.retarget_canvas_edge(b, mission_graph::REQUIRE, 0, d).unwrap();
    assert_eq!(e.rows()[b][mission_graph::REQUIRE], json!([0, 3]));
    assert_eq!(e.rows()[d][mission_graph::REQUIRE], json!([1, 0]));
    e.undo(false);
    assert_eq!(e.document, before);
    e.undo(true);
    assert_eq!(e.rows()[d][mission_graph::REQUIRE], json!([1, 0]));
}

#[test]
fn removing_opening_link_clears_legacy_repeats_and_undo_restores_them() {
    let (_dir, mut e) = fixture();
    let b = e.append_mission(true, false, "B", "B start").unwrap();
    let later = e.append_mission(false, false, "B", "B later").unwrap();
    e.connect_missions(0, b, None).unwrap();
    let mut rows = e.rows().to_vec();
    rows[later][mission_graph::REQUIRE] = json!([1, 0]);
    e.change(rows).unwrap();
    let before = e.document.clone();
    assert_eq!(mission_dependencies::opening_edges(e.rows()).len(), 1);
    e.graph_remove(b, mission_graph::REQUIRE, 0).unwrap();
    for row in [b, later] { assert_eq!(e.rows()[row][mission_graph::REQUIRE], json!([0, 0])); }
    e.undo(false);
    assert_eq!(e.document, before);
}

#[test]
fn neighboring_mission_cards_bracket_stages_without_colliding_with_task_ids() {
    let (_dir, mut e) = fixture();
    e.connect_stage(0, "m_iSUOutgoingTask", 1).unwrap();
    let prerequisite = e.append_mission(true, false, "Prerequisite", "Start").unwrap();
    let successor = e.append_mission(true, false, "Successor", "Start").unwrap();
    let mut rows = e.rows().to_vec();
    rows[prerequisite]["m_iHMissionID"] = json!(10); // Same as current task ID.
    e.change(rows).unwrap();
    e.connect_missions(prerequisite, 0, None).unwrap();
    e.connect_missions(0, successor, None).unwrap();
    e.row = Some(0);
    let scene = mission_canvas::scene(&e);
    use mission_scene::Key;
    let at = |key| scene.nodes.iter().find(|n| n.key == key).unwrap();
    assert!(at(Key::Mission(10)).position.y + at(Key::Mission(10)).size().y < at(Key::Task(10)).position.y);
    assert!(at(Key::Mission(3)).position.y > at(Key::Task(11)).position.y + mission_canvas::NODE_SIZE.y);
    assert!(scene.edges.iter().any(|edge| edge.from == Key::Mission(10) && edge.to == Key::Task(10)));
    assert!(scene.edges.iter().any(|edge| edge.from == Key::Task(11) && edge.to == Key::Mission(3)));
}

#[test]
fn failed_arrow_retarget_keeps_the_original_dependency_and_history() {
    let (_dir, mut e) = fixture();
    let b = e.append_mission(true, false, "B", "Start").unwrap();
    let c = e.append_mission(true, false, "C", "Start").unwrap();
    let d = e.append_mission(true, false, "D", "Start").unwrap();
    e.connect_missions(0, b, None).unwrap();
    e.connect_missions(b, d, None).unwrap();
    e.connect_missions(c, d, None).unwrap();
    let before = e.document.clone();
    let history = e.undo.len();
    assert!(e.retarget_canvas_edge(b, mission_graph::REQUIRE, 0, d).is_err());
    assert_eq!(e.document, before);
    assert_eq!(e.undo.len(), history);
    assert_eq!(e.rows()[b][mission_graph::REQUIRE], json!([1, 0]));
}
