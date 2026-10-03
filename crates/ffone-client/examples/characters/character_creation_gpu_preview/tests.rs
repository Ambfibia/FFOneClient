use super::*;
use image::Rgba;

fn face_test_image(cheek: [u8; 3]) -> image::RgbaImage {
    let mut image = image::RgbaImage::from_pixel(1264, 681, Rgba([74, 101, 148, 255]));
    fill_box(&mut image, FACE_NECK_BOUNDS, Rgba([90, 74, 61, 255]));
    for (x, y) in FACE_CHEEK_CENTERS {
        fill_box(
            &mut image,
            (
                x - FACE_PATCH_RADIUS,
                x + FACE_PATCH_RADIUS,
                y - FACE_PATCH_RADIUS,
                y + FACE_PATCH_RADIUS,
            ),
            Rgba([cheek[0], cheek[1], cheek[2], 255]),
        );
    }
    image
}

fn fill_box(
    image: &mut image::RgbaImage,
    (min_x, max_x, min_y, max_y): (u32, u32, u32, u32),
    color: Rgba<u8>,
) {
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            image.put_pixel(x, y, color);
        }
    }
}

#[test]
fn face_gate_accepts_skin_and_rejects_outline_or_background() {
    validate_face_render(&face_test_image([80, 66, 56]))
        .expect("skin-colored cheeks must pass");
    assert!(
        validate_face_render(&face_test_image([0, 0, 0]))
            .unwrap_err()
            .contains("nearBlack")
    );
    assert!(
        validate_face_render(&face_test_image([82, 106, 158]))
            .unwrap_err()
            .contains("blueResidualDelta")
    );
}
