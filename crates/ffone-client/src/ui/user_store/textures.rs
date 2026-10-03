use super::*;

#[must_use]
pub fn user_store_missing_checker_rgba_0104() -> Vec<u8> {
    const SIZE: u32 = 32;
    const QUADRANT: u32 = 8;
    let mut rgba = vec![0_u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let magenta = (x / QUADRANT + y / QUADRANT) % 2 == 0;
            let offset = ((y * SIZE + x) * 4) as usize;
            rgba[offset..offset + 4].copy_from_slice(if magenta {
                &[255, 0, 255, 255]
            } else {
                &[0, 0, 0, 255]
            });
        }
    }
    rgba
}
