//! Exercises actual swapchain resizing without writing user preferences.
use super::*;

pub(super) fn install(app: &mut App) {
    app.init_resource::<SelectionVerified>()
        .add_systems(Update, drive)
        .add_systems(Last, record_selection);
}
#[derive(Resource, Default)]
pub(super) struct SelectionVerified(pub bool);
fn drive(
    mut windows: Query<&mut Window>,
    capture: Res<Capture>,
    mut frame: Local<u32>,
    mut records: Local<Vec<serde_json::Value>>,
    mut commands: Commands,
) {
    if capture.ready.is_none() || capture.samples.is_empty() {
        return;
    }
    *frame += 1;
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    let step = match *frame {
        30 => Some((1280, 720, true)),
        110 => Some((1600, 900, true)),
        190 => Some((1280, 720, false)),
        270 => Some((1920, 1080, false)),
        350 => Some((1024, 768, true)),
        430 => Some((1920, 1080, true)),
        _ => None,
    };
    if let Some((width, height, windowed)) = step {
        info!("Resolution probe: {width}x{height}, windowed={windowed}");
        window.resolution.set_physical_resolution(width, height);
        window.mode = if windowed {
            WindowMode::Windowed
        } else {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        };
    }
    if matches!(*frame, 90 | 170 | 250 | 330 | 410 | 490) {
        records.push(serde_json::json!({"frame":*frame,"physical":[window.physical_width(),window.physical_height()],
            "logical":[window.width(),window.height()],"mode":format!("{:?}",window.mode)}));
        commands.spawn(Screenshot::primary_window()).observe(
            bevy::render::view::screenshot::save_to_disk(
                capture.output.join(format!("resolution-{}.png", *frame)),
            ),
        );
        fs::write(
            capture.output.join("resolution.json"),
            serde_json::to_vec_pretty(&*records).unwrap(),
        )
        .unwrap();
    }
}

fn record_selection(
    capture: Res<Capture>,
    state: Res<State<ClientState>>,
    windows: Query<&Window>,
    images: Query<(
        &ImageNode,
        &ComputedNode,
        &UiGlobalTransform,
        Option<&InheritedVisibility>,
    )>,
    assets: Res<AssetServer>,
    mut saved: ResMut<SelectionVerified>,
    mut commands: Commands,
    mut entered: Local<Option<Instant>>,
) {
    if saved.0 || *state.get() != ClientState::CharacterSelect || capture.selection_since.is_none()
    {
        return;
    }
    // The offline fixture enters selection for the first time on return. Its
    // deferred tree is constructed in the following state transition, and
    // asset prewarming can block that first frame for several seconds.
    let elapsed = entered.get_or_insert_with(Instant::now).elapsed();
    if elapsed < Duration::from_secs(1) {
        return;
    }
    let window = windows.single().unwrap();
    let matching: Vec<_> = images
        .iter()
        .filter(|(image, _, _, _)| {
            assets.get_path(image.image.id()).is_some_and(|p| {
                let path = p.path().to_string_lossy();
                path.contains("FullscreenModeButton")
                    || path.contains("WindowModeButton")
                    || path.contains("CSFullScreenButton")
                    || path.contains("CSFullToWindow")
            })
        })
        .collect();
    fs::write(capture.output_root.join("selection-diagnostic.json"), serde_json::to_vec_pretty(&matching.iter().map(|(image,node,transform,visible)|serde_json::json!({"path":assets.get_path(image.image.id()).map(|p|p.to_string()),"visibility":visible.map(|v|v.get()),"size":node.size().to_array(),"center":transform.translation.to_array()})).collect::<Vec<_>>()).unwrap()).unwrap();
    let candidates: Vec<_> = matching
        .into_iter()
        .filter(|(_, _, _, visible)| visible.is_none_or(|v| v.get()))
        .collect();
    if candidates.is_empty() && elapsed < Duration::from_secs(60) {
        return;
    }
    assert_eq!(
        candidates.len(),
        1,
        "fullscreen/windowed button missing after resolution change and reentry"
    );
    let (_, node, transform, _) = candidates[0];
    let center = transform.translation;
    let half = node.size() * 0.5;
    let size = Vec2::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
    );
    assert!(
        node.size().min_element() > 0.0
            && (center - half).min_element() >= 0.0
            && (center + half).cmple(size).all(),
        "fullscreen/windowed button outside window: center={center:?} half={half:?} window={size:?}"
    );
    fs::write(capture.output_root.join("selection-button.json"),serde_json::to_vec_pretty(&serde_json::json!({
        "visible":true,"center":center.to_array(),"size":node.size().to_array(),"window":size.to_array()})).unwrap()).unwrap();
    commands.spawn(Screenshot::primary_window()).observe(
        bevy::render::view::screenshot::save_to_disk(
            capture.output_root.join("selection-after-resize.png"),
        ),
    );
    saved.0 = true;
}
