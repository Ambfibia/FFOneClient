//! Inspector fields stay in their sections while values and references are edited.
use super::*;
use view::{button, dynamic_button, field, label, property};

pub(super) fn stack() -> Node { Node {flex_direction:FlexDirection::Column,min_width:px(0),min_height:px(0),row_gap:px(6),..default()} }
pub(super) fn row() -> Node { Node {width:percent(100),flex_shrink:0.,column_gap:px(6),align_items:AlignItems::Center,flex_wrap:FlexWrap::Wrap,..default()} }
pub(super) fn action(p:&mut ChildSpawnerCommands,f:&EditorFonts,a:Action,key:&str,en:&str) {
    button(p,f,a,&format!("ui.editor.xdt.{key}"),en,false,0.);
}
pub(super) fn record_label(e:&XdtEditor,t:usize,r:usize,l:&Localization,lang:&Language)->String {
    let id = schema::identity(&e.tables[t].label);
    let value = e.document.pointer(&e.tables[t].pointer).and_then(|v|v.get(r));
    let number = id.and_then(|id|value.and_then(|v|v.get(id))).map(display).unwrap_or_else(||r.to_string());
    let kind = if id.is_some() {"ID".into()} else {presentation::tr(l,lang,"mission.index","index")};
    format!("{} ({kind}: {number})",e.record_name(t,r))
}
pub(super) fn value_label(e:&XdtEditor,col:&str,value:&Value,l:&Localization,lang:&Language)->String {
    if let Some(text)=schema::value_name(&e.tables[e.table].label,col,value,l,lang) {return text;}
    if let Some(values)=value.as_array() {
        let filled=values.iter().filter(|v| !authoring::neutral(v)).count();
        return format!("{filled} / {}",values.len());
    }
    if let Some((t,id))=e.reference_target(col) {
        if (id.is_some() || relations::optional_index(col)) && value.as_i64()==Some(0) {return presentation::tr(l,lang,"not_assigned","Not assigned");}
        let rows=e.document.pointer(&e.tables[t].pointer).and_then(Value::as_array);
        let target=rows.and_then(|rows|if let Some(id)=id {rows.iter().position(|r|r.get(id)==Some(value))}
            else {value.as_u64().map(|n|n as usize).filter(|n|*n<rows.len())});
        return target.map(|r|record_label(e,t,r,l,lang)).unwrap_or_else(||format!("{} ({})",presentation::tr(l,lang,"graph.missing","Missing record"),display(value)));
    }
    display(value)
}

pub(super) fn draw_field(p:&mut ChildSpawnerCommands,f:&EditorFonts,e:&XdtEditor,l:&Localization,lang:&Language,col:&str,value:&Value) {
    let active=e.picker_field.as_deref()==Some(col);
    let focus=if e.draft.is_some(){Focus::Draft(col.into())}else{Focus::Cell(col.into())};
    p.spawn((mission_canvas::Hit::Field(col.into()),Node {width:percent(100),flex_shrink:0.,..stack()})).with_children(|p|{
        p.spawn(row()).with_children(|line|{
            line.spawn(Node {flex_grow:1.,flex_basis:px(0),min_width:px(0),..default()}).with_children(|p|
                property(p,f,Action::Field(focus.clone()),schema::field_name(col,l,lang),value_label(e,col,value,l,lang),active));
            if value.is_number() && e.reference_target(col).is_some_and(|(t,_)|t!=e.table) {
                button(line,f,Action::Mission(e.quick_create_command(col)),"ui.editor.xdt.mission.plus","+",false,32.);
            }
        });
        if !active {return;}
        if let Some(values)=value.as_array() {
            for (slot,item) in values.iter().enumerate() {
                property(p,f,Action::ArrayCell(col.into(),slot),format!("{} {}",presentation::tr(l,lang,"list_element","Element"),slot+1),value_label(e,col,item,l,lang),e.array_slot==Some(slot));
                if e.array_slot==Some(slot) {draw_editor(p,f,e,l,lang,col,item);}
            }
            if e.array_slot.is_none() {action(p,f,Action::CancelEdit,"close_field","Close field");}
        } else {draw_editor(p,f,e,l,lang,col,value);}
    });
}

fn draw_editor(p:&mut ChildSpawnerCommands,f:&EditorFonts,e:&XdtEditor,l:&Localization,lang:&Language,col:&str,value:&Value) {
    let tr=|key: &str,en: &str|presentation::tr(l,lang,key,en);
    let options=schema::choices(&e.tables[e.table].label,col);
    if !options.is_empty() {
        for (code,name) in options {
            dynamic_button(p,f,Action::Choice(col.into(),*code),format!("{} ({code})",tr(&format!("attribute.{name}"),name)),value.as_i64()==Some(*code),0.);
        }
        action(p,f,Action::CancelEdit,"close_field","Close field");
        return;
    }
    if let Some((target,id))=e.reference_target(col) {
        p.spawn(row()).with_children(|line|{
            line.spawn(Node {flex_grow:1.,min_width:px(0),..default()}).with_children(|p|field(p,f,e,Focus::ReferenceSearch,36.));
            if target!=e.table && e.array_slot.is_some() {
                button(line,f,Action::Mission(e.quick_create_command(col)),"ui.editor.xdt.mission.plus","+",false,32.);
            }
        });
        if id.is_some() || relations::optional_index(col) { action(p,f,Action::ClearReference(col.into()),"clear_reference","No linked record"); }
        let candidates=e.reference_candidates(target,col);
        p.spawn((ScrollRegion(4),Node {height:px(160),overflow:Overflow::clip(),..stack()})).with_children(|p|{
            if candidates.is_empty() {label(p,f,tr("mission.no_results","No matching records"),14.,Color::srgb(0.7,0.75,0.78));}
            for r in candidates.into_iter().skip(e.reference_offset).take(4) {
                dynamic_button(p,f,Action::PickReference(col.into(),target,r),record_label(e,target,r,l,lang),false,0.);
            }
        });
        if mission_journal::is_link(col) && value.as_i64().is_some_and(|id|id>0) {
            action(p,f,Action::Mission(mission_workspace::Command::EditJournal(col.into())),"mission.edit_journal","Edit journal entry");
        }
        if let Some(text)=e.text_target(col).filter(|_| !relations::optional_index(col) || value.as_i64().is_some_and(|id|id>0)) {
            label(p,f,format!("{}: {}",tr("text_users","Linked records"),text.users),13.,Color::srgb(0.6,0.76,0.74));
            action(p,f,Action::CopyText(col.into()),"copy_text","Create a private text copy");
            action(p,f,Action::EditSharedText(col.into()),"edit_shared","Edit linked text");
        }
        action(p,f,Action::CancelEdit,"close_field","Close field");
        return;
    }
    if let Some(focus)=&e.focus {
        field(p,f,e,focus.clone(),if value.is_string(){88.}else{36.});
        if let Some(limits)=schema::limits(&e.tables[e.table].label,col) {
            label(p,f,format!("{}: {}…{}",tr("field_range","Allowed range"),limits.min,limits.max.map(|n|n.to_string()).unwrap_or_else(||"∞".into())),12.,Color::srgb(0.65,0.74,0.77));
        }
        p.spawn(row()).with_children(|p|{
            button(p,f,Action::Apply,"ui.editor.xdt.apply","Apply",false,125.);
            button(p,f,Action::CancelEdit,"ui.editor.xdt.cancel","Cancel",false,100.);
        });
    }
    if !e.status.is_empty() {label(p,f,schema::status(&e.status,l,lang),13.,Color::srgb(1.,0.63,0.45));}
}

pub(super) fn draft(p:&mut ChildSpawnerCommands,f:&EditorFonts,e:&XdtEditor,l:&Localization,lang:&Language) {
    let Some(draft)=&e.draft else{return};
    let shared=e.workspace.quick_existing.is_some();
    let journal=e.tables[e.table].label.ends_with("/m_pJournalData");
    label(p,f,presentation::tr(l,lang,if shared&&journal{"mission.edit_journal"}else if shared{"edit_shared"}else if draft.placeholder{"new_placeholder"}else{"mission.create_use"},if shared&&journal{"Edit journal entry"}else if shared{"Edit linked text"}else if draft.placeholder{"Create placeholder"}else{"Create and use"}),19.,mission_skin::CYAN);
    label(p,f,schema::table_name(&e.tables[e.table].label,l,lang),14.,Color::srgb(0.7,0.77,0.8));
    if schema::identity(&e.tables[e.table].label).is_none() {
        let id=e.workspace.quick_existing.unwrap_or(e.rows().len());
        label(p,f,format!("{}: {id}",presentation::tr(l,lang,if journal{"mission.journal_id"}else{"mission.string_id"},if journal{"Journal ID"}else{"Text ID"})),14.,mission_skin::BLUE);
    }
    if e.workspace.quick_text.is_some() {
        for (index,locale) in ["EN","RU"].into_iter().enumerate(){
            label(p,f,locale,13.,Color::srgb(0.64,0.77,0.8));
            field(p,f,e,Focus::Locale(index),72.);
        }
        label(p,f,presentation::tr(l,lang,"mission.locale_fallback","An empty translation uses the other language."),12.,Color::srgb(0.64,0.72,0.77));
    } else if draft.name.is_some() {field(p,f,e,Focus::NewName,40.);}
    label(p,f,format!("{}: {}",presentation::tr(l,lang,"template","Settings copied from"),e.record_name(e.table,draft.template)),12.,Color::srgb(0.64,0.72,0.77));
    for col in schema::ordered_fields(draft.value.as_object().into_iter().flat_map(|o|o.keys().cloned())) {
        if draft.name.as_ref().is_some_and(|n|n.field==col) {continue;}
        if e.workspace.quick_text.is_some()&&matches!(col.as_str(),"m_pstrNameString"|"m_strName"){continue;}
        if !draft.required.contains(&col) && !e.advanced && !e.tables[e.table].label.ends_with("/m_pRewardData") {continue;}
        let value=&draft.value[&col];
        if schema::identity(&e.tables[e.table].label)==Some(col.as_str()) {
            label(p,f,format!("ID: {}",display(value)),14.,Color::srgb(0.6,0.77,0.78));
        } else {draw_field(p,f,e,l,lang,&col,value);}
    }
    for (field,reason) in e.draft_errors() {
        label(p,f,format!("{}: {}",schema::field_name(&field,l,lang),presentation::tr(l,lang,&format!("error.{reason}"),&reason)),13.,Color::srgb(1.,0.65,0.48));
    }
    action(p,f,Action::Create,if shared{"apply"}else{"mission.create_use"},if shared{"Apply"}else{"Create and use"});
    action(p,f,Action::Cancel,"cancel","Cancel");
}
