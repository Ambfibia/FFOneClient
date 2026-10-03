use super::*;

#[test]
fn grenade_bounces_on_native_terrain_and_only_local_expiry_sends_a_hit() {
    for local in [true, false] {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<TutorialEffectRuntime>()
            .init_resource::<crate::world_behaviour::WorldGameplayIntentQueue>()
            .add_systems(Update, update_projectiles);
        app.world_mut().spawn((crate::native_terrain::NativeHeightmapCollider::test_heightfield(2, 2, &[0.0;4]), GlobalTransform::IDENTITY));
        let id = app.world_mut().resource_mut::<TutorialEffectRuntime>().allocate_instance(false, None);
        let projectile = app.world_mut().spawn((NativeRoot { instance_id: id }, Transform::IDENTITY,
            NativeProjectile { target: Vec3::ZERO, impact: None, waiting_for_mesh_surface: false, waiting_for_trail_prewarm: false,
                motion: NativeProjectileMotion::Warhead { position: Vec3::new(-0.5, 0.5, 0.5), velocity: Vec3::NEG_Y * 4.0,
                    gravity: true, elapsed: 0.0, duration: 0.5,
                    authority: local.then_some(super::super::super::WarheadAuthority { bullet_id: 2, blast_radius: 2.0, target_capacity: 3 }),
                }
            })).id();
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(0.1));
        app.update();
        let entity = app.world().entity(projectile);
        let p = entity.get::<NativeProjectile>().unwrap();
        assert!(p.position().y > 0.0);
        assert!(matches!(p.motion, NativeProjectileMotion::Warhead { velocity, .. } if velocity.y > 0.0));
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(0.5));
        app.update();
        assert!(app.world().get_entity(projectile).is_err());
        let packets = app.world_mut().resource_mut::<crate::world_behaviour::WorldGameplayIntentQueue>().take_all();
        assert_eq!(packets.len(), usize::from(local));
        app.update();
        assert!(app.world_mut().resource_mut::<crate::world_behaviour::WorldGameplayIntentQueue>().take_all().is_empty());
    }
}
