//! Production boundary between the clean QuitMenu reducer and protocol-0104.
//!
//! The visual module intentionally emits typed destinations without owning a
//! socket or application state. This small boundary retains the requested
//! destination until the shard confirms `REP_PC_EXIT_SUCC(exitCode=1)`, and
//! classifies every fixed-size exit reply without losing the raw network frame.

use bevy::prelude::Resource;
use ffone_protocol::{DecodedFrame, PcExitPacket0104, decode_pc_exit_packet_0104};
use std::time::{Duration, Instant};

const PC_EXIT_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QuitMenuDestination {
    ChangeCharacter,
    QuitGame,
    QuitAndLogout,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuitMenuExitReply {
    Failure { pc_id: i32, error_code: i32 },
    Success { pc_id: i32, exit_code: i32 },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct QuitMenuRuntime {
    pending: Option<QuitMenuDestination>,
    pending_since: Option<Instant>,
    open_intent: bool,
}

impl QuitMenuRuntime {
    #[must_use]
    pub const fn pending(&self) -> Option<QuitMenuDestination> {
        self.pending
    }

    #[must_use]
    pub const fn is_waiting_for_server(&self) -> bool {
        self.pending.is_some()
    }

    pub fn begin(&mut self, destination: QuitMenuDestination) -> bool {
        if self.pending.is_some() {
            return false;
        }
        self.pending = Some(destination);
        self.pending_since = Some(Instant::now());
        true
    }

    #[must_use]
    pub fn exit_timed_out(&self) -> bool {
        self.pending_since
            .is_some_and(|since| since.elapsed() >= PC_EXIT_TIMEOUT)
    }

    pub fn finish(&mut self) -> Option<QuitMenuDestination> {
        self.pending_since = None;
        self.pending.take()
    }

    pub fn reset(&mut self) {
        self.pending = None;
        self.pending_since = None;
        self.open_intent = false;
    }

    pub fn capture_open_intent(&mut self, requested: bool) {
        self.open_intent = requested;
    }

    pub fn take_open_intent(&mut self) -> bool {
        std::mem::take(&mut self.open_intent)
    }
}

/// Strictly classify only the request/reply PC-exit handshake.
///
/// A client request appearing on the inbound frame stream is a protocol-order
/// error rather than a server reply. Unknown packet types remain available to
/// the rest of the production router through `Ok(None)`.
pub fn decode_quit_menu_exit_reply(
    frame: &DecodedFrame,
) -> Result<Option<QuitMenuExitReply>, String> {
    let Some(packet) = decode_pc_exit_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 PC-exit reply: {error}"))?
    else {
        return Ok(None);
    };
    match packet {
        PcExitPacket0104::Request(_) => {
            Err("protocol-0104 PC-exit request appeared on the inbound frame stream".to_owned())
        }
        PcExitPacket0104::Failure(failure) => Ok(Some(QuitMenuExitReply::Failure {
            pc_id: failure.pc_id,
            error_code: failure.error_code,
        })),
        PcExitPacket0104::Success(success) => Ok(Some(QuitMenuExitReply::Success {
            pc_id: success.pc_id,
            exit_code: success.exit_code,
        })),
    }
}

/// Exact primary TableData `m_pMessageData` text selected by
/// `GameFrame.ReceivePacket` for non-successful `iExitCode` values.
#[must_use]
pub const fn clean_pc_exit_code_message(exit_code: i32) -> Option<&'static str> {
    match exit_code {
        0 => Some("Socket disconnected."),
        2 => Some("You have lost your connection with the server."),
        3 => Some("Your connection has been terminated by a moderator."),
        4 => Some("Hacking has been terminated."),
        5 => Some("Packet error."),
        6 => Some("Live check termination"),
        7 => Some(
            "You have been disconnected due to an error. Please report what caused this on the forums. Thanks!",
        ),
        99 => Some("Server disconnection"),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
