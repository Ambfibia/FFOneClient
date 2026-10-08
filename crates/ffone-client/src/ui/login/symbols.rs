use super::*;

#[derive(Clone, Copy)]
pub(super) enum Icon { Pencil, Back, Refresh }

fn stroke(p: &mut ChildSpawnerCommands, x: f32, y: f32, w: f32, h: f32, angle: f32) {
    p.spawn((Node { position_type: PositionType::Absolute, left: px(x), top: px(y),
        width: px(w), height: px(h), ..default() }, BackgroundColor(Color::WHITE),
        UiTransform::from_rotation(Rot2::radians(angle)), bevy::ui::FocusPolicy::Pass));
}

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, icon: Icon) {
    parent.spawn((Node { width: px(18), height: px(18), flex_shrink: 0.0, ..default() },
        bevy::ui::FocusPolicy::Pass)).with_children(|p| match icon {
        Icon::Pencil => {
            stroke(p, 4.0, 7.0, 12.0, 4.0, -std::f32::consts::FRAC_PI_4);
            stroke(p, 2.5, 13.0, 3.0, 2.0, -std::f32::consts::FRAC_PI_4);
        }
        Icon::Back => {
            stroke(p, 4.0, 8.0, 11.0, 2.0, 0.0);
            stroke(p, 2.0, 5.5, 7.0, 2.0, -std::f32::consts::FRAC_PI_4);
            stroke(p, 2.0, 10.5, 7.0, 2.0, std::f32::consts::FRAC_PI_4);
        }
        Icon::Refresh => {
            for i in 0..9 {
                let angle = (i as f32 * 30.0 + 15.0).to_radians();
                stroke(p, 7.5 + 6.0*angle.cos(), 8.0 + 6.0*angle.sin(), 3.8, 2.0,
                    angle + std::f32::consts::FRAC_PI_2);
            }
            stroke(p, 12.0, 3.0, 5.0, 2.0, std::f32::consts::FRAC_PI_4);
            stroke(p, 14.0, 4.0, 2.0, 5.0, 0.0);
        }
    });
}
