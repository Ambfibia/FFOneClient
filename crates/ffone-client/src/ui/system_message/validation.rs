use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SystemMessageIconIndexError(pub i32);

impl fmt::Display for SystemMessageIconIndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid legacy system-message icon index {}",
            self.0
        )
    }
}

impl Error for SystemMessageIconIndexError {}
