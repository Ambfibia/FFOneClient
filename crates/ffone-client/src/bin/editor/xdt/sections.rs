//! Independent table/mission navigation and unfinished forms over one shared document.
use super::*;

pub(super) struct Navigation {
    table: usize,
    row: Option<usize>,
    search: String,
    column_search: String,
    focus: Option<Focus>,
    edit: String,
    cursor: usize,
    anchor: usize,
    offset: usize,
    column_offset: usize,
    picker_field: Option<String>,
    array_slot: Option<usize>,
    reference_search: String,
    reference_offset: usize,
    draft: Option<forms::Draft>,
    creation: Vec<mission_creation::Frame>,
    quick: Option<mission_workspace::QuickOrigin>,
    quick_text: Option<mission_text::TextDraft>,
    quick_existing: Option<usize>,
    filter: Option<i64>,
}
impl XdtEditor {
    fn take_navigation(&mut self) -> Navigation {
        Navigation {
            table: self.table,
            row: self.row,
            search: std::mem::take(&mut self.search),
            column_search: std::mem::take(&mut self.column_search),
            focus: self.focus.take(),
            edit: std::mem::take(&mut self.edit),
            cursor: self.cursor,
            anchor: self.anchor,
            offset: self.offset,
            column_offset: self.column_offset,
            picker_field: self.picker_field.take(),
            array_slot: self.array_slot.take(),
            reference_search: std::mem::take(&mut self.reference_search),
            reference_offset: self.reference_offset,
            draft: self.draft.take(),
            creation: std::mem::take(&mut self.workspace.creation),
            quick: self.workspace.quick.take(),
            quick_text: self.workspace.quick_text.take(),
            quick_existing: self.workspace.quick_existing.take(),
            filter: self.mission_filter.take(),
        }
    }
    fn restore_navigation(&mut self, n: Navigation) {
        self.table = n.table;
        self.row = n.row;
        self.search = n.search;
        self.column_search = n.column_search;
        self.focus = n.focus;
        self.edit = n.edit;
        self.cursor = n.cursor;
        self.anchor = n.anchor;
        self.offset = n.offset;
        self.column_offset = n.column_offset;
        self.picker_field = n.picker_field;
        self.array_slot = n.array_slot;
        self.reference_search = n.reference_search;
        self.reference_offset = n.reference_offset;
        self.draft = n.draft;
        self.workspace.creation = n.creation;
        self.workspace.quick = n.quick;
        self.workspace.quick_text = n.quick_text;
        self.workspace.quick_existing = n.quick_existing;
        self.mission_filter = n.filter;
    }
    pub(in super::super) fn open_missions(&mut self) {
        if self.mission_workspace_table().is_some() {
            return;
        }
        self.table_tab = Some(self.take_navigation());
        if let Some(n) = self.mission_tab.take() {
            self.restore_navigation(n);
        } else if let Some(table) = self
            .tables
            .iter()
            .position(|t| t.label.ends_with("/m_pMissionTable/m_pMissionData"))
        {
            self.select_table(table);
        }
        self.workspace.enabled = true;
        self.refresh();
        self.revision += 1;
    }
    pub(in super::super) fn open_tables(&mut self) {
        if !self.workspace.enabled {
            return;
        }
        self.mission_tab = Some(self.take_navigation());
        if let Some(n) = self.table_tab.take() {
            self.restore_navigation(n);
        } else if let Some(table) = self
            .tables
            .iter()
            .position(|t| t.label.ends_with("/m_pNpcTable/m_pNpcData"))
        {
            self.select_table(table);
        }
        self.workspace.enabled = false;
        self.refresh();
        self.revision += 1;
    }
}
