use super::*;

#[cfg(test)]
pub(super) enum RetainedLoginSelection {
    Frame(DecodedFrame),
    TransportFailed(String),
}

#[cfg(test)]
pub(super) const RETAINED_LOGIN_SELECTION_TIMEOUT: Duration = Duration::from_secs(65);
