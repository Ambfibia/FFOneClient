use super::*;

pub(super) struct ProductionHarness {
    pub(super) production: EmailProductionRuntime0104,
    pub(super) model: EmailUiModel,
    pub(super) actions: EmailUiOutbox,
    pub(super) audio: EmailUiAudioOutbox,
    pub(super) transport: EmailTransportOutbox,
    pub(super) network: EmailNetworkRuntime0104,
    pub(super) inbox: EmailNetworkInbox0104,
    pub(super) inventory: EmailInventoryAuthority0104,
}
