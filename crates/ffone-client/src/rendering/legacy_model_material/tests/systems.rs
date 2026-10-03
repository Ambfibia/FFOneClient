use super::*;

#[test]
fn sky_rimlight_sync_marks_only_materials_that_actually_change() {
    let textures = LegacyModelTextures::default();
    let sky_params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::Toon);
    let fixed_params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToonRim);
    let mut materials = Assets::<LegacyModelMaterial>::default();
    let sky_material = materials.add(
        sky_params
            .material_for_pass(sky_params.render_plan().passes[0], &textures)
            .unwrap(),
    );
    let fixed_material = materials.add(
        fixed_params
            .material_for_pass(fixed_params.render_plan().passes[0], &textures)
            .unwrap(),
    );
    let sampled_sky = LinearRgba::new(0.12, 0.34, 0.56, 1.0);

    assert_eq!(
        sync_legacy_sky_rimlight_assets(&mut materials, sampled_sky),
        1
    );
    assert_eq!(
        materials.get(&sky_material).unwrap().uniform.rim_color,
        sampled_sky
    );
    assert_ne!(
        materials.get(&fixed_material).unwrap().uniform.rim_color,
        sampled_sky
    );
    assert_eq!(
        sync_legacy_sky_rimlight_assets(&mut materials, sampled_sky),
        0,
        "an unchanged frame must not enqueue any material modifications"
    );
}

#[test]
fn streamed_entity_insert_ignores_despawn_before_deferred_apply() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        commands.entity(entity).despawn();
        try_insert_streamed_entity(&mut commands, entity, StreamingSafeInsertMarker);
    }

    queue.apply(&mut world);
    assert!(world.get_entity(entity).is_err());
}
