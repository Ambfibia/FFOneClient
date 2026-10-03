use super::*;

pub(super) fn sync_tutorial_choreography_overlay(
    presentation: Res<TutorialChoreographyPresentation>,
    mut overlays: Query<(
        &TutorialChoreographyOverlay,
        &mut BackgroundColor,
        &mut Visibility,
    )>,
) {
    if !presentation.is_changed() {
        return;
    }
    for (marker, mut color, mut visibility) in &mut overlays {
        let alpha = match marker {
            TutorialChoreographyOverlay::Fade => presentation.overlay_alpha,
            TutorialChoreographyOverlay::TopBar | TutorialChoreographyOverlay::BottomBar => {
                presentation.cinematic_alpha
            }
        }
        .clamp(0.0, 1.0);
        color.0 = Color::srgba(0.0, 0.0, 0.0, alpha);
        *visibility = if alpha > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn sync_tutorial_pan_strip(
    time: Res<Time>,
    mut presentation: ResMut<TutorialChoreographyPresentation>,
    assets: Res<TutorialPanAssets>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut frames: Query<(
        &TutorialPanFrame,
        &mut Node,
        &mut ImageNode,
        &mut Visibility,
    )>,
) {
    // Handles are intentionally retained after preload. `pan_loaded` mirrors
    // the legacy coroutine's logical ownership, not permission to evict and
    // reload the package while the tutorial is running.
    if presentation.pan_active {
        presentation.pan_elapsed_seconds += time.delta_secs().max(0.0);
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let height = window.height();
    let width = window.width();
    // Exact `PanScene` OnGUI:
    // t=(Time.time-fPanStartTime)/23; offset=(height*8-width)*t.
    let offset = (height * 8.0 - width) * (presentation.pan_elapsed_seconds / 23.0);
    for (marker, mut node, mut image, mut visibility) in &mut frames {
        let Some(handles) = assets.handles.as_ref() else {
            image.image = Handle::default();
            *visibility = Visibility::Hidden;
            continue;
        };
        image.image = handles[marker.0].clone();
        node.left = Val::Px(-offset + marker.0 as f32 * height);
        node.top = Val::Px(0.0);
        node.width = Val::Px(height);
        node.height = Val::Px(height);
        *visibility = if presentation.pan_active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}
