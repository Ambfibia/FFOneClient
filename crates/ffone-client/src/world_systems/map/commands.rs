
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldMapOutboxEvent {
    RequestPresentNpcTypes { last_sync_time: u64 },
    ExitMode,
}
