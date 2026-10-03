use super::*;

pub(super) fn remove_key_recursively(value: &mut serde_json::Value, key: &str) {
    match value {
        serde_json::Value::Object(object) => {
            object.remove(key);
            for child in object.values_mut() {
                remove_key_recursively(child, key);
            }
        }
        serde_json::Value::Array(array) => {
            for child in array {
                remove_key_recursively(child, key);
            }
        }
        _ => {}
    }
}

pub(super) fn assert_vec3_exact(left: Vec3, right: Vec3) {
    for (actual, expected) in left.to_array().into_iter().zip(right.to_array()) {
        assert!(
            (actual - expected).abs() <= f32::EPSILON,
            "expected {right:?}, got {left:?}"
        );
    }
}

#[test]
fn legacy_queue_tags_partition_complete_map_12_03_control_quartets() {
    let descriptor_path = "map/tiles/map_12_03/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let descriptor: NativeTerrainDescriptor = serde_json::from_slice(&bytes).unwrap();
    let modes = descriptor
        .splat
        .layers
        .iter()
        .map(|layer| layer.mode)
        .collect::<Vec<_>>();
    assert_eq!(modes, [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 1]);
    let group_modes = modes.chunks(4).map(|group| group[0]).collect::<Vec<_>>();
    assert_eq!(group_modes, [0, 0, 1, 1, 1]);
    assert_eq!(
        &modes[12..16],
        [1, 0, 0, 0],
        "the trailing Splat values are placeholders inside one Blend quartet"
    );

    // Queue tags FirstPass Geometry-100/AddPass Geometry-99/BlendPass
    // Geometry-98 mean the later Splat quartet is still accumulated before
    // the earlier serialized Blend quartet. Zero-weight placeholders keep
    // the original four-channel boundaries explicit.
    let placeholder = LegacyTerrainCompositorLayer {
        mode: LegacyTerrainTextureMode::Splat,
        weight: 0.0,
        texture_rgb: Vec3::ONE,
    };
    let mixed = [
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Splat,
            weight: 1.0,
            texture_rgb: Vec3::X,
        },
        placeholder,
        placeholder,
        placeholder,
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Blend,
            weight: 0.5,
            texture_rgb: Vec3::Y,
        },
        placeholder,
        placeholder,
        placeholder,
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Splat,
            weight: 1.0,
            texture_rgb: Vec3::Z,
        },
        placeholder,
        placeholder,
        placeholder,
    ];
    assert_vec3_exact(
        legacy_terrain_pass_composite(&mixed),
        Vec3::new(0.5, 0.25, 0.5),
    );
}

#[test]
fn future_pokey_oaks_north_pool_tile_keeps_primary_border_surface() {
    let descriptor_path = "map/tiles/map_12_02/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let descriptor: NativeTerrainDescriptor = serde_json::from_slice(&bytes).unwrap();

    let border = &descriptor.splat.layers[9];
    assert_eq!(border.index, 9);
    assert_eq!(border.mode, LegacyTerrainTextureMode::Blend as i64);
    assert_eq!(border.true_texture_name, "Tile_block_stone1_01");
    assert_eq!(
        (border.weight.map_index, border.weight.channel.as_str()),
        (2, "g")
    );
    assert_eq!(
        border.albedo.canonical_rgba_blake3,
        "blake3:2e5117fdd9fc0ee4f9d41251dad6cb28cf96486df60699391ae78a18f22eaf32"
    );

    let damaged_border = &descriptor.splat.layers[10];
    assert_eq!(damaged_border.index, 10);
    assert_eq!(damaged_border.mode, LegacyTerrainTextureMode::Blend as i64);
    assert_eq!(damaged_border.true_texture_name, "Tile_block_stone1_01D");
    assert_eq!(
        (
            damaged_border.weight.map_index,
            damaged_border.weight.channel.as_str(),
        ),
        (2, "b")
    );
    assert_eq!(
        damaged_border.albedo.canonical_rgba_blake3,
        "blake3:296ef24a2d64bf3f2e440aa893e6fcc9b6fbb0aa75d4492d655b74ef78d45ebb"
    );

    let pool = &descriptor.splat.layers[11];
    assert_eq!(pool.index, 11);
    assert_eq!(pool.mode, LegacyTerrainTextureMode::Blend as i64);
    assert_eq!(pool.true_texture_name, "Surface_01D_1");
    assert_eq!(
        (pool.weight.map_index, pool.weight.channel.as_str()),
        (2, "a")
    );
    assert_eq!(
        pool.albedo.canonical_rgba_blake3,
        "blake3:1ee6924d7ab872d1591cc8853396035a3d0d9fba14cb48d81def9c98862e0caa"
    );

    let weight = &descriptor.splat.weight_maps[2];
    assert_eq!(
        weight.canonical_rgba_blake3,
        "blake3:afcfdf0d2d0d0fb0b7126600588c3b2b3f17437750ceb7f5ddce3744bfb79537"
    );
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(asset_root().join("map/tiles/map_12_02/tile.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(manifest["terrain"]["path"], descriptor_path);
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        manifest["terrain"]["blake3"].as_str().unwrap(),
    )
    .unwrap();

    // These adjacent texels are in the central Pokey Oaks North pool
    // visible from the user's report: green is the intact stone curb,
    // while blue/alpha retain the damaged curb and pool surface.
    let border_texel = (119 * 256 + 138) * 4;
    let pool_texel = (120 * 256 + 139) * 4;
    assert_eq!(
        &terrain.weight_maps[2].pixels[border_texel..border_texel + 4],
        &[0, 255, 0, 0]
    );
    assert_eq!(
        &terrain.weight_maps[2].pixels[pool_texel..pool_texel + 4],
        &[0, 0, 255, 255]
    );

    let uploaded = weight_array_image(&terrain);
    let uploaded = uploaded.data.as_ref().unwrap();
    let map_stride = 256 * 256 * 4;
    assert_eq!(
        &uploaded[2 * map_stride + border_texel..2 * map_stride + border_texel + 4],
        &[0, 255, 0, 0],
        "the north-pool curb channel must survive the GPU-array upload"
    );
    assert_eq!(
        &uploaded[2 * map_stride + pool_texel..2 * map_stride + pool_texel + 4],
        &[0, 0, 255, 255],
        "the north-pool damaged-curb and surface channels must survive upload"
    );
}

#[test]
fn every_pool_fixture_tile_keeps_its_primary_stone_curb_on_the_gpu() {
    struct PoolCurbExpectation {
        tile: &'static str,
        fixture_count: usize,
        layer: usize,
        texture: &'static str,
        map: usize,
        channel: usize,
        weight_hash: &'static str,
        texture_hash: &'static str,
    }

    const STONE_1: &str =
        "blake3:2e5117fdd9fc0ee4f9d41251dad6cb28cf96486df60699391ae78a18f22eaf32";
    const STONE_6: &str =
        "blake3:79015b059cfc7e90fa5ba9b547a449f7338e99d26b5f600e3d547039143ff0fe";
    let expectations = [
        PoolCurbExpectation {
            tile: "map_06_06",
            fixture_count: 2,
            layer: 11,
            texture: "Tile_block_stone6_01",
            map: 2,
            channel: 3,
            weight_hash: "blake3:dde9a217a0c9b7a0d11432fc72a4f376688653fbaf583bfa780b7b0102aba826",
            texture_hash: STONE_6,
        },
        PoolCurbExpectation {
            tile: "map_07_06",
            fixture_count: 12,
            layer: 9,
            texture: "Tile_block_stone6_01",
            map: 2,
            channel: 1,
            weight_hash: "blake3:38abf4e956102fc0faf73fea72db744d46281bdd4f379615e166866f02663317",
            texture_hash: STONE_6,
        },
        PoolCurbExpectation {
            tile: "map_07_07",
            fixture_count: 7,
            layer: 9,
            texture: "Tile_block_stone6_01",
            map: 2,
            channel: 1,
            weight_hash: "blake3:56764985c0e4d4f802a91a69e55b9d653cbcdcf689308434696025262f70d4d3",
            texture_hash: STONE_6,
        },
        PoolCurbExpectation {
            tile: "map_07_08",
            fixture_count: 3,
            layer: 11,
            texture: "Tile_block_stone6_01",
            map: 2,
            channel: 3,
            weight_hash: "blake3:74b1cf24d4864d860f0ce0f7bc3d351c1bb8f55e4edad56aa0807ca2798131d8",
            texture_hash: STONE_6,
        },
        PoolCurbExpectation {
            tile: "map_09_07",
            fixture_count: 6,
            layer: 10,
            texture: "Tile_block_stone6_01",
            map: 2,
            channel: 2,
            weight_hash: "blake3:97c2fa50c1519238cbf0d6c33e6d740c876cf3e5d9ae08e8e1f6c658cc357066",
            texture_hash: STONE_6,
        },
        PoolCurbExpectation {
            tile: "map_09_08",
            fixture_count: 3,
            layer: 5,
            texture: "Tile_block_stone6_01",
            map: 1,
            channel: 1,
            weight_hash: "blake3:3c94b2874acf5dae2e463bf84a91d5cfa3e39326297476b2ee447691c6343680",
            texture_hash: STONE_6,
        },
        PoolCurbExpectation {
            tile: "map_11_01",
            fixture_count: 4,
            layer: 10,
            texture: "Tile_block_stone1_01",
            map: 2,
            channel: 2,
            weight_hash: "blake3:a55f058a8a646c4a977039f591a4ce9d250c984b392e5c11b838b3b6b867f914",
            texture_hash: STONE_1,
        },
        PoolCurbExpectation {
            tile: "map_12_01",
            fixture_count: 12,
            layer: 9,
            texture: "Tile_block_stone1_01",
            map: 2,
            channel: 1,
            weight_hash: "blake3:32168ce0941e09d2a62a87c65bc26ce7352f6cad7ba9e7037ec9a4d44ecdda26",
            texture_hash: STONE_1,
        },
        PoolCurbExpectation {
            tile: "map_12_02",
            fixture_count: 9,
            layer: 9,
            texture: "Tile_block_stone1_01",
            map: 2,
            channel: 1,
            weight_hash: "blake3:afcfdf0d2d0d0fb0b7126600588c3b2b3f17437750ceb7f5ddce3744bfb79537",
            texture_hash: STONE_1,
        },
        PoolCurbExpectation {
            tile: "map_12_03",
            fixture_count: 2,
            layer: 9,
            texture: "Tile_block_stone1_01",
            map: 2,
            channel: 1,
            weight_hash: "blake3:b9381105853f7b863a34371ac6e11282541e621c7e10fb99c04ea899df368476",
            texture_hash: STONE_1,
        },
        PoolCurbExpectation {
            tile: "map_14_02",
            fixture_count: 3,
            layer: 9,
            texture: "Tile_block_stone1_01",
            map: 2,
            channel: 1,
            weight_hash: "blake3:0b4b2c7c6726c14683e898f1192eab7f8018b129c7fbc46c354ee8bae25a5447",
            texture_hash: STONE_1,
        },
    ];

    for expected in expectations {
        let scene_path = format!("map/tiles/{}/scene.json", expected.tile);
        let scene: serde_json::Value =
            serde_json::from_slice(&fs::read(asset_root().join(&scene_path)).unwrap()).unwrap();
        let fixtures = scene["visuals"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|visual| {
                visual["name"]
                    .as_str()
                    .is_some_and(|name| name.contains("dds2-pool_set"))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            fixtures.len(),
            expected.fixture_count,
            "{} pool-fixture inventory drifted",
            expected.tile
        );

        let descriptor_path = format!("map/tiles/{}/terrain/terrain.json", expected.tile);
        let descriptor: NativeTerrainDescriptor =
            serde_json::from_slice(&fs::read(asset_root().join(&descriptor_path)).unwrap())
                .unwrap();
        let curb = &descriptor.splat.layers[expected.layer];
        assert_eq!(curb.index, expected.layer, "{} curb index", expected.tile);
        assert_eq!(
            curb.mode,
            LegacyTerrainTextureMode::Blend as i64,
            "{} curb pass",
            expected.tile
        );
        assert_eq!(
            curb.true_texture_name, expected.texture,
            "{} curb texture",
            expected.tile
        );
        assert_eq!(
            (curb.weight.map_index, curb.weight.channel_index),
            (expected.map, expected.channel),
            "{} curb control channel",
            expected.tile
        );
        assert_eq!(
            curb.albedo.canonical_rgba_blake3, expected.texture_hash,
            "{} curb texture bytes",
            expected.tile
        );
        assert_eq!(
            descriptor.splat.weight_maps[expected.map].canonical_rgba_blake3,
            expected.weight_hash,
            "{} pool control-map bytes",
            expected.tile
        );

        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(asset_root().join(format!("map/tiles/{}/tile.json", expected.tile)))
                .unwrap(),
        )
        .unwrap();
        let terrain = NativeTerrain::open(
            asset_root(),
            &descriptor_path,
            manifest["terrain"]["blake3"].as_str().unwrap(),
        )
        .unwrap();

        let root_translation = &scene["nativeTerrain"]["rootChain"]["nodes"][0]["nativeLocalTransform"]
            ["translation"];
        let root_x = root_translation[0].as_f64().unwrap();
        let root_z = root_translation[2].as_f64().unwrap();
        let weights = &terrain.weight_maps[expected.map];
        let width = weights.width as isize;
        let height = weights.height as isize;
        let mut authored_peak = 0;
        for fixture in fixtures {
            let translation = &fixture["transform"]["translation"];
            let world_x = translation[0].as_f64().unwrap();
            let world_z = translation[2].as_f64().unwrap();
            let pixel_x = (((root_x - world_x) / descriptor.scale.extent_x * width as f64)
                - 0.5)
                .round() as isize;
            let pixel_y = (((1.0 - (world_z - root_z) / descriptor.scale.extent_z)
                * height as f64)
                - 0.5)
                .round() as isize;
            for y in (pixel_y - 12).max(0)..=(pixel_y + 12).min(height - 1) {
                for x in (pixel_x - 12).max(0)..=(pixel_x + 12).min(width - 1) {
                    let offset = (y * width + x) as usize * 4 + expected.channel;
                    authored_peak = authored_peak.max(weights.pixels[offset]);
                }
            }
        }
        assert_eq!(
            authored_peak, 255,
            "{} has no opaque primary curb control near any pool fixture",
            expected.tile
        );

        let uploaded_weights = weight_array_image(&terrain);
        let uploaded_weights = uploaded_weights.data.as_ref().unwrap();
        let map_stride = weights.width as usize * weights.height as usize * 4;
        assert_eq!(
            &uploaded_weights[expected.map * map_stride..(expected.map + 1) * map_stride],
            weights.pixels.as_ref(),
            "{} curb control map must survive the GPU upload",
            expected.tile
        );
        let uploaded_layers = layer_array_image(&terrain);
        let uploaded_layers = uploaded_layers.data.as_ref().unwrap();
        let layer_stride = terrain.layer_images[expected.layer].width as usize
            * terrain.layer_images[expected.layer].height as usize
            * 4;
        assert_eq!(
            &uploaded_layers
                [expected.layer * layer_stride..(expected.layer + 1) * layer_stride],
            terrain.layer_images[expected.layer].pixels.as_ref(),
            "{} curb texture must survive the GPU upload",
            expected.tile
        );
    }
}

#[test]
fn map_07_08_restores_the_primary_sector_v_pool_vertical_wall() {
    let descriptor_path = "map/tiles/map_07_08/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        blake3::hash(&bytes).to_hex().as_str(),
    )
    .unwrap();
    let geometry = terrain.geometry();

    assert_eq!(terrain.descriptor().heightmap.vertex_shifts.len(), 1_033);
    assert!(
        terrain
            .descriptor()
            .heightmap
            .vertex_shifts
            .contains(&NativeTerrainVertexShift {
                flags: 9,
                column: 79,
                row: 92,
            })
    );

    // This shifted corner is on the clean-primary Sector V pool rim. It
    // lands on the same X/Z as the unshifted sample diagonally inside the
    // pool but retains its own height, forming Unity's vertical wall.
    let rim = geometry.positions()[92 * 129 + 79];
    let floor = geometry.positions()[93 * 129 + 78];
    assert_eq!([rim[0], rim[2]], [floor[0], floor[2]]);
    assert!((rim[1] - floor[1]).abs() > 1.0);
}

#[test]
fn map_05_05_materializes_every_authored_grass_density_instance() {
    let descriptor_path = "map/tiles/map_05_05/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        blake3::hash(&bytes).to_hex().as_str(),
    )
    .unwrap();
    assert_eq!(terrain.grass_layers.len(), 18);
    let instance_count = terrain
        .grass_layers
        .iter()
        .flat_map(|layer| layer.density.iter())
        .map(|value| usize::from(*value))
        .sum::<usize>();
    assert_eq!(instance_count, 15_381);
    let chunks = terrain
        .grass_layers
        .iter()
        .flat_map(|layer| build_grass_chunk_meshes(&terrain, &[layer], 80.0))
        .collect::<Vec<_>>();
    let chunk_instance_count = chunks
        .iter()
        .map(|chunk| chunk.instance_count)
        .sum::<usize>();
    let (vertex_count, index_count) =
        chunks.iter().fold((0, 0), |(vertices, indices), chunk| {
            (
                vertices + chunk.mesh.count_vertices(),
                indices + chunk.mesh.indices().unwrap().len(),
            )
        });
    assert_eq!(chunk_instance_count, instance_count);
    assert_eq!(vertex_count, instance_count * 8);
    assert_eq!(index_count, instance_count * 12);
    assert!(chunks.len() > terrain.grass_layers.len());
    assert!(
        chunks
            .iter()
            .all(|chunk| chunk.mesh.count_vertices() <= NATIVE_TERRAIN_GRASS_VERTICES_PER_FRAME)
    );
    assert!(
        chunks
            .iter()
            .all(|chunk| chunk.visibility_end > 80.0 && chunk.visibility_end < 176.0)
    );
}

#[test]
fn single_slice_tutorial_textures_keep_explicit_array_views() {
    let descriptor_path = "map/tiles/map_00_00/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        blake3::hash(&bytes).to_hex().as_str(),
    )
    .unwrap();
    assert_eq!(terrain.weight_maps.len(), 1);
    assert_eq!(terrain.layer_images.len(), 1);

    for image in [weight_array_image(&terrain), layer_array_image(&terrain)] {
        assert_eq!(image.texture_descriptor.size.depth_or_array_layers, 1);
        assert_eq!(
            image
                .texture_view_descriptor
                .as_ref()
                .and_then(|view| view.dimension),
            Some(TextureViewDimension::D2Array)
        );
    }
}

#[test]
fn four_by_four_layer_is_losslessly_padded_and_repeat_bilinear_equivalent() {
    let descriptor_path = "map/tiles/map_12_03/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        blake3::hash(&bytes).to_hex().as_str(),
    )
    .unwrap();
    let layer_index = terrain
        .descriptor
        .splat
        .layers
        .iter()
        .position(|layer| layer.albedo.width == 4 && layer.albedo.height == 4)
        .unwrap();
    let source = &terrain.layer_images[layer_index];
    let padded = layer_array_image(&terrain);
    let padded_width = padded.texture_descriptor.size.width as usize;
    let padded_height = padded.texture_descriptor.size.height as usize;
    let padded_data = padded.data.as_deref().unwrap();
    let slice_stride = padded_width * padded_height * 4;
    let padded_slice =
        &padded_data[layer_index * slice_stride..(layer_index + 1) * slice_stride];

    for row in 0..4 {
        assert_eq!(
            &padded_slice[row * padded_width * 4..row * padded_width * 4 + 16],
            &source.pixels[row * 16..row * 16 + 16]
        );
        assert!(
            padded_slice[row * padded_width * 4 + 16..(row + 1) * padded_width * 4]
                .iter()
                .all(|byte| *byte == 0)
        );
    }

    fn sample(
        pixels: &[u8],
        storage_width: usize,
        source_width: usize,
        source_height: usize,
        uv: Vec2,
    ) -> [f32; 4] {
        let texel = Vec2::new(
            uv.x.fract().rem_euclid(1.0) * source_width as f32 - 0.5,
            uv.y.fract().rem_euclid(1.0) * source_height as f32 - 0.5,
        );
        let base_x = texel.x.floor() as i32;
        let base_y = texel.y.floor() as i32;
        let fraction = texel - texel.floor();
        let load = |x: i32, y: i32| {
            let x = x.rem_euclid(source_width as i32) as usize;
            let y = y.rem_euclid(source_height as i32) as usize;
            let offset = (y * storage_width + x) * 4;
            [
                pixels[offset] as f32 / 255.0,
                pixels[offset + 1] as f32 / 255.0,
                pixels[offset + 2] as f32 / 255.0,
                pixels[offset + 3] as f32 / 255.0,
            ]
        };
        let c00 = load(base_x, base_y);
        let c10 = load(base_x + 1, base_y);
        let c01 = load(base_x, base_y + 1);
        let c11 = load(base_x + 1, base_y + 1);
        std::array::from_fn(|channel| {
            let top = c00[channel] + (c10[channel] - c00[channel]) * fraction.x;
            let bottom = c01[channel] + (c11[channel] - c01[channel]) * fraction.x;
            top + (bottom - top) * fraction.y
        })
    }

    for uv in [
        Vec2::new(0.5 / 4.0, 1.5 / 4.0),
        Vec2::new(0.999, 0.125),
        Vec2::new(-0.001, 0.125),
        Vec2::new(1.0, 0.5),
    ] {
        let expected = sample(&source.pixels, 4, 4, 4, uv);
        let actual = sample(padded_slice, padded_width, 4, 4, uv);
        for channel in 0..4 {
            assert!((expected[channel] - actual[channel]).abs() <= f32::EPSILON);
        }
    }
}

#[test]
fn legacy_sample_height_hits_vertices_and_applies_combined_owner_y_once() {
    let collider = legacy_height_test_collider();
    // This GlobalTransform represents the already-combined root + terrain
    // owner transform. MapAttributeTable adds its Y after SampleHeight.
    let owner_and_root = GlobalTransform::from(Transform::from_xyz(100.0, -30.0, 200.0));

    let h10 = require_legacy_height(collider.legacy_interpolated_height(
        &owner_and_root,
        98.0,
        200.0,
    ));
    let h01 = require_legacy_height(collider.legacy_interpolated_height(
        &owner_and_root,
        100.0,
        204.0,
    ));
    assert!((h10 - -20.0).abs() <= f32::EPSILON);
    assert!((h01 - -10.0).abs() <= f32::EPSILON);
}

#[test]
fn legacy_sample_height_rejects_non_finite_and_singular_queries() {
    let collider = legacy_height_test_collider();
    let identity = GlobalTransform::IDENTITY;
    assert_eq!(
        collider.legacy_interpolated_height(&identity, f32::NAN, 0.0),
        NativeTerrainLegacyInterpolatedHeightSample::Rejected(
            NativeTerrainLegacyInterpolatedHeightRejection::NonFiniteInput
        )
    );

    let non_finite = GlobalTransform::from(Transform::from_xyz(f32::INFINITY, 0.0, 0.0));
    assert_eq!(
        collider.legacy_interpolated_height(&non_finite, 0.0, 0.0),
        NativeTerrainLegacyInterpolatedHeightSample::Rejected(
            NativeTerrainLegacyInterpolatedHeightRejection::NonFiniteTerrainTransform
        )
    );

    let singular = GlobalTransform::from(Transform::from_scale(Vec3::new(0.0, 1.0, 1.0)));
    assert_eq!(
        collider.legacy_interpolated_height(&singular, 0.0, 0.0),
        NativeTerrainLegacyInterpolatedHeightSample::Rejected(
            NativeTerrainLegacyInterpolatedHeightRejection::SingularTerrainTransform
        )
    );
}
