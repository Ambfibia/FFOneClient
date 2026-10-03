use super::*;
use crate::legacy_model_material::{
    LegacyModelMaterialParams, LegacyModelTextures, LegacyShaderKind,
};

fn stage_from_assets(
    assets: &Assets<LegacyModelMaterial>,
    updates: &NativeMaterialUniformUpdates,
    extracted: &mut ExtractedAssets<MeshMaterial3d<LegacyModelMaterial>>,
    gpu: &mut NativeGpuUniforms,
) {
    let current = updates
        .dirty
        .iter()
        .filter_map(|id| assets.get(*id).map(|m| (*id, m.uniform)))
        .collect();
    stage_uniforms(updates.enabled, current, extracted, gpu);
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
fn uniform_ownership_is_assigned_after_deferred_asset_extraction() {
    use bevy::ecs::schedule::ScheduleBuildSettings;
    #[derive(Resource)]
    struct TestId(AssetId<LegacyModelMaterial>);
    fn publish(mut commands: Commands, id: Res<TestId>) {
        let mut extracted = ExtractedAssets::<MeshMaterial3d<LegacyModelMaterial>>::default();
        extracted.extracted.push((id.0, material()));
        commands.insert_resource(extracted);
    }
    fn stage(mut commands: Commands, id: Res<TestId>) {
        let mut uniform = material().uniform;
        uniform.emission.red = 0.625;
        queue_uniform_staging(&mut commands, true, HashMap::from([(id.0, uniform)]));
    }
    let mut assets = Assets::<LegacyModelMaterial>::default();
    let handle = assets.add(material());
    let mut world = World::new();
    world.insert_resource(TestId(handle.id()));
    world.init_resource::<NativeGpuUniforms>();
    world.init_resource::<ExtractedAssets<MeshMaterial3d<LegacyModelMaterial>>>();
    let mut schedule = Schedule::new(ExtractSchedule);
    // Mirror Bevy's actual Extract schedule, including both deferred settings.
    schedule.set_build_settings(ScheduleBuildSettings {
        auto_insert_apply_deferred: false,
        ..default()
    });
    schedule.set_apply_final_deferred(false);
    schedule.add_systems((
        publish.in_set(AssetExtractionSystems),
        stage.after(AssetExtractionSystems),
    ));
    schedule.run(&mut world);
    assert!(world.resource::<NativeGpuUniforms>().pending.is_empty());
    assert!(
        world
            .resource::<ExtractedAssets<MeshMaterial3d<LegacyModelMaterial>>>()
            .extracted
            .is_empty()
    );
    // This is what RenderSystems::ExtractCommands does before PrepareAssets.
    schedule.apply_deferred(&mut world);
    let extracted = world.resource::<ExtractedAssets<MeshMaterial3d<LegacyModelMaterial>>>();
    assert_eq!(extracted.extracted[0].1.uniform_owner.0, Some(handle.id()));
    assert_eq!(
        world.resource::<NativeGpuUniforms>().pending[&handle.id()]
            .emission
            .red,
        0.625
    );
}

#[test]
fn extracted_clones_own_distinct_buffers_without_changing_material_equality() {
    let mut assets = Assets::<LegacyModelMaterial>::default();
    let a = assets.add(material());
    let b = assets.add(assets.get(&a).unwrap().clone());
    let mut extracted = ExtractedAssets::<MeshMaterial3d<LegacyModelMaterial>>::default();
    extracted.extracted = vec![
        (a.id(), assets.get(&a).unwrap().clone()),
        (b.id(), assets.get(&b).unwrap().clone()),
    ];
    let updates = NativeMaterialUniformUpdates {
        enabled: true,
        dirty: default(),
    };
    let mut gpu = NativeGpuUniforms::default();
    stage_from_assets(&assets, &updates, &mut extracted, &mut gpu);
    assert_eq!(extracted.extracted[0].1.uniform_owner.0, Some(a.id()));
    assert_eq!(extracted.extracted[1].1.uniform_owner.0, Some(b.id()));
    assert_eq!(extracted.extracted[0].1, extracted.extracted[1].1);
    assert_eq!(assets.get(&a).unwrap().uniform_owner.0, None);
    assert_eq!(gpu.pending.len(), 2);
}

#[test]
fn pending_final_pose_survives_loading_and_normal_edits_then_retires() {
    let mut assets = Assets::<LegacyModelMaterial>::default();
    let handle = assets.add(material());
    let id = handle.id();
    let mut updates = NativeMaterialUniformUpdates {
        enabled: true,
        dirty: default(),
    };
    let mut extracted = ExtractedAssets::<MeshMaterial3d<LegacyModelMaterial>>::default();
    let mut gpu = NativeGpuUniforms::default();
    assets.get_mut_untracked(id).unwrap().uniform.emission.red = 0.375;
    assert!(updates.enqueue(id));
    stage_from_assets(&assets, &updates, &mut extracted, &mut gpu);
    updates.dirty.clear();
    stage_from_assets(&assets, &updates, &mut extracted, &mut gpu);
    assert_eq!(gpu.pending[&id].emission.red, 0.375);
    // A later full material edit supersedes the last animation, even if the
    // original texture load has not finished and there is no buffer yet.
    assets.get_mut(id).unwrap().uniform.emission.red = 0.75;
    extracted
        .extracted
        .push((id, assets.get(id).unwrap().clone()));
    stage_from_assets(&assets, &updates, &mut extracted, &mut gpu);
    assert_eq!(gpu.pending[&id].emission.red, 0.75);
    extracted.extracted.clear();
    extracted.removed.insert(id);
    stage_from_assets(&assets, &updates, &mut extracted, &mut gpu);
    assert!(gpu.pending.is_empty());
}

#[test]
fn uniform_bytes_preserve_all_thirteen_vec4_fields_exactly() {
    let mut uniform = material().uniform;
    uniform.emission = LinearRgba::new(-0.0, 0.375, 1.5, -2.0);
    uniform.uv_animation = Vec4::new(0.25, -0.75, 180.0, 1234.0);
    let fields = [
        uniform.base_color.to_f32_array(),
        uniform.tint_color.to_f32_array(),
        uniform.ambient_color.to_f32_array(),
        uniform.emission.to_f32_array(),
        uniform.rim_color.to_f32_array(),
        uniform.rim_effect.to_array(),
        uniform.custom_effect.to_array(),
        uniform.uv_scale_offset.to_array(),
        uniform.uv_pivot_rotation.to_array(),
        uniform.uv_animation.to_array(),
        uniform.light_direction_family.to_array(),
        uniform.alpha_effect.to_array(),
        uniform.legacy_effect.to_array(),
    ];
    let expected: Vec<u8> = fields
        .into_iter()
        .flatten()
        .flat_map(f32::to_le_bytes)
        .collect();
    assert_eq!(uniform_bytes(&uniform), expected);
    assert_eq!(expected.len(), 208);
}

#[test]
fn diagnostic_baseline_keeps_normal_material_extraction() {
    let mut assets = Assets::<LegacyModelMaterial>::default();
    let handle = assets.add(material());
    let mut updates = NativeMaterialUniformUpdates {
        enabled: false,
        dirty: default(),
    };
    assert!(!updates.enqueue(handle.id()));
    let mut extracted = ExtractedAssets::<MeshMaterial3d<LegacyModelMaterial>>::default();
    extracted
        .extracted
        .push((handle.id(), assets.get(&handle).unwrap().clone()));
    let mut gpu = NativeGpuUniforms::default();
    stage_from_assets(&assets, &updates, &mut extracted, &mut gpu);
    assert_eq!(extracted.extracted[0].1.uniform_owner.0, None);
    assert!(gpu.pending.is_empty());
}

#[test]
fn batched_uniforms_preserve_independent_byte_ranges_and_reuse_scratch_storage() {
    let a = material().uniform;
    let mut b = a;
    b.base_color.red = -0.0;
    b.emission.blue = 0.3125;
    let mut bytes = Vec::new();
    let first = append_uniform(&mut bytes, &a) as usize;
    let second = append_uniform(&mut bytes, &b) as usize;
    assert_eq!(first, 0);
    assert_eq!(&bytes[first..second], uniform_bytes(&a));
    assert_eq!(&bytes[second..], uniform_bytes(&b));
    assert_eq!(second % 4, 0);
    let capacity = bytes.capacity();
    bytes.clear();
    assert_eq!(append_uniform(&mut bytes, &b), 0);
    assert_eq!(bytes, uniform_bytes(&b));
    assert_eq!(bytes.capacity(), capacity);
}
