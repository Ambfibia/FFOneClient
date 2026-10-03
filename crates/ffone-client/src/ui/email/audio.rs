use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmailUiAudioCue {
    Open,
    Close,
    ButtonSound,
    TabClick,
    EmailArrived,
    OutgoingChat,
    ActionSuccess,
    CharacterLimitMax,
}

#[derive(Default, Resource)]
pub struct EmailUiAudioOutbox(pub VecDeque<EmailUiAudioCue>);

impl EmailUiAudioOutbox {
    pub fn push(&mut self, cue: EmailUiAudioCue) {
        self.0.push_back(cue);
    }

    pub fn pop(&mut self) -> Option<EmailUiAudioCue> {
        self.0.pop_front()
    }
}
