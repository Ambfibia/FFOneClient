
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TutorialLocaleBranch {
    /// `(int)localized.local == 0` in the reference client.
    #[default]
    OriginalEnglish,
    /// Every non-zero value of `localized.local`.
    Localized,
}
