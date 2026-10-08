//! Fixed account list with the inventory skin and a clipped, draggable viewport.
use super::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};
use bevy::input::mouse::AccumulatedMouseScroll;
#[derive(Component)]
pub(super) struct Track;
#[derive(Component)]
pub(super) struct Thumb;

fn image(handle: Handle<Image>) -> ImageNode {
    ImageNode { image: handle, visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer { border: BorderRect {
            min_inset: Vec2::new(2., 4.), max_inset: Vec2::new(2., 4.) },
            center_scale_mode: SliceScaleMode::Stretch, sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0 }), ..default() }
}

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, assets: &LoginUiAssets,
    rows: impl FnOnce(&mut ChildSpawnerCommands)) {
    parent.spawn((account_view::AccountMenu, RelativeCursorPosition::default(), Interaction::None,
        ScrollPosition::default(), Node { width: px(222), height: percent(100),
            overflow: Overflow::scroll_y(), flex_direction: FlexDirection::Column,
            row_gap: px(3), ..default() })).with_children(rows);
    parent.spawn((Track, Button, RelativeCursorPosition::default(), Node {
        position_type: PositionType::Absolute, right: px(3), top: px(3),
        width: px(14), height: px(79), ..default() }, image(assets.scroll_track.clone())))
        .with_children(|p| { p.spawn((Thumb, FocusPolicy::Pass, Node {
            position_type: PositionType::Absolute, width: percent(100), height: px(20), ..default()
        }, image(assets.scroll_thumb.clone()))); });
}

pub(super) fn scroll(
    model: Res<LoginUiModel>, options: Res<OptionUiModel>,
    wheel: Option<Res<AccumulatedMouseScroll>>, mouse: Option<Res<ButtonInput<MouseButton>>>,
    mut view: Query<(&ComputedNode, &RelativeCursorPosition, &mut ScrollPosition), With<account_view::AccountMenu>>,
    mut track: Query<(&mut Node, &Interaction, &RelativeCursorPosition), (With<Track>, Without<Thumb>)>,
    mut thumb: Query<&mut Node, (With<Thumb>, Without<Track>)>,
    mut dragging: Local<bool>,
) {
    let (Ok((computed, cursor_over, mut position)), Ok((mut bar, pressed, cursor)), Ok(mut handle)) =
        (view.single_mut(), track.single_mut(), thumb.single_mut())
    else {
        return;
    };
    let height = computed.size().y * computed.inverse_scale_factor();
    let maximum = ((computed.content_size().y - computed.size().y)
        * computed.inverse_scale_factor())
    .max(0.0);
    let display = if maximum > 0.0 {
        Display::Flex
    } else {
        Display::None
    };
    if bar.display != display {
        bar.display = display;
    }
    if !model.accepts_manual_input() || options.visible {
        return;
    }
    let size = (height * height / (height + maximum).max(1.0)).clamp(20.0_f32.min(height), height);
    let travel = height - size;
    if mouse.as_ref().is_none_or(|m| !m.pressed(MouseButton::Left)) {
        *dragging = false;
    }
    if *pressed == Interaction::Pressed {
        *dragging = true;
    }
    let mut next = position.y;
    if *dragging && travel > 0.0 {
        if let Some(point) = cursor.normalized {
            next = ((point.y + 0.5) * height - size * 0.5) / travel * maximum;
        }
    } else if cursor_over.normalized.is_some_and(|p| p.x.abs() <= 0.5 && p.y.abs() <= 0.5) || *pressed != Interaction::None {
        next -= wheel.as_ref().map_or(0.0, |wheel| {
            wheel.delta.y
                * match wheel.unit {
                    bevy::input::mouse::MouseScrollUnit::Line => 30.0,
                    bevy::input::mouse::MouseScrollUnit::Pixel => 1.0,
                }
        });
    }
    next = next.clamp(0.0, maximum);
    if position.y != next {
        position.y = next;
    }
    let thumb_height = px(size);
    let thumb_top = px(if maximum > 0.0 {
        next / maximum * travel
    } else {
        0.0
    });
    if handle.height != thumb_height {
        handle.height = thumb_height;
    }
    if handle.top != thumb_top {
        handle.top = thumb_top;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wheel_and_drag_clamp_accounts_and_hide_bar_for_short_list() {
        let mut app = App::new();
        app.insert_resource(LoginUiModel { visible: true, ..default() })
            .init_resource::<OptionUiModel>().init_resource::<AccumulatedMouseScroll>()
            .init_resource::<ButtonInput<MouseButton>>().add_systems(Update, scroll);
        let view = app.world_mut().spawn((account_view::AccountMenu, ComputedNode {
            size: Vec2::new(222., 79.), content_size: Vec2::new(222., 277.),
            inverse_scale_factor: 1., ..default() },
            RelativeCursorPosition { normalized: Some(Vec2::ZERO), ..default() }, ScrollPosition::default())).id();
        let track = app.world_mut().spawn((Track, Node::default(), Interaction::None,
            RelativeCursorPosition::default())).id();
        app.world_mut().spawn((Thumb, Node::default()));
        app.world_mut().resource_mut::<AccumulatedMouseScroll>().delta.y = -2.;
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(view).unwrap().y, 60.);
        app.world_mut().resource_mut::<AccumulatedMouseScroll>().delta = Vec2::ZERO;
        app.world_mut().resource_mut::<ButtonInput<MouseButton>>().press(MouseButton::Left);
        *app.world_mut().get_mut::<Interaction>(track).unwrap() = Interaction::Pressed;
        app.world_mut().get_mut::<RelativeCursorPosition>(track).unwrap().normalized = Some(Vec2::new(0., 0.5));
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(view).unwrap().y, 198.);
        app.world_mut().get_mut::<ComputedNode>(view).unwrap().content_size.y = 50.;
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(view).unwrap().y, 0.);
        assert_eq!(app.world().get::<Node>(track).unwrap().display, Display::None);
    }
}
