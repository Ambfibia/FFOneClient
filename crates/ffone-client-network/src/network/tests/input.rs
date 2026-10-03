use super::*;

pub(super) fn read_tutorial_client(stream: &mut TcpStream, key: u64, sequence: u16) -> DecodedFrame {
    decode_client_frame(&read_tutorial_wire(stream), key, sequence).unwrap()
}
