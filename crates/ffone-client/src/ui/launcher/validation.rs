use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LauncherUiOpenError {
    NonFiniteTrigger,
    InvalidPowerRange,
    NegativeAimLimit,
    NonFiniteAvatarOrCamera,
}

impl fmt::Display for LauncherUiOpenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NonFiniteTrigger => "launcher trigger contains a non-finite field",
            Self::InvalidPowerRange => "launcher maxPower must be greater than minPower",
            Self::NegativeAimLimit => "launcher maxRotate x/y must be non-negative",
            Self::NonFiniteAvatarOrCamera => {
                "launcher avatar position and camera height must be finite"
            }
        })
    }
}

impl Error for LauncherUiOpenError {}
