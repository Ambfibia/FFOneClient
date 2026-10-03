use super::*;

pub(super) fn spawn_gameplay_reader(
    events: &SyncSender<NetworkEvent>,
    shutting_down: &Arc<AtomicBool>,
    reader_finished: &Arc<GameplayReaderFinished>,
    mut read_next: impl FnMut() -> ffone_net::Result<DecodedFrame> + Send + 'static,
) {
    let events = events.clone();
    let shutting_down = Arc::clone(shutting_down);
    let reader_finished = Arc::clone(reader_finished);
    thread::Builder::new()
        .name("ffone-shard-reader".to_owned())
        .spawn(move || {
            loop {
                match read_next() {
                    Ok(frame) => {
                        if events
                            .send(NetworkEvent::incoming_frame_0104(
                                NetworkStream0104::Shard,
                                frame,
                            ))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(error) => {
                        if !shutting_down.load(Ordering::Acquire) {
                            let _ = events.send(NetworkEvent::Disconnected {
                                reason: DisconnectReason0104::ShardTransportFailed(
                                    error.to_string(),
                                ),
                            });
                        }
                        break;
                    }
                }
            }
            reader_finished.finish();
        })
        .expect("spawn FFOne shard reader");
}

#[cfg(test)]
pub(super) fn spawn_login_world_reader(
    events: &SyncSender<NetworkEvent>,
    shutting_down: &Arc<AtomicBool>,
    selection_events: Sender<RetainedLoginSelection>,
    mut read_next: impl FnMut() -> ffone_net::Result<DecodedFrame> + Send + 'static,
) {
    let events = events.clone();
    let shutting_down = Arc::clone(shutting_down);
    thread::Builder::new()
        .name("ffone-login-reader".to_owned())
        .spawn(move || {
            loop {
                match read_next() {
                    Ok(frame) => {
                        if matches!(
                            frame.packet_type,
                            packet::P_LS2CL_REP_SHARD_SELECT_SUCC
                                | packet::P_LS2CL_REP_SHARD_SELECT_FAIL
                                | packet::P_LS2CL_REP_CHAR_SELECT_SUCC
                                | packet::P_LS2CL_REP_CHAR_SELECT_FAIL
                        ) {
                            if selection_events
                                .send(RetainedLoginSelection::Frame(frame))
                                .is_err()
                            {
                                break;
                            }
                            continue;
                        }
                        if events
                            .send(NetworkEvent::incoming_frame_0104(
                                NetworkStream0104::Login,
                                frame,
                            ))
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(error) => {
                        let error = error.to_string();
                        let expected_shutdown = shutting_down.swap(true, Ordering::AcqRel);
                        let _ = selection_events
                            .send(RetainedLoginSelection::TransportFailed(error.clone()));
                        if !expected_shutdown {
                            let _ = events.send(NetworkEvent::Disconnected {
                                reason: DisconnectReason0104::LoginTransportFailed(error),
                            });
                        }
                        break;
                    }
                }
            }
        })
        .expect("spawn FFOne retained login reader");
}
