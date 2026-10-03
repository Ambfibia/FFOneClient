use super::*;

#[test]
fn semantic_texture_copies_and_font_remain_byte_exact() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for evidence in USER_EQUIP_SOURCE_TEXTURES {
        let converted = fs::read(root.join(evidence.converted_path)).unwrap();
        let semantic = fs::read(root.join(evidence.runtime_path)).unwrap();
        assert_eq!(semantic, converted, "{:?}", evidence.role);
        assert_eq!(
            format!("{:X}", Sha256::digest(&semantic)),
            evidence.sha256,
            "{:?}",
            evidence.role
        );
        let image = image::load_from_memory(&semantic).unwrap();
        assert_eq!(
            (image.width(), image.height()),
            (evidence.width, evidence.height),
            "{:?}",
            evidence.role
        );
    }
    assert_eq!(
        USER_EQUIP_POPUP_TEXTURE_EVIDENCE.map(|(_, path_id, _, _)| path_id),
        [390, 385, 282, 190]
    );
    for (path, _, expected_hash, expected_dimensions) in USER_EQUIP_POPUP_TEXTURE_EVIDENCE {
        let bytes = fs::read(root.join(path)).unwrap();
        assert_eq!(format!("{:X}", Sha256::digest(&bytes)), expected_hash);
        let image = image::load_from_memory(&bytes).unwrap();
        assert_eq!((image.width(), image.height()), expected_dimensions);
    }
    let font = fs::read(root.join(USER_EQUIP_FONT_PATH)).unwrap();
    assert_eq!(
        format!("{:X}", Sha256::digest(&font)),
        USER_EQUIP_FONT_SHA256
    );
}
