//! Winit applies programmatic sizes in Last, after Bevy's normal camera update.
//! Refresh window camera targets before extraction uses the new swapchain size.
use super::*;
use bevy::{
    app::MainScheduleOrder, ecs::schedule::ScheduleLabel, render::camera::camera_system,
    window::WindowResized,
};

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct WindowRenderSync;
#[derive(Resource, Default)]
struct ResizePending(bool);

pub(super) fn install(app: &mut App) {
    app.init_resource::<ResizePending>()
        .init_schedule(WindowRenderSync);
    app.world_mut()
        .resource_mut::<MainScheduleOrder>()
        .insert_after(Last, WindowRenderSync);
    app.add_systems(
        WindowRenderSync,
        (
            publish_dimensions,
            camera_system.run_if(|pending: Res<ResizePending>| pending.0),
        )
            .chain(),
    );
}
fn publish_dimensions(
    windows: Query<(Entity, &Window)>,
    mut previous: Local<Vec<(Entity, UVec2, f32)>>,
    mut pending: ResMut<ResizePending>,
    mut resized: MessageWriter<WindowResized>,
) {
    let current: Vec<_> = windows
        .iter()
        .map(|(e, w)| (e, w.resolution.physical_size(), w.scale_factor()))
        .collect();
    let changed = *previous != current;
    if pending.0 != changed {
        pending.0 = changed;
    }
    if !changed {
        return;
    }
    for (entity, window) in &windows {
        resized.write(WindowResized {
            window: entity,
            width: window.width(),
            height: window.height(),
        });
    }
    *previous = current;
}
