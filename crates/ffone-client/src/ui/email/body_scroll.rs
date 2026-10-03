//! Read mail at its normal font size, with a clipped viewport and draggable bar.
use super::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

#[derive(Component)]
struct Viewport;
#[derive(Component)]
struct Track;
#[derive(Component)]
struct Thumb;

pub(super) fn install(app: &mut App) {
    app.add_systems(
        Update,
        scroll.after(EmailUiSet::Bind).after(LocalizationSet::Apply),
    );
}

pub(super) fn spawn(
    parent: &mut ChildSpawnerCommands,
    rect: EmailUiRect,
    localized: LocalizedText,
    assets: &EmailUiAssets,
    style: EmailUiTextStyle,
    color: Color,
) {
    parent
        .spawn((rect.node(), Pickable::IGNORE, FocusPolicy::Pass))
        .with_children(|root| {
            root.spawn((
                Viewport,
                Interaction::None,
                ScrollPosition::default(),
                Node {
                    width: px(rect.width - 18.0),
                    height: percent(100),
                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
            ))
            .with_children(|body| {
                spawn_text_entity(
                    body,
                    Some(EmailUiTextRole::DetailBody),
                    localized,
                    assets,
                    style,
                    color,
                    true,
                    None,
                );
            });
            root.spawn((
                Track,
                Button,
                RelativeCursorPosition::default(),
                Node {
                    position_type: PositionType::Absolute,
                    right: px(0),
                    top: px(0),
                    width: px(14),
                    height: percent(100),
                    ..default()
                },
                sliced_image(
                    assets.scrollbar_track.clone(),
                    BorderRect {
                        min_inset: Vec2::new(2., 4.),
                        max_inset: Vec2::new(2., 4.),
                    },
                ),
            ))
            .with_children(|track| {
                track.spawn((
                    Thumb,
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                    Node {
                        position_type: PositionType::Absolute,
                        width: percent(100),
                        height: px(20),
                        ..default()
                    },
                    sliced_image(
                        assets.scrollbar_thumb.clone(),
                        BorderRect {
                            min_inset: Vec2::new(2., 4.),
                            max_inset: Vec2::new(2., 4.),
                        },
                    ),
                ));
            });
        });
}

fn scroll(
    model: Res<EmailUiModel>,
    wheel: Option<Res<AccumulatedMouseScroll>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    mut view: Query<(&ComputedNode, &Interaction, &mut ScrollPosition), With<Viewport>>,
    mut track: Query<
        (&mut Node, &Interaction, &RelativeCursorPosition),
        (With<Track>, Without<Thumb>),
    >,
    mut thumb: Query<&mut Node, (With<Thumb>, Without<Track>)>,
    texts: Query<(&EmailUiTextElement, &Text)>,
    mut previous: Local<String>,
    mut dragging: Local<bool>,
    mut selection: Local<Option<(EmailFolder, usize, i8, Option<usize>)>>,
) {
    let (Ok((computed, hover, mut position)), Ok((mut bar, pressed, cursor)), Ok(mut handle)) =
        (view.single_mut(), track.single_mut(), thumb.single_mut())
    else {
        return;
    };
    let text = texts
        .iter()
        .find(|(e, _)| e.role == EmailUiTextRole::DetailBody)
        .map(|(_, t)| t.0.as_str())
        .unwrap_or("");
    let selected = (
        model.folder,
        model.guide_page,
        model.player_page,
        model.selected_row,
    );
    if !model.visible || previous.as_str() != text || *selection != Some(selected) {
        *selection = Some(selected);
        if position.y != 0.0 {
            position.y = 0.0;
        }
        *dragging = false;
        *previous = text.to_owned();
    }
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
    if !model.visible || !model.input_enabled() {
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
    } else if *hover != Interaction::None || *pressed != Interaction::None {
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
mod tests;
