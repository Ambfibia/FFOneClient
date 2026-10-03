//! Explicit camera/hover and authoritative buff fixture for visual regressions.
use super::*;

#[derive(Resource)]
struct VisualFixture {
    yaw: Option<f32>,
    pitch: Option<f32>,
    hover: Option<Vec3>,
    buff: Option<u32>,
    effect: Option<i32>,
}

pub(super) fn install(app: &mut App) {
    fn number(key: &str) -> Option<f32> {
        env::var(key).ok().map(|v| {
            let value: f32 = v.parse().expect("finite camera angle");
            assert!(value.is_finite());
            value
        })
    }
    let yaw = number("FFONE_PERF_YAW");
    let pitch = number("FFONE_PERF_PITCH");
    let hover = env::var_os("FFONE_PERF_HOVER").map(|_| {
        let position: Vec<f32> = env::var("FFONE_PERF_POSITION")
            .unwrap()
            .split_whitespace()
            .map(|v| v.parse().unwrap())
            .collect();
        assert_eq!(position.len(), 3);
        assert!(position.iter().all(|v| v.is_finite()));
        Vec3::from_slice(&position)
    });
    let buff = env::var("FFONE_PERF_BUFF").ok().map(|v| {
        let index: u32 = v.parse().expect("buff index 1..25");
        assert!((1..=25).contains(&index));
        1 << (index - 1)
    });
    let effect = env::var("FFONE_PERF_EFFECT")
        .ok()
        .map(|v| v.parse().expect("effect id"));
    if yaw.is_none() && pitch.is_none() && hover.is_none() && buff.is_none() && effect.is_none() {
        return;
    }
    app.insert_resource(VisualFixture {
        yaw,
        pitch,
        hover,
        buff,
        effect,
    })
    .add_systems(
        Update,
        drive_visual_fixture
            .after(NetworkSessionLifecycleSet::Apply)
            .after(consume_network_entity_lifecycle_0104)
            .before(world_skill_effects::sync_world_skill_effects),
    ).add_systems(Last, record_senses.before(super::measure));
}

fn drive_visual_fixture(
    fixture: Res<VisualFixture>,
    mut cameras: Query<&mut LegacyOrbitCamera>,
    mut player: Query<(Entity, &mut Transform), With<LocalPlayer>>,
    mut buffs: ResMut<SkillBuffUiModel>,
    mut npcs: Query<&mut NetworkNpcAppearance0104>,
    time: Res<Time>,
    mut elapsed: Local<f32>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    for mut camera in &mut cameras {
        if let Some(yaw) = fixture.yaw {
            camera.yaw_degrees = yaw;
        }
        if let Some(pitch) = fixture.pitch {
            camera.pitch_degrees = pitch;
        }
    }
    if let Some(position) = fixture.hover {
        for (_, mut transform) in &mut player {
            transform.translation = position;
        }
    }
    if let Some(mask) = fixture.buff {
        buffs.local_condition_bit_flag = mask;
        for mut npc in &mut npcs {
            npc.0.condition_bit_flag = mask as i32;
        }
    }
    *elapsed += time.delta_secs();
    if let Some(effect_id) = fixture.effect
        && *elapsed >= 1.0
    {
        *elapsed = 0.0;
        for (entity, transform) in &mut player {
            effects.enqueue(TutorialEffectRuntimeCommand::Add {
                effect_id,
                placement: TutorialEffectPlacement::ExactEntityWorld {
                    root_entity: entity,
                    position: transform.translation,
                    rotation: transform.rotation,
                },
                scale: 1.0,
                tracked: false,
                name: None,
                destroy_after_seconds: Some(1.0),
                source_line: 0,
            });
        }
    }
}

// Inspect the production HUD projection after Update, including the two sense
// abilities whose markers can overlap the avatar arrow in a close-range capture.
fn record_senses(
    capture: Res<Capture>,
    buffs: Res<SkillBuffUiModel>,
    hud: Res<GameplayUiModel>,
    mut recorded: Local<bool>,
) {
    if *recorded || capture.samples.len() < 580 { return; }
    *recorded = true;
    let mob_markers = hud.minimap.markers.iter().filter(|marker|
        matches!(marker.icon, MinimapMarkerIcon::TableData(icon) if icon.index() == 20)).count();
    let egg_markers = hud.minimap.markers.iter().filter(|marker|
        marker.icon == MinimapMarkerIcon::Shiny).count();
    if env::var_os("FFONE_PERF_SPECIAL_SKILLS").is_some() && buffs.reveals_mobs() {
        assert!(mob_markers > 0, "radar must project the ordinary replay enemy onto the HUD");
    }
    if env::var_os("FFONE_PERF_COCO").is_some() && buffs.reveals_shinies() {
        assert_eq!(egg_markers, 2, "egg search must project both authoritative eggs after respawn");
    }
    fs::write(capture.output.join("sense-projection.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "radar":buffs.reveals_mobs(), "eggSearch":buffs.reveals_shinies(),
        "enemyMarkers":mob_markers, "eggMarkers":egg_markers,
        "markers":format!("{:?}",hud.minimap.markers),
    })).unwrap()).unwrap();
}
