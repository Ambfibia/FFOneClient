use super::*;

pub(super) fn publish_character_entry_failure(
    events: &SyncSender<NetworkEvent>,
    route: CharacterEntryRoute,
    pc_uid: i64,
    error: String,
) {
    let event = match route {
        CharacterEntryRoute::Selection => NetworkEvent::Error(error),
        CharacterEntryRoute::TutorialCompletion => {
            NetworkEvent::TutorialExitFailed { pc_uid, error }
        }
    };
    let _ = events.send(event);
}

pub(super) fn publish_world_then_spawn_gameplay_reader(
    events: &SyncSender<NetworkEvent>,
    shutting_down: &Arc<AtomicBool>,
    reader_finished: &Arc<GameplayReaderFinished>,
    world: WorldReady,
    read_next: impl FnMut() -> ffone_net::Result<DecodedFrame> + Send + 'static,
) -> bool {
    // The channel send happens before the reader thread exists, so every consumer observes the
    // complete typed bootstrap before the first live gameplay frame.
    if events.send(NetworkEvent::WorldReady(world)).is_err() {
        return false;
    }
    spawn_gameplay_reader(events, shutting_down, reader_finished, read_next);
    true
}
