//! Canvas gestures use stable task IDs, independently of rebuilt UI entities.
use super::*;
use mission_canvas::Hit;
use mission_workspace::Context;

#[derive(Default)]
pub(super) struct Gesture {
    start: Option<Vec2>,
    last: Vec2,
    hit: Option<Hit>,
    positions: Vec<(String, Vec2)>,
    hover: Option<String>,
    hover_since: f64,
    tooltip: bool,
    layout_before: Option<BTreeMap<String, [f32; 2]>>,
    right: Option<RightPan>,
    objective_click:Option<(usize,f64)>,
}

struct RightPan {
    origin: Vec2,
    last: Vec2,
    dragged: bool,
    context: Option<Context>,
}
impl RightPan {
    fn new(origin: Vec2, context: Option<Context>) -> Self {
        Self { origin, last: origin, dragged: false, context }
    }
    fn advance(&mut self, cursor: Vec2) -> Vec2 {
        if !self.dragged && cursor.distance(self.origin) < 5. { return Vec2::ZERO; }
        self.dragged = true;
        let delta = cursor - self.last;
        self.last = cursor;
        delta
    }
}

fn open_context(editor: &mut XdtEditor, at: Vec2, context: Option<Context>) {
    match &context {
        Some(Context::Node(row)) => {
            editor.row = Some(*row);
            editor.workspace.selected = BTreeSet::from([*row]);
        }
        Some(Context::Edge(row, field, slot)) => {
            editor.workspace.selected_edge = Some((*row, field.clone(), *slot));
        }
        _ => {}
    }
    editor.workspace.context = context.map(|kind| (at, kind));
    editor.revision += 1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_drag_moves_the_entire_view_and_never_turns_back_into_a_click() {
        let mut pan = RightPan::new(Vec2::new(100., 100.), Some(Context::Canvas));
        assert_eq!(pan.advance(Vec2::new(102., 101.)), Vec2::ZERO);
        assert!(!pan.dragged);
        assert_eq!(pan.advance(Vec2::new(140., 120.)), Vec2::new(40., 20.));
        assert_eq!(pan.advance(Vec2::new(150., 110.)), Vec2::new(10., -10.));
        assert_eq!(pan.advance(Vec2::new(100., 100.)), Vec2::new(-50., -10.));
        assert!(pan.dragged, "Returning to the origin must not open a context menu");
    }

    #[test]
    fn short_right_click_keeps_its_original_context_and_position() {
        let mut pan = RightPan::new(Vec2::new(100., 100.), Some(Context::Node(7)));
        assert_eq!(pan.advance(Vec2::new(102., 102.)), Vec2::ZERO);
        assert!(!pan.dragged);
        assert!(matches!(pan.context, Some(Context::Node(7))));
        assert_eq!(pan.origin, Vec2::new(100., 100.));
    }
}
fn priority(hit: &Hit) -> u8 {
    match hit {
        Hit::Canvas => 0,
        Hit::Node(..) | Hit::Neighbor(..) => 1,
        Hit::Field(_) | Hit::Preview(..) | Hit::Objective(..) => 2,
        Hit::Edge(..) => 3,
        Hit::Port(..) => 4,
        _ => 5,
    }
}

fn contains(size: Vec2, transform: &UiGlobalTransform, point: Vec2) -> bool {
    transform.try_inverse().is_some_and(|inverse| {
        Rect::from_center_size(Vec2::ZERO, size).contains(inverse.transform_point2(point))
    })
}

pub(super) fn pointer(
    state: Res<EditorState>,
    mut editor: ResMut<XdtEditor>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    time: Res<Time>,
    hits: Query<(&Hit, &ComputedNode, &UiGlobalTransform)>,
    menus: Query<(&mission_context::Panel, &ComputedNode, &UiGlobalTransform)>,
    mut wheels: MessageReader<MouseWheel>,
    mut gesture: Local<Gesture>,
    mut window_events: MessageReader<bevy::window::WindowEvent>,
) {
    if !state.xdt_open || editor.mission_workspace_table().is_none() {
        gesture.right = None;
        wheels.clear();
        window_events.clear();
        return;
    }
    let cursor = window.cursor_position().unwrap_or(gesture.last);
    let mut event_cursor = gesture.last;
    let mut press_cursor = None;
    let mut release_cursor = None;
    for event in window_events.read() {
        match event {
            bevy::window::WindowEvent::CursorMoved(event) => event_cursor = event.position,
            bevy::window::WindowEvent::MouseButtonInput(event)
                if event.state == ButtonState::Pressed =>
            {
                press_cursor = Some(event_cursor)
            }
            bevy::window::WindowEvent::MouseButtonInput(event)
                if event.state == ButtonState::Released =>
            {
                release_cursor = Some(event_cursor)
            }
            _ => {}
        }
    }
    let cursor =
        if mouse.just_released(MouseButton::Left) || mouse.just_released(MouseButton::Middle) || mouse.just_released(MouseButton::Right) {
            release_cursor.unwrap_or(cursor)
        } else {
            cursor
        };
    let scale = window.resolution.scale_factor();
    let rect = |node: &ComputedNode, transform: &UiGlobalTransform| {
        Rect::from_center_size(
            Vec2::from(transform.translation) / scale,
            node.size() / scale,
        )
    };
    let target = hits
        .iter()
        .filter(|(_, n, t)| contains(n.size(), t, cursor * scale))
        .max_by_key(|(h, _, _)| priority(h))
        .map(|(h, n, t)| (h.clone(), rect(n, t)));
    let canvas = hits
        .iter()
        .find(|(h, _, _)| matches!(h, Hit::Canvas))
        .map(|(_, n, t)| rect(n, t));
    let over_canvas = canvas.is_some_and(|r| r.contains(cursor));
    let menu = menus.iter().find(|(panel, _, _)| panel.interactive);
    // Button actions run first. Still consume this press if an action just closed
    // the menu, so it cannot also select or drag the canvas beneath the old panel.
    if mouse.just_pressed(MouseButton::Left) && menu.is_some() {
        let (_, node, transform) = menu.unwrap();
        if !contains(
            node.size(),
            transform,
            press_cursor.unwrap_or(cursor) * scale,
        ) {
            editor.dismiss_mission_context();
        }
        gesture.start = None;
        gesture.hit = None;
        gesture.positions.clear();
        gesture.layout_before = None;
        gesture.last = cursor;
        wheels.clear();
        return;
    }
    if let (Some(fit), Some(canvas)) = (
        editor.workspace.view_request,
        canvas.filter(|c| c.height() > 1. && c.width() > 1.),
    ) {
        editor.workspace.view_request = None;
        let scene = mission_canvas::scene(&editor);
        let selected = editor.mission_row();
        let nodes: Vec<_> = scene
            .nodes
            .iter()
            .filter(|n| fit || n.row == selected)
            .collect();
        if !nodes.is_empty() {
            let min = nodes
                .iter()
                .fold(Vec2::splat(f32::INFINITY), |v, n| v.min(n.position));
            let max = nodes.iter().fold(Vec2::splat(f32::NEG_INFINITY), |v, n| {
                v.max(n.position + n.size())
            });
            if fit {
                editor.graph_zoom = ((canvas.size() - Vec2::splat(48.)) / (max - min))
                    .min_element()
                    .clamp(0.5, 1.0);
            }
            editor.workspace.pan = canvas.size() * 0.5 - (min + max) * 0.5 * editor.graph_zoom;
            // Keep the header reachable when the minimum zoom cannot fit tall previews.
            if (max.y - min.y) * editor.graph_zoom > canvas.height() - 48. {
                editor.workspace.pan.y = 24. - min.y * editor.graph_zoom;
            }
            editor.workspace.canvas_revision += 1;
        }
    }
    if mouse.just_pressed(MouseButton::Left)
        && std::env::var_os("FFONE_MISSION_POINTER_TRACE").is_some()
    {
        eprintln!(
            "mission pointer cursor={cursor:?} scale={scale} canvas={canvas:?} target={:?}",
            target.as_ref().map(|(h, r)| (priority(h), *r))
        );
    }
    let hovered = if let Some((Hit::Field(field), _)) = &target {
        Some(field.clone())
    } else {
        None
    };
    if gesture.hover != hovered || (cursor - gesture.last).length() > 2. {
        if gesture.tooltip {
            editor.workspace.help = None;
            editor.revision += 1;
            gesture.tooltip = false;
        }
        gesture.hover = hovered;
        gesture.hover_since = time.elapsed_secs_f64();
    } else if gesture.hover.is_some()
        && !gesture.tooltip
        && time.elapsed_secs_f64() - gesture.hover_since >= 1.5
        && !mouse.pressed(MouseButton::Left)
        && !mouse.pressed(MouseButton::Right)
        && editor.workspace.context.is_none()
    {
        editor.workspace.help = gesture.hover.clone();
        editor.revision += 1;
        gesture.tooltip = true;
    }
    if keys.just_pressed(KeyCode::F1) {
        editor.workspace.help = editor.picker_field.clone().or(gesture.hover.clone());
        editor.revision += 1;
        gesture.tooltip = false;
    }
    for wheel in wheels.read() {
        if over_canvas && editor.workspace.context.is_none() {
            let delta = match wheel.unit {
                MouseScrollUnit::Line => wheel.y * 0.08,
                MouseScrollUnit::Pixel => wheel.y * 0.002,
            };
            let old = editor.graph_zoom;
            editor.graph_zoom = (old + delta).clamp(0.5, 1.5);
            let local = cursor - canvas.unwrap().min;
            editor.workspace.pan =
                local - (local - editor.workspace.pan) * (editor.graph_zoom / old);
            editor.workspace.canvas_revision += 1;
        }
    }
    if editor.workspace.quick.is_some() {
        gesture.right = None;
        gesture.last = cursor;
        return;
    }
    if keys.just_pressed(KeyCode::Escape) { gesture.right = None; }
    if mouse.just_pressed(MouseButton::Right) {
        if !editor.apply() {
            return;
        }
        editor.workspace.help = None;
        editor.delete_confirm = false;
        gesture.tooltip = false;
        gesture.start = None;
        gesture.hit = None;
        let at = press_cursor.unwrap_or(cursor);
        let under_menu = menu.is_some_and(|(_, n, t)| contains(n.size(), t, at * scale));
        let hit = hits
            .iter()
            .filter(|(_, n, t)| contains(n.size(), t, at * scale))
            .max_by_key(|(h, _, _)| priority(h))
            .map(|(h, _, _)| h);
        let previous = editor
            .workspace
            .context
            .as_ref()
            .map(|(_, kind)| kind.clone());
        let context = if under_menu {
            previous
        } else {
            match hit {
                Some(Hit::Neighbor(row)) => Some(Context::Mission(*row)),
                Some(Hit::Node(row)) if !editor.graph_stages => Some(Context::Mission(*row)),
                Some(Hit::Node(row)) => Some(Context::Node(*row)),
                Some(Hit::Edge(row, field, slot)) => {
                    Some(Context::Edge(*row, field.clone(), *slot))
                }
                Some(Hit::Field(field)) => Some(Context::Field(field.clone())),
                Some(Hit::Preview(row)) => Some(Context::Node(*row)),
                Some(Hit::Canvas) => Some(Context::Canvas),
                _ => previous,
            }
        };
        if canvas.is_some_and(|r| r.contains(at)) && !under_menu {
            if editor.workspace.context.take().is_some() { editor.revision += 1; }
            gesture.right = Some(RightPan::new(at, context));
        } else {
            gesture.right = None;
            open_context(&mut editor, at, context);
        }
    }
    if let Some(mut pan) = gesture.right.take() {
        let released = mouse.just_released(MouseButton::Right);
        if mouse.pressed(MouseButton::Right) || released {
            let delta = pan.advance(cursor);
            if delta != Vec2::ZERO {
                editor.workspace.pan += delta;
                editor.workspace.canvas_revision += 1;
            }
            if released {
                if !pan.dragged { open_context(&mut editor, pan.origin, pan.context); }
            } else {
                gesture.right = Some(pan);
            }
        }
        gesture.last = cursor;
        return;
    }
    if editor.workspace.context.is_some() {
        gesture.last = cursor;
        return;
    }
    if mouse.just_pressed(MouseButton::Middle) || mouse.just_pressed(MouseButton::Left) {
        let cursor = press_cursor.unwrap_or(cursor);
        let target = hits
            .iter()
            .filter(|(_, n, t)| contains(n.size(), t, cursor * scale))
            .max_by_key(|(h, _, _)| priority(h))
            .map(|(h, n, t)| (h.clone(), rect(n, t)));
        gesture.last = cursor;
        let pan = mouse.just_pressed(MouseButton::Middle) || keys.pressed(KeyCode::Space);
        if pan && over_canvas {
            gesture.hit = Some(Hit::Canvas);
            gesture.start = Some(cursor);
            gesture.positions.clear();
        } else if let Some((hit, box_)) = target.clone() {
            match &hit {
                Hit::Node(row) => {
                    if !editor.apply() {
                        return;
                    }
                    if let Some((source, field)) = editor.workspace.pending_link.clone() {
                        match editor.connect_canvas(source, &field, *row) {
                            Ok(()) => editor.workspace.pending_link = None,
                            Err(error) => editor.status = error,
                        }
                    }
                    let shift =
                        keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
                    if shift {
                        if !editor.workspace.selected.remove(row) {
                            editor.workspace.selected.insert(*row);
                        }
                    } else if !editor.workspace.selected.contains(row) {
                        editor.workspace.selected = BTreeSet::from([*row]);
                    }
                    editor.row = Some(*row);
                    editor.picker_field = None;
                    editor.rebuild_links();
                    if cursor.y - box_.min.y < 44. * editor.graph_zoom {
                        gesture.layout_before = Some(editor.workspace.positions.clone());
                        gesture.positions = mission_canvas::scene(&editor)
                            .nodes
                            .into_iter()
                            .filter(|n| {
                                n.row
                                    .is_some_and(|r| editor.workspace.selected.contains(&r))
                            })
                            .map(|n| (n.layout_key(&editor), n.position))
                            .collect();
                        gesture.hit = Some(hit);
                        gesture.start = Some(cursor);
                    }
                    editor.revision += 1;
                }
                Hit::Objective(row) => {
                    let now=time.elapsed_secs_f64();
                    if gesture.objective_click.is_some_and(|(previous,at)|previous==*row && now-at<=0.45) {
                        editor.row=Some(*row);editor.array_slot=None;
                        if let Err(error)=editor.start_mission_text_edit("m_iHCurrentObjective",true){editor.status=error;}
                        gesture.objective_click=None;
                    } else {gesture.objective_click=Some((*row,now));editor.row=Some(*row);editor.workspace.selected=BTreeSet::from([*row]);editor.revision+=1;}
                }
                Hit::Neighbor(row) => {
                    if !editor.apply() {
                        return;
                    }
                    if let Some((source, field)) = editor.workspace.pending_link.clone() {
                        match editor.connect_canvas(source, &field, *row) {
                            Ok(()) => editor.workspace.pending_link = None,
                            Err(error) => editor.status = error,
                        }
                    } else {
                        editor.row = Some(*row);
                        editor.workspace.selected = BTreeSet::from([*row]);
                        editor.workspace.view_request = Some(true);
                        editor.rebuild_links();
                    }
                    editor.revision += 1;
                }
                Hit::Port(row, field) => {
                    gesture.hit = Some(hit.clone());
                    gesture.start = Some(cursor);
                    editor.workspace.pending_link = Some((*row, field.clone()));
                    editor.revision += 1;
                }
                Hit::Edge(row, field, slot) => {
                    editor.workspace.selected_edge = Some((*row, field.clone(), *slot));
                    gesture.hit = Some(hit);
                    gesture.start = Some(cursor);
                    editor.revision += 1;
                }
                Hit::CatalogDivider | Hit::InspectorDivider => {
                    gesture.hit = Some(hit);
                    gesture.start = Some(cursor);
                }
                Hit::Canvas => {
                    gesture.hit = Some(hit);
                    gesture.start = Some(cursor);
                    editor.workspace.selected.clear();
                }
                _ => {}
            }
        }
    }
    if let Some(start) = gesture.start {
        let held = mouse.pressed(MouseButton::Left) || mouse.pressed(MouseButton::Middle);
        let delta = cursor - gesture.last;
        if (held
            || mouse.just_released(MouseButton::Left)
            || mouse.just_released(MouseButton::Middle))
            && delta != Vec2::ZERO
        {
            match gesture.hit.clone() {
                Some(Hit::Node(..)) => {
                    let offset = (cursor - start) / editor.graph_zoom;
                    for (key, at) in &gesture.positions {
                        editor
                            .workspace
                            .positions
                            .insert(key.clone(), (*at + offset).to_array());
                    }
                    editor.workspace.layout_dirty = true;
                    editor.workspace.canvas_revision += 1;
                }
                Some(Hit::Canvas) => {
                    if mouse.pressed(MouseButton::Middle) || keys.pressed(KeyCode::Space) {
                        editor.workspace.pan += delta;
                    } else {
                        let selection = Rect::from_corners(start, cursor);
                        if let Some(canvas) = canvas {
                            editor.workspace.marquee =
                                Some((start - canvas.min, cursor - canvas.min));
                        }
                        editor.workspace.selected = hits
                            .iter()
                            .filter_map(|(hit, node, transform)| {
                                if let Hit::Node(row) = hit {
                                    let b = rect(node, transform);
                                    selection
                                        .intersect(b)
                                        .is_empty()
                                        .then_some(None)
                                        .unwrap_or(Some(*row))
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }
                    editor.workspace.canvas_revision += 1;
                }
                Some(Hit::CatalogDivider) => {
                    editor.workspace.catalog_width = (editor.workspace.catalog_width + delta.x)
                        .clamp(180., window.width() * 0.28);
                    editor.workspace.layout_dirty = true;
                    editor.revision += 1;
                }
                Some(Hit::InspectorDivider) => {
                    editor.workspace.inspector_width = (editor.workspace.inspector_width - delta.x)
                        .clamp(300., window.width() * 0.45);
                    editor.workspace.layout_dirty = true;
                    editor.revision += 1;
                }
                _ => {}
            }
        }
        if !held {
            if (cursor - start).length() > 5. {
                let connection = match gesture.hit.clone() {
                    Some(Hit::Port(row, field)) => Some((row, field, None)),
                    Some(Hit::Edge(row, field, slot)) => Some((row, field, Some(slot))),
                    _ => None,
                };
                if let Some((row, field, slot)) = connection {
                    let node = hits.iter().find_map(|(hit, n, t)| match hit {
                        Hit::Node(target) | Hit::Neighbor(target)
                            if rect(n, t).contains(cursor) =>
                        {
                            Some(*target)
                        }
                        _ => None,
                    });
                    if let Some(target) = node {
                        let result = if let Some(slot) = slot {
                            editor.retarget_canvas_edge(row, &field, slot, target)
                        } else {
                            editor.connect_canvas(row, &field, target)
                        };
                        match result {
                            Ok(()) => {
                                editor.workspace.pending_link = None;
                                editor.workspace.selected_edge = None;
                            }
                            Err(error) => editor.status = error,
                        }
                    } else if over_canvas && !(field == mission_graph::REQUIRE && slot.is_some()) {
                        editor.workspace.pending_link = Some((row, field));
                        editor.workspace.context = Some((cursor, Context::Canvas));
                    }
                    editor.revision += 1;
                }
            }
            if let Some(before) = gesture.layout_before.take() {
                if before != editor.workspace.positions {
                    let after = serde_json::to_value(&editor.workspace.positions).unwrap();
                    editor.undo.push(Change {
                        pointer: "@mission-layout".into(),
                        before: serde_json::to_value(before).unwrap(),
                        after,
                    });
                    editor.redo.clear();
                }
            }
            if editor.workspace.marquee.take().is_some() {
                editor.revision += 1;
            }
            if editor.workspace.layout_dirty {
                if let Err(error) = editor.save_layout() {
                    editor.status = error;
                }
            }
            gesture.start = None;
            gesture.hit = None;
            gesture.positions.clear();
        }
    }
    if editor.focus.is_none() && editor.draft.is_none() {
        if keys.just_pressed(KeyCode::Delete) {
            editor.delete_confirm = true;
            editor.workspace.context = Some((
                Vec2::new(280., 160.),
                Context::Node(editor.row.unwrap_or(0)),
            ));
            editor.revision += 1;
        }
    }
    gesture.last = cursor;
}
