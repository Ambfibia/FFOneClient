use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum LegacyQuickSlotMode {
    #[default]
    None = 0,
    Quick = 1,
}

impl LegacyQuickSlotMode {
    #[must_use]
    pub const fn draws_component(self) -> bool {
        matches!(self, Self::Quick)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum LegacyMacroChatMode {
    None = 0,
    #[default]
    Use = 1,
}

#[derive(Clone, Default, Resource)]
pub(super) struct QuickSlotUiRuntimeAssets(pub(super) Option<QuickSlotUiAssets>);
