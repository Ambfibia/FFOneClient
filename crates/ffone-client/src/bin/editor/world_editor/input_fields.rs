//! Inspector and editor text fields.
use super::*;

pub(super) fn begin(e: &mut WorldEditor, field: Field) {
    let p = e.selected();
    let text = match field {
        Field::SquareName => e.square_settings()["location"].as_str().unwrap_or_default().to_owned(),
        Field::SquareMusic => e.square_settings()["music"].as_str().unwrap_or_default().to_owned(),
        Field::PickerSearch => e
            .type_picker
            .as_ref()
            .map(|p| p.filter.clone())
            .unwrap_or_default(),
        Field::RouteSpeed => e
            .routes
            .as_ref()
            .map(|r| r.speed.to_string())
            .unwrap_or_default(),
        Field::RouteTarget => e
            .routes
            .as_ref()
            .map(|r| r.target.to_string())
            .unwrap_or_default(),
        Field::RouteX | Field::RouteY | Field::RouteZ | Field::RouteStop => e
            .routes
            .as_ref()
            .and_then(|r| r.point.and_then(|i| r.points.get(i)))
            .map(|p| {
                p[match field {
                    Field::RouteX => "iX",
                    Field::RouteY => "iY",
                    Field::RouteZ => "iZ",
                    _ => "iStopTicks",
                }]
                .to_string()
            })
            .unwrap_or_default(),
        Field::Search => e.search.clone(),
        Field::Type => e.type_id.to_string(),
        Field::Snap => e.snap.to_string(),
        Field::BrushRadius => e.brush_radius.to_string(),
        Field::BrushStrength => e.brush_strength.to_string(),
        Field::X => p
            .map(|p| (-p.position.x * 100.).to_string())
            .unwrap_or_default(),
        Field::Y => p
            .map(|p| (p.position.z * 100.).to_string())
            .unwrap_or_default(),
        Field::Z => p
            .map(|p| (p.position.y * 100.).to_string())
            .unwrap_or_default(),
        Field::Angle => p
            .map(|p| {
                if p.kind == 4 {
                    e.object_rotation(p).y
                } else {
                    p.angle
                }
            })
            .map(|a| a.to_string())
            .unwrap_or_default(),
        Field::AngleX => p
            .map(|p| e.object_rotation(p).x.to_string())
            .unwrap_or_default(),
        Field::AngleZ => p
            .map(|p| e.object_rotation(p).z.to_string())
            .unwrap_or_default(),
        Field::InstanceName => String::new(),
        Field::EntityInstance => p.map(|p| p.instance.to_string()).unwrap_or_default(),
    };
    e.focus = Some(field);
    e.edit = text;
    e.select_text = true;
    e.revision += 1;
}
fn instance(text: &str) -> Result<u32, String> {
    text.parse()
        .map_err(|_| "Instance must be an unsigned integer".into())
}
pub(super) fn apply(e: &mut WorldEditor) -> Result<(), String> {
    let Some(field) = e.focus else {
        return Ok(());
    };
    match field {
        Field::SquareName => e.change_square("location",Value::from(e.edit.clone()))?,
        Field::SquareMusic => e.change_square("music",Value::from(e.edit.clone()))?,
        Field::PickerSearch => {
            let p = e.type_picker.as_mut().ok_or("Open type picker")?;
            p.filter = e.edit.clone();
            p.page = 0;
        }
        Field::RouteSpeed => {
            let n: i32 = e
                .edit
                .parse()
                .map_err(|_| "Speed must be a positive integer")?;
            if n <= 0 {
                return Err("Speed must be positive".into());
            }
            e.routes.as_mut().ok_or("Open path editor")?.speed = n;
        }
        Field::RouteTarget => {
            let n: i32 = e
                .edit
                .parse()
                .map_err(|_| "Transport route ID must be an integer")?;
            if n < 0 {
                return Err("Transport route ID must be nonnegative".into());
            }
            e.routes.as_mut().ok_or("Open path editor")?.target = i64::from(n);
        }
        Field::RouteX | Field::RouteY | Field::RouteZ | Field::RouteStop => {
            let n: i32 = e
                .edit
                .parse()
                .map_err(|_| "Point value must be an integer")?;
            if field == Field::RouteStop && n < 0 {
                return Err("Stop ticks must be nonnegative".into());
            }
            let r = e.routes.as_mut().ok_or("Open path editor")?;
            let point = r
                .point
                .and_then(|i| r.points.get_mut(i))
                .ok_or("Select a route point")?;
            point[match field {
                Field::RouteX => "iX",
                Field::RouteY => "iY",
                Field::RouteZ => "iZ",
                _ => "iStopTicks",
            }] = Value::from(n);
            if r.kind == 2 && field == Field::RouteStop {
                point["bStop"] = Value::from(n > 0);
            }
        }
        Field::Search => {
            e.search = e.edit.clone();
            e.page = 0;
        }
        Field::Type => {
            e.type_id = e.edit.parse().map_err(|_| "Type ID must be an integer")?;
            if e.type_id <= 0 {
                return Err("Type ID must be positive".into());
            }
        }
        Field::Snap => {
            e.snap = e.edit.parse().map_err(|_| "Grid step must be numeric")?;
            if !e.snap.is_finite() || e.snap < 0. {
                return Err("Grid step must be finite and nonnegative".into());
            }
        }
        Field::BrushRadius | Field::BrushStrength => {
            let value: f32 = e
                .edit
                .parse()
                .map_err(|_| "Brush setting must be numeric")?;
            if !value.is_finite() || !(0.01..=1000.).contains(&value) {
                return Err("Brush setting must be in 0.01…1000".into());
            }
            if field == Field::BrushRadius {
                e.brush_radius = value;
            } else {
                e.brush_strength = value;
            }
        }
        Field::InstanceName => {
            let name = e.edit.trim().to_owned();
            if name.is_empty() {
                return Err("Enter an instance name".into());
            }
            let id = e
                .instances
                .keys()
                .next_back()
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or("Instance IDs exhausted")?;
            e.register_instance(id)?;
            e.instances.insert(id, name);
            if e.instance_menu.take() == Some(true) {
                let index = e.selected.ok_or("Select an entity")?;
                let p = e.selected().ok_or("Select an entity")?.clone();
                e.transform(index, p.position, p.angle, id)?;
            } else {
                e.instance = id;
                e.selected = None;
                e.selected_point = None;
            }
            e.search = e.list_search[e.list_mode as usize].clone();
        }
        _ => {
            let index = e.selected.ok_or("Select an entity")?;
            let p = e.selected().ok_or("Select an entity")?.clone();
            let mut position = p.position;
            let mut angle = p.angle;
            let mut map = p.instance;
            if field == Field::EntityInstance {
                map = instance(&e.edit)?;
            } else {
                let number: f32 = e.edit.parse().map_err(|_| "Enter a number")?;
                if !number.is_finite() {
                    return Err("Enter a finite number".into());
                }
                if p.kind == 4 && matches!(field, Field::Angle | Field::AngleX | Field::AngleZ) {
                    e.rotate_object(
                        index,
                        match field {
                            Field::AngleX => 0,
                            Field::AngleZ => 2,
                            _ => 1,
                        },
                        number,
                        true,
                    )?;
                    e.focus = None;
                    e.revision += 1;
                    return Ok(());
                }
                match field {
                    Field::X => position.x = -number * 0.01,
                    Field::Y => position.z = number * 0.01,
                    Field::Z => position.y = number * 0.01,
                    Field::Angle => angle = number,
                    _ => {}
                }
            }
            e.transform(index, position, angle, map)?;
        }
    }
    e.focus = None;
    e.revision += 1;
    Ok(())
}
