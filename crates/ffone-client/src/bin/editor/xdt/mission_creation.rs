//! Nested quick creation is a single reversible transaction from the mission.
use super::*;

pub(super) struct Frame {
    pub table:usize,
    pub row:Option<usize>,
    pub draft:Option<forms::Draft>,
    origin:Option<mission_workspace::QuickOrigin>,
    text:Option<mission_text::TextDraft>,
    existing:Option<usize>,
    pub search:String,
    filter:Option<i64>,
    document:Value,
    locales:BTreeMap<String,[Option<String>;2]>,
    undo_len:usize,
    redo:Vec<Change>,
}

impl XdtEditor {
    pub(super) fn push_creation(&mut self){
        self.workspace.creation.push(Frame{
            table:self.table,row:self.row,draft:self.draft.take(),origin:self.workspace.quick.take(),
            text:self.workspace.quick_text.take(),existing:self.workspace.quick_existing.take(),
            search:self.search.clone(),filter:self.mission_filter,document:self.document.clone(),
            locales:self.workspace.locale_drafts.clone(),undo_len:self.undo.len(),redo:self.redo.clone(),
        });
    }
    pub(super) fn finish_creation(&mut self,link:Option<Value>){
        let Some(mut frame)=self.workspace.creation.pop()else{return};
        let origin=self.workspace.quick.take();
        if let Some(link)=link {
            if let (Some(draft),Some(origin))=(frame.draft.as_mut(),origin.as_ref()){
                if let Some(slot)=origin.slot{draft.value[&origin.field][slot]=link;}
                else{draft.value[&origin.field]=link;}
            }
            // Fold child records, translated strings and the final owner link into
            // one history entry. Disk writes stay disabled until the root closes.
            if self.workspace.creation.is_empty(){
                self.undo.truncate(frame.undo_len);
                let keys:BTreeSet<_>=frame.locales.keys().chain(self.workspace.locale_drafts.keys()).cloned().collect();
                let effective=|map:&BTreeMap<String,[Option<String>;2]>,key:&String|
                    map.get(key).cloned().unwrap_or_else(||std::array::from_fn(|i|self.workspace.locale_base[i].get(key).cloned()));
                let mut before=BTreeMap::new();let mut after=BTreeMap::new();
                for key in keys{
                    let old=effective(&frame.locales,&key);let new=effective(&self.workspace.locale_drafts,&key);
                    if old!=new{before.insert(key.clone(),old);after.insert(key,new);}
                }
                let localized=!before.is_empty();
                if frame.document!=self.document||localized{
                    self.undo.push(Change{
                        pointer:if localized{"@mission-localized".into()}else{String::new()},
                        before:if localized{serde_json::json!({"document":frame.document,"locales":before})}else{frame.document},
                        after:if localized{serde_json::json!({"document":self.document,"locales":after})}else{self.document.clone()},
                    });
                    self.redo.clear();
                }
            }
        }else{
            self.document=frame.document;
            self.workspace.locale_drafts=frame.locales;
            self.undo.truncate(frame.undo_len);self.redo=frame.redo;
        }
        self.table=frame.table;self.row=frame.row;self.draft=frame.draft;
        self.workspace.quick=frame.origin;self.workspace.quick_text=frame.text;
        self.workspace.quick_existing=frame.existing;
        self.search=frame.search;self.mission_filter=frame.filter;
        self.focus=None;self.picker_field=None;self.array_slot=None;
        self.reindex();self.refresh();self.revision+=1;
    }
}
