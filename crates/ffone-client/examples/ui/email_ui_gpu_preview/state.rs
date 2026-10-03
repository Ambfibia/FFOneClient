use super::*;

#[derive(Resource)]
pub(super) struct PreviewState {
    pub(super) frames: u32,
    pub(super) ready_frame: Option<u32>,
    pub(super) ready_at: Option<Instant>,
    pub(super) capture_issued: bool,
    pub(super) capture_saved: bool,
    pub(super) capture_failed: bool,
    pub(super) text_audited: bool,
    pub(super) started_at: Instant,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            frames: 0,
            ready_frame: None,
            ready_at: None,
            capture_issued: false,
            capture_saved: false,
            capture_failed: false,
            text_audited: false,
            started_at: Instant::now(),
        }
    }
}

pub(super) fn inventory_item(
    item_type: i16,
    item_id: i16,
    option: i32,
    icon_path: &str,
    count_label: Option<&str>,
) -> EmailInventorySlotView {
    EmailInventorySlotView {
        item: EmailWireItem {
            item_type,
            item_id,
            option,
            time_limit: 0,
        },
        icon_path: Some(icon_path.to_owned()),
        count_label: count_label.map(str::to_owned),
    }
}
