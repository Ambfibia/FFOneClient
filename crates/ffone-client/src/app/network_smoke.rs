//! Headless network smoke run.

use super::login::{ClientConfig, PendingLogin};
use ffone_client::{
    coordinates::ProtocolPosition,
    network::{CharacterEntryLocation0104, NetworkBridge, NetworkCommand, NetworkEvent},
    quit_menu_runtime::{QuitMenuExitReply, decode_quit_menu_exit_reply},
};
use std::{
    process::ExitCode,
    thread,
    time::{Duration, Instant},
};

pub(super) fn run_network_smoke(config: &ClientConfig, mut pending: PendingLogin) -> ExitCode {
    let Some(credentials) = pending.credentials.take() else {
        eprintln!("network smoke cannot start: {}", pending.note);
        return ExitCode::from(2);
    };
    let bridge = NetworkBridge::start();
    let requested_exit_cycles = std::env::var("FFONE_NETWORK_SMOKE_EXIT_CYCLES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    let mut completed_exit_cycles = 0;
    let mut active_player_id = None;
    let mut last_character_uid = None;
    if let Err(error) = bridge.send(NetworkCommand::Login {
        login_address: config.login_address.clone(),
        username: credentials.username,
        password: credentials.password,
    }) {
        eprintln!("network smoke failed: {error}");
        return ExitCode::FAILURE;
    }

    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        for event in bridge.drain() {
            match event {
                NetworkEvent::Connecting => {
                    eprintln!("connecting to {}", config.login_address);
                }
                NetworkEvent::LoginMetadata { payment_flag } => {
                    eprintln!("login entitlement received: payment_flag={payment_flag}");
                }
                NetworkEvent::Characters(characters) => {
                    eprintln!("login accepted; {} character(s)", characters.len());
                    let selected = config
                        .character_uid
                        .and_then(|uid| characters.iter().find(|character| character.pc_uid == uid))
                        .or_else(|| characters.first());
                    let Some(selected) = selected else {
                        eprintln!("network smoke failed: account has no selectable characters");
                        return ExitCode::FAILURE;
                    };
                    eprintln!(
                        "selecting uid={} level={} name={} {}",
                        selected.pc_uid, selected.level, selected.first_name, selected.last_name
                    );
                    if let Err(error) = bridge.send(NetworkCommand::SelectCharacter {
                        pc_uid: selected.pc_uid,
                        location: CharacterEntryLocation0104::Saved,
                    }) {
                        eprintln!("network smoke failed: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                NetworkEvent::EnteringWorld { pc_uid } => {
                    eprintln!("entering world as uid={pc_uid}");
                }
                NetworkEvent::WorldReady(world) => {
                    let load_style = world.load.style();
                    if load_style.pc_uid != world.pc_uid {
                        eprintln!(
                            "network smoke failed: selected uid {} != shard load uid {}",
                            world.pc_uid, load_style.pc_uid
                        );
                        return ExitCode::FAILURE;
                    }
                    let native_position = ProtocolPosition::new(world.position).to_native();
                    let login_style_flags = [
                        world.login_style.appearance_flag,
                        world.login_style.tutorial_flag,
                        world.login_style.payzone_flag,
                    ];
                    eprintln!(
                        "world ready: uid={} player={} name={} {} level={} map={} hp={} \
                         login_style_flags={:?} nano_slots={:?} active_nano_slot={} \
                         position={:?} native={:?} bootstrap={} malformed={}",
                        world.pc_uid,
                        world.player_id,
                        load_style.first_name.to_string_lossy(),
                        load_style.last_name.to_string_lossy(),
                        world.load.level(),
                        world.map_number,
                        world.hp,
                        login_style_flags,
                        world.load.nano_slots(),
                        world.load.active_nano_slot(),
                        world.position,
                        native_position,
                        world.bootstrap.packets.len(),
                        world.bootstrap.decode_errors().count()
                    );
                    if requested_exit_cycles > 0 {
                        active_player_id = Some(world.player_id);
                        last_character_uid = Some(world.pc_uid);
                        if let Err(error) = bridge.send(NetworkCommand::ExitWorld) {
                            eprintln!("network smoke failed to request world exit: {error}");
                            return ExitCode::FAILURE;
                        }
                        continue;
                    }
                    return ExitCode::SUCCESS;
                }
                NetworkEvent::ReturnedToCharacterSelection(characters) => {
                    completed_exit_cycles += 1;
                    eprintln!("returned to character selection: cycle {completed_exit_cycles}");
                    if completed_exit_cycles == requested_exit_cycles {
                        return ExitCode::SUCCESS;
                    }
                    let selected = characters
                        .iter()
                        .find(|character| Some(character.pc_uid) != last_character_uid)
                        .or_else(|| characters.first());
                    let Some(selected) = selected else {
                        eprintln!("network smoke failed: retained roster is empty");
                        return ExitCode::FAILURE;
                    };
                    if let Err(error) = bridge.send(NetworkCommand::SelectCharacter {
                        pc_uid: selected.pc_uid,
                        location: CharacterEntryLocation0104::Saved,
                    }) {
                        eprintln!("network smoke failed to reselect character: {error}");
                        return ExitCode::FAILURE;
                    }
                }
                NetworkEvent::Frame(frame) if requested_exit_cycles > 0 => {
                    match decode_quit_menu_exit_reply(&frame) {
                        Ok(Some(QuitMenuExitReply::Success { pc_id, exit_code: 1 }))
                            if Some(pc_id) == active_player_id =>
                        {
                            if let Err(error) =
                                bridge.send(NetworkCommand::ReturnToCharacterSelection)
                            {
                                eprintln!("network smoke failed to return to roster: {error}");
                                return ExitCode::FAILURE;
                            }
                        }
                        Ok(Some(reply)) => {
                            eprintln!("network smoke failed: unexpected exit reply {reply:?}");
                            return ExitCode::FAILURE;
                        }
                        Ok(None) => {}
                        Err(error) => {
                            eprintln!("network smoke failed: {error}");
                            return ExitCode::FAILURE;
                        }
                    }
                }
                NetworkEvent::TutorialExitFailed { pc_uid, error } => {
                    eprintln!("network smoke failed while exiting tutorial for {pc_uid}: {error}");
                    return ExitCode::FAILURE;
                }
                NetworkEvent::Error(error) => {
                    eprintln!("network smoke failed: {error}");
                    return ExitCode::FAILURE;
                }
                NetworkEvent::Disconnected { reason } => {
                    eprintln!("network smoke failed: disconnected before world ready: {reason}");
                    return ExitCode::FAILURE;
                }
                NetworkEvent::MalformedFrame0104 {
                    stream,
                    frame,
                    expected_payload_size,
                } => {
                    eprintln!(
                        "network smoke failed: malformed {stream} packet 0x{:08x}: expected {} payload bytes, got {}",
                        frame.packet_type,
                        expected_payload_size,
                        frame.payload.len()
                    );
                    return ExitCode::FAILURE;
                }
                NetworkEvent::CharacterNameChecked(_)
                | NetworkEvent::CharacterNameSaved(_)
                | NetworkEvent::CharacterCreated { .. }
                | NetworkEvent::CharacterDeleted { .. }
                | NetworkEvent::CharacterNameChanged(_)
                | NetworkEvent::DuplicateSessionExitRequested
                | NetworkEvent::CharacterOperationRejected { .. }
                | NetworkEvent::LoginFrame(_)
                | NetworkEvent::Frame(_) => {}
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    eprintln!("network smoke timed out after 30 seconds");
    ExitCode::FAILURE
}
