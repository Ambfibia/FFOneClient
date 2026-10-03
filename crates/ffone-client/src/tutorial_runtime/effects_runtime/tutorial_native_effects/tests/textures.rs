use super::*;

#[test]
fn bc3_decoder_preserves_endpoint_color_and_alpha() {
    let block = [255, 0, 0, 0, 0, 0, 0, 0, 0x00, 0xf8, 0x00, 0x00, 0, 0, 0, 0];
    let decoded = decode_bc3_level(&block, 4, 4).unwrap();
    assert_eq!(&decoded[0..4], &[255, 0, 0, 255]);
    assert_eq!(decoded.len(), 64);
}

#[test]
fn bc1_decoder_preserves_opaque_and_transparent_modes() {
    let mut transparent_mode = [0_u8; 8];
    transparent_mode[0..2].copy_from_slice(&0x0000_u16.to_le_bytes());
    transparent_mode[2..4].copy_from_slice(&0xffff_u16.to_le_bytes());
    transparent_mode[4..8].copy_from_slice(&0x0000_00e4_u32.to_le_bytes());
    let decoded = decode_bc1_level(&transparent_mode, 4, 4).unwrap();
    assert_eq!(&decoded[0..4], &[0, 0, 0, 255]);
    assert_eq!(&decoded[4..8], &[255, 255, 255, 255]);
    assert_eq!(&decoded[8..12], &[127, 127, 127, 255]);
    assert_eq!(&decoded[12..16], &[0, 0, 0, 0]);

    let mut opaque_mode = [0_u8; 8];
    opaque_mode[0..2].copy_from_slice(&0xf800_u16.to_le_bytes());
    opaque_mode[2..4].copy_from_slice(&0x0000_u16.to_le_bytes());
    opaque_mode[4..8].copy_from_slice(&3_u32.to_le_bytes());
    let decoded = decode_bc1_level(&opaque_mode, 4, 4).unwrap();
    assert_eq!(&decoded[0..4], &[85, 0, 0, 255]);
}

#[test]
fn unity_texture_dispatch_and_encoded_sizes_are_exact() {
    assert_eq!(texture_format_layout(2), Ok(("ARGB4444", None)));
    assert_eq!(texture_format_layout(10), Ok(("DXT1", Some(8))));
    assert_eq!(texture_format_layout(12), Ok(("DXT5", Some(16))));
    assert!(texture_format_layout(11).is_err());
    assert_eq!(argb4444_level_size(1, 1), Ok(2));
    assert_eq!(argb4444_level_size(4, 4), Ok(32));
    assert!(argb4444_level_size(0, 4).is_err());
    assert_eq!(compressed_level_size(8, 1, 1), Ok(8));
    assert_eq!(compressed_level_size(8, 4, 4), Ok(8));
    assert_eq!(compressed_level_size(8, 5, 4), Ok(16));
    assert_eq!(compressed_level_size(16, 4, 4), Ok(16));
    assert!(compressed_level_size(8, 0, 4).is_err());
    assert!(rgba_level_size(4, 0).is_err());
}

#[test]
fn exact_texture_installs_the_complete_mip_chain_before_upload() {
    let texture = ExactTexture {
        key: ("effect.asset".to_owned(), 7, "hash".to_owned()),
        width: 2,
        height: 2,
        mip_count: 2,
        rgba_mips: vec![255; 20].into(),
    };
    let mut images = Assets::<Image>::default();
    let mut visual_assets = NativeVisualAssets::default();

    let handle = texture_handle(&texture, &mut images, &mut visual_assets);
    let image = images.get(&handle).unwrap();
    assert_eq!(image.texture_descriptor.mip_level_count, 2);
    assert_eq!(image.data.as_ref().map(Vec::len), Some(20));
}
