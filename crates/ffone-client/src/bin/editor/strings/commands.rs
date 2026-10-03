use super::*;

#[derive(Component, Clone)]
pub(super) enum Action {
    Save,
    Sync,
    Discard,
    Search,
    Filter,
    Edit(String, bool),
    FilterMode,
    ReplaceCase,
    ReplaceWord,
    ReplaceLocale,
    SelectRow(String, bool),
    ExportRows,
    ImportRows,
    Replacement,
    ReplaceAll,
    Find(bool),
    ReplaceOne,
    CloseReplace,
    Sort(u8),
}
