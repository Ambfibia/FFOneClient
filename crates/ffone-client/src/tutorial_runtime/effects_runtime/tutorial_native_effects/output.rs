use super::*;

pub(in super::super) fn install(app: &mut App) {
    vehicle_trails::install(app);
    embedded_asset!(app, "tutorial_native_effects.wgsl");
    app.init_resource::<NativeVisualAssets>()
        .init_resource::<NativeEffectPreloadCache>()
        .init_resource::<NativeParticleRandomStream>()
        .init_resource::<NativeProjectileVisualPrewarm>()
        .init_resource::<TutorialProjectileVisualReadiness>()
        .add_plugins(MaterialPlugin::<TutorialParticleMaterial>::default())
        // A named attachment can be recursively despawned with a rebuilt NPC
        // visual while its runtime instance ID remains active. Reconcile the
        // real renderer roots before gameplay systems decide whether an icon
        // needs to be respawned.
        .add_systems(PreUpdate, reconcile_named_native_root_liveness)
        .add_systems(
            Update,
            (
                prewarm_native_projectile_visuals,
                apply_native_requests,
                sync_native_preload_completion,
                update_projectiles,
                simulate_particles,
                update_animated_particle_materials,
                cleanup_expired_particles,
                update_trails,
                prepare_native_mesh_effect_materials,
                play_native_mesh_effect_animations,
                animate_native_mesh_effect_materials,
                mark_native_effect_billboards,
                orient_native_effect_billboards,
                cleanup_effect_roots,
                cleanup_effect_meshes,
                cleanup_stream_owned_native_effects,
                prepare_native_sword_trails,
                configure_native_sword_trail_texture,
            )
                .chain()
                .after(process_tutorial_effect_runtime),
        )
        .add_systems(
            PostUpdate,
            (update_emitters, update_native_sword_trails)
                .chain()
                .after(TransformSystems::Propagate),
        );
}
