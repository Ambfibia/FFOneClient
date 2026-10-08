//! The NPC viewer edits the shared XDT document through the existing field forms.
use super::*;

impl XdtEditor {
    pub(crate) fn finish_npc_field(&mut self) -> bool {
        if self.draft.is_some() {
            self.status = "Finish or cancel the new record first".into();
            self.revision += 1;
            return false;
        }
        if !self.apply() {
            return false;
        }
        self.focus = None;
        self.picker_field = None;
        self.array_slot = None;
        self.revision += 1;
        true
    }
    fn inspect_npc(&mut self, id: i64) -> Result<(), String> {
        if self.draft.is_some() {
            return Ok(());
        }
        let table = self
            .tables
            .iter()
            .position(|t| t.label.ends_with("/m_pNpcTable/m_pNpcData"))
            .ok_or("Missing NPC table")?;
        let row = self
            .document
            .pointer(&self.tables[table].pointer)
            .and_then(Value::as_array)
            .and_then(|rows| {
                rows.iter()
                    .position(|v| v["m_iNpcNumber"].as_i64() == Some(id))
            })
            .ok_or("Missing NPC type in XDT")?;
        if self.table == table && self.row == Some(row) {
            return Ok(());
        }
        if !self.finish_npc_field() {
            return Err(self.status.clone());
        }
        self.select_table(table);
        self.row = Some(row);
        self.workspace.enabled = false;
        self.column_search.clear();
        self.rebuild_links();
        self.revision += 1;
        Ok(())
    }
}

pub(super) fn select(
    state: Res<EditorState>,
    catalog: Res<EditorCatalog>,
    mut e: ResMut<XdtEditor>,
    appearance: Res<hnpc::HnpcEditor>,
    icons: Res<icon_generator::IconGenerator>,
) {
    if !state.npc_editing() || appearance.active() || icons.active {
        return;
    }
    if let Some(id) = catalog
        .entries
        .get(state.selected)
        .and_then(|p| p.network_id)
    {
        if let Err(error) = e.inspect_npc(id) {
            if e.status != error {
                e.status = error;
                e.revision += 1;
            }
        }
    }
}

pub(super) fn draw(
    mut commands: Commands,
    state: Res<EditorState>,
    e: Res<XdtEditor>,
    catalog: Res<EditorCatalog>,
    fonts: Option<Res<EditorFonts>>,
    l: Res<Localization>,
    lang: Res<Language>,
    mut panels: Query<(Entity, &mut Node, Option<&Children>), With<NpcEditSection>>,
    mut tabs: Query<&mut Node, (With<NpcInspectorTabs>, Without<NpcEditSection>)>,
    mut shown: Local<Option<(Entity, usize, NpcInspectorTab, u64, String)>>,
) {
    for mut node in &mut tabs {
        node.display = if state.kind == CatalogKind::Npc {
            Display::Flex
        } else {
            Display::None
        };
    }
    let active = state.npc_editing();
    for (_, mut node, _) in &mut panels {
        node.display = if active { Display::Flex } else { Display::None };
    }
    if !active {
        *shown = None;
        return;
    }
    let Some(f) = fonts else {
        return;
    };
    let Some((panel, _, _)) = panels.iter().next() else {
        *shown = None;
        return;
    };
    let next = (
        panel,
        state.selected,
        state.npc_inspector,
        e.revision,
        lang.effective.clone(),
    );
    if shown.as_ref() == Some(&next) {
        return;
    }
    *shown = Some(next);
    for (panel, _, children) in &panels {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }
        commands.entity(panel).with_children(|p| {
            use mission_inline::{action, draw_field, row};
            use view::{button, field, label};
            let tr = |key: &str, en: &str| presentation::tr(&l, &lang, key, en);
            p.spawn(row()).with_children(|p| {
                button(
                    p,
                    &f,
                    Action::Save,
                    "ui.editor.xdt.save",
                    "Save work",
                    false,
                    144.,
                );
                button(
                    p,
                    &f,
                    Action::Rewrite,
                    "ui.editor.xdt.rewrite",
                    "Rewrite",
                    false,
                    144.,
                );
                button(
                    p,
                    &f,
                    Action::Undo,
                    "ui.editor.xdt.undo",
                    "Undo",
                    false,
                    144.,
                );
                button(
                    p,
                    &f,
                    Action::Redo,
                    "ui.editor.xdt.redo",
                    "Redo",
                    false,
                    144.,
                );
            });
            if !e.status.is_empty() {
                label(
                    p,
                    &f,
                    schema::status(&e.status, &l, &lang),
                    13.,
                    mission_skin::MUTED,
                );
            }
            if e.draft.is_some() {
                mission_inline::draft(p, &f, &e, &l, &lang);
                return;
            }
            let Some(row) = e.row.and_then(|i| e.rows().get(i)).filter(|r| {
                r["m_iNpcNumber"].as_i64() == catalog.entries[state.selected].network_id
            }) else {
                label(
                    p,
                    &f,
                    tr(
                        "select_help",
                        "Select a record to edit its parameters and see what uses it.",
                    ),
                    14.,
                    mission_skin::MUTED,
                );
                return;
            };
            label(
                p,
                &f,
                e.record_name(e.table, e.row.unwrap()),
                18.,
                mission_skin::CYAN,
            );
            if e.hnpc_value().is_some() {
                action(
                    p,
                    &f,
                    Action::Appearance,
                    "hnpc.appearance",
                    "HNPC appearance",
                );
            }
            action(p,&f,Action::Duplicate,"npc_template.duplicate","Duplicate NPC / mob");
            if row.get("m_iComment").is_some() {
                let caption =
                    mission_inline::value_label(&e, "m_iComment", &row["m_iComment"], &l, &lang);
                label(
                    p,
                    &f,
                    format!("{}: {caption}", schema::field_name("m_iComment", &l, &lang)),
                    14.,
                    mission_skin::BLUE,
                );
                action(
                    p,
                    &f,
                    Action::Field(Focus::Cell("m_iComment".into())),
                    "npc_template.choose_voice",
                    "Choose voice type",
                );
                action(
                    p,
                    &f,
                    Action::Mission(mission_workspace::Command::NewVoiceType),
                    "npc_template.new_voice",
                    "Create voice type…",
                );
                if e.picker_field.as_deref() == Some("m_iComment") {
                    draw_field(p, &f, &e, &l, &lang, "m_iComment", &row["m_iComment"]);
                }
            }
            field(p, &f, &e, Focus::Columns, 34.);
            let query = e.column_search.to_lowercase();
            for block in authoring::blocks(&e.tables[e.table].label, &e.columns, row, true) {
                let fields: Vec<_> = block
                    .fields
                    .iter()
                    .filter(|col| {
                        query.is_empty()
                            || col.to_lowercase().contains(&query)
                            || schema::field_name(col, &l, &lang)
                                .to_lowercase()
                                .contains(&query)
                    })
                    .collect();
                if fields.is_empty() {
                    continue;
                }
                label(
                    p,
                    &f,
                    tr(&format!("block.{}", block.key), block.key),
                    15.,
                    mission_skin::CYAN,
                );
                for col in fields {
                    if col == "m_iComment" {
                        continue;
                    }
                    if col == "m_iNpcNumber" {
                        label(
                            p,
                            &f,
                            format!(
                                "{}: {}",
                                schema::field_name(col, &l, &lang),
                                display(&row[col])
                            ),
                            14.,
                            mission_skin::TEXT,
                        );
                    } else {
                        draw_field(p, &f, &e, &l, &lang, col, &row[col]);
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn npc_inspector_selects_by_type_id_and_uses_shared_edits_and_voice_creation() {
        let (_dir, mut e) = super::super::tests::production_editor("m_pNpcTable/m_pNpcData");
        let before = e.document.clone();
        let table = e.table;
        let id = e.rows()[1]["m_iNpcNumber"].as_i64().unwrap();
        e.inspect_npc(id).unwrap();
        assert_eq!(e.row, Some(1));
        let original = e.rows()[1]["m_iHP"].as_i64().unwrap();
        e.begin(Focus::Cell("m_iHP".into()));
        e.edit = (original + 1).to_string();
        assert!(e.finish_npc_field());
        assert_eq!(e.rows()[1]["m_iHP"], original + 1);
        e.undo(false);
        assert_eq!(e.document, before);
        e.inspect_npc(id).unwrap();
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let (localization, lang) = Localization::open(&root, "en").unwrap();
        e.mission_command(
            mission_workspace::Command::NewVoiceType,
            &localization,
            &lang,
        )
        .unwrap();
        assert!(!e.finish_npc_field());
        e.edit = "NewNpcVoice".into();
        e.create_draft().unwrap();
        assert_eq!(e.table, table);
        assert_eq!(e.row, Some(1));
        let index = e.rows()[1]["m_iComment"].as_u64().unwrap() as usize;
        let target = e.reference_target("m_iComment").unwrap().0;
        assert_eq!(
            e.document.pointer(&e.tables[target].pointer).unwrap()[index]["m_strComment2"],
            "NewNpcVoice"
        );
        e.undo(false);
        assert_eq!(e.document, before);
    }
}
