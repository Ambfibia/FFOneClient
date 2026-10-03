//! Mutating acceptance probe for a replacement server; use an isolated database only.
//! All credentials and the endpoint come from the environment, as in local_smoke.
use std::{env, error::Error};

use ffone_net::{LoginSession, ShardSession};
use ffone_protocol::{
    CharacterCreateRequest0104, CharacterNameCheckRequest0104, CharacterNameSaveRequest0104,
    CharacterTutorialSaveRequest0104, OnItem0104, OnItemIndex0104, PcStyle0104,
    PresentNpcTypesReply0104, PresentNpcTypesRequest0104, WirePayload, packet,
    wire_0104::NanoBookSubsetReply0104,
};

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("FFONE_ISOLATED_TEST_DATABASE").as_deref() != Ok("1") {
        return Err("set FFONE_ISOLATED_TEST_DATABASE=1 for an isolated test server".into());
    }
    let mut login = LoginSession::connect_password(
        env::var("FFONE_LOGIN_ADDRESS")?,
        &env::var("FFONE_USERNAME")?,
        &env::var("FFONE_PASSWORD")?,
    )?;
    println!("PASS login: {} characters", login.characters().len());
    if login.characters().is_empty() {
        create_test_character(&mut login, 1, "Proto", "Hero")?;
        println!("PASS character creation: expanded palette limits");
    }
    if env::var_os("FFONE_SMOKE_CREATE_SECOND").is_some() && login.characters().len() == 1 {
        create_test_character(&mut login, 2, "Proto", "Switch")?;
        println!("PASS second character creation");
    }
    let uid = login.first_character_uid()?;
    let tutorial_entry = env::var_os("FFONE_SMOKE_TUTORIAL_ENTRY").is_some();
    if tutorial_entry {
        assert_eq!(
            login.characters()[0].style().tutorial_flag,
            0,
            "tutorial entry requires a fresh, unfinished test character"
        );
    } else if login.characters()[0].style().tutorial_flag == 0 {
        login.send_tutorial_completion(&CharacterTutorialSaveRequest0104 {
            pc_uid: uid,
            tutorial_flag: 1,
        })?;
    }
    let mut shard = ShardSession::connect(login.select_character(uid)?)?;
    let player_id = shard.player_id();
    println!("PASS shard entry: uid={uid}, player_id={player_id}");
    if tutorial_entry {
        // Match the native tutorial's scripted entry before LOADING_COMPLETE.
        shard.send_move(&ffone_protocol::PcMoveRequest0104 {
            position: [54_700, 65_500, -10_540],
            angle: 270,
            client_time: 0,
            velocity: [0.0; 3],
            key_value: 0,
            speed: 0,
        })?;
    }
    let loaded = shard.complete_loading()?;
    let mut failures = Vec::new();
    let mut nano_pages = 0;
    for frame in &loaded.prelude {
        if frame.packet_type == packet::P_FE2CL_REP_NANO_BOOK_SUBSET {
            let page = NanoBookSubsetReply0104::decode(&frame.payload)?;
            nano_pages += 1;
            if page.pcuid != i64::from(player_id) {
                failures.push(format!(
                    "Nano book owner {} differs from player ID {player_id}",
                    page.pcuid
                ));
            }
        }
    }
    if nano_pages == 0 {
        failures.push("no Nano book pages received".to_owned());
    }
    let bootstrap = loaded.into_world_bootstrap();
    if bootstrap.has_decode_errors() {
        failures.push("initial world entity decoding failed".to_owned());
    }
    println!("PASS loading complete; inspected {nano_pages} Nano book pages");
    let (sender, mut receiver) = shard.into_gameplay()?.into_parts();
    sender.send_present_npc_types(&PresentNpcTypesRequest0104::default())?;
    loop {
        let frame = receiver.read_next()?;
        if frame.packet_type == packet::P_FE2CL_REP_PRESENT_NPC_TYPES {
            let reply = PresentNpcTypesReply0104::decode(&frame.payload)?;
            println!(
                "PASS NPC presence: {} types in first page",
                reply.npc_types.len()
            );
            break;
        }
    }
    sender.send_pc_exit(player_id)?;
    loop {
        if receiver.read_next()?.packet_type == packet::P_FE2CL_REP_PC_EXIT_SUCC {
            break;
        }
    }
    println!("PASS clean world exit");
    failures.sort();
    failures.dedup();
    if !failures.is_empty() {
        return Err(failures.join("; ").into());
    }
    println!("PASS replacement server smoke");
    Ok(())
}

fn create_test_character(
    login: &mut LoginSession,
    slot: i8,
    first_name: &str,
    last_name: &str,
) -> Result<(), Box<dyn Error>> {
    let check = CharacterNameCheckRequest0104::new(first_name, last_name, 0, 0, 0)?;
    let checked = login.check_character_name(&check)?;
    let saved = login.save_character_name(&CharacterNameSaveRequest0104::from_check(
        slot, 1, &check, &checked,
    ))?;
    login.create_character(&CharacterCreateRequest0104 {
        style: PcStyle0104 {
            pc_uid: saved.pc_uid,
            name_check: 0,
            first_name: saved.first_name,
            last_name: saved.last_name,
            gender: 1,
            face_style: 1,
            hair_style: 1,
            hair_color: 54,
            skin_color: 36,
            eye_color: 10,
            height: 0,
            body: 0,
            class: 0,
        },
        equipped: OnItem0104 {
            upper_body_id: 1,
            lower_body_id: 1,
            foot_id: 1,
            ..Default::default()
        },
        selected_indices: OnItemIndex0104 {
            upper_body_index: 0,
            lower_body_index: 0,
            foot_index: 0,
            face_style_index: 0,
            hair_style_index: 0,
        },
    })?;
    Ok(())
}
