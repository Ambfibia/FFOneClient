//! Edit the server's consumed paths.json, with stable route IDs and shared history.
use super::*;
use model::{Patch, Source};

pub(super) struct Editor {
    pub source: usize,
    pub kind: u8,
    pub id: i64,
    pub pointer: Option<String>,
    pub points: Vec<Value>,
    pub point: Option<usize>,
    pub speed: i32,
    pub looped: bool,
    pub assignments: bool,
    pub drawing: bool,
    pub target: i64,
    pub page: usize,
}
pub(super) struct Row {
    pub pointer: String,
    pub id: i64,
    pub label: String,
}
fn container(kind: u8) -> &'static str {
    match kind {
        0 => "npc",
        1 => "skyway",
        _ => "slider",
    }
}
impl WorldEditor {
    pub(super) fn open_routes(&mut self) -> Result<(), String> {
        self.finish_brush()?;
        if self.routes.take().is_some() {
            self.revision += 1;
            return Ok(());
        }
        let path = self
            .folder
            .as_ref()
            .ok_or("Select a server folder")?
            .join("paths.json");
        let source = if let Some(i) = self.sources.iter().position(|s| s.path == path) {
            i
        } else {
            let base = if path.exists() {
                terrain::read_source(&path)?
            } else {
                Value::Null
            };
            let draft = if base.is_null() {
                serde_json::json!({"npc":{},"skyway":{},"slider":{}})
            } else {
                base.clone()
            };
            self.sources.push(Source { path, base, draft });
            self.sources.len() - 1
        };
        self.routes = Some(Editor {
            source,
            kind: 0,
            id: 0,
            pointer: None,
            points: vec![],
            point: None,
            speed: 300,
            looped: true,
            assignments: false,
            drawing: false,
            target: 0,
            page: 0,
        });
        self.terrain_tool = None;
        self.placing = false;
        self.focus = None;
        self.revision += 1;
        Ok(())
    }
    pub(super) fn route_rows(&self) -> Vec<Row> {
        let Some(r) = &self.routes else {
            return vec![];
        };
        if r.kind == 2 {
            return vec![Row {
                pointer: "/slider".into(),
                id: 0,
                label: "Slider · 0".into(),
            }];
        }
        self.sources[r.source].draft[container(r.kind)]
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(key, row)| {
                let id = key.parse().ok()?;
                let label = if r.kind == 0 {
                    format!(
                        "ID {id} · {}",
                        row["aPoints"].as_array().map_or(0, Vec::len)
                    )
                } else {
                    format!("ID {id} · Skyway {}", row["iRouteID"])
                };
                Some(Row {
                    pointer: format!("/{}/{key}", container(r.kind)),
                    id,
                    label,
                })
            })
            .collect()
    }
    pub(super) fn choose_route(&mut self, index: usize) -> Result<(), String> {
        let row = self
            .route_rows()
            .into_iter()
            .nth(index)
            .ok_or("Missing route")?;
        let r = self.routes.as_mut().ok_or("Open path editor")?;
        let value = self.sources[r.source]
            .draft
            .pointer(&row.pointer)
            .ok_or("Missing route")?;
        r.points = if r.kind == 2 {
            value
                .as_object()
                .ok_or("Invalid slider")?
                .values()
                .cloned()
                .collect()
        } else {
            value["aPoints"]
                .as_array()
                .ok_or("Missing route points")?
                .clone()
        };
        r.id = row.id;
        r.pointer = Some(row.pointer);
        r.point = None;
        r.drawing = true;
        r.speed = value[if r.kind == 0 {
            "iBaseSpeed"
        } else {
            "iMonkeySpeed"
        }]
        .as_i64()
        .unwrap_or(1200) as i32;
        r.looped = value["bLoop"]
            .as_bool()
            .unwrap_or(r.kind == 2 || r.points.first() == r.points.last());
        r.target = value["iRouteID"].as_i64().unwrap_or(0);
        self.revision += 1;
        Ok(())
    }
    pub(super) fn new_route(&mut self) -> Result<(), String> {
        let rows = self.route_rows();
        let r = self.routes.as_mut().ok_or("Open path editor")?;
        r.id = if r.kind == 2 {
            0
        } else {
            rows.iter()
                .map(|r| r.id)
                .max()
                .unwrap_or(-1)
                .checked_add(1)
                .filter(|i| *i <= i32::MAX as i64)
                .ok_or("Route IDs exhausted")?
        };
        r.pointer = None;
        r.points.clear();
        r.point = None;
        r.drawing = true;
        r.assignments = false;
        r.speed = if r.kind == 0 { 300 } else { 1500 };
        r.looped = r.kind != 1;
        if r.kind == 1 {
            r.target = self.sources[r.source].draft["skyway"]
                .as_object()
                .into_iter()
                .flatten()
                .filter_map(|(_, v)| v["iRouteID"].as_i64())
                .max()
                .unwrap_or(-1)
                + 1;
        }
        self.selected = None;
        self.selected_point = None;
        self.revision += 1;
        Ok(())
    }
    pub(super) fn route_click(&mut self, point: Vec3) -> Result<bool, String> {
        if !self
            .routes
            .as_ref()
            .is_some_and(|r| r.drawing && !r.assignments)
        {
            return Ok(false);
        }
        let point = self.snap_to_ground(point)?;
        let coords = [-point.x * 100., point.z * 100., point.y * 100.];
        if !point.is_finite()
            || coords
                .iter()
                .any(|v| *v < i32::MIN as f32 || *v >= i32::MAX as f32)
        {
            return Err("Invalid path point".into());
        }
        let r = self.routes.as_mut().unwrap();
        r.points.push(serde_json::json!({"iX":coords[0].round() as i32,"iY":coords[1].round() as i32,"iZ":coords[2].round() as i32,"iStopTicks":0,"bStop":false}));
        r.point = Some(r.points.len() - 1);
        self.revision += 1;
        Ok(true)
    }
    pub(super) fn save_route(&mut self) -> Result<(), String> {
        let r = self.routes.as_ref().ok_or("Open path editor")?;
        if r.points.len() < 2
            || !r
                .points
                .iter()
                .filter_map(model::native)
                .any(|p| Some(p) != r.points.first().and_then(model::native))
        {
            return Err("A path needs at least two different points".into());
        }
        let pointer = r.pointer.clone().unwrap_or_else(|| {
            if r.kind == 2 {
                "/slider".into()
            } else {
                format!("/{}/{}", container(r.kind), r.id)
            }
        });
        let before = self.sources[r.source].draft.pointer(&pointer).cloned();
        let mut after = before.clone().unwrap_or_else(|| serde_json::json!({}));
        if r.kind == 2 {
            after = Value::Object(
                r.points
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (format!("{i:06}"), p.clone()))
                    .collect(),
            );
        } else {
            after["aPoints"] = Value::Array(r.points.clone());
            if r.kind == 0 {
                after["iBaseSpeed"] = Value::from(r.speed);
                after["bLoop"] = Value::from(r.looped);
                if after.get("aNPCTypes").is_none() {
                    after["aNPCTypes"] = serde_json::json!([]);
                    after["aNPCIDs"] = serde_json::json!([]);
                }
            } else {
                after["iMonkeySpeed"] = Value::from(r.speed);
                after["iRouteID"] = Value::from(
                    before
                        .as_ref()
                        .and_then(|v| v["iRouteID"].as_i64())
                        .unwrap_or(r.target),
                );
            }
        }
        self.commit(vec![Patch {
            source: r.source,
            pointer: pointer.clone(),
            before,
            after: Some(after),
        }])?;
        self.routes.as_mut().unwrap().pointer = Some(pointer);
        Ok(())
    }
    pub(super) fn assign_route(&mut self) -> Result<(), String> {
        let r = self.routes.as_ref().ok_or("Open path editor")?;
        let pointer = r.pointer.as_ref().ok_or("Save the route first")?;
        if self.sources[r.source].draft.pointer(pointer).is_none() {
            return Err("Save the route first".into());
        }
        if r.kind == 0 {
            let p = self
                .selected()
                .filter(|p| p.kind <= 2)
                .ok_or("Select an NPC, mob or group in the left list")?;
            let mut row = self.sources[p.source]
                .draft
                .pointer(&p.pointer)
                .ok_or("Missing NPC")?
                .clone();
            let before = row.clone();
            row["iPathID"] = Value::from(r.id);
            self.commit(vec![Patch {
                source: p.source,
                pointer: p.pointer.clone(),
                before: Some(before),
                after: Some(row),
            }])?;
        } else if r.kind == 1 {
            let row = self.sources[r.source]
                .draft
                .pointer(pointer)
                .unwrap()
                .clone();
            let (key, old) = self.sources[r.source].draft["skyway"]
                .as_object()
                .into_iter()
                .flatten()
                .find(|(_, v)| v["iRouteID"].as_i64() == Some(r.target))
                .ok_or("Choose an existing Skyway route ID")?;
            let mut after = old.clone();
            after["aPoints"] = row["aPoints"].clone();
            after["iMonkeySpeed"] = row["iMonkeySpeed"].clone();
            self.commit(vec![Patch {
                source: r.source,
                pointer: format!("/skyway/{key}"),
                before: Some(old.clone()),
                after: Some(after),
            }])?;
        } else {
            return Err("Slider points already edit the shared transport circuit".into());
        }
        Ok(())
    }
}
pub(super) fn action(e: &mut WorldEditor, action: Action) -> Result<(), String> {
    match action {
        Action::Routes => e.open_routes(),
        Action::RouteNew => e.new_route(),
        Action::RouteSelect(i) => e.choose_route(i),
        Action::RouteSave => e.save_route(),
        Action::RouteAssign => e.assign_route(),
        Action::RouteTab(tab) => {
            let r = e.routes.as_mut().ok_or("Open path editor")?;
            r.assignments = tab;
            e.focus = None;
            e.revision += 1;
            Ok(())
        }
        Action::RouteKind(kind) => {
            let r = e.routes.as_mut().ok_or("Open path editor")?;
            r.kind = kind;
            r.drawing = false;
            r.pointer = None;
            r.points.clear();
            r.point = None;
            r.page = 0;
            e.revision += 1;
            Ok(())
        }
        Action::RoutePage(next) => {
            let pages = e.route_rows().len().div_ceil(5).max(1);
            let r = e.routes.as_mut().ok_or("Open path editor")?;
            r.page = if next {
                (r.page + 1).min(pages - 1)
            } else {
                r.page.saturating_sub(1)
            };
            e.revision += 1;
            Ok(())
        }
        Action::RouteLoop => {
            let r = e.routes.as_mut().ok_or("Open path editor")?;
            r.looped = !r.looped;
            e.revision += 1;
            Ok(())
        }
        Action::RoutePoint(i) => {
            let r = e.routes.as_mut().ok_or("Open path editor")?;
            r.point = Some(i);
            e.revision += 1;
            Ok(())
        }
        Action::RouteRemovePoint => {
            let r = e.routes.as_mut().ok_or("Open path editor")?;
            if let Some(i) = r.point.filter(|i| *i < r.points.len()) {
                r.points.remove(i);
            } else {
                r.points.pop();
            }
            r.point = None;
            e.revision += 1;
            Ok(())
        }
        _ => Err("Unknown path action".into()),
    }
}
pub(super) fn draw(p: &mut ChildSpawnerCommands, f: &EditorFonts, e: &WorldEditor) {
    use view::{MUTED, WHITE, button, field, label, row, value};
    let Some(r) = &e.routes else {
        return;
    };
    label(
        p,
        f,
        LocalizedText::new("ui.editor.world.routes", "Path editor"),
        18.,
        WHITE,
    );
    p.spawn(row()).with_children(|p| {
        button(
            p,
            f,
            Action::RouteTab(false),
            "route_points",
            "Points",
            125.,
            !r.assignments,
        );
        button(
            p,
            f,
            Action::RouteTab(true),
            "route_assignments",
            "Assignments",
            125.,
            r.assignments,
        );
    });
    for (kind, key, en) in [
        (0, "route_npc", "NPC / Mob / Group"),
        (1, "route_skyway", "Skyway"),
        (2, "route_slider", "Slider"),
    ] {
        button(p, f, Action::RouteKind(kind), key, en, 0., r.kind == kind);
    }
    value(p, f, format!("ID {} · {}", r.id, r.points.len()), 14.);
    if r.assignments {
        if r.kind == 0 {
            label(
                p,
                f,
                LocalizedText::new(
                    "ui.editor.world.route_assign_help",
                    "Select an NPC, mob or group on the left, then assign this saved route.",
                ),
                13.,
                MUTED,
            );
        }
        if r.kind == 1 {
            field(
                p,
                f,
                e,
                Field::RouteTarget,
                "route_target",
                r.target.to_string(),
            );
        }
        button(
            p,
            f,
            Action::RouteAssign,
            "route_assign",
            "Assign route",
            0.,
            false,
        );
    } else {
        button(p, f, Action::RouteNew, "route_new", "New route", 0., false);
        if r.drawing {
            label(
                p,
                f,
                LocalizedText::new(
                    "ui.editor.world.route_click_help",
                    "Click in the world to add points. Save the route, then open Assignments.",
                ),
                13.,
                MUTED,
            );
            if r.kind != 2 {
                field(
                    p,
                    f,
                    e,
                    Field::RouteSpeed,
                    "route_speed",
                    r.speed.to_string(),
                );
            }
            if r.kind == 0 {
                button(p, f, Action::RouteLoop, "route_loop", "Loop", 0., r.looped);
            }
            button(
                p,
                f,
                Action::RouteSave,
                "route_save",
                "Save route",
                0.,
                false,
            );
            button(
                p,
                f,
                Action::RouteRemovePoint,
                "route_remove_point",
                "Remove point",
                0.,
                false,
            );
            for (i, point) in r.points.iter().enumerate() {
                button(
                    p,
                    f,
                    Action::RoutePoint(i),
                    "value",
                    &format!(
                        "{} · {}, {}, {}",
                        i + 1,
                        point["iX"],
                        point["iY"],
                        point["iZ"]
                    ),
                    0.,
                    r.point == Some(i),
                );
            }
            if let Some(point) = r.point.and_then(|i| r.points.get(i)) {
                for (field, key, value) in [
                    (Field::RouteX, "X", &point["iX"]),
                    (Field::RouteY, "Y", &point["iY"]),
                    (Field::RouteZ, "Z", &point["iZ"]),
                    (Field::RouteStop, "route_stop", &point["iStopTicks"]),
                ] {
                    view::field(p, f, e, field, key, value.to_string());
                }
            }
        }
        let rows = e.route_rows();
        p.spawn(row()).with_children(|p| {
            button(p, f, Action::RoutePage(false), "previous", "←", 45., false);
            button(p, f, Action::RoutePage(true), "next", "→", 45., false);
            value(
                p,
                f,
                format!("{}/{}", r.page + 1, rows.len().div_ceil(5).max(1)),
                14.,
            );
        });
        for (i, row) in rows.iter().enumerate().skip(r.page * 5).take(5) {
            button(
                p,
                f,
                Action::RouteSelect(i),
                "value",
                &row.label,
                0.,
                r.pointer.as_ref() == Some(&row.pointer),
            );
        }
    }
    if e.focus.is_some() {
        button(p, f, Action::Apply, "apply", "Apply", 0., false);
    }
}
pub(super) fn gizmos(e: Res<WorldEditor>, state: Res<EditorState>, mut gizmos: Gizmos) {
    if state.world_open != Some(true) || e.map_picker {
        return;
    }
    let Some(r) = &e.routes else {
        return;
    };
    let points: Vec<_> = r.points.iter().filter_map(model::native).collect();
    for p in &points {
        gizmos.sphere(
            Isometry3d::from_translation(*p + Vec3::Y * 0.1),
            0.6,
            Color::srgb(0.1, 0.8, 1.),
        );
    }
    for pair in points.windows(2) {
        gizmos.line(
            pair[0] + Vec3::Y * 0.1,
            pair[1] + Vec3::Y * 0.1,
            Color::srgb(0.1, 0.8, 1.),
        );
    }
    if r.looped && points.len() > 1 {
        gizmos.line(
            *points.last().unwrap(),
            points[0],
            Color::srgb(0.1, 0.8, 1.),
        );
    }
}

pub(super) fn draw_2d(p: &mut ChildSpawnerCommands, f: &EditorFonts, e: &WorldEditor, size: Vec2) {
    let Some(r) = &e.routes else {
        return;
    };
    let points: Vec<_> = r
        .points
        .iter()
        .filter_map(model::native)
        .map(|at| view::screen(e, at, size))
        .collect();
    let color = Color::srgb(0.1, 0.8, 1.);
    let mut segments: Vec<_> = points.windows(2).map(|a| (a[0], a[1])).collect();
    if r.looped && points.len() > 1 {
        segments.push((*points.last().unwrap(), points[0]));
    }
    for (a, b) in segments {
        let count = ((b - a).length() / 6.).ceil().clamp(1., 1000.) as usize;
        for i in 0..=count {
            let at = a.lerp(b, i as f32 / count as f32);
            if at.cmpge(Vec2::ZERO).all() && at.cmple(size).all() {
                p.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(at.x - 1.5),
                        top: px(at.y - 1.5),
                        width: px(3),
                        height: px(3),
                        ..default()
                    },
                    BackgroundColor(color),
                ));
            }
        }
    }
    for (i, at) in points.iter().enumerate() {
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(at.x - 5.),
                top: px(at.y - 5.),
                width: px(10),
                height: px(10),
                ..default()
            },
            BackgroundColor(if r.point == Some(i) {
                Color::WHITE
            } else {
                color
            }),
        ));
        p.spawn(Node {
            position_type: PositionType::Absolute,
            left: px(at.x + 7.),
            top: px(at.y - 8.),
            ..default()
        })
        .with_children(|p| view::value(p, f, (i + 1).to_string(), 13.));
    }
}
