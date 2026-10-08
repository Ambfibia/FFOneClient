//! Axes use the server's XY plane: native -X / +Z, with native Y as height.
use super::*;

pub(super) fn axis(keys: &ButtonInput<KeyCode>) -> Option<(Vec3, Color)> {
    if keys.pressed(KeyCode::KeyZ) {
        Some((Vec3::Y, Color::srgb(0.2, 0.45, 1.)))
    } else if keys.pressed(KeyCode::KeyX) {
        Some((Vec3::NEG_X, Color::srgb(1., 0.2, 0.2)))
    } else if keys.pressed(KeyCode::KeyC) {
        Some((Vec3::Z, Color::srgb(0.2, 1., 0.35)))
    } else { None }
}

pub(super) fn gizmos(e: Res<WorldEditor>, state: Res<EditorState>, keys: Res<ButtonInput<KeyCode>>, mut gizmos: Gizmos) {
    if state.world_open != Some(true) || e.focus.is_some() || e.type_picker.is_some() { return; }
    let Some(p) = e.selected() else { return; };
    let Some((axis, color)) = axis(&keys) else { return; };
    let length = (e.distance * 0.12).clamp(3., 60.);
    gizmos.arrow(p.position, p.position + axis * length, color);
}

impl WorldEditor {
    pub(super) fn click_entity(&mut self, index: usize) -> bool {
        let key = self.entities[index].key.clone();
        let double = self.last_entity_click.as_ref().is_some_and(|(old, at)| *old == key && at.elapsed().as_millis() < 450);
        self.last_entity_click = Some((key, Instant::now()));
        self.select_entity(index);
        if double {
            self.center_selected();
            self.distance = 40.;
            self.zoom = self.zoom.max(2.);
            self.revision += 1;
        }
        double
    }
}
