use super::*;

pub(super) fn read_client(
    stream: &mut TcpStream,
    key: u64,
    sequence: u16,
) -> ffone_protocol::DecodedFrame {
    decode_client_frame(&read_wire(stream), key, sequence).unwrap()
}
