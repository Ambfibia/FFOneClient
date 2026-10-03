use super::*;
#[test]
fn corruption_styles_load_original_projectiles_without_an_impact() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut runtime = TutorialEffectRuntime::default();
    runtime.load_native_particle_catalog(&crate::assets::AssetLocator::open(root).unwrap()).unwrap();
    for (id, particle) in [(167, 18), (168, 830), (169, 831)] {
        runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
            bullet_type: id, source: Vec3::ZERO, target: Vec3::X,
            target_exists: true, source_style: -1, target_style: -1,
            motion: TutorialProjectileMotion::BulletMove, source_line: 0,
        });
        runtime.process_pending();
        assert_eq!(runtime.drain_issues().count(), 0);
        let tutorial_native_effects::NativeSpawnRequest::LinearProjectile {
            effect_id, impact, carried_effect, motion, ..
        } = runtime.native_spawns.pop_front().unwrap() else {panic!("not a projectile")};
        assert_eq!(effect_id, particle);
        assert!(impact.is_none());
        assert!(carried_effect.is_some());
        assert!(matches!(motion, tutorial_native_effects::NativeLinearProjectileMotion::BulletMove {
            duration_seconds: 0.5, ..
        }));
    }
}
#[test]
fn healing_and_egg_projectiles_queue_from_native_assets_without_legacy_catalog() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut runtime = TutorialEffectRuntime::default();
    runtime
        .load_native_particle_catalog(&crate::assets::AssetLocator::open(root).unwrap())
        .unwrap();
    for (id, particle, impact) in [
        (88, 827, 407),
        (92, 0, 407),
        (97, 828, 438),
        (105, 502, 500),
    ] {
        runtime.enqueue(TutorialEffectRuntimeCommand::Projectile {
            bullet_type: id,
            source: Vec3::ZERO,
            target: Vec3::X,
            target_exists: true,
            source_style: -1,
            target_style: -1,
            motion: TutorialProjectileMotion::BulletMove,
            source_line: 0,
        });
        runtime.process_pending();
        assert_eq!(runtime.drain_issues().count(), 0);
        let tutorial_native_effects::NativeSpawnRequest::LinearProjectile {
            effect_id,
            impact: hit,
            carried_effect,
            ..
        } = runtime.native_spawns.pop_front().unwrap()
        else {
            panic!("not a projectile")
        };
        assert_eq!(effect_id, particle);
        assert_eq!(hit.unwrap().effect_id, impact);
        assert_eq!(carried_effect.is_some(), particle != 0);
    }
}
