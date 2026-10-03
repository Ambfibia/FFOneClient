use super::*;

pub(super) fn buddy_static_localized(value: &'static str) -> LocalizedText {
    let key = match value {
        BUDDY_LIST_TITLE => "ui.buddy.title",
        BUDDY_DELETE_LABEL => "ui.common.delete",
        BUDDY_WARP_LABEL => "ui.buddy.warp",
        BUDDY_ADD_LABEL => "ui.buddy.add",
        BUDDY_CANCEL_LABEL => "ui.common.cancel",
        BUDDY_ADD_TITLE => "ui.buddy.add_title",
        BUDDY_ADD_INSTRUCTION => "ui.buddy.add_instruction",
        _ => unreachable!("Buddy static copy must have a semantic localization key"),
    };
    LocalizedText::new(key, value)
}

pub(super) fn buddy_row_localized(row: Option<&BuddyRowView>) -> LocalizedText {
    match row {
        Some(row) if row.name_is_verified => {
            LocalizedText::new("ui.buddy.row.verified", "    {display_name}")
                .with_arg("display_name", &row.display_name)
        }
        Some(row) => LocalizedText::new("ui.buddy.row.player", "    Player {pc_uid}")
            .with_arg("pc_uid", row.target.pc_uid.to_string()),
        None => LocalizedText::new("ui.buddy.row.empty", ""),
    }
}

pub(super) fn buddy_add_name_localized(name: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.buddy.add_name_input", "{name}").with_arg("name", name)
}
