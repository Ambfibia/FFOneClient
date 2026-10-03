//! Local flight and GM chat command parsing.

use bevy::prelude::*;
use ffone_protocol::{
    DecodedFrame, GM_SET_VALUE_SPEED_0104, GmSetValueRequest0104, decode_gm_set_value_reply_0104,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FlightCommand {
    Toggle,
    Set(bool),
    Invalid,
}

pub(super) fn parse_local_flight_command(message: &str) -> Option<FlightCommand> {
    let mut parts = message.split_whitespace();
    if !parts.next()?.eq_ignore_ascii_case("/fly") {
        return None;
    }
    let command = match (parts.next(), parts.next()) {
        (None, None) => FlightCommand::Toggle,
        (Some(value), None) if value.eq_ignore_ascii_case("on") => FlightCommand::Set(true),
        (Some(value), None) if value.eq_ignore_ascii_case("off") => FlightCommand::Set(false),
        _ => FlightCommand::Invalid,
    };
    Some(command)
}

pub(super) const GM_CHAT_MAX_USER_LEVEL: i16 = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GmSpeedCommand {
    Set(i32),
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GmSpeedRequestError {
    Usage,
    AccessDenied,
    MissingPlayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GmNanoCommand {
    Give(i16),
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GmNanoRequestError {
    Usage,
    AccessDenied,
    UnknownNano(i16),
}

pub(super) fn parse_gm_nano_command(message: &str) -> Option<GmNanoCommand> {
    let mut parts = message.split_whitespace();
    if parts.next()? != "/nano" {
        return None;
    }
    let command = match (parts.next(), parts.next()) {
        (Some(value), None) => value
            .parse::<i16>()
            .ok()
            .filter(|nano_id| *nano_id > 0)
            .map(GmNanoCommand::Give)
            .unwrap_or(GmNanoCommand::Invalid),
        _ => GmNanoCommand::Invalid,
    };
    Some(command)
}

pub(super) fn build_gm_nano_request(
    command: GmNanoCommand,
    user_level: i16,
    nano_exists: impl FnOnce(i16) -> bool,
) -> Result<ffone_protocol::wire_0104::PcGiveNanoRequest0104, GmNanoRequestError> {
    let GmNanoCommand::Give(nano_id) = command else {
        return Err(GmNanoRequestError::Usage);
    };
    if user_level > GM_CHAT_MAX_USER_LEVEL {
        return Err(GmNanoRequestError::AccessDenied);
    }
    if !nano_exists(nano_id) {
        return Err(GmNanoRequestError::UnknownNano(nano_id));
    }
    Ok(ffone_protocol::wire_0104::PcGiveNanoRequest0104 { nano_id })
}

pub(super) fn nano_create_failure_message(error_code: i32) -> super::LocalizedText {
    // RustyFusion uses the existing 0104 failure field; unknown codes remain generic.
    match error_code {
        1 => super::LocalizedText::new("ui.chat.command.nano.unknown_server", "The server does not recognize this Nano ID."),
        2 => super::LocalizedText::new("ui.chat.command.nano.already_owned", "You already own this Nano."),
        _ => super::LocalizedText::new("ui.chat.command.nano.rejected", "Nano request rejected by the server (error {error}).")
            .with_arg("error", error_code.to_string()),
    }
}

#[cfg(test)]
mod nano_tests {
    use super::*;

    #[test]
    fn nano_50_is_an_id_request() {
        let command = parse_gm_nano_command("/nano 50").unwrap();
        assert_eq!(command, GmNanoCommand::Give(50));
        let request = build_gm_nano_request(command, 30, |id| id == 50).unwrap();
        assert_eq!(request.nano_id, 50);
        assert_eq!(build_gm_nano_request(command, 30, |_| false), Err(GmNanoRequestError::UnknownNano(50)));
    }

    #[test]
    fn ownership_is_only_reported_for_server_duplicate_error() {
        assert_eq!(nano_create_failure_message(2).key, "ui.chat.command.nano.already_owned");
        assert_eq!(nano_create_failure_message(1).key, "ui.chat.command.nano.unknown_server");
        assert_eq!(nano_create_failure_message(0).key, "ui.chat.command.nano.rejected");
    }
}

pub(super) fn is_server_chat_command_0104(message: &str) -> bool {
    message
        .strip_prefix('/')
        .is_some_and(|command| !command.is_empty())
}

pub(super) fn parse_gm_speed_command(message: &str) -> Option<GmSpeedCommand> {
    let mut parts = message.split_whitespace();
    if parts.next()? != "/speed" {
        return None;
    }
    let command = match (parts.next(), parts.next()) {
        (Some(value), None) => value
            .parse::<i32>()
            .map(GmSpeedCommand::Set)
            .unwrap_or(GmSpeedCommand::Invalid),
        _ => GmSpeedCommand::Invalid,
    };
    Some(command)
}

pub(super) fn build_gm_speed_request(
    command: GmSpeedCommand,
    user_level: i16,
    pc_id: Option<i32>,
) -> Result<GmSetValueRequest0104, GmSpeedRequestError> {
    let GmSpeedCommand::Set(speed) = command else {
        return Err(GmSpeedRequestError::Usage);
    };
    if user_level > GM_CHAT_MAX_USER_LEVEL {
        return Err(GmSpeedRequestError::AccessDenied);
    }
    let pc_id = pc_id.ok_or(GmSpeedRequestError::MissingPlayer)?;
    Ok(GmSetValueRequest0104::speed(pc_id, speed))
}

pub(super) fn decode_local_gm_speed_reply_0104(
    frame: &DecodedFrame,
    local_pc_id: Option<i32>,
) -> Result<Option<i32>, String> {
    let reply = decode_gm_set_value_reply_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 GM set-value reply: {error}"))?;
    let Some(reply) = reply else {
        return Ok(None);
    };
    if local_pc_id != Some(reply.pc_id) {
        return Err(format!(
            "GM set-value reply targets PC {}, but the local runtime PC is {:?}",
            reply.pc_id, local_pc_id
        ));
    }
    if reply.value_type != GM_SET_VALUE_SPEED_0104 {
        return Err(format!(
            "unsupported GM set-value reply type {} for PC {}",
            reply.value_type, reply.pc_id
        ));
    }
    Ok(Some(reply.value))
}
