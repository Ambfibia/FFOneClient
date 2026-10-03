use super::*;

#[test]
fn server_music_commands_are_consumed_before_chat_and_reject_malformed_packets() {
    let mut requests = ffone_client::world_audio::RetrobutionMusicRequests::default();
    for message_type in [11, 12] {
        let mut frame = DecodedFrame {
            packet_type: packet::P_FE2CL_PC_MOTD_LOGIN,
            flags: 0,
            checksum: 0,
            payload: ffone_protocol::ServerMessage0104 {
                message_type,
                message: FixedUtf16::from_str("Vs Weeper.ogg").unwrap(),
            }
            .encode(),
        };
        assert_eq!(apply_music_frame_0104(&frame, &mut requests), Ok(true));
        frame.payload.pop();
        assert!(apply_music_frame_0104(&frame, &mut requests).is_err());
    }
    let frame = DecodedFrame {
        packet_type: packet::P_FE2CL_REP_PC_FREECHAT_SUCC,
        flags: 0,
        checksum: 0,
        payload: FreeChatSuccess0104 {
            pc_id: 1,
            emote_code: 666,
            message: FixedUtf16::from_str("MissionTheme1.ogg").unwrap(),
        }
        .encode(),
    };
    assert_eq!(apply_music_frame_0104(&frame, &mut requests), Ok(true));
}
