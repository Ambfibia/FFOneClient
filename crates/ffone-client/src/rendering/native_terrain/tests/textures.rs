use super::*;

#[test]
fn legacy_realtime_blend_weights_all_rgba_channels_and_sums_pass_alpha() {
    let layers = [
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Blend,
            weight: 0.90,
            texture_rgb: Vec3::X,
        },
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Blend,
            weight: 0.25,
            texture_rgb: Vec3::Y,
        },
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Blend,
            weight: 0.50,
            texture_rgb: Vec3::Z,
        },
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Blend,
            weight: 0.75,
            texture_rgb: Vec3::ONE,
        },
    ];
    let (source, alpha) = legacy_realtime_blend_group(&layers);
    let expected_source = Vec3::X * 0.90 + Vec3::Y * 0.25 + Vec3::Z * 0.50 + Vec3::ONE * 0.75;
    assert_vec3_exact(source, expected_source);
    assert_eq!(alpha, 0.90 + 0.25 + 0.50 + 0.75);
    assert!(alpha > 1.0, "the source pass does not normalize alpha");

    let mut changed_r = layers;
    changed_r[0].weight = 0.10;
    let (source_with_changed_r, alpha_with_changed_r) = legacy_realtime_blend_group(&changed_r);
    assert_vec3_exact(source_with_changed_r, expected_source - Vec3::X * 0.80);
    assert_eq!(alpha_with_changed_r, 0.10 + 0.25 + 0.50 + 0.75);
}

#[test]
fn future_pokey_oaks_pool_tile_keeps_border_surface_and_rgba_masks() {
    let descriptor_path = "map/tiles/map_12_01/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let descriptor: NativeTerrainDescriptor = serde_json::from_slice(&bytes).unwrap();
    let border = &descriptor.splat.layers[9];
    assert_eq!(border.index, 9);
    assert_eq!(border.mode, LegacyTerrainTextureMode::Blend as i64);
    assert_eq!(border.true_texture_name, "Tile_block_stone1_01");
    assert_eq!(border.weight.map_index, 2);
    assert_eq!(border.weight.channel_index, 1);
    assert_eq!(border.weight.channel, "g");
    assert_eq!(
        border.albedo.canonical_rgba_blake3,
        "blake3:2e5117fdd9fc0ee4f9d41251dad6cb28cf96486df60699391ae78a18f22eaf32"
    );
    let damaged_border = &descriptor.splat.layers[10];
    assert_eq!(damaged_border.index, 10);
    assert_eq!(damaged_border.mode, LegacyTerrainTextureMode::Blend as i64);
    assert_eq!(damaged_border.true_texture_name, "Tile_block_stone1_01D");
    assert_eq!(damaged_border.weight.map_index, 2);
    assert_eq!(damaged_border.weight.channel_index, 2);
    assert_eq!(damaged_border.weight.channel, "b");
    assert_eq!(
        damaged_border.albedo.canonical_rgba_blake3,
        "blake3:296ef24a2d64bf3f2e440aa893e6fcc9b6fbb0aa75d4492d655b74ef78d45ebb"
    );
    let pool = &descriptor.splat.layers[11];
    assert_eq!(pool.index, 11);
    assert_eq!(pool.mode, LegacyTerrainTextureMode::Blend as i64);
    assert_eq!(pool.true_texture_name, "Surface_01D_1");
    assert_eq!(pool.weight.map_index, 2);
    assert_eq!(pool.weight.channel_index, 3);
    assert_eq!(pool.weight.channel, "a");
    assert_eq!(
        pool.albedo.canonical_rgba_blake3,
        "blake3:1ee6924d7ab872d1591cc8853396035a3d0d9fba14cb48d81def9c98862e0caa"
    );
    let weight = &descriptor.splat.weight_maps[2];
    assert_eq!(
        weight.canonical_rgba_blake3,
        "blake3:32168ce0941e09d2a62a87c65bc26ce7352f6cad7ba9e7037ec9a4d44ecdda26"
    );

    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(asset_root().join("map/tiles/map_12_01/tile.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["terrain"]["path"], descriptor_path);
    let descriptor_blake3 = manifest["terrain"]["blake3"].as_str().unwrap();
    let terrain =
        NativeTerrain::open(asset_root(), descriptor_path, descriptor_blake3).unwrap();
    assert!(Arc::ptr_eq(
        &terrain.weight_maps[2].pixels,
        &terrain.weight_map_mips[2][0].pixels,
    ));
    let border_texel = (177 * 256 + 113) * 4;
    assert_eq!(
        &terrain.weight_maps[2].pixels[border_texel..border_texel + 4],
        &[0, 255, 0, 0],
        "primary TerrainData_12_01 stores the pool border in control-map 2 green"
    );
    let texel = (177 * 256 + 115) * 4;
    assert_eq!(
        &terrain.weight_maps[2].pixels[texel..texel + 4],
        &[0, 0, 255, 255]
    );
    let uploaded = weight_array_image(&terrain);
    let uploaded = uploaded.data.as_ref().unwrap();
    let map_stride = 256 * 256 * 4;
    assert_eq!(
        &uploaded[2 * map_stride + border_texel..2 * map_stride + border_texel + 4],
        &[0, 255, 0, 0],
        "the GPU control array must retain the pool-border green channel"
    );
    assert_eq!(
        &uploaded[2 * map_stride + texel..2 * map_stride + texel + 4],
        &[0, 0, 255, 255],
        "the GPU array base mip must retain the pool's alpha channel"
    );
    let layers = layer_array_image(&terrain);
    let layers = layers.data.as_ref().unwrap();
    let layer_stride = 256 * 256 * 4;
    assert_eq!(
        &layers[9 * layer_stride..10 * layer_stride],
        terrain.layer_images[9].pixels.as_ref(),
        "the pool Tile_block_stone1_01 border bytes must remain array layer 9"
    );
    assert_eq!(
        &layers[10 * layer_stride..11 * layer_stride],
        terrain.layer_images[10].pixels.as_ref(),
        "the damaged border bytes must remain array layer 10"
    );
    assert_eq!(
        &layers[11 * layer_stride..12 * layer_stride],
        terrain.layer_images[11].pixels.as_ref(),
        "the pool Surface_01D_1 bytes must remain array layer 11"
    );

    let weight_path = asset_root().join("map/tiles/map_12_01/terrain/weights/weights_02.png");
    let first = load_verified_rgba_image(
        weight_path.clone(),
        256,
        256,
        &weight.png_blake3,
        &weight.canonical_rgba_blake3,
        "pool control map cache regression",
    )
    .unwrap();
    let second = load_verified_rgba_image(
        weight_path.clone(),
        256,
        256,
        &weight.png_blake3,
        &weight.canonical_rgba_blake3,
        "pool control map cache regression",
    )
    .unwrap();
    assert!(
        Arc::ptr_eq(&first.pixels, &second.pixels),
        "the verified base and mip-0 route must share one decoded RGBA buffer"
    );
    assert!(
        load_verified_rgba_image(
            weight_path,
            256,
            256,
            "blake3:0000000000000000000000000000000000000000000000000000000000000000",
            &weight.canonical_rgba_blake3,
            "pool control map wrong-hash regression",
        )
        .is_err(),
        "cache identity must include the authoritative hash"
    );
}

#[test]
fn shared_detail_texture_route_is_not_joined_below_the_tile() {
    assert_eq!(
        native_terrain_detail_texture_asset_path(
            "map/tiles/map_12_01/terrain/terrain.json",
            "map/shared/terrain/details/etc_plant_11/mips/mip_00.png",
        ),
        "map/shared/terrain/details/etc_plant_11/mips/mip_00.png"
    );
    assert_eq!(
        native_terrain_detail_texture_asset_path(
            "map/tiles/map_12_01/terrain/terrain.json",
            "details/textures/ETC_plant_11/texture.png",
        ),
        "map/tiles/map_12_01/terrain/details/textures/ETC_plant_11/texture.png"
    );
}
