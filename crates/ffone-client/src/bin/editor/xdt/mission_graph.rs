//! Mission prerequisites use mission IDs; stage transitions use task IDs.
use super::presentation::tr;
use super::view::{button, dynamic_button, label};
use super::*;

pub(super) const REQUIRE: &str = "m_iCSTReqMission";
#[derive(Clone, Debug)]
pub(super) struct Edge {
    pub from: i64,
    pub to: i64,
    pub row: usize,
    pub field: String,
    pub slot: usize,
}
// Presentation follows the normal chain without reordering the native task array.
pub(super) fn ordered_stages(rows:&[Value],candidates:&[usize])->Vec<usize>{
    let by_id:BTreeMap<_,_>=candidates.iter().filter_map(|r|rows[*r]["m_iHTaskID"].as_i64().map(|id|(id,*r))).collect();
    let incoming:BTreeSet<_>=candidates.iter().filter_map(|r|rows[*r]["m_iSUOutgoingTask"].as_i64())
        .filter_map(|id|by_id.get(&id).copied()).collect();
    let mut visited=BTreeSet::new();let mut order=Vec::new();
    for &start in candidates.iter().filter(|r|!incoming.contains(*r)).chain(candidates.iter()){
        let mut current=start;
        while visited.insert(current){
            order.push(current);
            let next=rows[current]["m_iSUOutgoingTask"].as_i64().filter(|id|*id>0).and_then(|id|by_id.get(&id));
            let Some(&next)=next else{break;};
            current=next;
        }
    }
    order
}
pub(super) fn edges(rows: &[Value], stages: bool) -> Vec<Edge> {
    let mut edges = Vec::new();
    for (row, value) in rows.iter().enumerate() {
        let key = if stages {
            "m_iHTaskID"
        } else {
            "m_iHMissionID"
        };
        let Some(id) = value[key].as_i64().filter(|v| *v > 0) else {
            continue;
        };
        if stages {
            for field in ["m_iSUOutgoingTask", "m_iFOutgoingTask"] {
                if let Some(next) = value[field].as_i64().filter(|v| *v > 0) {
                    edges.push(Edge {
                        from: id,
                        to: next,
                        row,
                        field: field.into(),
                        slot: 0,
                    });
                }
            }
        } else {
            for (slot, required) in value[REQUIRE].as_array().into_iter().flatten().enumerate() {
                if let Some(required) = required.as_i64().filter(|v| *v > 0) {
                    edges.push(Edge {
                        from: required,
                        to: id,
                        row,
                        field: REQUIRE.into(),
                        slot,
                    });
                }
            }
        }
    }
    edges
}
fn reachable(edges: &[Edge], from: i64, to: i64) -> bool {
    let mut seen = BTreeSet::new();
    let mut pending = vec![from];
    while let Some(id) = pending.pop() {
        if id == to {
            return true;
        }
        if seen.insert(id) {
            pending.extend(edges.iter().filter(|e| e.from == id).map(|e| e.to));
        }
    }
    false
}
pub(super) fn validate(before: &[Value], after: &[Value]) -> Result<(), String> {
    let old_tasks=edges(before,true);
    let new_tasks=edges(after,true);
    let success:Vec<_>=new_tasks.iter().filter(|e|e.field=="m_iSUOutgoingTask").cloned().collect();
    for edge in &new_tasks {
        if old_tasks.iter().any(|old|old.from==edge.from&&old.to==edge.to&&old.field==edge.field){continue;}
        let from=after.iter().find(|r|r["m_iHTaskID"].as_i64()==Some(edge.from));
        let to=after.iter().find(|r|r["m_iHTaskID"].as_i64()==Some(edge.to));
        if let (Some(from),Some(to))=(from,to) {
            if from["m_iHMissionID"]!=to["m_iHMissionID"] {
                return Err(schema::task_error(from, "Transitions must stay within the same mission"));
            }
        }
        if edge.field=="m_iSUOutgoingTask"&&reachable(&success,edge.to,edge.from) {
            return Err(schema::task_error(&after[edge.row], "Success transitions cannot create a cycle"));
        }
    }
    let old = edges(before, false);
    let new = edges(after, false);
    for edge in &new {
        if !old.iter().any(|e| e.from == edge.from && e.to == edge.to)
            && reachable(&new, edge.to, edge.from)
        {
            return Err(schema::task_error(&after[edge.row], "Mission prerequisite would create a cycle"));
        }
    }
    Ok(())
}
impl XdtEditor {
    pub(super) fn graph_add(&mut self, field: &str) -> Result<(), String> {
        if field == REQUIRE && self.mission_workspace_table().is_some() {
            if let Some(id) = self.row.and_then(|r| self.rows()[r]["m_iHMissionID"].as_i64()) {
                self.row = mission_dependencies::first_rows(self.rows()).get(&id).copied();
            }
        }
        let value = self
            .row
            .and_then(|r| self.rows().get(r))
            .ok_or("Select a mission task")?;
        if field == REQUIRE {
            let slot = value[field]
                .as_array()
                .ok_or("Missing prerequisite slots")?
                .iter()
                .position(|v| v.as_i64() == Some(0))
                .ok_or("All prerequisite slots are occupied")?;
            self.begin_element(field.into(), slot)?;
        } else {
            self.begin(Focus::Cell(field.into()));
        }
        Ok(())
    }
    pub(super) fn graph_remove(
        &mut self,
        row: usize,
        field: &str,
        slot: usize,
    ) -> Result<(), String> {
        let mut rows = self.rows().to_vec();
        if field == REQUIRE {
            mission_dependencies::clear(&mut rows, row, slot)?;
        } else if matches!(field, "m_iSUOutgoingTask" | "m_iFOutgoingTask") {
            let value = rows.get_mut(row).ok_or("Row no longer exists")?;
            value[field] = Value::from(0);
        } else {
            return Err("Unsupported graph edge".into());
        }
        self.change(rows)
    }
}

#[derive(Default)]
pub(super) struct GraphScroll {
    pub context: Option<(usize, Option<usize>, bool, u32)>,
    pub position: Vec2,
}
pub(super) fn draw(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    position: Vec2,
) {
    let stages = e.graph_stages;
    let key = if stages {
        "m_iHTaskID"
    } else {
        "m_iHMissionID"
    };
    let selected = e
        .row
        .and_then(|r| e.rows().get(r))
        .and_then(|r| r[key].as_i64());
    p.spawn(Node {
        column_gap: px(6),
        flex_wrap: FlexWrap::Wrap,
        ..default()
    })
    .with_children(|bar| {
        button(
            bar,
            f,
            Action::GraphMode(false),
            "ui.editor.xdt.graph.missions",
            "Mission chains",
            !stages,
            175.,
        );
        button(
            bar,
            f,
            Action::GraphMode(true),
            "ui.editor.xdt.graph.stages",
            "Stage transitions",
            stages,
            175.,
        );
        dynamic_button(bar, f, Action::GraphZoom(-0.15), "−".into(), false, 35.);
        dynamic_button(bar, f, Action::GraphZoom(0.15), "+".into(), false, 35.);
        button(
            bar,
            f,
            Action::GraphAll,
            if e.graph_all {
                "ui.editor.xdt.graph.nearby"
            } else {
                "ui.editor.xdt.graph.entire"
            },
            "Entire chain",
            e.graph_all,
            150.,
        );
    });
    label(
        p,
        f,
        tr(
            l,
            lang,
            if stages {
                "graph.stage_help"
            } else {
                "graph.help"
            },
            "Arrow: completed prerequisite → unlocked mission. Wheel: vertical; Shift+wheel: horizontal.",
        ),
        14.,
        Color::srgb(0.65, 0.76, 0.8),
    );
    let all = edges(e.rows(), stages);
    let mut representatives = BTreeMap::new();
    for (i, row) in e.rows().iter().enumerate() {
        if let Some(id) = row[key].as_i64().filter(|id| *id > 0) {
            representatives.entry(id).or_insert(i);
        }
    }
    if let (Some(id), Some(row)) = (selected, e.row) {
        representatives.insert(id, row);
    }
    let mut visible = BTreeSet::new();
    if let Some(id) = selected {
        visible.insert(id);
        // Show the neighborhood of the selected node. Selecting a node continues exploration.
        for _ in 0..if e.graph_all {
            representatives.len().max(1)
        } else {
            2
        } {
            let mut next = visible.clone();
            for edge in &all {
                if visible.contains(&edge.from) || visible.contains(&edge.to) {
                    next.insert(edge.from);
                    next.insert(edge.to);
                }
            }
            if next == visible {
                break;
            }
            visible = next;
        }
    }
    if visible.len() > 100 && !e.graph_all {
        let mut limited = BTreeSet::new();
        if let Some(id) = selected {
            limited.insert(id);
        }
        for edge in &all {
            if Some(edge.from) == selected || Some(edge.to) == selected {
                limited.insert(edge.from);
                limited.insert(edge.to);
            }
        }
        visible = limited;
    }
    let links: Vec<_> = all
        .iter()
        .filter(|a| visible.contains(&a.from) && visible.contains(&a.to))
        .collect();
    label(
        p,
        f,
        format!(
            "{}: {}",
            tr(l, lang, "graph.nodes", "Visible nodes"),
            visible.len()
        ),
        12.,
        Color::srgb(0.65, 0.76, 0.8),
    );
    let mut levels: BTreeMap<i64, usize> = visible.iter().map(|id| (*id, 0)).collect();
    let mut completed = BTreeSet::new();
    for _ in 0..visible.len() {
        let ready: Vec<_> = visible
            .iter()
            .filter(|id| {
                !completed.contains(*id)
                    && links
                        .iter()
                        .filter(|edge| edge.to == **id)
                        .all(|edge| completed.contains(&edge.from))
            })
            .copied()
            .collect();
        if ready.is_empty() {
            break;
        }
        for id in ready {
            let level = links
                .iter()
                .filter(|edge| edge.to == id)
                .map(|edge| levels[&edge.from] + 1)
                .max()
                .unwrap_or(0);
            levels.insert(id, level);
            completed.insert(id);
        }
    }
    let zoom = e.graph_zoom;
    let mut counts = BTreeMap::new();
    let mut positions = BTreeMap::new();
    for id in &visible {
        let level = levels[id];
        let slot = counts.entry(level).or_insert(0);
        positions.insert(
            *id,
            Vec2::new(
                (24. + level as f32 * 300.) * zoom,
                (24. + *slot as f32 * 145.) * zoom,
            ),
        );
        *slot += 1;
    }
    let width = positions
        .values()
        .map(|p| p.x + 260. * zoom)
        .fold(500., f32::max);
    let height = positions
        .values()
        .map(|p| p.y + 130. * zoom)
        .fold(300., f32::max);
    p.spawn((
        ScrollRegion(5),
        ScrollPosition(position),
        Node {
            flex_grow: 1.,
            min_height: px(200),
            overflow: Overflow::scroll(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.025, 0.04, 0.055)),
    ))
    .with_children(|clip| {
        clip.spawn(Node {
            width: px(width),
            min_width: px(width),
            height: px(height),
            min_height: px(height),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|canvas| {
            let mut painted = BTreeSet::new();
            for edge in &links {
                if !painted.insert((edge.from, edge.to, edge.field.as_str())) {
                    continue;
                }
                let color = if edge.field == "m_iFOutgoingTask" {
                    Color::srgb(0.85, 0.48, 0.35)
                } else {
                    Color::srgb(0.28, 0.7, 0.68)
                };
                let a = positions[&edge.from] + Vec2::new(230. * zoom, 48. * zoom);
                let b = positions[&edge.to] + Vec2::new(0., 48. * zoom);
                let mid = (a.x + b.x) * 0.5;
                for (x, y, w, h) in [
                    (a.x.min(mid), a.y, (a.x - mid).abs(), 2.),
                    (mid, a.y.min(b.y), 2., (a.y - b.y).abs()),
                    (mid.min(b.x), b.y, (mid - b.x).abs(), 2.),
                ] {
                    canvas.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(x),
                            top: px(y),
                            width: px(w.max(2.)),
                            height: px(h.max(2.)),
                            ..default()
                        },
                        BackgroundColor(color),
                    ));
                }
                canvas
                    .spawn(Node {
                        position_type: PositionType::Absolute,
                        left: px(b.x - 14.),
                        top: px(b.y - 10.),
                        width: px(20),
                        ..default()
                    })
                    .with_children(|arrow| label(arrow, f, "▶", 16., color));
            }
            for id in &visible {
                let at = positions[id];
                let current = Some(*id) == selected;
                let missing = !representatives.contains_key(id);
                let tint = if missing {
                    Color::srgb(0.32, 0.12, 0.1)
                } else if current {
                    Color::srgb(0.1, 0.3, 0.32)
                } else {
                    Color::srgb(0.075, 0.12, 0.16)
                };
                let mut node = canvas.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(at.x),
                        top: px(at.y),
                        width: px(230. * zoom),
                        height: px(100. * zoom),
                        padding: UiRect::all(px(10. * zoom)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(8)),
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(tint),
                    view::ButtonTint(tint),
                    BorderColor::all(if current {
                        Color::srgb(0.4, 0.85, 0.8)
                    } else {
                        Color::srgb(0.18, 0.27, 0.3)
                    }),
                ));
                if let Some(row) = representatives.get(id) {
                    node.insert((Button, Action::GraphNode(*row)));
                }
                node.with_children(|n| {
                    label(
                        n,
                        f,
                        format!(
                            "{} {}",
                            tr(
                                l,
                                lang,
                                if stages {
                                    "graph.task_id"
                                } else {
                                    "graph.mission_id"
                                },
                                "ID"
                            ),
                            id
                        ),
                        13. * zoom,
                        Color::srgb(0.52, 0.8, 0.8),
                    );
                    let title = representatives
                        .get(id)
                        .map(|r| e.record_name(e.table, *r))
                        .unwrap_or_else(|| tr(l, lang, "graph.missing", "Missing record"));
                    label(n, f, title, 16. * zoom, Color::WHITE);
                    if !completed.contains(id) {
                        label(
                            n,
                            f,
                            tr(l, lang, "graph.cycle", "Cycle / repeated stage"),
                            12. * zoom,
                            Color::srgb(1., 0.65, 0.4),
                        );
                    }
                });
            }
        });
    });
    if e.row.is_some() {
        p.spawn(Node {
            column_gap: px(6),
            flex_wrap: FlexWrap::Wrap,
            ..default()
        })
        .with_children(|bar| {
            if stages {
                button(
                    bar,
                    f,
                    Action::GraphAdd("m_iSUOutgoingTask".into()),
                    "ui.editor.xdt.graph.success",
                    "Success →",
                    false,
                    160.,
                );
                button(
                    bar,
                    f,
                    Action::GraphAdd("m_iFOutgoingTask".into()),
                    "ui.editor.xdt.graph.failure",
                    "Failure →",
                    false,
                    160.,
                );
            } else {
                button(
                    bar,
                    f,
                    Action::GraphAdd(REQUIRE.into()),
                    "ui.editor.xdt.graph.add",
                    "+ Prerequisite",
                    false,
                    210.,
                );
            }
        });
        for edge in links.iter().filter(|edge| Some(edge.row) == e.row) {
            let text = format!("×  {} → {}", edge.from, edge.to);
            dynamic_button(
                p,
                f,
                Action::GraphRemove(edge.row, edge.field.clone(), edge.slot),
                text,
                false,
                0.,
            );
        }
    }
}
