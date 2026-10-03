//! One search contract for catalog records and inline reference pickers.
use super::*;

impl XdtEditor {
    pub(super) fn rebuild_search_index(&mut self) {
        self.workspace.locale_text.clear();
        for index in 0..2 {
            for (key,value) in &self.workspace.locale_base[index] {
                if key.starts_with("content.")&&!self.workspace.locale_drafts.contains_key(key){
                    self.workspace.locale_text.entry(key.clone()).or_default().push_str(&format!(" {value}"));
                }
            }
        }
        for (key,values) in &self.workspace.locale_drafts {
            for value in values.iter().flatten(){self.workspace.locale_text.entry(key.clone()).or_default().push_str(&format!(" {value}"));}
        }
        let mut index:Vec<Vec<String>>=self.tables.iter().enumerate().map(|(t,table)|{
            self.document.pointer(&table.pointer).and_then(Value::as_array).into_iter().flatten().enumerate().map(|(r,value)|{
                let mut text=format!("{r} {} {}",self.record_name(t,r),value);
                if table.label.ends_with("/m_pMissionStringData") {
                    let key=format!("content.tabledata.mission.mission_string.{r}.str_name_string");
                    if let Some(translation)=self.workspace.locale_text.get(&key) {text.push(' ');text.push_str(translation);}
                }
                let domain=if table.label.ends_with("/m_pMissionData"){Some(("mission.task","m_iHTaskID"))}
                    else if table.label.ends_with("/m_pNpcData"){Some(("npc","m_iNpcNumber"))}
                    else if table.label.contains("/m_pQuestItemTable/")&&table.label.ends_with("/m_pItemData"){Some(("quest_item","m_iItemNumber"))}else{None};
                if let Some((domain,id))=domain {
                    if let Some(id)=value[id].as_i64(){
                        let prefix=format!("content.{domain}.{id}.");
                        for (_,translation) in self.workspace.locale_text.range(prefix.clone()..).take_while(|(key,_)|key.starts_with(&prefix)) {
                            text.push(' ');text.push_str(translation);
                        }
                    }
                }
                text.to_lowercase()
            }).collect()
        }).collect();
        // Texts first, then journals: mission -> journal -> localized text.
        for journals in [false,true] {
        for link in &self.references {
            let Some(target)=link.target_row else{continue};
            let leaf=if journals{"/m_pJournalData"}else{"StringData"};
            if self.tables[link.target_table].label.ends_with(leaf) {
                if let Some(value)=index.get(link.target_table).and_then(|v|v.get(target)).cloned() {
                    if let Some(text)=index.get_mut(link.table).and_then(|v|v.get_mut(link.row)) {
                        text.push(' ');text.push_str(&value);
                    }
                }
            }
        }
        }
        self.workspace.index=index;
    }
    pub(super) fn matches_search(&self,table:usize,row:usize,query:&str)->bool {
        let query=query.to_lowercase();
        let text=self.workspace.index.get(table).and_then(|v|v.get(row));
        query.split_whitespace().all(|word|text.is_some_and(|text|text.contains(word)))
    }
    pub(super) fn search_rank(&self,table:usize,row:usize,query:&str)->u8 {
        let query=query.trim().to_lowercase();
        if query.is_empty(){return 3;}
        let value=self.document.pointer(&self.tables[table].pointer).and_then(|v|v.get(row));
        let id=schema::identity(&self.tables[table].label).and_then(|id|value.and_then(|v|v.get(id))).map(display).unwrap_or_else(||row.to_string());
        if id==query{return 0;}
        let name=self.record_name(table,row).to_lowercase();
        if name.starts_with(&query){1}else if name.contains(&query){2}else{3}
    }
}
