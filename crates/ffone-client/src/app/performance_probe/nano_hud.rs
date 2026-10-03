//! Focused tall-Nano HUD fixture in the full offline client.
use super::*;

pub(super) fn install(app: &mut App) {
    app.add_systems(
        Update,
        equip.before(super::super::tutorial_presentation::TutorialPresentationSet::GameplayHud),
    )
    .add_systems(Last, capture_sizes.before(super::measure));
    if env::var_os("FFONE_PERF_NANO_HUD_BASELINE").is_some() {
        app.add_systems(
            PostUpdate,
            restore_square_view.before(bevy::ui::UiSystems::Layout),
        );
    }
}

// A/B reproduction of the former square in the same executable and scene.
fn restore_square_view(
    portraits: Res<ffone_client::gameplay_ui::GameplayNanoPortraitImages>,
    mut images: ResMut<Assets<Image>>,
    mut nodes: Query<(&ImageNode, &mut Node)>,
    mut cameras: Query<(&Name, &mut Camera)>,
) {
    for handle in portraits.0.iter().flatten() {
        if let Some(mut image) = images.get_mut(handle) {
            if image.height() != 144 {
                image.resize(bevy::render::render_resource::Extent3d {
                    width: 144,
                    height: 144,
                    depth_or_array_layers: 1,
                });
            }
        }
        for (image, mut node) in &mut nodes {
            if image.image == *handle {
                node.top = px(12);
                node.height = px(72);
            }
        }
    }
    for (name, mut camera) in &mut cameras {
        if name.as_str().starts_with("Gameplay Nano portrait ") {
            camera.sub_camera_view = None;
        }
    }
}

fn capture_sizes(
    capture: Res<Capture>,
    mut windows: Query<&mut Window>,
    portraits: Res<ffone_client::gameplay_ui::GameplayNanoPortraitImages>,
    images: Res<Assets<Image>>,
    mut commands: Commands,
    mut previous: Local<usize>,
) {
    let frame = capture.samples.len();
    if frame == *previous {
        return;
    }
    *previous = frame;
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    match frame {
        1 => window.resolution.set_physical_resolution(1280, 720),
        151 => window.resolution.set_physical_resolution(1024, 768),
        301 => window.resolution.set_physical_resolution(1920, 1080),
        100 | 250 | 450 => {
            for handle in &portraits.0 {
                let image = images
                    .get(handle.as_ref().expect("HUD portrait target"))
                    .expect("loaded HUD portrait target");
                let height = if env::var_os("FFONE_PERF_NANO_HUD_BASELINE").is_some() {
                    144
                } else {
                    216
                };
                assert_eq!(image.size(), UVec2::new(144, height));
            }
            commands.spawn(Screenshot::primary_window()).observe(
                bevy::render::view::screenshot::save_to_disk(capture.output.join(format!(
                    "nano-hud-{}x{}.png",
                    window.physical_width(),
                    window.physical_height()
                ))),
            );
        }
        _ => {}
    }
}

fn equip(mut runtime: ResMut<RuntimeStatus>, content: Res<TutorialMissionContent>) {
    let ids: Vec<i32> = env::var("FFONE_PERF_NANO_HUD_IDS")
        .unwrap_or_else(|_| "24 42 43".into())
        .split_whitespace()
        .map(|id| id.parse().expect("Nano ID"))
        .collect();
    assert_eq!(ids.len(), 3);
    for (index, id) in ids.into_iter().enumerate() {
        let skill_id = content.journal_nano(id).expect("Nano table row").skills[0].skill_id;
        runtime.nano_slots[index] = super::super::runtime_status::RuntimeNanoSlot {
            nano_id: Some(id as i16),
            skill_id: skill_id as i16,
            stamina: 150,
            active: index == 0,
        };
    }
    runtime.nano_battery = 15;
    runtime.weapon_battery = 30;
}
