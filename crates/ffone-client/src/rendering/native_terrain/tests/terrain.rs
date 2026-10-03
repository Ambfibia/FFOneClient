use super::*;

pub(super) fn sanitize_runtime_terrain(value: &mut serde_json::Value) {
    let object = value.as_object_mut().unwrap();
    object.remove("source");
    object.remove("sceneInstance");
    if let Some(attributes) = object
        .get_mut("gameplayAttributes")
        .and_then(serde_json::Value::as_object_mut)
    {
        attributes.remove("rawParsedDocument");
        attributes.remove("source");
    }
    if let Some(details) = object
        .get_mut("detailAndTrees")
        .and_then(serde_json::Value::as_object_mut)
    {
        details.remove("assetClosure");
        details.remove("rawDocument");
        if let Some(trees) = details.get_mut("trees") {
            remove_key_recursively(trees, "rawDocument");
        }
    }
    if let Some(splat) = object
        .get_mut("splat")
        .and_then(serde_json::Value::as_object_mut)
    {
        if let Some(weight_maps) = splat
            .get_mut("weightMaps")
            .and_then(serde_json::Value::as_array_mut)
        {
            for weight in weight_maps {
                let weight = weight.as_object_mut().unwrap();
                weight.remove("source");
                if let Some(mips) = weight
                    .get_mut("mips")
                    .and_then(serde_json::Value::as_array_mut)
                {
                    for mip in mips {
                        mip.as_object_mut().unwrap().remove("sourceEncoded");
                    }
                }
            }
        }
        if let Some(layers) = splat
            .get_mut("layers")
            .and_then(serde_json::Value::as_array_mut)
        {
            for layer in layers {
                let layer = layer.as_object_mut().unwrap();
                layer.remove("modeSource");
                layer.remove("modeEvidence");
                if let Some(albedo) = layer
                    .get_mut("albedo")
                    .and_then(serde_json::Value::as_object_mut)
                {
                    albedo.remove("source");
                    if let Some(mips) = albedo
                        .get_mut("mips")
                        .and_then(serde_json::Value::as_array_mut)
                    {
                        for mip in mips {
                            mip.as_object_mut().unwrap().remove("sourceEncoded");
                        }
                    }
                }
            }
        }
    }
    if let Some(lightmap) = object
        .get_mut("lightmap")
        .and_then(serde_json::Value::as_object_mut)
    {
        lightmap.remove("sourcePointer");
        lightmap.remove("source");
        if let Some(mips) = lightmap
            .get_mut("mips")
            .and_then(serde_json::Value::as_array_mut)
        {
            for mip in mips {
                mip.as_object_mut().unwrap().remove("sourceEncoded");
            }
        }
    }
}

#[test]
fn semantic_terrain_descriptor_hashes() {
    for (relative, expected) in [
        (
            "map/tiles/map_08_06/terrain/terrain.json",
            "f26131c022bbd20a724fc67e6ca5c501593b73d3395d4246a375fb33bbf73b41",
        ),
        (
            "map/tiles/map_12_03/terrain/terrain.json",
            "310d78d69770be67ab8cf3f741c00246ed768eb4f6e52db9479672c0634f8914",
        ),
    ] {
        let bytes = fs::read(asset_root().join(relative)).unwrap();
        assert_eq!(blake3::hash(&bytes).to_hex().as_str(), expected);
    }
}

#[test]
fn legacy_terrain_shader_evidence_hashes_are_exact() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for evidence in LEGACY_TERRAIN_SHADER_EVIDENCE {
        let bytes = fs::read(project_root.join(evidence.project_relative_path)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            evidence.sha256,
            "{}",
            evidence.project_relative_path
        );
    }
}

#[test]
fn legacy_render_mode_indices_match_the_native_splat_database_table() {
    assert_eq!(
        LegacyTerrainRenderMode::try_from(0).unwrap(),
        LegacyTerrainRenderMode::VertexLit
    );
    assert_eq!(
        LegacyTerrainRenderMode::try_from(1).unwrap(),
        LegacyTerrainRenderMode::Lightmap
    );
    assert_eq!(
        LegacyTerrainRenderMode::try_from(2).unwrap(),
        LegacyTerrainRenderMode::Realtime
    );
    assert!(LegacyTerrainRenderMode::try_from(3).is_err());
}

#[test]
fn legacy_splat_first_and_add_passes_saturate_between_control_map_groups() {
    let layers = [
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Splat,
            weight: 0.80,
            texture_rgb: Vec3::new(1.0, 0.0, 0.0),
        },
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Splat,
            weight: 0.70,
            texture_rgb: Vec3::new(0.0, 1.0, 0.0),
        },
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Splat,
            weight: 0.60,
            texture_rgb: Vec3::new(0.0, 0.0, 1.0),
        },
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Splat,
            weight: 0.50,
            texture_rgb: Vec3::ONE,
        },
        // Fifth layer is the first Blend One One AddPass input.
        LegacyTerrainCompositorLayer {
            mode: LegacyTerrainTextureMode::Splat,
            weight: 0.90,
            texture_rgb: Vec3::new(1.0, 0.5, 0.25),
        },
    ];
    let actual = legacy_terrain_pass_composite(&layers);
    assert_vec3_exact(actual, Vec3::ONE);
}

#[test]
fn splat_sampling_walks_the_footprint_long_axis_when_anisotropy_is_enabled() {
    let shader = include_str!("../native_terrain.wgsl");
    for anisotropic_operation in [
        "let max_taps = max(material.render_quality.x, 1.0);",
        "let ratio = clamp(long_length / max(short_length, 0.000001), 1.0, max_taps);",
        "let taps = u32(clamp(round(ratio), 1.0, max_taps));",
        "log2(max(long_length / f32(taps), 1.0)) + source.w,",
        "for (var tap = 0u; tap < sample_coordinates.taps; tap += 1u)",
    ] {
        assert!(
            shader.contains(anisotropic_operation),
            "missing anisotropic operation {anisotropic_operation:?}"
        );
    }
    assert!(
        NATIVE_TERRAIN_ISOTROPIC_TAPS == 1.0,
        "one tap must reproduce the exact legacy single-mip footprint"
    );
    assert!(NATIVE_TERRAIN_ANISOTROPIC_TAPS > NATIVE_TERRAIN_ISOTROPIC_TAPS);
}

#[test]
fn native_terrain_wgsl_contains_only_the_proven_pass_compositor() {
    let shader = include_str!("../native_terrain.wgsl");
    for exact_source_operation in [
        "for (var group_start = 0u; group_start < layer_count; group_start += 4u)",
        "if layer_mode(group_start) == 0u",
        "let weights = sample_weight_map(group_start / 4u, terrain_uv);",
        "let weight = splat_channel(weights, layer - group_start);",
        "if weight > 0.0",
        "let sample_coordinates = layer_sample_coordinates(layer, terrain_uv);",
        "splat_source += layer_rgb(layer, sample_coordinates) * weight;",
        "let pass_color = clamp(splat_source, vec3<f32>(0.0), vec3<f32>(1.0));",
        "if layer_mode(group_start) == 1u",
        "blend_source += source * weight;",
        "mix(blend_source, source, weight)",
        "blend_alpha += weight;",
        "blend_alpha = clamp(blend_alpha, 0.0, 1.0);",
        "blend_source * blend_alpha + framebuffer * (1.0 - blend_alpha)",
        "if terrain_render_mode() == 1u",
        "} else if terrain_render_mode() == 2u {",
        "framebuffer *= lightmap.rgb;",
        "material.ambience_light.rgb,",
        "lightmap.rgb,",
        "framebuffer * vertex_pass,",
        "let view_position = view.view_from_world * mesh.world_position;",
        "1.0 - exp(-(density_depth * density_depth))",
        "return vec4<f32>(srgb_to_linear(framebuffer), 1.0);",
        "FirstPass Geometry-100",
        "AddPass Geometry-99",
        "BlendPass Geometry-98",
    ] {
        assert!(
            shader.contains(exact_source_operation),
            "missing shader operation {exact_source_operation:?}"
        );
    }
    assert_eq!(
        shader
            .matches("let weights = sample_weight_map(group_start / 4u, terrain_uv);")
            .count(),
        2,
        "each splat/blend quartet must fetch its control map once per pass"
    );
    for removed_preview_heuristic in [
        "world_normal",
        "light_direction",
        "effective_weight",
        "accumulated_weight",
        "base_albedo",
        "composited /=",
        "clamp(weight",
        "let ambient_pass = mix(",
        "framebuffer + surface * vertex_pass,",
        "material.metadata.y > 0.5",
    ] {
        assert!(
            !shader.contains(removed_preview_heuristic),
            "preview heuristic survived in shader: {removed_preview_heuristic:?}"
        );
    }
}

#[test]
fn streamed_terrain_preparation_and_materialization_are_backpressured() {
    assert_eq!(native_heightmap_preparation_slots(0), 1);
    assert_eq!(native_heightmap_preparation_slots(1), 0);
    assert_eq!(native_heightmap_preparation_slots(2), 0);
    assert_eq!(native_heightmap_preparation_slots(usize::MAX), 0);
    assert_eq!(native_terrain_grass_preparation_slots(0), 1);
    assert_eq!(native_terrain_grass_preparation_slots(1), 0);
    assert_eq!(native_terrain_grass_preparation_slots(usize::MAX), 0);
    assert_eq!(NATIVE_TERRAIN_MATERIALIZATIONS_PER_FRAME, 1);
}

#[test]
fn map_12_03_heightmap_is_full_resolution_upward_and_exactly_bounded() {
    let descriptor_path = "map/tiles/map_12_03/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        blake3::hash(&bytes).to_hex().as_str(),
    )
    .unwrap();
    let geometry = terrain.geometry();
    assert_eq!(geometry.vertex_count(), 129 * 129);
    assert_eq!(geometry.index_count(), 128 * 128 * 6);
    assert_eq!(&geometry.indices()[0..6], &[0, 1, 129, 1, 130, 129]);
    let minimum = geometry
        .positions()
        .iter()
        .map(|position| Vec3::from_array(*position))
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let maximum = geometry
        .positions()
        .iter()
        .map(|position| Vec3::from_array(*position))
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    assert!((minimum.x + 512.0).abs() <= f32::EPSILON);
    assert!(minimum.z.abs() <= f32::EPSILON);
    assert!(maximum.x.abs() <= f32::EPSILON);
    assert!((maximum.z - 512.0).abs() <= f32::EPSILON);
    assert!(geometry.normals().iter().all(|normal| normal[1] > 0.0));
    // Published control and baked-light PNG rows require the legacy GPU V
    // orientation. Geometry rows still advance along +nativeZ, while V
    // descends from one to zero; changing this rotates both maps together.
    assert_eq!(geometry.uvs()[0], [0.0, 1.0]);
    assert_eq!(geometry.uvs()[128], [1.0, 1.0]);
    assert_eq!(geometry.uvs()[128 * 129], [0.0, 0.0]);
    assert_eq!(geometry.uvs()[129 * 129 - 1], [1.0, 0.0]);

    for triangle in geometry.indices().chunks_exact(3).take(64) {
        let a = Vec3::from_array(geometry.positions()[triangle[0] as usize]);
        let b = Vec3::from_array(geometry.positions()[triangle[1] as usize]);
        let c = Vec3::from_array(geometry.positions()[triangle[2] as usize]);
        assert!((b - a).cross(c - a).y > 0.0);
    }
}

#[test]
fn terrain_json_and_every_png_fail_closed_on_hash_or_schema_drift() {
    let descriptor_path = "map/tiles/map_12_03/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let hash = blake3::hash(&bytes).to_hex().to_string();
    let terrain = NativeTerrain::open(asset_root(), descriptor_path, &hash).unwrap();
    let environment = terrain
        .environment()
        .expect("production v5 terrain must carry its verified environment");
    assert_eq!(environment.schema, NATIVE_TERRAIN_ENVIRONMENT_SCHEMA);
    assert_eq!(environment.status, "complete");
    assert_eq!(environment.scope, "worldMap");
    assert_eq!(environment.tile_id, "12_03");
    assert_eq!(environment.ambience.grid_coordinates, [12, 3]);
    assert!((environment.ambience.fog_depth - 0.44).abs() < 0.000_001);
    let render = &environment
        .terrain_detail
        .serialized_terrain_render_baseline
        .serialized_fields;
    assert_eq!(render.render_mode, 1);
    assert_eq!(render.use_lightmap, 0);
    assert_eq!(render.cast_shadows, 1);
    let mut wrong = hash;
    wrong.replace_range(0..1, if &wrong[0..1] == "0" { "1" } else { "0" });
    assert!(NativeTerrain::open(asset_root(), descriptor_path, &wrong).is_err());

    let mut json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    json["unexpectedRuntimeFixup"] = serde_json::json!(true);
    assert!(serde_json::from_value::<NativeTerrainDescriptor>(json).is_err());
}

#[test]
fn splat_material_preserves_all_source_slots_and_tilings() {
    let descriptor_path = "map/tiles/map_12_03/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        blake3::hash(&bytes).to_hex().as_str(),
    )
    .unwrap();
    assert_eq!(terrain.weight_maps.len(), 5);
    assert_eq!(terrain.layer_images.len(), 17);
    assert!(
        terrain.weight_map_mips.iter().all(|mips| mips.len() == 9),
        "Map_12_03 must retain every serialized weight-map mip"
    );
    assert_eq!(
        layer_array_image(&terrain)
            .texture_descriptor
            .mip_level_count,
        9
    );
    assert_eq!(
        lightmap_image(&terrain).texture_descriptor.mip_level_count,
        11
    );
    let uniform = terrain_material_uniform(&terrain);
    assert_eq!(
        uniform.metadata.y, 1.0,
        "Map_12_03 selects the saved Lightmap shader family even though its separate m_UseLightmap field is zero"
    );
    assert_eq!(uniform.metadata.z, 150.0);
    for (index, layer) in terrain.descriptor.splat.layers.iter().enumerate() {
        assert_eq!(
            uniform.layer_tile_scale_mode[index].truncate().truncate(),
            Vec2::new(
                (terrain.descriptor.scale.extent_x / layer.tile_size.x) as f32,
                (terrain.descriptor.scale.extent_z / layer.tile_size.y) as f32,
            )
        );
        assert_eq!(uniform.layer_tile_scale_mode[index].z, layer.mode as f32);
        assert_eq!(
            uniform.layer_source_size[index].truncate().truncate(),
            Vec2::new(layer.albedo.width as f32, layer.albedo.height as f32)
        );
    }
}

#[test]
fn terrain_segment_hit_uses_the_exact_rendered_cell_triangles() {
    let collider = legacy_height_test_collider();
    let owner_and_root = GlobalTransform::from(Transform::from_xyz(100.0, -30.0, 200.0));
    let start = Vec3::new(99.0, 20.0, 202.0);
    let end = Vec3::new(99.0, -50.0, 202.0);

    let hit = collider
        .segment_hit(&owner_and_root, start, end)
        .expect("vertical camera segment must hit the rendered terrain triangle");
    assert!((hit.fraction - 0.5).abs() < 0.000_01);
    assert!(
        hit.point
            .abs_diff_eq(Vec3::new(99.0, -15.0, 202.0), 0.000_01)
    );
    assert!(hit.normal.is_normalized());
    assert!(hit.normal.y > 0.0);
    let ground = collider
        .ground_contact(&owner_and_root, 99.0, 202.0, -50.0, 20.0)
        .expect("player ground query must retain the terrain triangle contact");
    assert_eq!(ground.point, hit.point);
    assert_eq!(ground.normal, hit.normal);
    assert_ne!(
        ground.normal,
        Vec3::Y,
        "TerrainCollider contact must not be flattened before slope handling"
    );

    assert!(
        collider
            .segment_hit(
                &owner_and_root,
                Vec3::new(101.0, 20.0, 202.0),
                Vec3::new(101.0, -50.0, 202.0),
            )
            .is_none(),
        "segment outside the finite TerrainCollider must not be clamped onto its edge"
    );
}

#[test]
fn map_12_03_heightfield_samples_exact_render_triangles_in_constant_time() {
    let descriptor_path = "map/tiles/map_12_03/terrain/terrain.json";
    let bytes = fs::read(asset_root().join(descriptor_path)).unwrap();
    let terrain = NativeTerrain::open(
        asset_root(),
        descriptor_path,
        blake3::hash(&bytes).to_hex().as_str(),
    )
    .unwrap();
    let geometry = terrain.geometry().clone();
    let collider = NativeHeightmapCollider {
        source_mesh: Handle::default(),
        source_descriptor_path: descriptor_path.to_owned(),
        geometry: geometry.clone(),
        width: 129,
        height: 129,
        sample_spacing_x: terrain.descriptor().scale.sample_spacing_x as f32,
        sample_spacing_z: terrain.descriptor().scale.sample_spacing_z as f32,
        terrain_size_x: terrain.descriptor().scale.extent_x as f32,
        terrain_size_z: terrain.descriptor().scale.extent_z as f32,
        gameplay_attributes: None,
    };
    let tile_and_owner = GlobalTransform::from(Transform::from_xyz(-6144.0, -300.0, 1536.0));
    assert_eq!(
        collider.gameplay_attribute(&tile_and_owner, -6144.0, 1536.0),
        NativeTerrainGameplayAttributeSample::SidecarMissing
    );
    let corner_height = geometry.positions()[0][1] - 300.0;
    assert_eq!(
        collider.legacy_interpolated_height(&tile_and_owner, -6144.0, 1536.0),
        NativeTerrainLegacyInterpolatedHeightSample::Height(corner_height),
        "legacy Terrain.SampleHeight must apply the owner/root Y exactly once"
    );

    let column = 44_usize;
    let row = 83_usize;
    let vertex = Vec3::from_array(geometry.positions()[row * 129 + column]);
    let world = tile_and_owner.transform_point(vertex);
    let sampled = collider
        .ground_height(
            &tile_and_owner,
            world.x,
            world.z,
            world.y - 1.0,
            world.y + 1.0,
        )
        .unwrap();
    assert!((sampled - world.y).abs() <= 0.000_01);

    // A point in the first source triangle must use h00/u/h10/v/h01,
    // not bilinear interpolation or the opposite diagonal.
    let u = 0.25_f32;
    let v = 0.35_f32;
    let i00 = row * 129 + column;
    let h00 = geometry.positions()[i00][1];
    let h10 = geometry.positions()[i00 + 1][1];
    let h01 = geometry.positions()[i00 + 129][1];
    let expected_local = h00 + u * (h10 - h00) + v * (h01 - h00);
    let local_x = -(column as f32 + u) * 4.0;
    let local_z = (row as f32 + v) * 4.0;
    let expected_world = expected_local - 300.0;
    let sampled = collider
        .ground_height(
            &tile_and_owner,
            -6144.0 + local_x,
            1536.0 + local_z,
            expected_world - 1.0,
            expected_world + 1.0,
        )
        .unwrap();
    assert!((sampled - expected_world).abs() <= 0.000_02);

    assert!(
        collider
            .ground_height(&tile_and_owner, -6656.1, 1536.0, -1000.0, 1000.0)
            .is_none()
    );
    assert_eq!(collider.vertex_count(), 129 * 129);
    assert_eq!(collider.index_count(), 128 * 128 * 6);

    let mut values = vec![0_u8; 129 * 129];
    values[row * 129 + column] = 0x5a;
    let attribute_collider = NativeHeightmapCollider {
        gameplay_attributes: Some(Arc::new(VerifiedGameplayAttributes {
            width: 129,
            height: 129,
            values: values.into(),
        })),
        ..collider
    };
    let attribute_probe = tile_and_owner.transform_point(Vec3::new(
        -(column as f32 * 4.0 + 1.5),
        0.0,
        row as f32 * 4.0 + 2.0,
    ));
    assert_eq!(
        attribute_collider.gameplay_attribute(
            &tile_and_owner,
            attribute_probe.x,
            attribute_probe.z
        ),
        NativeTerrainGameplayAttributeSample::Value(0x5a)
    );
    assert_eq!(
        attribute_collider.gameplay_attribute(&tile_and_owner, -6660.1, 1536.0),
        NativeTerrainGameplayAttributeSample::OutsideTerrain
    );
}
