use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PreviewScene {
    Player,
    Compose,
    Buddy,
    Calculator,
}

impl PreviewScene {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Player => "player",
            Self::Compose => "compose",
            Self::Buddy => "buddy",
            Self::Calculator => "calculator",
        }
    }

    pub(super) const fn hover(self) -> EmailUiButtonKind {
        match self {
            Self::Player => EmailUiButtonKind::SendMail,
            Self::Compose | Self::Buddy => EmailUiButtonKind::ComposeSend,
            Self::Calculator => EmailUiButtonKind::CalculatorDigit(5),
        }
    }
}

impl std::str::FromStr for PreviewScene {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "player" => Ok(Self::Player),
            "compose" => Ok(Self::Compose),
            "buddy" => Ok(Self::Buddy),
            "calculator" => Ok(Self::Calculator),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub(super) struct PreviewCli {
    pub(super) scene: PreviewScene,
    pub(super) language: String,
    pub(super) output: PathBuf,
}

#[derive(Resource)]
pub(super) struct PreviewAssets {
    pub(super) images: Vec<Handle<Image>>,
    pub(super) fonts: [Handle<Font>; 2],
}
