use std::{env, io};

use ffone_net::{LoginSession, ShardSession};

fn required_env(name: &'static str) -> Result<String, io::Error> {
    env::var(name).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("required environment variable {name} is not set"),
        )
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The address may come from the optional environment override or the first CLI argument.
    // No endpoint or credential is compiled into this smoke test.
    let login_address = env::var("FFONE_LOGIN_ADDRESS")
        .ok()
        .or_else(|| env::args().nth(1))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "set FFONE_LOGIN_ADDRESS or pass the login address as the first argument",
            )
        })?;
    let username = required_env("FFONE_USERNAME")?;
    let password = required_env("FFONE_PASSWORD")?;

    let mut login = LoginSession::connect_password(&login_address, &username, &password)?;
    eprintln!(
        "login accepted; OpenFusion returned {} character(s)",
        login.characters().len()
    );
    for character in login.characters() {
        eprintln!(
            "uid={} level={} name={} {}",
            character.pc_uid(),
            character.level(),
            character.first_name().to_string_lossy(),
            character.last_name().to_string_lossy()
        );
    }

    let pc_uid = match env::var("FFONE_CHARACTER_UID") {
        Ok(value) => value.parse::<i64>().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("FFONE_CHARACTER_UID is not a valid i64: {error}"),
            )
        })?,
        Err(_) => login.first_character_uid()?,
    };
    let ticket = login.select_character(pc_uid)?;
    eprintln!(
        "selected uid={pc_uid}; shard endpoint={}",
        ticket.endpoint()
    );
    drop(login);

    let mut shard = ShardSession::connect(ticket)?;
    eprintln!(
        "entered shard as player id={} map={} position={:?}",
        shard.player_id(),
        shard.load_data().map_number(),
        shard.load_data().position()
    );
    let loaded = shard.complete_loading()?;
    eprintln!(
        "loading complete; {} initial world packet(s) arrived before acknowledgement",
        loaded.prelude.len()
    );
    Ok(())
}
