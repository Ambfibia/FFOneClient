//! Reference traversal includes technical names, quest items, journals and message channels.
use super::*;

fn snake(name:&str)->String {
    let mut out=String::new();
    for (i,c) in name.chars().enumerate(){if c.is_uppercase()&&i>0{out.push('_');}out.extend(c.to_lowercase());}
    out
}
impl XdtEditor {
    fn text_keys(&self,table:usize,row:usize,keys:&BTreeMap<String,String>)->BTreeSet<String> {
        let t=&self.tables[table];
        let value=&self.document.pointer(&t.pointer).unwrap()[row];
        let parts:Vec<_>=t.label.split('/').collect();
        let leaf=parts.last().copied().unwrap_or("").trim_start_matches("m_p").trim_end_matches("Data");
        let group=parts.get(parts.len().saturating_sub(2)).copied().unwrap_or("").trim_start_matches("m_p").trim_end_matches("Table");
        let mut prefixes=vec![format!("content.tabledata.{}.{}.{row}.",snake(group),snake(leaf))];
        if t.label.ends_with("/m_pNpcData") {if let Some(id)=value["m_iNpcNumber"].as_i64(){prefixes.push(format!("content.npc.{id}."));}}
        if t.label.ends_with("/m_pMissionData") {if let Some(id)=value["m_iHTaskID"].as_i64(){prefixes.push(format!("content.mission.task.{id}."));}}
        if t.label.contains("/m_pQuestItemTable/")&&t.label.ends_with("/m_pItemData") {if let Some(id)=value["m_iItemNumber"].as_i64(){prefixes.push(format!("content.quest_item.{id}."));}}
        prefixes.into_iter().flat_map(|p|keys.range(p.clone()..).take_while(move |(k,_)|k.starts_with(&p)).map(|(k,_)|k.clone())).collect()
    }
    pub(crate) fn related_string_keys(&self,mode:u8,id:i64,keys:&BTreeMap<String,String>)->BTreeSet<String> {
        let mut pending=Vec::new();
        for (t,table) in self.tables.iter().enumerate(){
            let field=if mode==1&&table.label.ends_with("/m_pMissionData"){"m_iHMissionID"}
                else if mode==2&&table.label.ends_with("/m_pNpcData"){"m_iNpcNumber"}else{continue;};
            for (r,row) in self.document.pointer(&table.pointer).and_then(Value::as_array).into_iter().flatten().enumerate(){if row[field].as_i64()==Some(id){pending.push((t,r));}}
        }
        if mode==2 {
            pending.extend(self.references.iter().filter(|link|link.value==id&&self.tables[link.table].label.ends_with("/m_pMissionData")&&self.tables[link.target_table].label.ends_with("/m_pNpcData")).map(|link|(link.table,link.row)));
        }
        let mut visited=BTreeSet::new();let mut result=BTreeSet::new();
        while let Some(record)=pending.pop(){
            if !visited.insert(record){continue;}
            result.extend(self.text_keys(record.0,record.1,keys));
            pending.extend(self.references.iter().filter(|link|(link.table,link.row)==record&&!self.tables[link.target_table].label.ends_with("/m_pMissionData")).filter_map(|link|link.target_row.map(|r|(link.target_table,r))));
        }
        result
    }
    pub(crate) fn string_speakers(&self,keys:&BTreeMap<String,String>)->BTreeMap<String,i64> {
        let mut speakers:BTreeMap<String,BTreeSet<i64>>=BTreeMap::new();
        for link in &self.references {
            let Some(target)=link.target_row else{continue;};
            let owner=&self.document.pointer(&self.tables[link.table].pointer).unwrap()[link.row];
            let npc=if let Some(slot)=link.field.strip_prefix("m_iMentorEmailID[").and_then(|s|s.strip_suffix(']')).and_then(|s|s.parse::<usize>().ok()){[707,728,731,732,730].get(slot).copied()}
                else if let Some(field)=mission_events::speaker(&link.field){owner[field].as_i64()}
                else if self.tables[link.table].label.ends_with("/m_pNpcData")&&matches!(link.field.as_str(),"m_iNpcName"|"m_iComment"|"m_iBarkerNumber"){owner["m_iNpcNumber"].as_i64()}else{None};
            let Some(npc)=npc.filter(|id|*id>0) else{continue;};
            for key in self.text_keys(link.target_table,target,keys){speakers.entry(key).or_default().insert(npc);}
        }
        speakers.into_iter().filter_map(|(key,npcs)|(npcs.len()==1).then(||(key,*npcs.first().unwrap()))).collect()
    }
    pub(crate) fn npc_reference_count(&self,id:i64)->usize {
        self.references.iter().filter(|r|r.value==id&&self.tables[r.target_table].label.ends_with("/m_pNpcData")).map(|r|(r.table,r.row)).collect::<BTreeSet<_>>().len()
    }
}
