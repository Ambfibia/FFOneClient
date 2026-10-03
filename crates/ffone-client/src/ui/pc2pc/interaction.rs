use super::*;

pub const PC2PC_CHAT_SEND_HOVER_PATH: &str = "ui/en/pc2pc/chat-send-hover.png";

pub const PC2PC_BUTTON_PATH: &str = "ui/en/pc2pc/button.png";

pub const PC2PC_BUTTON_HOVER_PATH: &str = "ui/en/pc2pc/button-hover.png";

pub const PC2PC_CHAT_MAX_INPUT_CHARS: usize = 64;

pub const PC2PC_BUTTON_PADDING_LEFT: f32 = 6.0;

pub const PC2PC_BUTTON_PADDING_RIGHT: f32 = 6.0;

pub const PC2PC_BUTTON_PADDING_TOP: f32 = 3.0;

pub const PC2PC_BUTTON_PADDING_BOTTOM: f32 = 3.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pc2pcButtonLabel {
    Submit,
    Accept,
}

impl Pc2pcButtonLabel {
    #[must_use]
    pub const fn text(self) -> &'static str {
        match self {
            Self::Submit => "SUBMIT",
            Self::Accept => "ACCEPT",
        }
    }

    #[must_use]
    pub(super) fn localized_text(self) -> LocalizedText {
        match self {
            Self::Submit => LocalizedText::new("ui.pc2pc.button.submit", self.text()),
            Self::Accept => LocalizedText::new("ui.pc2pc.button.accept", self.text()),
        }
    }
}
