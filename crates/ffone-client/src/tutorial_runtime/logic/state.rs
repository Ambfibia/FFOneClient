
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TutorialJournalMode {
    #[default]
    Hidden,
    /// `eJM_Allow`, which is `CheckUI(11)`.
    Allow,
    /// `eJM_Reward`, which is `CheckUI(12)`.
    Reward,
    /// Any visible journal mode other than Allow or Reward: `CheckUI(13)`.
    Other,
}
