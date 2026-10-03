use super::*;

pub const EMAIL_UI_GAME_MODE_VALUE: u8 = 18;

pub const EMAIL_INVENTORY_SLOT_COUNT: usize = 50;

pub const EMAIL_TIME_TRAVEL_SPECIAL_STATE: u64 = 64;

pub const EMAIL_UI_INVENTORY_COLUMNS: usize = 5;

pub const EMAIL_UI_INVENTORY_SLOT_SIZE: f32 = 67.0;

pub const EMAIL_UI_INVENTORY_SLOT_STRIDE: f32 = 69.0;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmailInventorySlotView {
    pub item: EmailWireItem,
    pub icon_path: Option<String>,
    pub count_label: Option<String>,
}

#[must_use]
pub fn request_delete_selected_email(
    model: &mut EmailUiModel,
    actions: &mut EmailUiOutbox,
    audio: &mut EmailUiAudioOutbox,
) -> bool {
    let Some(read) = model.read_message.as_ref() else {
        audio.push(EmailUiAudioCue::ButtonSound);
        return false;
    };
    if model.folder != EmailFolder::Player || !model.input_enabled() || read.email_index <= 0 {
        return false;
    }
    actions.push(EmailUiAction::DeleteEmailConfirmation {
        message_id: 166,
        email_index: read.email_index,
    });
    audio.push(EmailUiAudioCue::ButtonSound);
    true
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiRowSelection {
    pub row: usize,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiInventorySlot {
    pub slot: usize,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiInventoryIcon {
    pub slot: usize,
}

pub(super) fn email_inventory_source_valid(model: &EmailUiModel, slot: usize) -> bool {
    model.screen == EmailScreen::Compose
        && model.popup == EmailPopup::None
        && model.compose_controls_open()
        && !email_inventory_slot_staged(model, slot)
        && model
            .inventory
            .get(slot)
            .and_then(Option::as_ref)
            .is_some_and(|view| !view.item.is_empty())
}

pub(super) fn email_inventory_slot_staged(model: &EmailUiModel, slot: usize) -> bool {
    model
        .draft
        .attachments
        .iter()
        .flatten()
        .any(|attachment| attachment.inventory_slot == slot as i32)
}

pub(super) fn first_free_email_inventory_slot(model: &EmailUiModel) -> Option<usize> {
    model.inventory.iter().position(Option::is_none)
}

pub(super) fn email_page_selection_text(page: usize, row: usize) -> LocalizedText {
    LocalizedText::new("ui.email.page.selection", "{page} - {row}")
        .with_arg("page", page.to_string())
        .with_arg("row", row.to_string())
}
