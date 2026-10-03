use super::*;

pub(super) fn connect_character_shard(
    pc_uid: i64,
    login_style: CharacterStyle0104,
    location: CharacterEntryLocation0104,
    ticket: ShardTicket,
) -> ffone_net::Result<(GameplayHandle, GameplayReceiver, WorldReady)> {
    let mut shard = ShardSession::connect(ticket)?;
    let player_id = shard.player_id();
    let server_time = shard.server_time();
    let load = shard.load_data().clone();
    let (position, angle, entry_move) =
        character_entry_pose(load.position(), load.angle(), location);
    if let Some(request) = entry_move {
        // Apply caller-owned entry poses before LOADING_COMPLETE so both the
        // server's initial chunk view and WorldReady observe the same place.
        shard.send_move(&request)?;
    }
    let bootstrap = shard.complete_loading()?.into_world_bootstrap();
    let connection = shard.into_gameplay()?;
    let (sender, receiver) = connection.into_parts();
    let shutting_down = Arc::new(AtomicBool::new(false));
    let reader_finished = Arc::new(GameplayReaderFinished::default());
    Ok((
        GameplayHandle {
            sender,
            player_id,
            shutting_down,
            reader_finished,
        },
        receiver,
        WorldReady {
            pc_uid,
            player_id,
            server_time,
            map_number: load.map_number(),
            hp: load.hp(),
            position,
            angle,
            login_style,
            load,
            bootstrap,
        },
    ))
}

pub(super) fn character_summaries(session: &LoginSession) -> Vec<CharacterSummary> {
    character_summaries_from(session.characters())
}

pub(super) fn character_summaries_from(characters: &[CharacterInfo0104]) -> Vec<CharacterSummary> {
    characters
        .iter()
        .map(|character| CharacterSummary {
            slot: character.slot(),
            level: character.level(),
            pc_uid: character.pc_uid(),
            first_name: character.first_name().to_string_lossy(),
            last_name: character.last_name().to_string_lossy(),
            position: character.position(),
            style: character.style(),
            equipment: character.equipment(),
        })
        .collect()
}

pub(super) fn send_login_required(events: &SyncSender<NetworkEvent>, operation: &str) {
    let _ = events.send(NetworkEvent::Error(format!(
        "{operation} requires a successful login"
    )));
}

pub(super) fn send_gameplay(
    gameplay: Option<&GameplayHandle>,
    events: &SyncSender<NetworkEvent>,
    operation: impl FnOnce(&GameplaySender) -> ffone_net::Result<()>,
) {
    let Some(gameplay) = gameplay else {
        let _ = events.send(NetworkEvent::Error(
            "gameplay packet requested before entering the world".to_owned(),
        ));
        return;
    };
    if let Err(error) = operation(&gameplay.sender) {
        let _ = events.send(NetworkEvent::Error(error.to_string()));
    }
}

pub(super) fn disconnect_gameplay(gameplay: &mut Option<GameplayHandle>) {
    if let Some(gameplay) = gameplay.take() {
        gameplay.stop();
    }
}
