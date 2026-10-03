//! Localized inline text drafts and recoverable, conflict-aware locale saving.
use super::*;

#[derive(Default)]
pub(super) struct TextDraft { pub values:[String;2] }

impl XdtEditor {
    pub(super) fn start_mission_text_edit(&mut self,field:&str,shared:bool)->Result<(),String>{
        let (table,id)=self.reference_target(field).ok_or("Missing text table")?;
        if id.is_some(){return Err("Select a text reference".into());}
        let row=self.field_value(field).and_then(Value::as_u64).ok_or("Missing text index")? as usize;
        if row == 0 && relations::optional_index(field) {
            return self.start_quick(field);
        }
        let value=self.document.pointer(&self.tables[table].pointer).and_then(|v|v.get(row)).cloned().ok_or("Missing text record")?;
        let fallback=value["m_pstrNameString"].as_str().or_else(||value["m_strName"].as_str()).unwrap_or_default().to_owned();
        self.start_quick(field)?;
        self.workspace.quick_existing=shared.then_some(row);
        self.draft.as_mut().unwrap().value=value;
        let key=format!("content.tabledata.mission.mission_string.{row}.str_name_string");
        let values=std::array::from_fn(|i|self.workspace.locale_drafts.get(&key).and_then(|v|v[i].clone())
            .or_else(||self.workspace.locale_base[i].get(&key).cloned()).unwrap_or_else(||fallback.clone()));
        self.workspace.quick_text=Some(TextDraft{values});
        self.begin(Focus::Locale(1));
        Ok(())
    }
    pub(super) fn apply_locale_text(&mut self,index:usize) -> bool {
        let Some(text)=self.workspace.quick_text.as_mut()else{return false};
        text.values[index]=self.edit.clone();
        let fallback=if text.values[0].trim().is_empty(){text.values[1].clone()}else{text.values[0].clone()};
        if let Some(draft)=self.draft.as_mut(){
            if let Some(name)=draft.name.as_mut(){name.text=fallback;}
            else if draft.value.get("m_pstrNameString").is_some(){draft.value["m_pstrNameString"]=Value::from(fallback);}
            else if draft.value.get("m_strName").is_some(){draft.value["m_strName"]=Value::from(fallback);}
        }
        self.focus=None;self.revision+=1;true
    }
    pub(super) fn quick_text_aliases(&self,value:&Value,row:usize)->Vec<String> {
        let table=&self.tables[self.table].label;
        let mut keys=Vec::new();
        if table.ends_with("/m_pMissionStringData") {
            keys.push(format!("content.tabledata.mission.mission_string.{row}.str_name_string"));
            if let Some(origin)=&self.workspace.quick {
                let task=self.document.pointer(&self.tables[origin.table].pointer).and_then(|v|v.get(origin.row)).and_then(|v|v["m_iHTaskID"].as_i64());
                let field=match origin.field.as_str(){"m_iHMissionName"=>Some("title"),"m_iHCurrentObjective"=>Some("objective"),_=>None};
                if let (Some(task),Some(field))=(task,field){keys.push(format!("content.mission.task.{task}.{field}"));}
            }
            if self.workspace.quick_existing.is_some(){
                for link in self.references.iter().filter(|r|r.target_table==self.table&&r.target_row==Some(row)) {
                    let field=match link.field.as_str(){"m_iHMissionName"=>"title","m_iHCurrentObjective"=>"objective",_=>continue};
                    if let Some(task)=self.document.pointer(&self.tables[link.table].pointer).and_then(|v|v.get(link.row)).and_then(|v|v["m_iHTaskID"].as_i64()){
                        keys.push(format!("content.mission.task.{task}.{field}"));
                    }
                }
            }
        }else if table.ends_with("/m_pNpcData") {
            if let Some(id)=value["m_iNpcNumber"].as_i64(){keys.push(format!("content.npc.{id}.name"));}
        }else if table.ends_with("/m_pQuestItemTable/m_pItemData") {
            if let Some(id)=value["m_iItemNumber"].as_i64(){keys.push(format!("content.quest_item.{id}.name"));}
        }
        keys
    }
    pub(super) fn attach_text_history(&mut self,keys:Vec<String>,mut values:[String;2])->Result<(),String> {
        if keys.is_empty(){return Ok(());}
        if values[0].is_empty(){values[0]=values[1].clone();}
        if values[1].is_empty(){values[1]=values[0].clone();}
        let tokens=|text:&str|text.split('{').skip(1).filter_map(|p|p.split_once('}')).map(|(key,_)|key.to_owned()).collect::<Vec<_>>();
        let mut en=tokens(&values[0]);let mut ru=tokens(&values[1]);en.sort();ru.sort();
        if en!=ru{return Err("Text placeholders must match in EN and RU".into());}
        let mut before=BTreeMap::new();
        let mut after=BTreeMap::new();
        for key in keys {
            before.insert(key.clone(),self.workspace.locale_drafts.get(&key).cloned().unwrap_or_else(||
                std::array::from_fn(|i|self.workspace.locale_base[i].get(&key).cloned())));
            let value=values.clone().map(Some);
            after.insert(key.clone(),value.clone());
            self.workspace.locale_drafts.insert(key,value);
        }
        if let Some(change)=self.undo.last_mut(){
            if change.pointer.is_empty(){
                change.pointer="@mission-localized".into();
                change.before=serde_json::json!({"document":change.before,"locales":before});
                change.after=serde_json::json!({"document":change.after,"locales":after});
            }
        }
        self.rebuild_search_index();
        Ok(())
    }
    pub(super) fn validate_text_draft(&self)->Result<(),String>{
        let Some(draft)=&self.workspace.quick_text else{return Ok(())};
        if draft.values.iter().any(|v|v.is_empty()){return Ok(());}
        let tokens=|text:&str|{let mut result:Vec<_>=text.split('{').skip(1).filter_map(|s|s.split_once('}')).map(|(s,_)|s.to_owned()).collect();result.sort();result};
        if tokens(&draft.values[0])!=tokens(&draft.values[1]){return Err("Text placeholders must match in EN and RU".into());}
        Ok(())
    }
    pub(super) fn prepare_locales(&self)->Result<Vec<(PathBuf,Value)>,String>{
        if self.workspace.locale_drafts.is_empty(){return Ok(Vec::new());}
        let root=self.path.parent().and_then(Path::parent).and_then(Path::parent).ok_or("Missing asset root")?;
        let mut result=Vec::new();
        for (index,locale) in ["en","ru"].into_iter().enumerate(){
            let path=root.join(format!("localization/{locale}.json"));
            let mut value:Value=serde_json::from_slice(&fs::read(&path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
            let entries=value["entries"].as_object_mut().ok_or("Missing localization entries")?;
            for (key,text) in &self.workspace.locale_drafts{
                let current=entries.get(key).and_then(Value::as_str);
                let base=self.workspace.locale_base[index].get(key).map(String::as_str);
                if current!=base&&current!=text[index].as_deref(){return Err(format!("Localization conflict: {locale} {key}"));}
                if let Some(text)=&text[index]{entries.insert(key.clone(),Value::from(text.clone()));}
                else{entries.remove(key);}
            }
            result.push((path,value));
        }
        Ok(result)
    }
    pub(super) fn save_locales(&mut self,prepared:Vec<(PathBuf,Value)>)->Result<(),String>{
        // All conflicts were checked before TableData is written. Successful files are
        // remembered so a partial disk failure can be retried without duplicate records.
        for (index,(path,value)) in prepared.into_iter().enumerate(){
            let mut temp=tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e|e.to_string())?;
            serde_json::to_writer_pretty(&mut temp,&value).map_err(|e|e.to_string())?;
            temp.write_all(b"\n").map_err(|e|e.to_string())?;
            temp.as_file().sync_all().map_err(|e|e.to_string())?;
            temp.persist(&path).map_err(|e|format!("Localization save incomplete ({}): {e}",path.display()))?;
            self.workspace.locale_base[index]=value["entries"].as_object().unwrap().iter().filter_map(|(k,v)|v.as_str().map(|v|(k.clone(),v.into()))).collect();
        }
        self.workspace.locale_drafts.clear();
        self.rebuild_search_index();
        Ok(())
    }
}
