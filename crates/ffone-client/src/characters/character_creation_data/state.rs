use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedCreatorSelection {
    pub style: PcStyle0104,
    pub equipped: OnItem0104,
    pub selected_indices: OnItemIndex0104,
    pub look: NativePlayerLook,
}
