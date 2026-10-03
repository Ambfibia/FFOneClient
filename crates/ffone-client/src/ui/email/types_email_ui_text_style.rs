use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmailOutgoingItem {
    pub inventory_slot: i32,
    pub item: EmailWireItem,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmailSystemTime {
    pub year: i32,
    pub month: i32,
    pub day_of_week: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub milliseconds: i32,
}

impl EmailSystemTime {
    #[must_use]
    pub fn short_date(self) -> String {
        format!("{:02}/{:02}/{:04}", self.month, self.day, self.year)
    }

    /// Reproduces `Panel_EmailList.GetDate(sSYSTEMTIME)`, including its
    /// deliberately approximate 30-day months and day-31 adjustment.
    #[must_use]
    pub fn legacy_day_label(self, now: Self) -> String {
        let days = self.legacy_day_difference(now);
        if days <= 0 {
            "today".to_owned()
        } else {
            format!("{days} day")
        }
    }

    pub(super) fn legacy_day_difference(self, now: Self) -> i32 {
        let mut days = now.day.saturating_sub(self.day);
        days = days.saturating_add(now.month.saturating_sub(self.month).saturating_mul(30));
        if self.day == 31 && (self.month != now.month || self.year != now.year) {
            days = days.saturating_add(1);
        }
        days.saturating_add(now.year.saturating_sub(self.year).saturating_mul(365))
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmailSummary {
    pub email_index: i64,
    pub from_pc_uid: i64,
    pub first_name: String,
    pub last_name: String,
    pub subject: String,
    pub read_flag: i32,
    pub send_time: EmailSystemTime,
    pub delete_time: EmailSystemTime,
    pub item_cash_flag: i32,
}

impl EmailSummary {
    #[must_use]
    pub fn sender_label(&self) -> String {
        // The clean player-mail list and detail pane display szFirstName only.
        self.first_name.clone()
    }

    #[must_use]
    pub fn truncated_subject(&self) -> String {
        truncate_legacy_list_text(&self.subject)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmailGuideMessage {
    pub mode: i32,
    pub mission_task_id: i32,
    pub sender_npc_id: i32,
    pub sender_name: String,
    pub subject: String,
    pub content: String,
    pub sender_icon_path: Option<String>,
    pub auto_delete_note: bool,
    pub subject_string_id: i32,
    pub content_string_id: i32,
}

impl EmailGuideMessage {
    #[must_use]
    pub fn localized_sender(&self) -> LocalizedText {
        LocalizedText::new(
            format!("content.npc.{}.name", self.sender_npc_id),
            &self.sender_name,
        )
    }

    #[must_use]
    pub fn localized_subject(&self) -> LocalizedText {
        LocalizedText::new(
            format!(
                "content.tabledata.mission.mission_string.{}.str_name_string",
                self.subject_string_id
            ),
            &self.subject,
        )
    }

    #[must_use]
    pub fn localized_content(&self) -> LocalizedText {
        LocalizedText::new(
            format!(
                "content.tabledata.mission.mission_string.{}.str_name_string",
                self.content_string_id
            ),
            &self.content,
        )
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EmailBuddy {
    pub pc_uid: i64,
    pub first_name: String,
    pub last_name: String,
    pub name_check_flag: i32,
}

impl EmailBuddy {
    #[must_use]
    pub fn list_label(&self) -> String {
        if self.name_check_flag == 1 {
            format!("{} {}", self.first_name, self.last_name)
                .trim()
                .to_owned()
        } else {
            format!("Player {}", self.pc_uid)
        }
    }

    #[must_use]
    pub fn selected_recipient_label(&self) -> String {
        if self.name_check_flag == 1 {
            // `Panel_NewEmail.ClickBuddy` concatenates these fields without a
            // separator even though its popup row uses one.
            format!("{}{}", self.first_name, self.last_name)
        } else {
            format!("Player {}", self.pc_uid)
        }
    }

    #[must_use]
    pub fn display_name(&self) -> String {
        self.selected_recipient_label()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailComposeDraft {
    pub recipient_pc_uid: i64,
    pub recipient_name: String,
    pub subject: String,
    pub content: String,
    pub attachments: [Option<EmailOutgoingItem>; EMAIL_ATTACHMENT_COUNT],
    pub cash: i32,
}

impl Default for EmailComposeDraft {
    fn default() -> Self {
        Self {
            recipient_pc_uid: 0,
            recipient_name: String::new(),
            subject: String::new(),
            content: String::new(),
            attachments: [None; EMAIL_ATTACHMENT_COUNT],
            cash: 0,
        }
    }
}

impl EmailComposeDraft {
    #[must_use]
    pub fn attachment_count(&self) -> usize {
        self.attachments.iter().flatten().count()
    }

    #[must_use]
    pub fn postage(&self) -> i32 {
        EMAIL_BASE_POSTAGE + self.attachment_count() as i32 * EMAIL_ITEM_POSTAGE
    }

    pub fn set_subject(&mut self, value: impl AsRef<str>) {
        self.subject = truncate_utf16(value.as_ref(), EMAIL_SUBJECT_INPUT_LIMIT);
    }

    pub fn set_content(&mut self, value: impl AsRef<str>) {
        self.content = truncate_utf16(value.as_ref(), EMAIL_CONTENT_INPUT_LIMIT);
    }

    pub fn request(&self, available_cash: i32) -> Result<EmailRequest, EmailComposeError> {
        if self.recipient_pc_uid <= 0 {
            return Err(EmailComposeError::MissingRecipient);
        }
        if self.cash < 0 {
            return Err(EmailComposeError::NegativeCash(self.cash));
        }
        let required = self.postage().saturating_add(self.cash);
        if required > available_cash {
            return Err(EmailComposeError::InsufficientCash {
                available: available_cash,
                required,
            });
        }
        if utf16_units(&self.subject) > EMAIL_SUBJECT_WIRE_UNITS {
            return Err(EmailComposeError::SubjectWireOverflow);
        }
        if utf16_units(&self.content) > EMAIL_CONTENT_WIRE_UNITS {
            return Err(EmailComposeError::ContentWireOverflow);
        }
        let subject = if self.subject.is_empty() {
            "No subject.".to_owned()
        } else {
            self.subject.clone()
        };
        let items = array::from_fn(|index| self.attachments[index].unwrap_or_default());
        Ok(EmailRequest::Send {
            recipient_pc_uid: self.recipient_pc_uid,
            subject,
            content: self.content.clone(),
            items,
            cash: self.cash,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EmailScreen {
    #[default]
    List,
    Compose,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EmailFolder {
    #[default]
    Guide,
    Player,
}

#[derive(Clone, Copy, Component, Debug, Default, Eq, Hash, PartialEq)]
pub enum EmailPopup {
    #[default]
    None,
    BuddyList,
    AddTaros,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EmailComposeFocus {
    #[default]
    None,
    Subject,
    Body,
}

#[derive(Default, Resource)]
pub struct EmailUiOutbox(pub VecDeque<EmailUiAction>);

impl EmailUiOutbox {
    pub fn push(&mut self, action: EmailUiAction) {
        self.0.push_back(action);
    }

    pub fn pop(&mut self) -> Option<EmailUiAction> {
        self.0.pop_front()
    }
}

#[derive(Default, Resource)]
pub struct EmailTransportOutbox(pub VecDeque<EmailRequest>);

impl EmailTransportOutbox {
    pub fn push(&mut self, request: EmailRequest) {
        self.0.push_back(request);
    }

    pub fn pop(&mut self) -> Option<EmailRequest> {
        self.0.pop_front()
    }
}

/// Production EmailMode request correlation. Kept beside the UI boundary so
/// standalone previews can install the plugin without opening a socket.
#[derive(Default, Resource)]
pub struct EmailNetworkRuntime0104(pub EmailTransportRuntime0104);

#[derive(Default, Resource)]
pub struct EmailNetworkInbox0104(pub VecDeque<DecodedFrame>);

impl EmailNetworkInbox0104 {
    pub const fn owns_packet(packet_id: u32) -> bool {
        matches!(
            packet_id,
            EMAIL_REP_NEW_ID
                | EMAIL_REP_READ_SUCCESS_ID
                | EMAIL_REP_READ_FAILURE_ID
                | EMAIL_REP_PAGE_LIST_SUCCESS_ID
                | EMAIL_REP_PAGE_LIST_FAILURE_ID
                | EMAIL_REP_DELETE_SUCCESS_ID
                | EMAIL_REP_DELETE_FAILURE_ID
                | EMAIL_REP_SEND_SUCCESS_ID
                | EMAIL_REP_SEND_FAILURE_ID
                | EMAIL_REP_RECEIVE_ITEM_SUCCESS_ID
                | EMAIL_REP_RECEIVE_ITEM_FAILURE_ID
                | EMAIL_REP_RECEIVE_CASH_SUCCESS_ID
                | EMAIL_REP_RECEIVE_CASH_FAILURE_ID
                | EMAIL_REP_RECEIVE_ALL_SUCCESS_ID
                | EMAIL_REP_RECEIVE_ALL_FAILURE_ID
        )
    }

    pub fn push(&mut self, frame: DecodedFrame) -> bool {
        if !Self::owns_packet(frame.packet_type) {
            return false;
        }
        self.0.push_back(frame);
        true
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum EmailCloseSource {
    Escape,
    EmailKey,
    RightPanelClose,
}

#[derive(Clone, Resource)]
pub(super) struct EmailUiAssets {
    pub(super) scrollbar_track: Handle<Image>,
    pub(super) scrollbar_thumb: Handle<Image>,
    pub(super) frame: Handle<Image>,
    pub(super) list: Handle<Image>,
    pub(super) guide_tab: Handle<Image>,
    pub(super) guide_tab_hover: Handle<Image>,
    pub(super) player_tab: Handle<Image>,
    pub(super) player_tab_hover: Handle<Image>,
    pub(super) inactive_tab_fill: Handle<Image>,
    pub(super) attachment: Handle<Image>,
    pub(super) buddy_popup: Handle<Image>,
    pub(super) calculator_pad: Handle<Image>,
    pub(super) calculator_popup: Handle<Image>,
    pub(super) data_back: Handle<Image>,
    pub(super) data_box: Handle<Image>,
    pub(super) taros: Handle<Image>,
    pub(super) previous: Handle<Image>,
    pub(super) previous_hover: Handle<Image>,
    pub(super) next: Handle<Image>,
    pub(super) next_hover: Handle<Image>,
    pub(super) compose: Handle<Image>,
    pub(super) selection: Handle<Image>,
    pub(super) button: Handle<Image>,
    pub(super) button_hover: Handle<Image>,
    pub(super) red_button: Handle<Image>,
    pub(super) red_button_hover: Handle<Image>,
    pub(super) backdrop: Handle<Image>,
    pub(super) right_panel: Handle<Image>,
    pub(super) inventory_panel: Handle<Image>,
    pub(super) slot_occupied: Handle<Image>,
    pub(super) slot_empty: Handle<Image>,
    pub(super) close: Handle<Image>,
    pub(super) close_hover: Handle<Image>,
    pub(super) font: Handle<Font>,
    pub(super) body_font: Handle<Font>,
}

impl EmailUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            scrollbar_track: asset_server.load("ui/en/user-equip/scroll-track.png"),
            scrollbar_thumb: asset_server.load("ui/en/user-equip/scroll-thumb.png"),
            frame: asset_server.load(EMAIL_UI_FRAME_PATH),
            list: asset_server.load(EMAIL_UI_LIST_PATH),
            guide_tab: asset_server.load(EMAIL_UI_GUIDE_TAB_PATH),
            guide_tab_hover: asset_server.load(EMAIL_UI_GUIDE_TAB_HOVER_PATH),
            player_tab: asset_server.load(EMAIL_UI_PLAYER_TAB_PATH),
            player_tab_hover: asset_server.load(EMAIL_UI_PLAYER_TAB_HOVER_PATH),
            inactive_tab_fill: asset_server.load(EMAIL_UI_INACTIVE_TAB_FILL_PATH),
            attachment: asset_server.load(EMAIL_UI_ATTACHMENT_PATH),
            buddy_popup: asset_server.load(EMAIL_UI_BUDDY_POPUP_PATH),
            calculator_pad: asset_server.load(EMAIL_UI_CALCULATOR_PAD_PATH),
            calculator_popup: asset_server.load(EMAIL_UI_CALCULATOR_POPUP_PATH),
            data_back: asset_server.load(EMAIL_UI_DATA_BACK_PATH),
            data_box: asset_server.load(EMAIL_UI_DATA_BOX_PATH),
            taros: asset_server.load(EMAIL_UI_TAROS_PATH),
            previous: asset_server.load(EMAIL_UI_PREVIOUS_PATH),
            previous_hover: asset_server.load(EMAIL_UI_PREVIOUS_HOVER_PATH),
            next: asset_server.load(EMAIL_UI_NEXT_PATH),
            next_hover: asset_server.load(EMAIL_UI_NEXT_HOVER_PATH),
            compose: asset_server.load(EMAIL_UI_COMPOSE_PATH),
            selection: asset_server.load(EMAIL_UI_LIST_SELECTION_PATH),
            button: asset_server.load(EMAIL_UI_BUTTON_PATH),
            button_hover: asset_server.load(EMAIL_UI_BUTTON_HOVER_PATH),
            red_button: asset_server.load(EMAIL_UI_RED_BUTTON_PATH),
            red_button_hover: asset_server.load(EMAIL_UI_RED_BUTTON_HOVER_PATH),
            backdrop: asset_server.load(EMAIL_UI_BACKDROP_PATH),
            right_panel: asset_server.load(EMAIL_UI_RIGHT_PANEL_PATH),
            inventory_panel: asset_server.load(EMAIL_UI_INVENTORY_PANEL_PATH),
            slot_occupied: asset_server.load(EMAIL_UI_SLOT_OCCUPIED_PATH),
            slot_empty: asset_server.load(EMAIL_UI_SLOT_EMPTY_PATH),
            close: asset_server.load(EMAIL_UI_CLOSE_PATH),
            close_hover: asset_server.load(EMAIL_UI_CLOSE_HOVER_PATH),
            font: asset_server.load(EMAIL_UI_FONT_PATH),
            body_font: asset_server.load(EMAIL_UI_BODY_FONT_PATH),
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiRoot;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiBackground;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiLeftBackplate;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiRightBackplate;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiListPanel;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiComposePanel;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiRightPanel;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiBuddyRow {
    pub row: usize,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiBuddyListContent;

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiRowAttachment {
    pub row: usize,
}

#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum EmailUiTextRole {
    FolderEmpty,
    RowFrom(usize),
    RowSubject(usize),
    RowDate(usize),
    PageSelection,
    PageNumber,
    DetailFrom,
    DetailSubject,
    DetailReceived,
    DetailBody,
    DetailTaros,
    ComposeTo,
    ComposeSubject,
    ComposeBody,
    ComposeTaros,
    ComposePostage,
    CalculatorValue,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiTextElement {
    pub role: EmailUiTextRole,
}

/// Exact clean-0104 `FusionFallInvenSkin` GUIStyle role carried by every
/// native Email `Text` entity.
///
/// Localization identity is deliberately separate: EN and RU copy keep the
/// primary style, source Rect and validated replacement-font calibration.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum EmailUiTextStyle {
    LabelUpperLeft,
    LabelMiddleCenter,
    LabelMiddleRight,
    BlankBoxUpperLeft,
    BlankBoxUpperCenter,
    BlankBoxMiddleLeft,
    BlankBoxMiddleRight,
    CenterLabel,
    RightLabel,
    ChaletLabelMiddleLeft,
    ChaletLabelMiddleCenter,
    TextArea,
    Button,
    ButtonLabelFont,
    RedButtonLabelFont,
    MailPageButton,
    CalculatorButton,
    PostageLabel,
}

impl EmailUiTextStyle {
    pub(super) fn font(self, assets: &EmailUiAssets) -> (TextFont, LineHeight) {
        let (font, font_size, line_height) = match self {
            Self::LabelUpperLeft
            | Self::LabelMiddleCenter
            | Self::LabelMiddleRight
            | Self::BlankBoxUpperLeft
            | Self::BlankBoxUpperCenter
            | Self::BlankBoxMiddleLeft
            | Self::BlankBoxMiddleRight
            | Self::ButtonLabelFont
            | Self::RedButtonLabelFont => (
                assets.font.clone(),
                EMAIL_UI_JEFFE_12_FONT_SIZE,
                EMAIL_UI_JEFFE_12_LINE_HEIGHT,
            ),
            Self::Button | Self::MailPageButton => (
                assets.font.clone(),
                EMAIL_UI_JEFFE_14_FONT_SIZE,
                EMAIL_UI_JEFFE_14_LINE_HEIGHT,
            ),
            Self::CalculatorButton => (
                assets.font.clone(),
                EMAIL_UI_JEFFE_16_FONT_SIZE,
                EMAIL_UI_JEFFE_16_LINE_HEIGHT,
            ),
            Self::PostageLabel => (
                assets.font.clone(),
                EMAIL_UI_JEFFE_06_FONT_SIZE,
                EMAIL_UI_JEFFE_06_LINE_HEIGHT,
            ),
            Self::CenterLabel
            | Self::RightLabel
            | Self::ChaletLabelMiddleLeft
            | Self::ChaletLabelMiddleCenter
            | Self::TextArea => (
                assets.body_font.clone(),
                EMAIL_UI_CHALET_SMALL_FONT_SIZE,
                EMAIL_UI_CHALET_SMALL_LINE_HEIGHT,
            ),
        };
        (
            TextFont {
                font: (font).into(),
                font_size: (font_size).into(),
                ..default()
            },
            LineHeight::Px(line_height),
        )
    }

    pub(super) fn layout(self) -> TextLayout {
        let justify = match self {
            Self::LabelMiddleCenter
            | Self::BlankBoxUpperCenter
            | Self::ChaletLabelMiddleCenter
            | Self::CenterLabel
            | Self::Button
            | Self::ButtonLabelFont
            | Self::RedButtonLabelFont
            | Self::MailPageButton
            | Self::CalculatorButton => Justify::Center,
            Self::LabelMiddleRight
            | Self::BlankBoxMiddleRight
            | Self::RightLabel
            | Self::PostageLabel => Justify::Right,
            Self::LabelUpperLeft
            | Self::BlankBoxUpperLeft
            | Self::BlankBoxMiddleLeft
            | Self::ChaletLabelMiddleLeft
            | Self::TextArea => Justify::Left,
        };
        let linebreak = match self {
            Self::BlankBoxUpperLeft
            | Self::BlankBoxUpperCenter
            | Self::BlankBoxMiddleLeft
            | Self::BlankBoxMiddleRight
            | Self::Button
            | Self::ButtonLabelFont
            | Self::RedButtonLabelFont
            | Self::MailPageButton
            | Self::CalculatorButton
            | Self::PostageLabel => LineBreak::NoWrap,
            Self::LabelUpperLeft
            | Self::LabelMiddleCenter
            | Self::LabelMiddleRight
            | Self::CenterLabel
            | Self::RightLabel
            | Self::ChaletLabelMiddleLeft
            | Self::ChaletLabelMiddleCenter
            | Self::TextArea => LineBreak::WordBoundary,
        };
        TextLayout::new(justify, linebreak)
    }

    pub(super) fn apply_to_container(self, node: &mut Node) {
        node.justify_content = match self {
            Self::LabelMiddleCenter
            | Self::BlankBoxUpperCenter
            | Self::ChaletLabelMiddleCenter
            | Self::CenterLabel
            | Self::Button
            | Self::ButtonLabelFont
            | Self::RedButtonLabelFont
            | Self::MailPageButton
            | Self::CalculatorButton => JustifyContent::Center,
            Self::LabelMiddleRight
            | Self::BlankBoxMiddleRight
            | Self::RightLabel
            | Self::PostageLabel => JustifyContent::FlexEnd,
            Self::LabelUpperLeft
            | Self::BlankBoxUpperLeft
            | Self::BlankBoxMiddleLeft
            | Self::ChaletLabelMiddleLeft
            | Self::TextArea => JustifyContent::FlexStart,
        };
        node.align_items = match self {
            Self::LabelUpperLeft
            | Self::BlankBoxUpperLeft
            | Self::BlankBoxUpperCenter
            | Self::CenterLabel
            | Self::TextArea => AlignItems::FlexStart,
            Self::LabelMiddleCenter
            | Self::LabelMiddleRight
            | Self::BlankBoxMiddleLeft
            | Self::BlankBoxMiddleRight
            | Self::RightLabel
            | Self::ChaletLabelMiddleLeft
            | Self::ChaletLabelMiddleCenter
            | Self::Button
            | Self::ButtonLabelFont
            | Self::RedButtonLabelFont
            | Self::MailPageButton
            | Self::CalculatorButton
            | Self::PostageLabel => AlignItems::Center,
        };
        node.padding = match self {
            Self::LabelUpperLeft
            | Self::LabelMiddleCenter
            | Self::LabelMiddleRight
            | Self::ChaletLabelMiddleLeft
            | Self::ChaletLabelMiddleCenter => UiRect::new(
                px(0),
                px(0),
                px(EMAIL_UI_LABEL_PADDING_TOP),
                px(EMAIL_UI_LABEL_PADDING_BOTTOM),
            ),
            Self::TextArea => UiRect::all(px(3)),
            Self::Button | Self::ButtonLabelFont => UiRect::new(
                px(EMAIL_UI_BUTTON_PADDING_LEFT),
                px(EMAIL_UI_BUTTON_PADDING_RIGHT),
                px(EMAIL_UI_BUTTON_PADDING_TOP),
                px(EMAIL_UI_BUTTON_PADDING_BOTTOM),
            ),
            Self::BlankBoxUpperLeft
            | Self::BlankBoxUpperCenter
            | Self::BlankBoxMiddleLeft
            | Self::BlankBoxMiddleRight
            | Self::CenterLabel
            | Self::RightLabel
            | Self::RedButtonLabelFont
            | Self::MailPageButton
            | Self::CalculatorButton
            | Self::PostageLabel => UiRect::ZERO,
        };
        node.overflow = Overflow::clip();
    }

    pub const fn y_offset(self) -> f32 {
        match self {
            Self::PostageLabel => EMAIL_UI_POSTAGE_LABEL_Y_OFFSET,
            _ => 0.0,
        }
    }

    pub const fn source_style_name(self) -> &'static str {
        match self {
            Self::LabelUpperLeft
            | Self::LabelMiddleCenter
            | Self::LabelMiddleRight
            | Self::ChaletLabelMiddleLeft
            | Self::ChaletLabelMiddleCenter => "label",
            Self::BlankBoxUpperLeft
            | Self::BlankBoxUpperCenter
            | Self::BlankBoxMiddleLeft
            | Self::BlankBoxMiddleRight => "blankbox",
            Self::CenterLabel => "centerLabel",
            Self::RightLabel => "rightLabel",
            Self::TextArea => "textArea",
            Self::Button | Self::ButtonLabelFont => "button",
            Self::RedButtonLabelFont => "RedButton",
            Self::MailPageButton => "mailprevbut/mailnextbut",
            Self::CalculatorButton => "cacubut*",
            Self::PostageLabel => "vendorlistback",
        }
    }

    pub(super) fn content_bounds(self, rect: EmailUiRect) -> Vec2 {
        let (horizontal, vertical) = match self {
            Self::LabelUpperLeft
            | Self::LabelMiddleCenter
            | Self::LabelMiddleRight
            | Self::ChaletLabelMiddleLeft
            | Self::ChaletLabelMiddleCenter => (
                0.0,
                EMAIL_UI_LABEL_PADDING_TOP + EMAIL_UI_LABEL_PADDING_BOTTOM,
            ),
            Self::TextArea => (6.0, 6.0),
            Self::Button | Self::ButtonLabelFont => (
                EMAIL_UI_BUTTON_PADDING_LEFT + EMAIL_UI_BUTTON_PADDING_RIGHT,
                EMAIL_UI_BUTTON_PADDING_TOP + EMAIL_UI_BUTTON_PADDING_BOTTOM,
            ),
            Self::BlankBoxUpperLeft
            | Self::BlankBoxUpperCenter
            | Self::BlankBoxMiddleLeft
            | Self::BlankBoxMiddleRight
            | Self::CenterLabel
            | Self::RightLabel
            | Self::RedButtonLabelFont
            | Self::MailPageButton
            | Self::CalculatorButton
            | Self::PostageLabel => (0.0, 0.0),
        };
        Vec2::new(
            (rect.width - horizontal).max(1.0),
            (rect.height - vertical).max(1.0),
        )
    }
}

/// Guide-tab sender icon layers drawn by `Panel_EmailList.DoData`.
#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum EmailUiGuideIcon {
    SlotBackdrop,
    Sender,
}

/// Detail label moved right by `rectGuideIcon.width` in the guide tab.
#[derive(Clone, Copy, Component, Debug, PartialEq)]
pub struct EmailUiGuideIconShift {
    pub left: f32,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct EmailUiAttachmentSlot {
    pub slot: usize,
    pub outgoing: bool,
}
