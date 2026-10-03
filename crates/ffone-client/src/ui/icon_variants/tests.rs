use super::*;
#[test]
fn hue_preserves_alpha_and_value_and_outline_does_not_cover_source() {
    assert_eq!(
        transform_icon(&[255, 0, 0, 128], 1, 1, IconVariant::Hue(120)).2,
        [0, 255, 0, 128]
    );
    let (w, h, pixels) = transform_icon(&[255, 0, 0, 128], 1, 1, IconVariant::Outline);
    assert_eq!((w, h), (3, 3));
    assert_eq!(pixels[4 * 4 + 3], 0);
    assert_eq!(pixels[3], 255);
    assert_eq!(
        transform_icon(&[255, 0, 0, 128], 1, 1, IconVariant::Grayscale).2,
        [76, 76, 76, 128]
    );
}
