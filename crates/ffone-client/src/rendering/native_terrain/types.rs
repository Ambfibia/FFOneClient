use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VerifiedGameplayAttributes {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) values: Arc<[u8]>,
}
