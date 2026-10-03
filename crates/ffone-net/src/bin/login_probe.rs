use std::{env, process::ExitCode};

use ffone_net::LoginSession;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let address = args.next().unwrap_or_else(|| "127.0.0.1:23000".to_owned());
    let username = args.next().unwrap_or_else(|| "Ambfibia".to_owned());
    let password = env::var("FFONE_PASSWORD").unwrap_or_default();
    match LoginSession::connect_password(address, &username, &password) {
        Ok(session) => {
            println!(
                "login ok: characters={}, selected_slot={}",
                session.characters().len(),
                session.login_success().selected_slot
            );
            for character in session.characters() {
                println!(
                    "slot={} uid={} name={} {} level={}",
                    character.slot(),
                    character.pc_uid(),
                    character.first_name().to_string_lossy(),
                    character.last_name().to_string_lossy(),
                    character.level(),
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
