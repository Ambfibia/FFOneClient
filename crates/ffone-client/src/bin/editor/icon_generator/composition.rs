//! Icon layers operate on premultiplied samples to avoid dark transparent fringes.
use image::{Rgba, RgbaImage, imageops};
use std::path::{Path, PathBuf};
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub(super) struct Layers {
    pub nano: bool,
    pub background: bool,
    pub outline: bool,
    pub locked: bool,
    pub badge: bool,
    /// 0: the exact extracted source; 1: level; 2: world; 3: item.
    pub badge_kind: u8,
    pub level: i32,
    pub affinity: bool,
}
impl Default for Layers {
    fn default() -> Self {
        Self {
            nano: false,
            background: false,
            outline: true,
            locked: false,
            badge: true,
            badge_kind: 0,
            level: 1,
            affinity: false,
        }
    }
}
pub(super) fn extract_badge(source: &RgbaImage) -> RgbaImage {
    let source = imageops::resize(source, 128, 128, imageops::FilterType::Nearest);
    RgbaImage::from_fn(128, 128, |x, y| {
        let pixel = *source.get_pixel(x, y);
        let [r, g, b, a] = pixel.0;
        // Ready portraits/backgrounds are neutral; acquisition marks are saturated
        // blue or bright white. Preserve their exact alpha and original location.
        if y >= 64 && (r.max(g).max(b) - r.min(g).min(b) > 12 || r.min(g).min(b) > 150) {
            Rgba([r, g, b, a])
        } else {
            Rgba([0, 0, 0, 0])
        }
    })
}
pub(super) fn compose(
    source: &RgbaImage,
    layers: &Layers,
    color: Option<Rgba<u8>>,
    badge: Option<&RgbaImage>,
) -> RgbaImage {
    let mut premultiplied = source.clone();
    for p in premultiplied.pixels_mut() {
        for c in 0..3 {
            p[c] = ((u16::from(p[c]) * u16::from(p[3]) + 127) / 255) as u8;
        }
    }
    let mut subject = imageops::resize(&premultiplied, 128, 128, imageops::FilterType::Lanczos3);
    for p in subject.pixels_mut() {
        if p[3] > 0 {
            for c in 0..3 {
                p[c] = (u32::from(p[c]) * 255 / u32::from(p[3])).min(255) as u8;
            }
        } else {
            p.0 = [0; 4]
        }
    }
    let type_color = color.unwrap_or(Rgba([82, 110, 255, 255]));
    let mut output = RgbaImage::from_pixel(
        128,
        128,
        if layers.nano && layers.background {
            if layers.locked {
                Rgba([19, 19, 19, 255])
            } else {
                type_color
            }
        } else {
            Rgba([0; 4])
        },
    );
    if layers.nano && layers.outline && !layers.locked {
        for y in 0..128 {
            for x in 0..128 {
                if subject.get_pixel(x, y)[3] > 16 {
                    continue;
                }
                let edge = (-2i32..=2).any(|dy| {
                    (-2i32..=2).any(|dx| {
                        let px = x as i32 + dx;
                        let py = y as i32 + dy;
                        dx * dx + dy * dy <= 4
                            && (0..128).contains(&px)
                            && (0..128).contains(&py)
                            && subject.get_pixel(px as u32, py as u32)[3] > 32
                    })
                });
                if edge {
                    output.put_pixel(x, y, type_color);
                }
            }
        }
    }
    if layers.locked {
        for p in subject.pixels_mut() {
            p[0] = 54;
            p[1] = 55;
            p[2] = 54;
        }
    }
    imageops::overlay(&mut output, &subject, 0, 0);
    if layers.nano && layers.badge {
        if let Some(badge) = badge {
            imageops::overlay(&mut output, badge, 0, 0);
        }
    }
    output
}
/// Acquisition symbols and numeral sprites recovered from actual ready icons.
/// Source portraits remain intact; only their non-neutral badge pixels are read.
pub(super) struct BadgeLibrary {
    level: RgbaImage,
    world: RgbaImage,
    item: RgbaImage,
    digits: Vec<Numeral>,
}
struct Numeral {
    image: RgbaImage,
    advance: i64,
}
impl BadgeLibrary {
    pub fn open(root: &Path) -> Result<Self, String> {
        let read = |slug: &str| -> Result<RgbaImage, String> {
            image::open(root.join(format!("icons/entities/nanos/ready/nanoready_{slug}.png")))
                .map(|i| extract_badge(&i.to_rgba8()))
                .map_err(|e| e.to_string())
        };
        let mut level = read("buttercup")?;
        for y in 90..128 {
            for x in 44..128 {
                level.put_pixel(x, y, Rgba([0; 4]));
            }
        }
        let mut digits = Vec::new();
        for slug in [
            "megas",
            "buttercup",
            "numbuh-two",
            "eddy",
            "eduardo",
            "blossom",
            "wilt",
            "dee-dee",
            "numbuh-five",
            "edd",
        ] {
            let src = read(slug)?;
            let ink = |x: u32, y: u32| src.get_pixel(x, y)[3] > 0;
            let mut x0 = 128;
            let mut x1 = 0;
            let mut y0 = 128;
            let mut y1 = 0;
            // Megas is level 10: its second numeral supplies zero.
            let left = if slug == "megas" { 54 } else { 44 };
            for y in 96..116 {
                for x in left..128 {
                    if ink(x, y) {
                        x0 = x0.min(x);
                        x1 = x1.max(x);
                        y0 = y0.min(y);
                        y1 = y1.max(y);
                    }
                }
            }
            if x0 > x1 || y0 > y1 {
                return Err(format!("Missing level numeral in {slug}"));
            }
            let mut white_min = 128;
            let mut white_max = 0;
            for y in 98..113 {
                for x in x0..=x1 {
                    let p = src.get_pixel(x, y);
                    if p[0] > 130 && p[1] > 170 && p[2] > 170 && p[3] > 0 {
                        white_min = white_min.min(x);
                        white_max = white_max.max(x);
                    }
                }
            }
            let advance =
                i64::from(white_max - white_min + 1) + if slug == "buttercup" { 4 } else { 3 };
            digits.push(Numeral {
                image: RgbaImage::from_fn(x1 - x0 + 1, 20, |x, y| *src.get_pixel(x + x0, y + 96)),
                advance,
            });
        }
        Ok(Self {
            level,
            world: read("belladonna")?,
            item: read("panini")?,
            digits,
        })
    }
    pub fn badge(&self, kind: u8, number: i32) -> RgbaImage {
        match kind {
            2 => self.world.clone(),
            3 => self.item.clone(),
            _ => {
                let mut result = self.level.clone();
                let mut x = 44;
                for digit in number.clamp(0, 99).to_string().bytes() {
                    let glyph = &self.digits[(digit - b'0') as usize];
                    imageops::overlay(&mut result, &glyph.image, x, 96);
                    x += glyph.advance;
                }
                result
            }
        }
    }
}
pub(super) fn unused_path(folder: &Path, name: &str) -> PathBuf {
    let mut path = folder.join(format!("{name}.png"));
    let mut n = 2;
    while path.exists() || path.with_extension("json").exists() {
        path = folder.join(format!("{name}_{n}.png"));
        n += 1;
    }
    path
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_acquisition_library_has_all_marks_and_ten_numerals() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        let library = BadgeLibrary::open(&root).unwrap();
        assert_eq!(library.digits.len(), 10);
        for kind in 1..=3 {
            let badge = library.badge(kind, 36);
            assert!(badge.pixels().any(|p| p[3] > 0));
        }
        assert_ne!(library.badge(1, 12), library.badge(1, 21));
        let source = image::open(root.join("icons/entities/nanos/ready/nanoready_buttercup.png"))
            .unwrap()
            .to_rgba8();
        let original = extract_badge(&source);
        let rebuilt = library.badge(1, 1);
        for (x, y, pixel) in original.enumerate_pixels().filter(|(_, _, p)| p[3] > 0) {
            assert_eq!(
                pixel,
                rebuilt.get_pixel(x, y),
                "original numeral styling at {x},{y}"
            );
        }
        for digit in &library.digits {
            assert!(digit.image.pixels().any(|p| p[3] > 0 && p[0] > 150));
            assert!(
                digit
                    .image
                    .pixels()
                    .any(|p| p[3] > 0 && p[2] > p[0].saturating_add(30)),
                "blue numeral outline"
            );
        }
    }
    #[test]
    fn badge_extraction_retains_color_and_level_but_not_silhouette() {
        let mut src = RgbaImage::from_pixel(128, 128, Rgba([19, 19, 19, 255]));
        src.put_pixel(50, 95, Rgba([220, 249, 255, 255]));
        src.put_pixel(3, 110, Rgba([0, 86, 131, 255]));
        src.put_pixel(60, 85, Rgba([54, 54, 54, 255]));
        let badge = extract_badge(&src);
        assert_eq!(badge.get_pixel(50, 95)[3], 255);
        assert_eq!(badge.get_pixel(3, 110)[3], 255);
        assert_eq!(badge.get_pixel(60, 85)[3], 0);
    }
    #[test]
    fn exports_square_with_transparent_border_and_locked_subject() {
        let mut src = RgbaImage::new(512, 512);
        for y in 100..400 {
            for x in 100..400 {
                src.put_pixel(x, y, Rgba([200, 80, 30, 255]));
            }
        }
        let layers = Layers {
            locked: true,
            nano: false,
            ..Default::default()
        };
        let result = compose(&src, &layers, None, None);
        assert_eq!(result.dimensions(), (128, 128));
        assert_eq!(result.get_pixel(0, 0).0, [0; 4]);
        assert_eq!(result.get_pixel(64, 64).0, [54, 55, 54, 255]);
    }
}
