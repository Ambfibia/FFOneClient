use super::*;

#[test]
fn legacy_projectile_api_requires_captured_unity_samples() {
    let mut runtime = TutorialEffectRuntime::with_library(library());
    runtime.enqueue(TutorialEffectRuntimeCommand::ProjectilePair {
        types: [76, 77],
        source: Vec3::ZERO,
        target: Vec3::ONE,
        oni: true,
        priority: 0,
        source_line: 4042,
    });
    runtime.process_pending();
    assert!(matches!(
        runtime.drain_issues().next(),
        Some(
            TutorialEffectRuntimeIssue::ProjectileRuntimeSamplesRequired {
                types: [76, 77],
                required_draw_count_normal: 6,
                required_draw_count_reverse: 4,
                ..
            }
        )
    ));

    runtime.enqueue(TutorialEffectRuntimeCommand::ProjectilePair {
        types: [77, 76],
        source: Vec3::ZERO,
        target: Vec3::ONE,
        oni: true,
        priority: 0,
        source_line: 4042,
    });
    runtime.process_pending();
    assert!(matches!(
        runtime.drain_issues().next(),
        Some(
            TutorialEffectRuntimeIssue::ProjectilePairOutsideExactContract {
                types: [77, 76],
                ..
            }
        )
    ));
}

#[test]
fn unity_draw_mapping_preserves_call_order_and_reverse_draw_count() {
    assert_eq!(
        tutorial_oni_velocity_samples_from_unity_unit_draws(
            false,
            &[0.0, 0.25, 0.5, 1.0, 0.75, 1.0],
        ),
        Some([Vec3::new(-10.0, 7.5, -5.0), Vec3::new(10.0, 10.0, 5.0),])
    );
    assert_eq!(
        tutorial_oni_velocity_samples_from_unity_unit_draws(true, &[0.0, 0.25, 1.0, 0.75],),
        Some([Vec3::new(-10.0, 0.0, -5.0), Vec3::new(10.0, 0.0, 5.0),])
    );
    assert!(tutorial_oni_velocity_samples_from_unity_unit_draws(true, &[0.0; 6]).is_none());
}
