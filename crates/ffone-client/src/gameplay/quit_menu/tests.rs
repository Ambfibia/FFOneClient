use super::PC_EXIT_TIMEOUT;
use crate::quit_menu_runtime::*;
use ffone_protocol::{
    PcExitFailure0104, PcExitRequest0104, PcExitSuccess0104, WirePayload, packet,
};
use std::time::Instant;

fn frame(packet_type: u32, payload: Vec<u8>) -> DecodedFrame {
    DecodedFrame {
        packet_type,
        flags: 0,
        checksum: 0,
        payload,
    }
}

#[test]
fn runtime_latches_one_destination_and_one_frame_open_intent() {
    let mut runtime = QuitMenuRuntime::default();
    runtime.capture_open_intent(true);
    assert!(runtime.take_open_intent());
    assert!(!runtime.take_open_intent());

    assert!(runtime.begin(QuitMenuDestination::ChangeCharacter));
    assert!(!runtime.begin(QuitMenuDestination::QuitGame));
    assert!(runtime.is_waiting_for_server());
    assert!(!runtime.exit_timed_out());
    assert_eq!(runtime.finish(), Some(QuitMenuDestination::ChangeCharacter));
    assert!(!runtime.is_waiting_for_server());
    assert!(!runtime.exit_timed_out());
}

#[test]
fn exit_wait_is_bounded_and_reset_after_failure() {
    let mut runtime = QuitMenuRuntime::default();
    assert!(runtime.begin(QuitMenuDestination::ChangeCharacter));
    runtime.pending_since = Some(Instant::now() - PC_EXIT_TIMEOUT);
    assert!(runtime.exit_timed_out());
    assert_eq!(runtime.finish(), Some(QuitMenuDestination::ChangeCharacter));
    assert!(!runtime.exit_timed_out());
    assert!(runtime.begin(QuitMenuDestination::ChangeCharacter));
    assert!(!runtime.exit_timed_out());
}

#[test]
fn decoder_accepts_only_strict_inbound_failure_and_success_replies() {
    let failure = PcExitFailure0104 {
        pc_id: 81,
        error_code: -7,
    };
    assert_eq!(
        decode_quit_menu_exit_reply(
            &frame(packet::P_FE2CL_REP_PC_EXIT_FAIL, failure.encode(),)
        ),
        Ok(Some(QuitMenuExitReply::Failure {
            pc_id: 81,
            error_code: -7,
        }))
    );

    let success = PcExitSuccess0104 {
        pc_id: 81,
        exit_code: 1,
    };
    assert_eq!(
        decode_quit_menu_exit_reply(
            &frame(packet::P_FE2CL_REP_PC_EXIT_SUCC, success.encode(),)
        ),
        Ok(Some(QuitMenuExitReply::Success {
            pc_id: 81,
            exit_code: 1,
        }))
    );
    assert_eq!(
        decode_quit_menu_exit_reply(&frame(packet::P_FE2CL_PC_EXIT, vec![0; 8])),
        Ok(None)
    );
    assert!(
        decode_quit_menu_exit_reply(&frame(packet::P_FE2CL_REP_PC_EXIT_SUCC, vec![0; 7],))
            .is_err()
    );
}

#[test]
fn inbound_request_is_rejected_and_clean_disconnect_messages_are_exact() {
    assert!(
        decode_quit_menu_exit_reply(&frame(
            packet::P_CL2FE_REQ_PC_EXIT,
            PcExitRequest0104 { pc_id: 81 }.encode(),
        ))
        .is_err()
    );
    assert_eq!(clean_pc_exit_code_message(0), Some("Socket disconnected."));
    assert_eq!(
        clean_pc_exit_code_message(3),
        Some("Your connection has been terminated by a moderator.")
    );
    assert_eq!(clean_pc_exit_code_message(99), Some("Server disconnection"));
    assert_eq!(clean_pc_exit_code_message(1), None);
    assert_eq!(clean_pc_exit_code_message(42), None);
}
