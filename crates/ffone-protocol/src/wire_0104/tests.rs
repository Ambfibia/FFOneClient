use super::*;
use crate::{fixed_payload_size, packet};

fn pattern(len: usize) -> Vec<u8> {
    (0..len).map(|index| (index * 7 + 3) as u8).collect()
}

fn round_trip<T: WirePayload + PartialEq + std::fmt::Debug>(declared: usize) {
    assert_eq!(T::SIZE, declared);
    let first = T::decode(&pattern(declared)).expect("declared size decodes");
    let wire = first.encode();
    assert_eq!(wire.len(), declared);
    assert_eq!(T::decode(&wire).unwrap(), first);
    assert!(T::decode(&pattern(declared + 1)).is_err());
    assert!(T::decode(&pattern(declared - 1)).is_err());
}
mod every_generated_layout_matches_its_declared_size_and_round_trips;

mod every_packet_id_matches_the_clean_csdefines_value;
