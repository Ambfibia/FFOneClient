use crate::email_runtime::*;
use crate::email_ui::{
    EMAIL_ATTACHMENT_COUNT, EMAIL_REQ_DELETE_ID, EMAIL_REQ_DELETE_SIZE, EMAIL_REQ_PAGE_LIST_ID,
    EMAIL_REQ_PAGE_LIST_SIZE, EMAIL_REQ_READ_ID, EMAIL_REQ_READ_SIZE, EMAIL_REQ_RECEIVE_ALL_ID,
    EMAIL_REQ_RECEIVE_ALL_SIZE, EMAIL_REQ_RECEIVE_CASH_ID, EMAIL_REQ_RECEIVE_CASH_SIZE,
    EMAIL_REQ_RECEIVE_ITEM_ID, EMAIL_REQ_RECEIVE_ITEM_SIZE, EMAIL_REQ_SEND_ID,
    EMAIL_REQ_SEND_SIZE, EMAIL_REQ_UPDATE_CHECK_ID, EMAIL_REQ_UPDATE_CHECK_SIZE,
};

mod operations;
mod assets;
mod types;
mod codec;
mod state;
mod input;
mod commands;
mod validation;
mod output;

use operations::{
    sample_item, sample_outgoing, base_item, empty_base_item, authority_with, open_harness,
    prepare_send, value_i32
};
use assets::TestCatalog;
use types::ProductionHarness;
use codec::{frame, page_success_frame, read_success_frame, send_success_frame};
use output::write_utf16_at;
