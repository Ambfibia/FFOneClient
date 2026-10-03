use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialPlayerPresentationError {
    UnsupportedProtocolGender(i8),
    UnsupportedDirectAvatarEmote(String),
}

impl fmt::Display for TutorialPlayerPresentationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProtocolGender(value) => {
                write!(formatter, "unsupported protocol player gender {value}")
            }
            Self::UnsupportedDirectAvatarEmote(name) => write!(
                formatter,
                "unsupported direct tutorial AvatarEmote {name:?}; expected staying, standup, or run"
            ),
        }
    }
}

impl Error for TutorialPlayerPresentationError {}
