//! Quest NPC roles use accepted native TableData templates.
use super::*;
fn mob_row(row:&Value)->bool{row["m_iNpcType"].as_i64()==Some(0)&&row["m_iHP"].as_i64().unwrap_or(0)>1&&(row["m_iAiType"].as_i64().unwrap_or(0)>0||matches!(row["m_iNpcStyle"].as_i64(),Some(1|2)))}

#[derive(Clone, Copy, Debug)]
pub(super) enum NpcTemplate { Area, Interactable, Character, Mob, Empty }

impl NpcTemplate {
    fn source(self) -> i64 {
        match self { Self::Area | Self::Empty => 1401, Self::Interactable => 870, Self::Character => 2564, Self::Mob => 55 }
    }
}

impl XdtEditor {
    pub(super) fn copy_mob_stats(&mut self, source:usize)->Result<(),String> {
        let row=self.rows().get(source).cloned().ok_or("Missing mob template")?;
        let table=self.tables[self.table].label.clone();
        let draft=self.draft.as_mut().ok_or("Duplicate or create an NPC first")?;
        for (field,value) in row.as_object().into_iter().flatten() {
            if field=="m_iNpcLevel"||authoring::group(&table,field)==Some("combat") {draft.value[field]=value.clone();}
        }
        self.workspace.mob_level_picker=false;self.focus=None;self.revision+=1;Ok(())
    }
    pub(crate) fn create_npc_from_catalog(&mut self) {
        let result = (|| {
            if self.draft.is_some() { return Err("Finish or cancel the new record first".to_owned()); }
            let table = self.tables.iter().position(|t| t.label.ends_with("/m_pNpcTable/m_pNpcData"))
                .ok_or("Select the NPC table")?;
            self.select_table(table);
            self.workspace.enabled = false;
            self.mission_filter = None;
            self.start_placeholder_draft()
        })();
        if let Err(error) = result { self.status = error; }
        self.revision += 1;
    }
    pub(super) fn choose_npc_template(&mut self, kind: NpcTemplate) -> Result<(), String> {
        if !self.tables[self.table].label.ends_with("/m_pNpcTable/m_pNpcData") {
            return Err("Select the NPC table".into());
        }
        let template = self.rows().iter().position(|row| row["m_iNpcNumber"].as_i64() == Some(kind.source()))
            .ok_or_else(|| format!("Missing NPC template: {}", kind.source()))?;
        let mut value = self.rows()[template].clone();
        if matches!(kind, NpcTemplate::Empty) {
            value = mission_fields::neutral(&value);
            for (field, number) in [("m_iMesh", 256), ("m_iNpcType", 111), ("m_iHP", 1),
                ("m_iHeight", 120), ("m_iRadius", 90)] { value[field] = Value::from(number); }
            for field in ["m_fScale", "m_fAnimationSpeed", "m_fRunAnimationSpeed", "m_fWalkAnimationSpeed"] {
                value[field] = Value::from(1.0);
            }
        }
        let draft = self.draft.as_mut().ok_or("No creation draft")?;
        for field in ["m_iNpcNumber", "m_iNpcName"] { value[field] = draft.value[field].clone(); }
        draft.value = value;
        if draft.value.get("m_iComment").is_some() {draft.required.insert("m_iComment".into());}
        draft.template = template;
        draft.placeholder = matches!(kind, NpcTemplate::Area | NpcTemplate::Empty);
        self.focus = None;
        self.status.clear();
        self.revision += 1;
        Ok(())
    }
}

pub(super) fn choices(p: &mut ChildSpawnerCommands, f: &EditorFonts, e: &XdtEditor,
    l: &Localization, lang: &Language) {
    if e.workspace.quick_existing.is_some() || !e.tables[e.table].label.ends_with("/m_pNpcTable/m_pNpcData") { return; }
    view::label(p, f, presentation::tr(l, lang, "npc_template.role", "Quest NPC role"), 14., mission_skin::BLUE);
    for (kind, key, label) in [
        (NpcTemplate::Area, "npc_template.area", "Area marker · Location A256"),
        (NpcTemplate::Interactable, "npc_template.interactable", "Interactive object · Protocore"),
        (NpcTemplate::Character, "npc_template.character", "Character · Ranger Melissa"),
        (NpcTemplate::Mob, "npc_template.mob", "Mob · Fusion Spawn"),
        (NpcTemplate::Empty, "npc_template.empty", "Empty · fill manually"),
    ] {
        view::button(p, f, Action::Mission(mission_workspace::Command::NpcTemplate(kind)),
            &format!("ui.editor.xdt.{key}"), label, false, 0.);
    }
    view::label(p, f, presentation::tr(l, lang, "npc_template.placement",
        "Place the new NPC in the server NPCs.json before saving its mission."), 12., mission_skin::MUTED);
    view::button(p,f,Action::MobLevels,"ui.editor.xdt.npc_template.level","Choose level / copy mob stats",e.workspace.mob_level_picker,0.);
    if e.workspace.mob_level_picker {
        if let Some(level)=e.workspace.mob_level {
            for (i,row) in e.rows().iter().enumerate().filter(|(_,r)|r["m_iNpcLevel"].as_i64()==Some(level)&&mob_row(r)) {
                view::dynamic_button(p,f,Action::MobStats(i),format!("{} · HP {} · {}",row["m_iNpcNumber"],row["m_iHP"],e.record_name(e.table,i)),false,0.);
            }
        } else {
            let levels:BTreeSet<_>=e.rows().iter().filter(|r|mob_row(r)).filter_map(|r|r["m_iNpcLevel"].as_i64()).collect();
            for level in levels {view::dynamic_button(p,f,Action::MobLevel(level),format!("{} {level}",presentation::tr(l,lang,"npc_template.level_short","Level")),false,0.);}
        }
    }
}
