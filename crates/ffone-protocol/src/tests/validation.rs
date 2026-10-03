use super::*;

#[test]
fn around_decoders_reject_missing_negative_and_mismatched_counts() {
    assert_eq!(
        decode_pc_around_0104(&[0; 3]),
        Err(AroundDecodeError::MissingCount { actual: 3 })
    );

    let mut negative = vec![0u8; PcAppearance0104::SIZE];
    write_i32(&mut negative, 0, -1);
    assert_eq!(
        decode_pc_around_0104(&negative),
        Err(AroundDecodeError::NegativeCount { count: -1 })
    );

    let mut legacy_four_byte_header = vec![0u8; 4 + PcAppearance0104::SIZE];
    write_i32(&mut legacy_four_byte_header, 0, 1);
    assert_eq!(
        decode_pc_around_0104(&legacy_four_byte_header),
        Err(AroundDecodeError::WrongSize {
            expected: PcAppearance0104::SIZE * 2,
            actual: 4 + PcAppearance0104::SIZE,
        })
    );

    let mut shiny_with_trailing_byte = vec![0u8; 5 + ShinyAppearance0104::SIZE];
    write_i32(&mut shiny_with_trailing_byte, 0, 1);
    assert_eq!(
        decode_shiny_around_0104(&shiny_with_trailing_byte),
        Err(AroundDecodeError::WrongSize {
            expected: 4 + ShinyAppearance0104::SIZE,
            actual: 5 + ShinyAppearance0104::SIZE,
        })
    );
}
