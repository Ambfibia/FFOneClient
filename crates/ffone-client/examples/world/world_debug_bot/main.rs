//! Reproducible OpenFusion world-session bot for native client visual checks.
//!
//! The command provisions one character through the same login packets as the
//! production UI, enters either the ordinary world or the tutorial through the
//! same typed entry contract as the production client, optionally walks a short
//! distance, and keeps the session alive. It never edits the OpenFusion database
//! directly.

use std::{
    env,
    path::PathBuf,
    process::ExitCode,
    thread,
    time::{Duration, Instant},
};

use ffone_client::{
    character_creation_data::CharacterCreationData,
    character_creation_ui::{CharacterAppearance, CharacterGender},
    network::{
        CharacterEntryLocation0104, CharacterSummary, NetworkBridge, NetworkCommand, NetworkEvent,
        WorldReady,
    },
};
use ffone_protocol::{
    CharacterCreateRequest0104, CharacterNameCheckRequest0104, PcMoveRequest0104,
    PcStopRequest0104,
    packet::{P_FE2CL_AROUND_DEL_PC, P_FE2CL_PC_MOVE, P_FE2CL_PC_NEW, P_FE2CL_PC_STOP},
};

const SESSION_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Debug)]
struct Config {
    asset_root: PathBuf,
    login_address: String,
    username: String,
    password_env: String,
    first_name: String,
    last_name: String,
    appearance_variant: u8,
    enter_tutorial: bool,
    provision_only: bool,
    move_x: i32,
    move_y: i32,
    move_delay: Duration,
    hold: Duration,
}

impl Config {
    fn parse() -> Result<Self, String> {
        let mut config = Self {
            asset_root: PathBuf::from("assets/game"),
            login_address: "127.0.0.1:23000".to_owned(),
            username: String::new(),
            password_env: "FFONE_PASSWORD".to_owned(),
            first_name: "Debug".to_owned(),
            last_name: "Player".to_owned(),
            appearance_variant: 0,
            enter_tutorial: false,
            provision_only: false,
            move_x: 0,
            move_y: 0,
            move_delay: Duration::from_secs(3),
            hold: Duration::from_secs(120),
        };
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            if matches!(flag.as_str(), "-h" | "--help") {
                print_help();
                std::process::exit(0);
            }
            if flag == "--provision-only" {
                config.provision_only = true;
                continue;
            }
            if flag == "--enter-tutorial" {
                config.enter_tutorial = true;
                continue;
            }
            let value = args
                .next()
                .ok_or_else(|| format!("{flag} requires a value"))?;
            match flag.as_str() {
                "--asset-root" => config.asset_root = PathBuf::from(value),
                "--login-address" => config.login_address = value,
                "--username" => config.username = value,
                "--password-env" => config.password_env = value,
                "--first-name" => config.first_name = value,
                "--last-name" => config.last_name = value,
                "--appearance-variant" => {
                    config.appearance_variant = value
                        .parse()
                        .map_err(|_| format!("invalid --appearance-variant value {value:?}"))?;
                }
                "--move-x" => {
                    config.move_x = value
                        .parse()
                        .map_err(|_| format!("invalid --move-x value {value:?}"))?;
                }
                "--move-y" => {
                    config.move_y = value
                        .parse()
                        .map_err(|_| format!("invalid --move-y value {value:?}"))?;
                }
                "--move-delay-ms" => {
                    let millis = value
                        .parse()
                        .map_err(|_| format!("invalid --move-delay-ms value {value:?}"))?;
                    config.move_delay = Duration::from_millis(millis);
                }
                "--hold-seconds" => {
                    let seconds = value
                        .parse()
                        .map_err(|_| format!("invalid --hold-seconds value {value:?}"))?;
                    config.hold = Duration::from_secs(seconds);
                }
                _ => return Err(format!("unknown option {flag:?}")),
            }
        }
        if config.username.is_empty() {
            return Err("--username is required".to_owned());
        }
        if config.password_env.is_empty() {
            return Err("--password-env may not be empty".to_owned());
        }
        if config.first_name.encode_utf16().count() > 8 {
            return Err("--first-name must fit FixedUtf16<9> (at most 8 UTF-16 units)".to_owned());
        }
        if config.last_name.encode_utf16().count() > 16 {
            return Err("--last-name must fit FixedUtf16<17> (at most 16 UTF-16 units)".to_owned());
        }
        if config.appearance_variant > 1 {
            return Err("--appearance-variant must be 0 or 1".to_owned());
        }
        Ok(config)
    }

    fn appearance(&self) -> CharacterAppearance {
        match self.appearance_variant {
            0 => CharacterAppearance::default(),
            1 => CharacterAppearance {
                gender: CharacterGender::Girl,
                skin_color: 6,
                hair_color: 9,
                eye_color: 4,
                body: 2,
                height: 3,
                hair: 8,
                face: 4,
                shirt: 9,
                pants: 10,
                shoes: 11,
            },
            _ => unreachable!("appearance variant was validated while parsing"),
        }
    }
}

fn print_help() {
    println!(
        "FFOne ordinary-world debug bot\n\n\
Usage:\n  cargo run -p ffone-client --example world_debug_bot -- [OPTIONS]\n\n\
Required:\n  --username <NAME>          OpenFusion account (autocreated when enabled)\n\n\
Options:\n  --asset-root <DIR>         Native assets [default: assets/game]\n  --login-address <ADDR>     Login server [default: 127.0.0.1:23000]\n  --password-env <NAME>      Password environment variable [default: FFONE_PASSWORD]\n  --first-name <NAME>        Character first name [default: Debug]\n  --last-name <NAME>         Character last name [default: Player]\n  --enter-tutorial           Enter a new character at the scripted tutorial pose\n  --provision-only           Create/finish/skip tutorial, then disconnect\n  --move-x <UNITS>           Walk by this many protocol X units after entry\n  --move-y <UNITS>           Walk by this many protocol Y units after entry\n  --move-delay-ms <MS>       Delay before MOVE [default: 3000]\n  --hold-seconds <SECONDS>   Keep the shard session alive [default: 120]\n"
    );
    println!("  --appearance-variant <0|1> 0=default boy, 1=distinct girl [default: 0]");
}

fn main() -> ExitCode {
    let config = match Config::parse() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("world debug bot: {error}");
            eprintln!("use --help for usage");
            return ExitCode::from(2);
        }
    };
    let password = match env::var(&config.password_env) {
        Ok(password) if !password.is_empty() => password,
        _ => {
            eprintln!(
                "world debug bot: environment variable {} is not set",
                config.password_env
            );
            return ExitCode::from(2);
        }
    };
    let character_data = match CharacterCreationData::open(&config.asset_root) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("world debug bot: native character data failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let (bridge, world) = match enter_world(&config, &password, &character_data) {
        Ok(session) => session,
        Err(error) => {
            eprintln!("world debug bot: {error}");
            return ExitCode::FAILURE;
        }
    };
    println!(
        "DEBUG_WORLD_READY username={} uid={} player={} map={} position={:?} tutorialFlag={}",
        config.username,
        world.pc_uid,
        world.player_id,
        world.map_number,
        world.position,
        world.login_style.tutorial_flag
    );
    let hand = world.load.equipment()[ffone_protocol::CharacterEquipSlot0104::Hand as usize];
    println!(
        "DEBUG_HAND_EQUIPMENT username={} type={} item={} option={}",
        config.username, hand.item_type, hand.item_id, hand.option
    );

    if config.provision_only {
        let _ = bridge.send(NetworkCommand::ExitWorld);
        println!("DEBUG_PROVISIONED username={}", config.username);
        return ExitCode::SUCCESS;
    }

    if config.move_x != 0 || config.move_y != 0 {
        drain_for(&bridge, config.move_delay);
        let mut destination = world.position;
        destination[0] = destination[0].saturating_add(config.move_x);
        destination[1] = destination[1].saturating_add(config.move_y);
        let velocity_x = config.move_x.signum() as f32;
        let velocity_y = config.move_y.signum() as f32;
        let movement = PcMoveRequest0104 {
            client_time: 0,
            position: destination,
            velocity: [velocity_x, velocity_y, 0.0],
            angle: if config.move_x.is_negative() { 0 } else { 180 },
            key_value: if config.move_y != 0 { 1 } else { 7 },
            speed: 600,
        };
        bridge
            .send(NetworkCommand::Move(movement))
            .map_err(|error| format!("MOVE failed: {error}"))
            .unwrap_or_else(|error| {
                eprintln!("world debug bot: {error}");
                std::process::exit(1);
            });
        println!(
            "DEBUG_MOVE_SENT username={} from={:?} to={destination:?}",
            config.username, world.position
        );
        drain_for(&bridge, Duration::from_millis(850));
        bridge
            .send(NetworkCommand::Stop(PcStopRequest0104 {
                client_time: 0,
                position: destination,
            }))
            .map_err(|error| format!("STOP failed: {error}"))
            .unwrap_or_else(|error| {
                eprintln!("world debug bot: {error}");
                std::process::exit(1);
            });
        println!(
            "DEBUG_STOP_SENT username={} position={destination:?}",
            config.username
        );
    }

    println!(
        "DEBUG_HOLDING username={} seconds={}",
        config.username,
        config.hold.as_secs()
    );
    drain_for(&bridge, config.hold);
    let _ = bridge.send(NetworkCommand::ExitWorld);
    ExitCode::SUCCESS
}

fn enter_world(
    config: &Config,
    password: &str,
    data: &CharacterCreationData,
) -> Result<(NetworkBridge, WorldReady), String> {
    let bridge = NetworkBridge::start();
    bridge.send(NetworkCommand::Login {
        login_address: config.login_address.clone(),
        username: config.username.clone(),
        password: password.to_owned(),
    })?;

    let deadline = Instant::now() + SESSION_TIMEOUT;
    let mut requested_name = false;
    let mut requested_appearance_uid = None;
    let mut requested_entry_uid = None;
    while Instant::now() < deadline {
        for event in bridge.drain() {
            match event {
                NetworkEvent::Characters(characters) => {
                    if let Some(character) = characters.first() {
                        if character.style.appearance_flag == 0 {
                            if requested_appearance_uid != Some(character.pc_uid) {
                                send_debug_appearance(
                                    &bridge,
                                    data,
                                    character,
                                    &config.appearance(),
                                )?;
                                requested_appearance_uid = Some(character.pc_uid);
                            }
                        } else if requested_entry_uid != Some(character.pc_uid) {
                            let command =
                                if character.style.tutorial_flag == 0 && config.enter_tutorial {
                                    println!(
                                        "DEBUG_ENTER_TUTORIAL username={} uid={}",
                                        config.username, character.pc_uid
                                    );
                                    NetworkCommand::SelectCharacter {
                                        pc_uid: character.pc_uid,
                                        location: CharacterEntryLocation0104::Scripted {
                                            position: [54_700, 65_500, -10_540],
                                            angle: 270,
                                        },
                                    }
                                } else if character.style.tutorial_flag == 0 {
                                    println!(
                                        "DEBUG_SKIP_TUTORIAL username={} uid={}",
                                        config.username, character.pc_uid
                                    );
                                    NetworkCommand::CompleteTutorial {
                                        pc_uid: character.pc_uid,
                                    }
                                } else {
                                    NetworkCommand::SelectCharacter {
                                        pc_uid: character.pc_uid,
                                        location: CharacterEntryLocation0104::Saved,
                                    }
                                };
                            bridge.send(command)?;
                            requested_entry_uid = Some(character.pc_uid);
                        }
                    } else if !requested_name {
                        let request = CharacterNameCheckRequest0104::new(
                            &config.first_name,
                            &config.last_name,
                            0,
                            0,
                            0,
                        )
                        .map_err(|error| format!("character name cannot be encoded: {error}"))?;
                        bridge.send(NetworkCommand::ReserveCharacterName {
                            request,
                            slot: 1,
                            gender: config.appearance().gender as i8,
                        })?;
                        requested_name = true;
                        println!(
                            "DEBUG_NAME_REQUESTED username={} name={} {}",
                            config.username, config.first_name, config.last_name
                        );
                    }
                }
                NetworkEvent::CharacterNameSaved(saved) => {
                    if requested_appearance_uid != Some(saved.pc_uid) {
                        let selection = data
                            .resolve_creator(
                                saved.pc_uid,
                                0,
                                &saved.first_name.to_string_lossy(),
                                &saved.last_name.to_string_lossy(),
                                &config.appearance(),
                            )
                            .map_err(|error| {
                                format!("default character appearance cannot resolve: {error}")
                            })?;
                        bridge.send(NetworkCommand::CreateCharacter(
                            CharacterCreateRequest0104 {
                                style: selection.style,
                                equipped: selection.equipped,
                                selected_indices: selection.selected_indices,
                            },
                        ))?;
                        requested_appearance_uid = Some(saved.pc_uid);
                        println!(
                            "DEBUG_APPEARANCE_REQUESTED username={} uid={}",
                            config.username, saved.pc_uid
                        );
                    }
                }
                NetworkEvent::CharacterCreated { response, .. } => {
                    println!(
                        "DEBUG_CHARACTER_CREATED username={} uid={}",
                        config.username, response.style.pc_uid
                    );
                }
                NetworkEvent::EnteringWorld { pc_uid } => {
                    println!(
                        "DEBUG_ENTERING_WORLD username={} uid={pc_uid}",
                        config.username
                    );
                }
                NetworkEvent::WorldReady(world) => return Ok((bridge, world)),
                NetworkEvent::CharacterOperationRejected { stage, error_code } => {
                    return Err(format!(
                        "character operation {stage:?} rejected with error {error_code}"
                    ));
                }
                NetworkEvent::TutorialExitFailed { pc_uid, error } => {
                    return Err(format!("tutorial exit failed for {pc_uid}: {error}"));
                }
                NetworkEvent::Error(error) => return Err(error),
                NetworkEvent::Disconnected { reason } => {
                    return Err(format!("server disconnected before WorldReady: {reason}"));
                }
                NetworkEvent::MalformedFrame0104 {
                    stream,
                    frame,
                    expected_payload_size,
                } => {
                    return Err(format!(
                        "malformed {stream} packet 0x{:08x}: expected {} payload bytes, got {}",
                        frame.packet_type,
                        expected_payload_size,
                        frame.payload.len()
                    ));
                }
                NetworkEvent::Connecting
                | NetworkEvent::LoginMetadata { .. }
                | NetworkEvent::CharacterNameChecked(_)
                | NetworkEvent::CharacterNameChanged(_)
                | NetworkEvent::DuplicateSessionExitRequested
                | NetworkEvent::CharacterDeleted { .. }
                | NetworkEvent::LoginFrame(_)
                | NetworkEvent::Frame(_) => {}
            }
        }
        thread::sleep(Duration::from_millis(10));
    }
    Err(format!(
        "timed out after {} seconds before WorldReady",
        SESSION_TIMEOUT.as_secs()
    ))
}

fn send_debug_appearance(
    bridge: &NetworkBridge,
    data: &CharacterCreationData,
    character: &CharacterSummary,
    appearance: &CharacterAppearance,
) -> Result<(), String> {
    let selection = data
        .resolve_creator(
            character.pc_uid,
            i8::from(character.style.name_check != 0),
            &character.first_name,
            &character.last_name,
            appearance,
        )
        .map_err(|error| format!("default character appearance cannot resolve: {error}"))?;
    bridge.send(NetworkCommand::CreateCharacter(
        CharacterCreateRequest0104 {
            style: selection.style,
            equipped: selection.equipped,
            selected_indices: selection.selected_indices,
        },
    ))
}

fn drain_for(bridge: &NetworkBridge, duration: Duration) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        for event in bridge.drain() {
            match event {
                NetworkEvent::Error(error) => eprintln!("DEBUG_NETWORK_ERROR {error}"),
                NetworkEvent::Disconnected { reason } => {
                    eprintln!("DEBUG_NETWORK_DISCONNECTED {reason}");
                    return;
                }
                NetworkEvent::Frame(frame)
                    if matches!(
                        frame.packet_type,
                        P_FE2CL_PC_NEW | P_FE2CL_PC_MOVE | P_FE2CL_PC_STOP | P_FE2CL_AROUND_DEL_PC
                    ) =>
                {
                    println!(
                        "DEBUG_REMOTE_PC_FRAME packet=0x{:08x} payloadBytes={}",
                        frame.packet_type,
                        frame.payload.len()
                    );
                }
                _ => {}
            }
        }
        thread::sleep(Duration::from_millis(20));
    }
}
