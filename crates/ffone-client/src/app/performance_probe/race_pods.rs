//! Offline race state, production world/materials, and a fixed pod camera.
//! Use FFONE_PERF_RACE_PODS=1, FFONE_PERF_POSITION="-2933 -50.7 5467";
//! FFONE_PERF_POD_DISTANCE=2 (near) or 12 (far), FFONE_PERF_ENTRIES=2 for reload.
use super::*;
use ffone_client::legacy_model_material::LegacyMaterialPassCompanion;
use ffone_client::legacy_model_material::LegacyModelMaterial;
use std::collections::BTreeMap;

const POD: Vec3 = Vec3::new(-2936.3145, -49.59778, 5469.9673);

pub(super) fn install(app: &mut App) {
    let distance = env::var("FFONE_PERF_POD_DISTANCE")
        .map(|value| value.parse::<f32>().expect("positive pod camera distance"))
        .unwrap_or(2.0);
    assert!(distance.is_finite() && distance > 0.0);
    app.add_systems(Update, activate.before(race::sync_race_world_rings))
        .add_systems(
            PostUpdate,
            (move |mut cameras: Query<&mut Transform, With<Camera3d>>| {
                let target = POD + Vec3::Y * 0.49778;
                for mut transform in &mut cameras {
                    *transform = Transform::from_translation(
                        target + Vec3::new(distance, 0.3 * distance, -distance),
                    )
                    .looking_at(target, Vec3::Y);
                }
            })
            .before(bevy::transform::TransformSystems::Propagate),
        )
        .add_systems(Last, (verify_passes, verify_pulse));
}

fn activate(capture: Res<Capture>, mut race: ResMut<race::RaceProductionRuntime>) {
    // Only the diagnostic fixture uses unconstrained course bounds. Ring
    // visibility still runs through the production race owner every frame.
    race.player.ring_race_active = env::var_os("FFONE_PERF_RACE_HUD_LIFECYCLE").is_none()
        || capture.samples.len() < 500;
    race.player.race_limit_time = 3600;
    race.course_bounds = Some([i32::MIN, i32::MAX, i32::MIN, i32::MAX]);
    // Exercise the confirmed-pickup owner once in this offline fixture.
    if capture.samples.len() == 350 && !race.collected_rings.contains(&i32::MAX) {
        race.pending_rings.insert(i32::MAX);
        assert!(race.confirm_ring(i32::MAX, 1));
    }
}

fn verify_pulse(
    capture: Res<Capture>,
    sources: Query<(&bevy::gltf::GltfMaterialName, &GlobalTransform)>,
    passes: Query<(&LegacyMaterialPassCompanion, &MeshMaterial3d<LegacyModelMaterial>)>,
    materials: Res<Assets<LegacyModelMaterial>>,
    hud: Res<ffone_client::race_ui::hud::RaceHudState>,
    mut commands: Commands,
    mut samples: Local<BTreeMap<Entity, (f32, f32)>>,
) {
    let frame = capture.samples.len();
    if frame == 355 { assert_eq!(hud.pods, 1, "confirmed pickup reaches HUD"); }
    if frame == 510 && env::var_os("FFONE_PERF_RACE_HUD_LIFECYCLE").is_some() {
        assert!(!hud.running && !hud.awaiting_start, "HUD stays hidden with retained course info after race exit");
        eprintln!("Race pod entry {}: HUD cleared after exit with course info retained", capture.entry);
    }
    if matches!(frame, 100 | 160 | 355) {
        commands.spawn(Screenshot::primary_window()).observe(
            bevy::render::view::screenshot::save_to_disk(capture.output.join(format!("race-{frame}.png"))),
        );
    }
    if frame == 99 { samples.clear(); }
    if !(100..=160).contains(&frame) { return; }
    for (pass, handle) in &passes {
        let Ok((name, transform)) = sources.get(pass.source_mesh_entity) else { continue; };
        if !name.0.starts_with("fusion_ring-02") || transform.translation().distance(POD) >= 0.1 { continue; }
        let material = materials.get(&handle.0).expect("pod color-pass material");
        let value = material.uniform.emission.red;
        let range = samples.entry(pass.source_mesh_entity).or_insert((value, value));
        range.0 = range.0.min(value);
        range.1 = range.1.max(value);
    }
    if frame == 160 {
        assert_eq!(samples.len(), 2, "shell and core emission samples");
        // Only the inner Sphere01h:1 owns an authored emission controller.
        // The shell has no curve and must not receive a synthetic sine pulse.
        assert_eq!(samples.values().filter(|(min, max)| max - min > 0.01).count(), 1,
            "one authored core pulse, one constant shell: {samples:?}");
        eprintln!("Race pod entry {}: authored core emission changes; shell remains stable: {samples:?}", capture.entry);
    }
}

fn verify_passes(
    capture: Res<Capture>,
    sources: Query<(&bevy::gltf::GltfMaterialName, &GlobalTransform)>,
    passes: Query<(
        &LegacyMaterialPassCompanion,
        &Visibility,
        &InheritedVisibility,
    )>,
) {
    if capture.samples.len() != 100 {
        return;
    }
    let mut checked = 0;
    for (pass, visibility, inherited) in &passes {
        let Ok((name, transform)) = sources.get(pass.source_mesh_entity) else {
            continue;
        };
        if name.0.starts_with("fusion_ring-02") && transform.translation().distance(POD) < 0.1 {
            assert_eq!(
                *visibility,
                Visibility::Inherited,
                "hidden pod pass: {}",
                name.0
            );
            assert!(
                inherited.get(),
                "pod pass failed to inherit visible root: {}",
                name.0
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 2, "expected shell and core transparent passes");
    eprintln!(
        "Race pod entry {}: both material color passes visible",
        capture.entry
    );
}
