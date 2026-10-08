use super::*;
use ffone_client::coordinates::ProtocolYawDegrees;

pub(super) struct Drag {
    index: usize,
    offset: Vec3,
    raw_position: Vec3,
    undo_start: usize,
    rotating: bool,
}
fn finish_drag(e: &mut WorldEditor) {
    let Some(drag) = e.drag.take() else {
        return;
    };
    if drag.undo_start >= e.undo.len() {
        return;
    }
    let changes: Vec<_> = e.undo.drain(drag.undo_start..).collect();
    let mut patches: BTreeMap<(usize, String), model::Patch> = BTreeMap::new();
    for change in changes {
        for p in change.patches {
            patches
                .entry((p.source, p.pointer.clone()))
                .and_modify(|old| old.after = p.after.clone())
                .or_insert(p);
        }
    }
    let patches: Vec<_> = patches
        .into_values()
        .filter(|p| p.before != p.after)
        .collect();
    if !patches.is_empty() {
        e.undo.push(model::Edit {
            patches,
            selection: e.entities.get(drag.index).map(|p| p.key.clone()),
        });
    }
}
#[path = "input_fields.rs"]
mod fields;
use fields::{apply, begin};

pub(super) fn buttons(
    mut e: ResMut<WorldEditor>,
    state: Res<EditorState>,
    mut xdt: ResMut<xdt::XdtEditor>,
    l: Res<Localization>,
    lang: Res<Language>,
    catalog: Res<EditorCatalog>,
    mouse: Res<ButtonInput<MouseButton>>,
    interactions: Query<(&Interaction, &Action), Changed<Interaction>>,
    window: Single<&Window>,
    mut icons_dirty: Local<bool>,
    lists:Query<&ComputedNode,With<ListViewport>>,
    preview:Res<preview::WorldPreview>,
    mesh_bounds:Query<(Entity,&bevy::camera::primitives::Aabb,&GlobalTransform)>,
    parents:Query<&ChildOf>,
) {
    *icons_dirty |= xdt.is_changed();
    if state.world_open.is_none() {
        return;
    }
    if e.npc_map_icons.is_empty() || *icons_dirty {
        let icons = xdt.npc_map_icons();
        if e.npc_map_icons != icons {
            e.npc_map_icons = icons;
            e.revision += 1;
        }
        let object_npc_types = xdt.object_npc_types();
        if e.object_npc_types != object_npc_types {
            e.object_npc_types = object_npc_types;
            e.revision += 1;
            e.geometry_revision += 1;
        }
        *icons_dirty = false;
    }
    if e.sources.is_empty() {
        if let Some(folder) = xdt.server_folder() {
            if let Err(error) = e.load_server(&folder) {
                e.error(error);
            }
        }
    }
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        e.ignore_pointer_press=true;
        finish_drag(&mut e);
        if let Err(error) = e.finish_brush() {
            e.error(error);
            continue;
        }
        let result = match *action {
            Action::SquareSettings => e.open_square_settings(),
            Action::SquareSkybox(mode) => e.change_square("skybox",Value::from(["","past","future"][mode as usize])),
            Action::SquareShader(mode) => e.change_square("terrainShader",if mode==3{Value::Null}else{Value::from(mode)}),
            Action::SquareChoices(mode) => e.open_square_choices(mode),
            Action::SquareChoice(index) => e.choose_square_value(index),
            Action::SquarePage(next) => {if let Some((mode,page))=e.square_choices {let last=e.square_values(mode).len().saturating_sub(1)/8;e.square_choices=Some((mode,if next{(page+1).min(last)}else{page.saturating_sub(1)}));}e.revision+=1;Ok(())},
            Action::CreateTerrain => { e.creating_terrain = !e.creating_terrain; e.revision += 1; Ok(()) },
            Action::ShowCollisions => { e.show_collisions = !e.show_collisions; e.revision += 1; e.geometry_revision += 1; Ok(()) },
            Action::ToggleCollision => e.toggle_object_collision(),
            Action::Select(i) => {
                if e.click_entity(i){picking::focus(&mut e,&preview,&mesh_bounds,&parents);}else{e.center_selected();}
                if state.world_open == Some(true) {
                    let tile =
                        atlas::tile_at(e.center).ok_or_else(|| "Invalid map position".to_owned());
                    tile.and_then(|tile| {
                        e.select_region(tile)?;
                        e.map_picker = false;
                        Ok(())
                    })
                } else {
                    Ok(())
                }
            }
            Action::Field(field) => {
                begin(&mut e, field);
                Ok(())
            }
            Action::Apply => apply(&mut e),
            Action::Cancel => {
                e.focus = None;
                e.revision += 1;
                Ok(())
            }
            Action::Kind(kind) => {
                e.placement_kind = kind;
                e.revision += 1;
                Ok(())
            }
            Action::Place => {
                if e.placing {
                    e.placing = false;
                    e.revision += 1;
                    Ok(())
                } else if e.placement_kind==4 {
                    if e.object_template.is_none(){Err("Choose a world model".into())}else{e.placing=true;e.focus=None;e.revision+=1;Ok(())}
                } else if !catalog.entries.iter().any(|entry| {
                    entry.kind == CatalogKind::Npc && entry.network_id == Some(e.type_id)
                }) && e.placement_kind != 3
                    && !xdt.has_npc_type(e.type_id)
                {
                    Err("Choose an existing NPC type from XDT".into())
                } else if e.placement_kind == 3
                    && !e
                        .entities
                        .iter()
                        .any(|p| p.kind == 3 && p.type_id == e.type_id)
                {
                    Err("Choose an existing shiny type".into())
                } else {
                    e.placing = true;
                    e.focus = None;
                    e.revision += 1;
                    Ok(())
                }
            }
            Action::Duplicate => {
                let position = e
                    .selected()
                    .map(|p| p.position + Vec3::new(-5., 0., 5.))
                    .ok_or_else(|| "Select an entity".to_owned());
                position.and_then(|p| e.place(p, true))
            }
            Action::Undo => e.undo(false),
            Action::ToggleKind(kind) => {
                e.actor_enabled[kind] = !e.actor_enabled[kind];
                e.revision += 1;
                Ok(())
            }
            Action::List(mode) => {
                e.switch_list(mode);
                Ok(())
            }
            Action::EditObjects(objects) => {
                e.routes=None;
                e.selected = None;
                e.selected_point = None;
                e.terrain_tool = None;
                e.edit_objects = objects;
                e.switch_list(if objects { 1 } else { 0 });
                Ok(())
            }
            Action::Terrain(tool) => {
                e.routes=None;
                e.terrain_tool = Some(tool);
                if tool==3 {
                    if let Some(layer)=atlas::tile_at(e.center).and_then(|at|e.ground.get(&atlas::tile_id(at))).and_then(|t|t.descriptor["splat"]["layers"].as_array()).and_then(|layers|layers.iter().position(|l|l["trueTextureName"].as_str().is_some_and(terrain::path_texture))){e.brush_layer=layer;}
                }
                e.selected = None;
                e.placing = false;
                e.selected_point = None;
                e.focus = None;
                e.revision += 1;
                Ok(())
            }
            Action::TerrainLayer(layer) => {
                e.brush_layer = layer;
                e.revision += 1;
                Ok(())
            }
            Action::RemoteObject(index) => e.select_remote_object(index),
            action @ (Action::ChooseType(_)|Action::PickerSelect(_)|Action::PickerConfirm|Action::PickerCancel|Action::PickerPage(_)|Action::Routes|Action::RouteTab(_)|Action::RouteKind(_)|Action::RoutePage(_)|Action::RouteNew|Action::RouteSelect(_)|Action::RouteSave|Action::RoutePoint(_)|Action::RouteRemovePoint|Action::RouteLoop|Action::RouteAssign)=>type_picker::action(&mut e,action,&catalog),
            Action::Delete => e.delete_selected(),
            Action::Copy => e.copy(),
            Action::Paste => e.paste(),
            Action::CopyCoordinates => e.copy_coordinates(),
            Action::PasteCoordinates => e.paste_coordinates(),
            Action::Instances(entity) => {
                let mode = e.list_mode as usize;
                e.list_search[mode] = e.search.clone();
                e.instance_menu = Some(entity);
                e.focus = None;
                e.search.clear();
                e.page = 0;
                e.revision += 1;
                Ok(())
            }
            Action::ChooseInstance(id) => {
                let entity = e.instance_menu.take() == Some(true);
                e.search = e.list_search[e.list_mode as usize].clone();
                e.page = 0;
                if entity {
                    let index = e.selected;
                    let p = e.selected().cloned().ok_or("Select an entity".to_owned());
                    p.and_then(|p| e.transform(index.unwrap(), p.position, p.angle, id))
                } else {
                    e.instance = id;
                    e.selected = None;
                    e.selected_point = None;
                    e.revision += 1;
                    Ok(())
                }
            }
            Action::NewInstance => {
                begin(&mut e, Field::InstanceName);
                Ok(())
            }
            Action::Redo => e.undo(true),
            Action::SaveWork => e.save_work(),
            Action::RestoreWork => {
                e.session_request = Some(session::Request::Restore);
                Ok(())
            }
            Action::Publish => e.publish(),
            Action::Server => {
                xdt.start_server_selection(&l, &lang);
                Ok(())
            }
            Action::Center => {
                e.center_selected();
                if state.world_open == Some(true) {
                    atlas::tile_at(e.center)
                        .ok_or_else(|| "Invalid map position".into())
                        .and_then(|tile| e.select_region(tile))
                } else {
                    Ok(())
                }
            }
            Action::Page(down) => {
                let pages = if e.instance_menu.is_some() {
                    e.instances
                        .iter()
                        .filter(|(id, name)| {
                            format!("{id} {name}")
                                .to_lowercase()
                                .contains(&e.search.to_lowercase())
                        })
                        .count()
                        .div_ceil(8)
                        .max(1)
                } else {
                    (e.filtered(&catalog).len() + e.remote_objects().len())
                        .div_ceil(view::list_capacity(lists.iter().next(),window.height(),state.world_open==Some(true),e.list_mode==1))
                        .max(1)
                };
                e.page = if down {
                    (e.page + 1).min(pages - 1)
                } else {
                    e.page.saturating_sub(1)
                };
                e.revision += 1;
                Ok(())
            }
            Action::Tile => atlas::tile_at(e.center)
                .ok_or_else(|| "Invalid map position".into())
                .and_then(|tile| e.select_region(tile)),
            Action::Map => {
                e.map_picker = !e.map_picker;
                if e.map_picker {
                    atlas::fit(
                        &mut e,
                        view::rect(Vec2::new(window.width(), window.height())).1,
                    );
                }
                e.revision += 1;
                Ok(())
            }
            Action::Overview => {
                atlas::fit(
                    &mut e,
                    view::rect(Vec2::new(window.width(), window.height())).1,
                );
                Ok(())
            }
        };
        if let Err(error) = result {
            e.error(error);
        } else if matches!(action,Action::PickerSelect(_)) {
            if let Some(picker)=e.type_picker.as_mut(){picker.grab=window.cursor_position();}
        }
    }
}
pub(super) fn keyboard(
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<EditorState>,
    mut e: ResMut<WorldEditor>,
    mut held: Local<ButtonInput<KeyCode>>,
) {
    if state.world_open.is_none() {
        events.clear();
        held.reset_all();
        return;
    }
    for event in events.read() {
        if event.state == ButtonState::Released {
            held.release(event.key_code);
            continue;
        }
        held.press(event.key_code);
        if let Err(error) = e.finish_brush() {
            e.error(error);
            continue;
        }
        let ctrl = held.pressed(KeyCode::ControlLeft)
            || held.pressed(KeyCode::ControlRight)
            || keys.pressed(KeyCode::ControlLeft)
            || keys.pressed(KeyCode::ControlRight);
        if event.key_code == KeyCode::Escape {
            finish_drag(&mut e);
            e.type_picker=None;
            e.focus = None;
            e.placing = false;
            e.creating_terrain = false;
            e.revision += 1;
            continue;
        }
        let result = if e.type_picker.is_some()&&e.focus.is_none(){Ok(())}else if ctrl && event.key_code == KeyCode::KeyS {
            e.save_work()
        } else if e.focus.is_none()
            && ctrl
            && matches!(event.key_code, KeyCode::KeyZ | KeyCode::KeyY)
        {
            e.undo(event.key_code == KeyCode::KeyY || keys.pressed(KeyCode::ShiftLeft))
        } else if e.focus.is_none() && ctrl && event.key_code == KeyCode::KeyD {
            let pos = e.selected().map(|p| p.position + Vec3::new(-5., 0., 5.));
            pos.ok_or("Select an entity".into())
                .and_then(|p| e.place(p, true))
        } else if e.focus.is_none() && ctrl && event.key_code == KeyCode::KeyC {
            e.copy()
        } else if e.focus.is_none() && ctrl && event.key_code == KeyCode::KeyV {
            e.paste()
        } else if e.focus.is_none() && event.key_code == KeyCode::Delete {
            e.delete_selected()
        } else if e.focus.is_none() && event.key_code == KeyCode::KeyF {
            e.center_selected();
            Ok(())
        } else if let Some(field) = e.focus {
            if event.key_code == KeyCode::Enter {
                apply(&mut e)
            } else {
                if ctrl && event.key_code == KeyCode::KeyA {
                    e.select_text = true;
                } else if ctrl && event.key_code == KeyCode::KeyV {
                    match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
                        Ok(text) => {
                            if e.select_text {
                                e.edit.clear();
                            }
                            e.edit.extend(text.chars().filter(|c| !c.is_control()));
                            e.select_text = false;
                        }
                        Err(error) => e.error(error.to_string()),
                    }
                } else if ctrl && event.key_code == KeyCode::KeyC {
                    if let Err(error) = arboard::Clipboard::new()
                        .and_then(|mut clipboard| clipboard.set_text(e.edit.clone()))
                    {
                        e.error(error.to_string());
                    }
                } else if matches!(event.key_code, KeyCode::Backspace | KeyCode::Delete) {
                    if e.select_text {
                        e.edit.clear();
                    } else {
                        e.edit.pop();
                    }
                    e.select_text = false;
                } else if !ctrl {
                    if let Some(text) = &event.text {
                        let text: String = text.chars().filter(|c| !c.is_control()).collect();
                        if !text.is_empty() {
                            if e.select_text {
                                e.edit.clear();
                            }
                            e.edit.push_str(&text);
                            e.select_text = false;
                        }
                    }
                }
                if field == Field::Search {
                    e.search = e.edit.clone();
                    e.page = 0;
                }
                if field==Field::PickerSearch {
                    let text=e.edit.clone();if let Some(p)=e.type_picker.as_mut(){p.filter=text;p.page=0;}
                }
                e.revision += 1;
                Ok(())
            }
        } else {
            Ok(())
        };
        if let Err(error) = result {
            e.error(error);
        }
    }
}
fn snapped(e: &WorldEditor, mut p: Vec3) -> Vec3 {
    let step = e.snap * 0.01;
    if step > 0. {
        p.x = (p.x / step).round() * step;
        p.z = (p.z / step).round() * step;
    }
    p
}
pub(super) fn pointer(
    mut e: ResMut<WorldEditor>,
    state: Res<EditorState>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheels: MessageReader<MouseWheel>,
    window: Single<&Window>,
    canvas: Query<(&ComputedNode, &UiGlobalTransform), With<Canvas>>,
    cameras: Query<(&Camera, &GlobalTransform), With<PreviewCamera>>,
    catalog: Res<EditorCatalog>,
    mut scrolls: Query<
        (&RelativeCursorPosition, &ComputedNode, &mut ScrollPosition),
        With<InspectorScroll>,
    >,
    mut events: MessageReader<bevy::window::WindowEvent>,
    time: Res<Time>,
    preview: Res<preview::WorldPreview>,
    mesh_bounds: Query<(Entity, &bevy::camera::primitives::Aabb, &GlobalTransform)>,
    parents: Query<&ChildOf>,
) {
    let Some(three_d) = state.world_open else {
        wheels.clear();
        events.clear();
        e.last_cursor = None;
        finish_drag(&mut e);
        return;
    };
    if e.ignore_pointer_press {
        e.ignore_pointer_press=false;events.clear();wheels.clear();e.last_cursor=window.cursor_position();return;
    }
    if let Some(picker)=e.type_picker.as_mut() {
        let moved=picker.grab.zip(window.cursor_position()).is_some_and(|(start,now)|start.distance(now)>10.);
        if mouse.pressed(MouseButton::Left)&&moved&&!picker.dragging {picker.dragging=true;e.revision+=1;}
        if !e.type_picker.as_ref().unwrap().dragging {
            if !mouse.pressed(MouseButton::Left){e.type_picker.as_mut().unwrap().grab=None;}
            wheels.clear();events.clear();e.last_cursor=window.cursor_position();return;
        }
    }
    if !mouse.pressed(MouseButton::Left) {
        if let Err(error) = e.finish_brush() {
            e.error(error);
        }
    }
    let raster = !three_d || e.map_picker;
    let Some(mut cursor) = window.cursor_position() else {
        return;
    };
    let mut event_cursor = e.last_cursor.unwrap_or(cursor);
    let mut press = None;
    let mut release = None;
    for event in events.read() {
        match event {
            bevy::window::WindowEvent::CursorMoved(event) => event_cursor = event.position,
            bevy::window::WindowEvent::MouseButtonInput(event)
                if event.button == MouseButton::Left =>
            {
                if event.state == ButtonState::Pressed {
                    press = Some(event_cursor);
                } else {
                    release = Some(event_cursor);
                }
            }
            _ => {}
        }
    }
    if let Some(end) = release {
        cursor = end;
    }
    let press_cursor = press.unwrap_or(cursor);
    let Some((node, global)) = canvas.iter().next() else {
        return;
    };
    let scale = node.inverse_scale_factor;
    let size = node.size() * scale;
    let min = global.translation * scale - size * 0.5;
    let over = cursor.cmpge(min).all() && cursor.cmple(min + size).all();
    let delta = e.last_cursor.map_or(Vec2::ZERO, |last| cursor - last);
    e.last_cursor = Some(cursor);
    if !mouse.pressed(MouseButton::Left)
        && !mouse.pressed(MouseButton::Right)
        && press.is_none()
        && release.is_none()
    {
        finish_drag(&mut e);
    }
    for wheel in wheels.read() {
        for (cursor, node, mut scroll) in &mut scrolls {
            if cursor.cursor_over() {
                let amount = if wheel.unit == MouseScrollUnit::Line {
                    wheel.y * 36.
                } else {
                    wheel.y
                };
                let max = (node.content_size.y - node.size().y).max(0.) * node.inverse_scale_factor;
                scroll.0.y = (scroll.0.y - amount).clamp(0., max);
            }
        }
        if over {
            let amount = match wheel.unit {
                MouseScrollUnit::Line => wheel.y,
                MouseScrollUnit::Pixel => wheel.y / 40.,
            };
            if !raster {
                e.distance = (e.distance * (-amount * 0.12).exp()).clamp(2., 2000.);
            } else {
                let (_, old) = atlas::viewport(&e);
                let zoom = (old * (amount * 0.12).exp()).clamp(0.001, 60.);
                let anchor = cursor - min - size * 0.5;
                let difference = anchor / old - anchor / zoom;
                let offset = Vec3::new(-difference.x, 0., -difference.y);
                if e.map_picker {
                    e.atlas_zoom = zoom;
                    e.atlas_center += offset;
                } else {
                    e.zoom = zoom;
                    e.center += offset;
                }
            }
            e.revision += 1;
        }
    }
    if !over {
        if mouse.just_released(MouseButton::Left){if let Some(picker)=e.type_picker.as_mut(){picker.dragging=false;picker.grab=None;e.revision+=1;}}
        return;
    }
    if mouse.pressed(MouseButton::Middle) || (raster && mouse.pressed(MouseButton::Right)) {
        let factor = if !raster {
            e.distance / size.y.max(1.)
        } else {
            1. / atlas::viewport(&e).1
        };
        let pan = Vec3::new(delta.x * factor, 0., delta.y * factor);
        let yaw = e.yaw;
        let offset = if !raster {
            Quat::from_rotation_y(-yaw) * pan
        } else {
            pan
        };
        if e.map_picker {
            e.atlas_center += offset;
        } else {
            e.center += offset;
        }
        e.revision += 1;
        return;
    }
    if !raster && mouse.pressed(MouseButton::Right) {
        if mouse.just_pressed(MouseButton::Right)
            && let Some(index) = e.selected
        {
            let p = &e.entities[index];
            let mesh = cameras
                .iter()
                .next()
                .and_then(|(c, t)| c.viewport_to_world(t, cursor).ok())
                .and_then(|ray| {
                    picking::entity(&e, &preview, ray, &mesh_bounds, &parents)
                });
            if (p.kind == 4) == e.edit_objects
                && (mesh == Some(index)
                    || cameras
                        .iter()
                        .next()
                        .and_then(|(c, t)| c.world_to_viewport(t, p.position).ok())
                        .is_some_and(|at| at.distance(cursor) <= 24.))
            {
                e.drag = Some(Drag {
                    index,
                    offset: Vec3::ZERO,
                    raw_position: p.position,
                    undo_start: e.undo.len(),
                    rotating: true,
                });
            }
        }
        if let Some(drag) = &e.drag
            && drag.rotating
        {
            let index = drag.index;
            let p = e.entities[index].clone();
            let result = if p.kind == 4 {
                e.rotate_object(
                    index,
                    if keys.pressed(KeyCode::KeyZ) { 0 } else { 1 },
                    delta.x * 0.5,
                    false,
                )
            } else {
                e.transform(index, p.position, p.angle + delta.x * 0.5, p.instance)
            };
            if let Err(error) = result {
                e.error(error);
                finish_drag(&mut e);
            }
            return;
        }
        e.yaw -= delta.x * 0.006;
        e.pitch = (e.pitch + delta.y * 0.006).clamp(0.05, 1.5);
        e.revision += 1;
        return;
    }
    if e.map_picker {
        if press.is_some() || mouse.just_pressed(MouseButton::Left) {
            let (center, zoom) = atlas::viewport(&e);
            let mut point = atlas::unproject(center, zoom, press_cursor - min, size);
            if let Some(tile) = atlas::tile_at(point) {
                if e.creating_terrain {match e.create_terrain(point){Ok(())=>{e.center=point;e.map_picker=false;e.revision+=1;},Err(error)=>e.error(error)}return;}
                match e.select_region(tile) {
                    Ok(()) => {
                        if let Some(nearest) = e
                            .entities
                            .iter()
                            .filter(|p| {
                                p.kind != 4
                                    && p.instance == e.instance
                                    && atlas::tile_at(p.position) == Some(tile)
                            })
                            .min_by(|a, b| {
                                a.position
                                    .distance_squared(point)
                                    .total_cmp(&b.position.distance_squared(point))
                            })
                        {
                            point.y = nearest.position.y;
                        }
                        e.center = point;
                        e.distance = e.distance.max(250.);
                        e.map_picker = false;
                        e.focus = None;
                        e.revision += 1;
                    }
                    Err(error) => e.error(error),
                }
            }
        }
        return;
    }
    let height = e.selected().map_or(e.center.y, |p| p.position.y);
    let unproject = |cursor: Vec2| {
        if three_d {
            let Some((camera, transform)) = cameras.iter().next() else {
                return None;
            };
            camera
                .viewport_to_world(transform, cursor)
                .ok()
                .and_then(|ray| {
                    if ray.direction.y.abs() < 0.00001 {
                        return None;
                    }
                    let t = (height - ray.origin.y) / ray.direction.y;
                    e.ground_ray(ray)
                        .or_else(|| (t > 0.).then(|| ray.origin + *ray.direction * t))
                })
        } else {
            let xy = (cursor - min - size * 0.5) / e.zoom;
            Some(e.center + Vec3::new(-xy.x, height - e.center.y, -xy.y))
        }
    };
    // Mesh selection and axis dragging do not require the cursor ray to hit
    // ground (e.g. when clicking the top of a building above the horizon).
    let Some(point) = unproject(cursor).or_else(|| {
        (three_d && e.terrain_tool.is_none() && e.routes.is_none() && !e.placing).then_some(e.center)
    }) else {
        return;
    };
    let clicked_point = unproject(press_cursor);
    let press_point = clicked_point.unwrap_or(point);
    if e.type_picker.as_ref().is_some_and(|p|p.dragging) {
        if release.is_some() || mouse.just_released(MouseButton::Left) {
            e.type_picker.as_mut().unwrap().target=None;
            let result=(||{
                let point=snapped(&e,point);
                let point=e.snap_to_ground(point)?;
                e.confirm_type()?;
                if e.terrain_tool==Some(4){let template=e.grass_template.clone().ok_or("Choose a model")?;e.paste_object(template,point)}else{e.place(point,false)}
            })();
            if let Err(error)=result{e.error(error);}
        }
        return;
    }
    if e.routes.as_ref().is_some_and(|r|r.drawing&&!r.assignments) {
        if press.is_some()||mouse.just_pressed(MouseButton::Left){if let Err(error)=e.route_click(press_point){e.error(error);}}
        return;
    }
    if e.creating_terrain {
        if press.is_some() || mouse.just_pressed(MouseButton::Left) {
            if let Err(error) = e.create_terrain(press_point) { e.error(error); }
        }
        return;
    }
    if e.terrain_tool.is_some() {
        if e.brush_cursor.and_then(atlas::tile_at) != atlas::tile_at(point) {
            e.revision += 1;
        }
        e.brush_cursor = Some(point);
        if mouse.pressed(MouseButton::Left) {
            if e.terrain_tool==Some(4) { if let Err(error)=e.grass_brush(point){e.error(error);} }
            else if let Err(error) = e.terrain_brush(
                point,
                time.delta_secs(),
                keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
            ) {
                e.error(error);
            }
        }
        return;
    }
    if press.is_some() || mouse.just_pressed(MouseButton::Left) {
        e.focus = None;
        if e.placing {
            let point = snapped(&e, point);
            let point = match e.snap_to_ground(point) {
                Ok(p) => p,
                Err(error) => {
                    e.error(error);
                    return;
                }
            };
            if let Err(error) = e.place(point, false) {
                e.error(error);
            }
            return;
        }
        if !three_d {
            let remote = e
                .remote_objects()
                .into_iter()
                .filter_map(|i| {
                    let distance = view::screen(&e, e.object_index[i].position, size)
                        .distance(press_cursor - min);
                    (distance <= 18.).then_some((i, distance))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|(i, _)| i);
            if let Some(i) = remote {
                if let Err(error) = e.select_remote_object(i) {
                    e.error(error);
                }
                return;
            }
        }
        let mesh = three_d
            .then(|| {
                cameras
                    .iter()
                    .next()
                    .and_then(|(c, t)| c.viewport_to_world(t, press_cursor).ok())
                    .and_then(|ray| {
                        picking::entity(&e, &preview, ray, &mesh_bounds, &parents)
                    })
            })
            .flatten();
        let selected = mesh.or_else(|| {
            e.displayed(&catalog)
                .into_iter()
                .filter(|i| !three_d || (e.entities[*i].kind == 4) == e.edit_objects)
                .filter_map(|i| {
                    let p = &e.entities[i];
                    let distance = if three_d {
                        let (camera, transform) = cameras.iter().next()?;
                        let projected = camera.world_to_viewport(transform, p.position).ok()?;
                        projected.distance(press_cursor)
                    } else {
                        view::screen(&e, p.position, size).distance(press_cursor - min)
                    };
                    (distance <= 18.).then_some((i, distance))
                })
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .map(|p| p.0)
        });
        if let Some(index) = selected {
            if e.click_entity(index) { picking::focus(&mut e,&preview,&mesh_bounds,&parents);return; }
            let position = e.entities[index].position;
            e.drag = Some(Drag {
                index,
                offset: position - if three_d {cameras.iter().next().and_then(|(c,t)|c.viewport_to_world(t,press_cursor).ok()).and_then(|ray|{
                    let distance=(position.y-ray.origin.y)/ray.direction.y;
                    (ray.direction.y.abs()>0.00001&&distance>0.).then(||ray.origin+*ray.direction*distance)
                }).unwrap_or(press_point)} else {press_point},
                raw_position: position,
                undo_start: e.undo.len(),
                rotating: false,
            });
            e.revision += 1;
        } else if let Err(error) = e.select_coordinates(clicked_point) {
            e.error(error);
        }
    }
    if (mouse.pressed(MouseButton::Left) || release.is_some())
        && (if press.is_some() {
            cursor.distance_squared(press_cursor) > 0.
        } else {
            delta.length_squared() > 0.
        })
        && let Some(drag) = &e.drag
    {
        let index = drag.index;
        if drag.rotating {
            return;
        }
        let raw=drag.raw_position;
        let position = if three_d {
            let p = &e.entities[index];
            if let Some((axis, _)) = manipulation::axis(&keys) {
              if let Some((camera, transform)) = cameras.iter().next() {
                if let (Ok(start), Ok(end)) = (
                    camera.world_to_viewport(transform, raw),
                    camera.world_to_viewport(transform, raw + axis * 10.),
                ) {
                    let projected = end - start;
                    raw
                        + axis * (delta.dot(projected) / projected.length_squared().max(0.01) * 10.)
                } else {
                    raw
                }
            } else {
                raw
            }
            } else {
                let plane=cameras.iter().next().and_then(|(c,t)|c.viewport_to_world(t,cursor).ok()).and_then(|ray|{
                    let distance=(p.position.y-ray.origin.y)/ray.direction.y;
                    (ray.direction.y.abs()>0.00001&&distance>0.).then(||ray.origin+*ray.direction*distance)
                });
                let mut target = plane.unwrap_or(point) + drag.offset;
                target.y = p.position.y;
                target
            }
        } else if keys.pressed(KeyCode::AltLeft) {
            point + drag.offset
        } else {
            snapped(&e, point + drag.offset)
        };
        if let Some(drag)=e.drag.as_mut(){drag.raw_position=position;}
        let position = if three_d && !keys.pressed(KeyCode::KeyZ) && !keys.any_pressed([KeyCode::AltLeft,KeyCode::AltRight]) {snapped(&e,position)}else{position};
        let position = if !three_d {
            let old = e.entities[index].position;
            if keys.pressed(KeyCode::KeyX) { Vec3::new(position.x, old.y, old.z) }
            else if keys.pressed(KeyCode::KeyC) { Vec3::new(old.x, old.y, position.z) }
            else if keys.pressed(KeyCode::KeyZ) { old + Vec3::Y * (-delta.y / e.zoom) }
            else { position }
        } else { position };
        let position = if !three_d && !keys.pressed(KeyCode::KeyZ) {
            match e.snap_to_ground(position) {
                Ok(p) => p,
                Err(error) => {
                    e.error(error);
                    finish_drag(&mut e);
                    return;
                }
            }
        } else {
            position
        };
        let p = e.entities[index].clone();
        if let Err(error) = e.transform(index, position, p.angle, p.instance) {
            e.error(error);
            finish_drag(&mut e);
        }
    }
    if release.is_some() {
        finish_drag(&mut e);
    }
    // Q/E rotate the selected entity without changing its server coordinate basis.
    if e.focus.is_none()
        && let Some(i) = e.selected
    {
        let step = if keys.just_pressed(KeyCode::KeyQ) {
            -15.
        } else if keys.just_pressed(KeyCode::KeyE) {
            15.
        } else {
            0.
        };
        if step != 0. {
            let p = e.entities[i].clone();
            let angle = ProtocolYawDegrees::new((p.angle + step) as i32).degrees() as f32;
            if let Err(error) = e.transform(i, p.position, angle, p.instance) {
                e.error(error);
            }
        }
    }
}
