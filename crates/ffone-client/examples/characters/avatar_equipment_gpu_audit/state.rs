use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ScreenshotMode {
    All,
    Failures,
}

pub(super) const fn status_label(status: NativeLookupStatus) -> &'static str {
    match status {
        NativeLookupStatus::VerifiedUnique => "verified_unique",
        NativeLookupStatus::VerifiedVariants => "verified_variants",
        NativeLookupStatus::Missing => "missing",
        NativeLookupStatus::Ambiguous => "ambiguous",
    }
}
