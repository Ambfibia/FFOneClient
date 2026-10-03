use super::*;

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct EmailUiModel {
    pub visible: bool,
    pub screen: EmailScreen,
    pub folder: EmailFolder,
    pub guide_messages: Vec<EmailGuideMessage>,
    pub guide_page: usize,
    pub player_messages: Vec<EmailSummary>,
    pub player_page: i8,
    pub selected_row: Option<usize>,
    pub read_message: Option<EmailReadMessage>,
    pub draft: EmailComposeDraft,
    pub buddies: Vec<EmailBuddy>,
    pub inventory: Vec<Option<EmailInventorySlotView>>,
    pub available_cash: i32,
    pub current_local_time: EmailSystemTime,
    pub popup: EmailPopup,
    pub compose_focus: EmailComposeFocus,
    pub calculator_value: i64,
    pub send_in_flight: bool,
    pub mail_send_in_flight: bool,
    pub system_popup_active: bool,
    pub help_active: bool,
    pub external_inventory_popup_active: bool,
    pub escape_gate_pending: bool,
    pub computress_query_pending: bool,
    pub opening_elapsed_seconds: f32,
    pub right_opening_elapsed_seconds: f32,
    pub(super) buddy_scroll_y: f32,
    pub(super) ui_scale_override: Option<f32>,
}

impl Default for EmailUiModel {
    fn default() -> Self {
        Self {
            visible: false,
            screen: EmailScreen::List,
            folder: EmailFolder::Guide,
            guide_messages: Vec::new(),
            guide_page: 0,
            player_messages: Vec::new(),
            player_page: 1,
            selected_row: None,
            read_message: None,
            draft: EmailComposeDraft::default(),
            buddies: Vec::new(),
            inventory: vec![None; EMAIL_INVENTORY_SLOT_COUNT],
            available_cash: 0,
            current_local_time: EmailSystemTime::default(),
            popup: EmailPopup::None,
            compose_focus: EmailComposeFocus::None,
            calculator_value: 0,
            send_in_flight: false,
            mail_send_in_flight: false,
            system_popup_active: false,
            help_active: false,
            external_inventory_popup_active: false,
            escape_gate_pending: false,
            computress_query_pending: false,
            opening_elapsed_seconds: EMAIL_UI_OPEN_SECONDS,
            right_opening_elapsed_seconds: EMAIL_UI_OPEN_SECONDS,
            buddy_scroll_y: 0.0,
            ui_scale_override: None,
        }
    }
}

impl EmailUiModel {
    #[must_use]
    pub fn input_boundary(&self) -> EmailInputBoundary {
        EmailInputBoundary {
            blocks_lower_ui: self.visible,
            blocks_gameplay_input: self.visible,
            requires_pointer: self.visible,
            cursor_locked_while_visible: false,
            cursor_locked_after_exit: false,
            mouse_controls_enabled: self.visible && self.input_enabled(),
            escape_close_gate_enabled: self.visible
                && !self.send_in_flight
                && !self.system_popup_active,
        }
    }

    #[must_use]
    pub fn input_enabled(&self) -> bool {
        self.visible
            && !self.send_in_flight
            && !self.mail_send_in_flight
            && !self.system_popup_active
            && !self.help_active
            && !self.external_inventory_popup_active
            && !self.escape_gate_pending
            && !self.computress_query_pending
    }

    #[must_use]
    pub fn page_count(&self) -> usize {
        match self.folder {
            EmailFolder::Guide => self.guide_messages.len().div_ceil(EMAIL_PAGE_SIZE).max(1),
            EmailFolder::Player => EMAIL_PAGE_MAX as usize,
        }
    }

    #[must_use]
    pub fn has_user_mail(&self) -> bool {
        !self.player_messages.is_empty()
    }

    #[must_use]
    pub fn player_can_next(&self) -> bool {
        self.player_page < EMAIL_PAGE_MAX && self.player_messages.len() == EMAIL_PAGE_SIZE
    }

    #[must_use]
    pub fn player_sender_label(&self, message: &EmailSummary) -> String {
        self.buddies
            .iter()
            .find(|buddy| buddy.pc_uid == message.from_pc_uid && buddy.name_check_flag != 1)
            .map(|buddy| format!("Player {}", buddy.pc_uid))
            .unwrap_or_else(|| message.sender_label())
    }

    #[must_use]
    pub fn selected_read_matches(&self) -> bool {
        self.selected_summary()
            .zip(self.read_message.as_ref())
            .is_some_and(|(summary, read)| summary.email_index == read.email_index)
    }

    #[must_use]
    pub fn compose_controls_open(&self) -> bool {
        self.opening_elapsed_seconds >= EMAIL_UI_OPEN_SECONDS
    }

    #[must_use]
    pub fn right_panel_controls_open(&self) -> bool {
        self.right_opening_elapsed_seconds >= EMAIL_UI_OPEN_SECONDS
    }

    #[must_use]
    pub const fn buddy_scroll_y(&self) -> f32 {
        self.buddy_scroll_y
    }

    pub fn apply_buddy_scroll_axis(&mut self, axis: f32) -> bool {
        if self.screen != EmailScreen::Compose
            || self.popup != EmailPopup::BuddyList
            || !self.input_enabled()
            || !self.compose_controls_open()
            || !axis.is_finite()
            || axis == 0.0
        {
            return false;
        }
        // `EmailMode.Update` clamps the axis to +/-1 and multiplies it by the
        // shared 200 velocity. `Panel_NewEmail.Scroll` then clamps that value
        // to +/-30 and subtracts it from `scrBuddyPos.y`.
        let shared_value = axis.clamp(-1.0, 1.0) * EMAIL_UI_SHARED_SCROLL_VELOCITY;
        let step = shared_value.clamp(
            -EMAIL_UI_BUDDY_SCROLL_STEP_LIMIT,
            EMAIL_UI_BUDDY_SCROLL_STEP_LIMIT,
        );
        self.buddy_scroll_y =
            (self.buddy_scroll_y - step).clamp(0.0, email_buddy_scroll_max(self.buddies.len()));
        true
    }

    #[must_use]
    pub fn visible_guide_messages(&self) -> &[EmailGuideMessage] {
        let start = self.guide_page.saturating_mul(EMAIL_PAGE_SIZE);
        let end = (start + EMAIL_PAGE_SIZE).min(self.guide_messages.len());
        self.guide_messages.get(start..end).unwrap_or(&[])
    }

    #[must_use]
    pub fn selected_summary(&self) -> Option<&EmailSummary> {
        self.selected_row
            .and_then(|index| self.player_messages.get(index))
    }

    #[must_use]
    pub fn selected_guide(&self) -> Option<&EmailGuideMessage> {
        let index = self
            .guide_page
            .saturating_mul(EMAIL_PAGE_SIZE)
            .saturating_add(self.selected_row?);
        self.guide_messages.get(index)
    }

    pub fn set_ui_scale_override(&mut self, scale: Option<f32>) {
        self.ui_scale_override = scale.filter(|value| value.is_finite() && *value > 0.0);
    }

    #[must_use]
    pub fn effective_ui_scale(&self, viewport_height: f32) -> f32 {
        self.ui_scale_override
            .unwrap_or_else(|| clean_email_ui_scale(viewport_height))
    }

    pub fn tick_opening(&mut self, delta_seconds: f32) {
        if !self.visible || !delta_seconds.is_finite() {
            return;
        }
        let delta_seconds = delta_seconds.max(0.0);
        self.right_opening_elapsed_seconds =
            (self.right_opening_elapsed_seconds + delta_seconds).min(EMAIL_UI_OPEN_SECONDS);
        if self.screen == EmailScreen::Compose {
            self.opening_elapsed_seconds =
                (self.opening_elapsed_seconds + delta_seconds).min(EMAIL_UI_OPEN_SECONDS);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sync_email_ui_model(
    model: Res<EmailUiModel>,
    asset_server: Res<AssetServer>,
    assets: Res<EmailUiAssets>,
    mut texts: Query<(&EmailUiTextElement, &mut LocalizedText)>,
    mut bindings: ParamSet<(
        Query<(&EmailUiRowSelection, &mut Node)>,
        Query<(&EmailUiRowAttachment, &mut Node)>,
        Query<(&EmailUiButton, &mut Node)>,
        Query<(&EmailPopup, &mut Node)>,
        Query<(&EmailUiAttachmentSlot, &mut ImageNode)>,
        Query<(&EmailUiInventorySlot, &mut ImageNode)>,
        Query<(&EmailUiInventoryIcon, &mut Node, &mut ImageNode)>,
    )>,
) {
    for (element, mut localized) in &mut texts {
        let next = email_localized_text(&model, element.role);
        if *localized != next {
            *localized = next;
        }
    }
    for (selection, mut node) in &mut bindings.p0() {
        node.display = if model.screen == EmailScreen::List
            && model.selected_row == Some(selection.row)
            && match model.folder {
                EmailFolder::Guide => selection.row < model.visible_guide_messages().len(),
                EmailFolder::Player => selection.row < model.player_messages.len(),
            } {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (attachment, mut node) in &mut bindings.p1() {
        node.display = if model.folder == EmailFolder::Player
            && model
                .player_messages
                .get(attachment.row)
                .is_some_and(|message| message.item_cash_flag != 0)
        {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (button, mut node) in &mut bindings.p2() {
        node.display = match button.kind {
            EmailUiButtonKind::AcceptAll
            | EmailUiButtonKind::AcceptTaros
            | EmailUiButtonKind::RemoveBuddy
            | EmailUiButtonKind::Delete
            | EmailUiButtonKind::Reply => {
                if model.folder == EmailFolder::Player && model.has_user_mail() {
                    Display::Flex
                } else {
                    Display::None
                }
            }
            EmailUiButtonKind::Previous => {
                let enabled = match model.folder {
                    EmailFolder::Guide => model.guide_page > 0,
                    EmailFolder::Player => model.player_page > 1,
                };
                if enabled {
                    Display::Flex
                } else {
                    Display::None
                }
            }
            EmailUiButtonKind::Next => {
                let enabled = match model.folder {
                    EmailFolder::Guide => model.guide_page + 1 < model.page_count(),
                    EmailFolder::Player => model.player_can_next(),
                };
                if enabled {
                    Display::Flex
                } else {
                    Display::None
                }
            }
            _ => node.display,
        };
    }
    for (popup, mut node) in &mut bindings.p3() {
        node.display = if model.screen == EmailScreen::Compose && model.popup == *popup {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (slot, mut image) in &mut bindings.p4() {
        let occupied = if slot.outgoing {
            model
                .draft
                .attachments
                .get(slot.slot)
                .and_then(|item| *item)
                .is_some_and(|item| !item.item.is_empty())
        } else {
            model
                .read_message
                .as_ref()
                .and_then(|read| read.items.get(slot.slot))
                .is_some_and(|item| !item.is_empty())
        };
        image.image = if occupied {
            assets.slot_occupied.clone()
        } else {
            assets.slot_empty.clone()
        };
    }
    for (slot, mut image) in &mut bindings.p5() {
        image.image = if model.inventory.get(slot.slot).is_some_and(Option::is_some)
            && !email_inventory_slot_staged(&model, slot.slot)
        {
            assets.slot_occupied.clone()
        } else {
            assets.slot_empty.clone()
        };
    }
    for (icon, mut node, mut image) in &mut bindings.p6() {
        let icon_path = (!email_inventory_slot_staged(&model, icon.slot))
            .then(|| model.inventory.get(icon.slot))
            .flatten()
            .and_then(Option::as_ref)
            .and_then(|slot| slot.icon_path.as_ref());
        if let Some(icon_path) = icon_path {
            image.image = asset_server.load(icon_path.clone());
            node.display = Display::Flex;
        } else {
            node.display = Display::None;
        }
    }
}
