use super::{LegacyStaticMaterialCache, SharedAssets, maintain_shared_static_assets};
use crate::legacy_model_material::LegacyModelMaterial;
use crate::legacy_model_material::{
    LegacyModelMaterialParams, LegacyModelTextures, LegacyShaderKind,
};
use bevy::prelude::*;
use bevy::{
    asset::RenderAssetUsages,
    image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    mesh::Indices,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use std::sync::Arc;

fn pixel() -> Image {
    Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![120, 80, 40, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

fn material() -> LegacyModelMaterial {
    let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::OpaqueNormal);
    params
        .material_for_pass(
            params.render_plan().passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap()
}

#[test]
fn shared_material_detaches_once_before_instance_texture_or_uniform_writes() {
    let mut assets = Assets::default();
    let mut cache = LegacyStaticMaterialCache::default();
    let original = material();
    let first = cache.admit_immutable(original.clone(), &mut assets);
    let mut second = cache.admit_immutable(original.clone(), &mut assets);
    assert_eq!(first, second);
    super::make_legacy_material_unique(&mut second, &mut assets);
    assert_ne!(first, second);
    assert!(assets.get(&second).unwrap().is_instance_private());
    let private = second.clone();
    for _ in 0..20 {
        super::make_legacy_material_unique(&mut second, &mut assets);
        assets.get_mut(&second).unwrap().uniform.base_color.alpha = 0.25;
    }
    assert_eq!(second, private);
    assert_eq!(assets.len(), 2);
    let mut images = Assets::<Image>::default();
    assets.get_mut(&second).unwrap().base_texture = Some(images.add(pixel()));
    assert_eq!(assets.get(&first).unwrap(), &original);
    let third = cache.admit_immutable(original, &mut assets);
    assert_eq!(third, first);
    assert!(assets.get(&first).unwrap().clone().is_instance_private());
}

#[test]
fn final_model_admission_preserves_order_and_early_private_bindings() {
    use crate::legacy_model_material::{
        LegacyMaterialRendererOrder, LegacyOutlineMaterial, PendingLegacyModelMaterial,
        apply_legacy_material_renderer_order,
    };
    let mut app = App::new();
    app.init_resource::<Assets<LegacyModelMaterial>>()
        .init_resource::<Assets<LegacyOutlineMaterial>>()
        .init_resource::<LegacyStaticMaterialCache>()
        .add_systems(Update, apply_legacy_material_renderer_order);
    let mut sources = Vec::new();
    for (index, order) in [0, 0, 1, 0].into_iter().enumerate() {
        let mut handle = app
            .world_mut()
            .resource_mut::<Assets<LegacyModelMaterial>>()
            .add(material());
        if index == 3 {
            super::make_legacy_material_unique(
                &mut handle,
                &mut app
                    .world_mut()
                    .resource_mut::<Assets<LegacyModelMaterial>>(),
            );
        }
        let pending = PendingLegacyModelMaterial {
            true_name: "test".into(),
            serialized_shader_name: "test".into(),
            declared_shader_name: "test".into(),
            params: LegacyModelMaterialParams::for_shader(LegacyShaderKind::OpaqueNormal),
            shader_texture_defaults: vec![],
            texture_bindings: vec![],
            source_render_queue: 3000,
            source_passes: vec![],
            source_pass_count: 0,
        };
        let entity = app
            .world_mut()
            .spawn((
                MeshMaterial3d(handle.clone()),
                pending,
                LegacyMaterialRendererOrder {
                    renderer_index: order,
                },
            ))
            .id();
        sources.push((entity, handle));
    }
    app.update();
    let handles: Vec<_> = sources
        .iter()
        .map(|(e, _)| {
            app.world()
                .get::<MeshMaterial3d<LegacyModelMaterial>>(*e)
                .unwrap()
                .0
                .clone()
        })
        .collect();
    assert_eq!(handles[0], handles[1]);
    assert_ne!(
        handles[0], handles[2],
        "renderer/pass order is part of equality"
    );
    assert_eq!(
        handles[3], sources[3].1,
        "an earlier writer's cached private handle remains valid"
    );
    assert_ne!(handles[0], handles[3]);
}

#[test]
fn renderer_rebinding_refreshes_sort_order_without_mutating_shared_peer() {
    use crate::legacy_model_material::{
        LegacyMaterialRendererOrder, LegacyMaterialSortOrderApplied, LegacyOutlineMaterial,
        PendingLegacyModelMaterial, apply_legacy_material_renderer_order,
    };
    let mut app = App::new();
    app.init_resource::<Assets<LegacyModelMaterial>>()
        .init_resource::<Assets<LegacyOutlineMaterial>>()
        .init_resource::<LegacyStaticMaterialCache>()
        .add_systems(Update, apply_legacy_material_renderer_order);
    let pending = PendingLegacyModelMaterial {
        true_name: "face".into(),
        serialized_shader_name: "test".into(),
        declared_shader_name: "test".into(),
        params: LegacyModelMaterialParams::for_shader(LegacyShaderKind::OpaqueNormal),
        shader_texture_defaults: vec![],
        texture_bindings: vec![],
        source_render_queue: 2900,
        source_passes: vec![],
        source_pass_count: 0,
    };
    let mut entities = Vec::new();
    for _ in 0..2 {
        let handle = app
            .world_mut()
            .resource_mut::<Assets<LegacyModelMaterial>>()
            .add(material());
        entities.push(
            app.world_mut()
                .spawn((
                    MeshMaterial3d(handle),
                    pending.clone(),
                    LegacyMaterialRendererOrder { renderer_index: 0 },
                ))
                .id(),
        );
    }
    app.update();
    let original = app
        .world()
        .get::<MeshMaterial3d<LegacyModelMaterial>>(entities[1])
        .unwrap()
        .0
        .clone();
    assert_eq!(
        app.world()
            .get::<MeshMaterial3d<LegacyModelMaterial>>(entities[0])
            .unwrap()
            .0,
        original
    );

    // A modular rebind assigns the renderer its new flattened actor index.
    app.world_mut()
        .entity_mut(entities[0])
        .insert(LegacyMaterialRendererOrder { renderer_index: 8 });
    app.update();
    assert_eq!(
        app.world()
            .get::<LegacyMaterialSortOrderApplied>(entities[0])
            .unwrap()
            .renderer_index,
        8
    );
    let rebound = &app
        .world()
        .get::<MeshMaterial3d<LegacyModelMaterial>>(entities[0])
        .unwrap()
        .0;
    let assets = app.world().resource::<Assets<LegacyModelMaterial>>();
    assert_eq!(assets.get(rebound).unwrap().sort_bias, 32.0);
    assert_eq!(assets.get(&original).unwrap().sort_bias, 0.0);
    assert_ne!(rebound, &original);
}

#[test]
fn renderer_rebinding_retries_until_the_outline_companion_is_ready() {
    use crate::legacy_model_material::{
        LegacyMaterialPassCompanion, LegacyMaterialRendererOrder, LegacyMaterialSortOrderApplied,
        LegacyOutlineMaterial, LegacyOutlineUniform, LegacyPassKind, PendingLegacyModelMaterial,
        apply_legacy_material_renderer_order,
    };
    let mut app = App::new();
    app.init_resource::<Assets<LegacyModelMaterial>>()
        .init_resource::<Assets<LegacyOutlineMaterial>>()
        .add_systems(Update, apply_legacy_material_renderer_order);
    let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToon);
    let plan = params.render_plan();
    let base = params
        .material_for_pass(plan.passes[0], &LegacyModelTextures::default())
        .unwrap()
        .clone();
    let base = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(base);
    let outline = app
        .world_mut()
        .resource_mut::<Assets<LegacyOutlineMaterial>>()
        .add(LegacyOutlineMaterial {
            uniform: LegacyOutlineUniform {
                color: LinearRgba::BLACK,
                width_fat: Vec4::ZERO,
                fog_params: Vec4::ZERO,
            },
            render_mode: plan.outline().unwrap().render_mode,
            sort_bias: 0.0,
        });
    let source = app
        .world_mut()
        .spawn((
            MeshMaterial3d(base.clone()),
            LegacyMaterialRendererOrder { renderer_index: 0 },
            PendingLegacyModelMaterial {
                true_name: "face-sub".into(),
                serialized_shader_name: "test".into(),
                declared_shader_name: "test".into(),
                params,
                shader_texture_defaults: vec![],
                texture_bindings: vec![],
                source_render_queue: 2900,
                source_passes: vec![],
                source_pass_count: 0,
            },
        ))
        .id();
    let pass = LegacyMaterialPassCompanion {
        source_mesh_entity: source,
        pass: LegacyPassKind::Outline,
    };
    let companion = app
        .world_mut()
        .spawn((pass.clone(), MeshMaterial3d(outline.clone())))
        .id();
    app.update();
    assert!(
        app.world()
            .get::<LegacyMaterialSortOrderApplied>(source)
            .is_some()
    );
    app.world_mut().entity_mut(companion).despawn();
    app.world_mut()
        .entity_mut(source)
        .insert(LegacyMaterialRendererOrder { renderer_index: 8 });
    app.update();
    assert!(
        app.world()
            .get::<LegacyMaterialSortOrderApplied>(source)
            .is_none()
    );
    app.world_mut()
        .spawn((pass, MeshMaterial3d(outline.clone())));
    app.update();
    assert_eq!(
        app.world()
            .get::<LegacyMaterialSortOrderApplied>(source)
            .unwrap()
            .renderer_index,
        8
    );
    assert_eq!(
        app.world()
            .resource::<Assets<LegacyModelMaterial>>()
            .get(&base)
            .unwrap()
            .sort_bias,
        32.0
    );
    assert_eq!(
        app.world()
            .resource::<Assets<LegacyOutlineMaterial>>()
            .get(&outline)
            .unwrap()
            .sort_bias,
        33.0
    );
}

#[test]
fn equal_pixels_share_but_sampler_color_space_and_layout_do_not() {
    let mut assets = Assets::<Image>::default();
    let a = assets.add(pixel());
    let b = assets.add(pixel());
    let mut cache = SharedAssets::<Image>::default();
    assert_eq!(cache.intern(&a, &assets), a);
    assert_eq!(cache.intern(&b, &assets), a);
    let mut variants = Vec::new();
    let mut linear = pixel();
    linear.texture_descriptor.format = TextureFormat::Rgba8Unorm;
    variants.push(linear);
    let mut clamped = pixel();
    clamped.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::ClampToEdge,
        ..default()
    });
    variants.push(clamped);
    let mut layout = pixel();
    layout.texture_descriptor.dimension = TextureDimension::D1;
    variants.push(layout);
    for variant in variants {
        let handle = assets.add(variant);
        assert_eq!(cache.intern(&handle, &assets), handle);
        assert_ne!(handle, a);
    }
    assert_eq!(cache.hashed, 5);
    cache.intern(&a, &assets);
    assert_eq!(
        cache.hashed, 5,
        "resident assets are not hashed every frame"
    );
}

#[test]
fn render_targets_are_never_shared_even_when_initialized_pixels_match() {
    let mut assets = Assets::<Image>::default();
    let mut cache = SharedAssets::<Image>::default();
    let mut target = pixel();
    target.texture_descriptor.usage |=
        bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT;
    let first = assets.add(target.clone());
    let second = assets.add(target);
    assert_eq!(cache.intern(&first, &assets), first);
    assert_eq!(cache.intern(&second, &assets), second);
    assert_eq!(cache.hashed, 0);
}

#[test]
fn exact_meshes_share_but_uv_normals_and_indices_remain_distinct() {
    let mut assets = Assets::<Mesh>::default();
    let mesh = Mesh::from(Rectangle::new(2.0, 3.0));
    let a = assets.add(mesh.clone());
    let b = assets.add(mesh.clone());
    let mut cache = SharedAssets::<Mesh>::default();
    cache.intern(&a, &assets);
    assert_eq!(cache.intern(&b, &assets), a);
    let mut uv = mesh.clone();
    uv.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0_f32, 0.0]; 4]);
    let mut normals = mesh.clone();
    normals.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0_f32, 1.0, 0.0]; 4]);
    let mut indices = mesh;
    indices.insert_indices(Indices::U32(vec![0, 2, 1, 1, 2, 3]));
    for variant in [uv, normals, indices] {
        let handle = assets.add(variant);
        assert_eq!(cache.intern(&handle, &assets), handle);
    }
}

#[test]
fn materials_preserve_every_uniform_render_state_and_texture_binding() {
    let mut assets = Assets::<LegacyModelMaterial>::default();
    let mut cache = SharedAssets::<LegacyModelMaterial>::default();
    let first = cache.insert(material(), &mut assets);
    assert_eq!(cache.insert(material(), &mut assets), first);
    assert_eq!(
        assets.len(),
        1,
        "reuse must avoid allocating a GPU material"
    );
    let mut uv = material();
    uv.uniform.uv_scale_offset.z = 0.125;
    let mut alpha = material();
    alpha.uniform.base_color.alpha = 0.5;
    let mut queue = material();
    queue.render_mode.source_queue += 1;
    let mut order = material();
    order.sort_bias = 2.0;
    let mut depth = material();
    depth.render_mode.depth_write = !depth.render_mode.depth_write;
    let mut animated = material();
    animated.gpu_uv_animation = true;
    for variant in [uv, alpha, queue, order, depth, animated] {
        assert_ne!(cache.insert(variant, &mut assets), first);
    }
    // The production animation binding uses this same clone-before-write boundary.
    let private = assets.add(assets.get(&first).unwrap().clone());
    assets.get_mut(&private).unwrap().uniform.base_color.alpha = 0.25;
    assert_eq!(assets.get(&first).unwrap().uniform.base_color.alpha, 1.0);
}

#[test]
fn caches_do_not_pin_streamed_assets_and_reject_modified_candidates() {
    let mut assets = Assets::<Image>::default();
    let mut cache = SharedAssets::<Image>::default();
    let a = assets.add(pixel());
    cache.intern(&a, &assets);
    assets.get_mut(&a).unwrap().data.as_mut().unwrap()[0] = 0;
    let b = assets.add(pixel());
    assert_eq!(
        cache.intern(&b, &assets),
        b,
        "hash collision/stale key is never equality"
    );
    let Handle::Strong(strong) = &b else {
        unreachable!()
    };
    assert_eq!(
        Arc::strong_count(strong),
        1,
        "the cache holds only weak references"
    );
    drop(a);
    drop(b);
    cache.prune();
    assert!(cache.candidates.is_empty());
    assert!(cache.fingerprints.is_empty());
}

#[test]
fn opacity_is_scanned_once_and_invalidated_on_image_edits() {
    let mut app = App::new();
    app.init_resource::<LegacyStaticMaterialCache>()
        .add_message::<AssetEvent<Image>>()
        .add_message::<AssetEvent<Mesh>>()
        .add_message::<AssetEvent<StandardMaterial>>()
        .add_message::<AssetEvent<LegacyModelMaterial>>()
        .add_systems(Update, maintain_shared_static_assets);
    let mut assets = Assets::<Image>::default();
    let handle = assets.add(pixel());
    {
        let mut cache = app.world_mut().resource_mut::<LegacyStaticMaterialCache>();
        for _ in 0..100 {
            assert!(cache.shared.opaque(&handle, assets.get(&handle).unwrap()));
        }
        assert_eq!(cache.shared.opacity_scans, 1);
    }
    assets.get_mut(&handle).unwrap().data.as_mut().unwrap()[3] = 0;
    app.world_mut()
        .write_message(AssetEvent::<Image>::Modified { id: handle.id() });
    app.update();
    let mut cache = app.world_mut().resource_mut::<LegacyStaticMaterialCache>();
    assert!(!cache.shared.opaque(&handle, assets.get(&handle).unwrap()));
    assert_eq!(cache.shared.opacity_scans, 2);
}

#[test]
fn production_highway_models_share_materials_without_merging_placements() {
    use crate::legacy_model_material::{
        PendingLegacyStaticWorldMaterial, apply_legacy_static_world_materials,
    };
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<Image>>()
        .init_resource::<Assets<StandardMaterial>>()
        .init_resource::<Assets<LegacyModelMaterial>>()
        .init_resource::<LegacyStaticMaterialCache>()
        .add_systems(Update, apply_legacy_static_world_materials);
    let mut entities = Vec::new();
    for index in 10..=14 {
        let path = root.join(format!(
            "objects/collections/dt_highway_02/models/dt_highway_01_{index}_default/visual.glb"
        ));
        let bytes = std::fs::read(&path).unwrap();
        let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let doc: serde_json::Value = serde_json::from_slice(&bytes[20..20 + json_length]).unwrap();
        let source = &doc["materials"][0];
        let pending = PendingLegacyStaticWorldMaterial::from_gltf_extras(
            source["name"].as_str(),
            &source["extras"].to_string(),
        )
        .unwrap();
        let uri = doc["images"][0]["uri"].as_str().unwrap();
        let image = Image::from_dynamic(
            image::load_from_memory(&std::fs::read(path.parent().unwrap().join(uri)).unwrap())
                .unwrap(),
            true,
            RenderAssetUsages::default(),
        );
        let texture = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        let standard = app
            .world_mut()
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color_texture: Some(texture),
                ..default()
            });
        let gltf = gltf::Gltf::from_slice(&bytes).unwrap();
        let primitive = gltf.meshes().next().unwrap().primitives().next().unwrap();
        let reader = primitive.reader(|_| gltf.blob.as_deref());
        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            reader.read_positions().unwrap().collect::<Vec<_>>(),
        );
        if let Some(normals) = reader.read_normals() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.collect::<Vec<_>>());
        }
        if let Some(uv) = reader.read_tex_coords(0) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv.into_f32().collect::<Vec<_>>());
        }
        if let Some(colors) = reader.read_colors(0) {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_COLOR,
                colors.into_rgba_f32().collect::<Vec<_>>(),
            );
        }
        mesh.insert_indices(Indices::U32(
            reader.read_indices().unwrap().into_u32().collect(),
        ));
        let mesh = app.world_mut().resource_mut::<Assets<Mesh>>().add(mesh);
        let transform = Transform::from_xyz(index as f32 * 10.0, 0.0, 0.0);
        let entity = app
            .world_mut()
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(standard),
                pending,
                transform,
                Visibility::Inherited,
            ))
            .id();
        entities.push((entity, mesh, transform));
    }
    app.update();
    let mut handles = std::collections::HashSet::new();
    for (entity, mesh, transform) in &entities {
        assert_eq!(app.world().get::<Transform>(*entity).unwrap(), transform);
        assert_eq!(
            *app.world().get::<Visibility>(*entity).unwrap(),
            Visibility::Inherited
        );
        let actual_mesh = &app.world().get::<Mesh3d>(*entity).unwrap().0;
        let meshes = app.world().resource::<Assets<Mesh>>();
        assert_eq!(meshes.get(actual_mesh), meshes.get(mesh));
        handles.insert(
            app.world()
                .get::<MeshMaterial3d<LegacyModelMaterial>>(*entity)
                .unwrap()
                .0
                .id(),
        );
    }
    assert_eq!(
        handles.len(),
        1,
        "five distinct published highway models must use one exact native material"
    );
    assert_eq!(
        app.world().resource::<Assets<LegacyModelMaterial>>().len(),
        1
    );
    assert_eq!(
        app.world()
            .resource::<LegacyStaticMaterialCache>()
            .shared
            .images
            .hits,
        4
    );
    assert_eq!(
        app.world()
            .resource::<LegacyStaticMaterialCache>()
            .shared
            .opacity_scans,
        1
    );
}
