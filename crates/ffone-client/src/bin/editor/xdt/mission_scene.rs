//! Display endpoints are typed: a mission ID can equal an unrelated task ID.
use super::*;
use mission_canvas::NODE_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Key {
    Task(i64),
    Mission(i64),
}

pub(super) struct CanvasNode {
    pub key: Key,
    pub compact: bool,
    pub id: i64,
    pub row: Option<usize>,
    pub position: Vec2,
    pub extent: Vec2,
}
impl CanvasNode {
    pub fn size(&self) -> Vec2 {
        if self.compact {
            Vec2::new(NODE_SIZE.x, 112.)
        } else {
            self.extent
        }
    }
    pub fn layout_key(&self, e: &XdtEditor) -> String {
        if self.compact {
            format!(
                "task-group:{}:mission:{}",
                e.mission_row()
                    .and_then(|r| e.mission_rows()[r]["m_iHMissionID"].as_i64())
                    .unwrap_or(0),
                self.id
            )
        } else {
            e.layout_key(self.id)
        }
    }
}
pub(super) struct CanvasEdge {
    pub from: Key,
    pub to: Key,
    pub link: mission_graph::Edge,
}
pub(super) struct Scene {
    pub nodes: Vec<CanvasNode>,
    pub edges: Vec<CanvasEdge>,
}

pub(super) fn scene(e: &XdtEditor) -> Scene {
    let rows = e.mission_rows();
    let key = if e.graph_stages {
        "m_iHTaskID"
    } else {
        "m_iHMissionID"
    };
    let selected = e.mission_row().and_then(|r| rows.get(r));
    let group = selected.and_then(|v| v["m_iHMissionID"].as_i64());
    if e.graph_stages && group.is_none() {
        return Scene {
            nodes: Vec::new(),
            edges: Vec::new(),
        };
    }
    let mut representatives = BTreeMap::new();
    for (r, value) in rows.iter().enumerate() {
        if e.graph_stages && group.is_some() && value["m_iHMissionID"].as_i64() != group {
            continue;
        }
        if let Some(id) = value[key].as_i64().filter(|n| *n > 0) {
            representatives.entry(id).or_insert(r);
        }
    }
    let edges = if e.graph_stages {
        mission_graph::edges(rows, true)
    } else {
        mission_dependencies::opening_edges(rows)
    };
    let mut visible: BTreeSet<i64> = if e.graph_stages {
        representatives.keys().copied().collect()
    } else {
        group.into_iter().collect()
    };
    if visible.is_empty() {
        visible.extend(representatives.keys().take(20).copied());
    }
    for _ in 0..if e.graph_all { 100 } else { 1 } {
        let before = visible.clone();
        for edge in &edges {
            if before.contains(&edge.from) || !e.graph_stages && before.contains(&edge.to) {
                visible.insert(edge.from);
                visible.insert(edge.to);
            }
        }
        if visible == before {
            break;
        }
    }
    let edges: Vec<_> = edges
        .into_iter()
        .filter(|a| visible.contains(&a.from) && visible.contains(&a.to))
        .collect();
    let mut levels: BTreeMap<i64, usize> = visible.iter().map(|id| (*id, 0)).collect();
    let mut done = BTreeSet::new();
    for _ in 0..visible.len() {
        let ready: Vec<_> = visible
            .iter()
            .filter(|id| {
                !done.contains(*id)
                    && edges
                        .iter()
                        .filter(|a| a.to == **id && a.field != "m_iFOutgoingTask")
                        .all(|a| done.contains(&a.from))
            })
            .copied()
            .collect();
        if ready.is_empty() {
            break;
        }
        for id in ready {
            let depth = edges
                .iter()
                .filter(|a| a.to == id && a.field != "m_iFOutgoingTask")
                .map(|a| levels[&a.from] + 1)
                .max()
                .unwrap_or(0);
            levels.insert(id, depth);
            done.insert(id);
        }
    }
    let extents:BTreeMap<_,_>=visible.iter().map(|id|{
        let size=representatives.get(id).filter(|_|e.graph_stages)
            .map(|row| if e.workspace.collapsed_stages.contains(id) {
                Vec2::new(mission_preview::WIDTH, 152.)
            } else {mission_preview::size(&rows[*row])}).unwrap_or(NODE_SIZE);
        (*id,size)
    }).collect();
    let mut heights:BTreeMap<usize,f32>=BTreeMap::new();
    for id in &visible {let height=heights.entry(levels[id]).or_default();*height=height.max(extents[id].y);}
    let mut offset=28.;
    let offsets:BTreeMap<_,_>=heights.into_iter().map(|(level,height)|{
        let y=offset;offset+=height+90.;(level,y)
    }).collect();
    let mut counts = BTreeMap::new();
    let mut nodes: Vec<CanvasNode> = visible
        .into_iter()
        .map(|id| {
            let level = levels[&id];
            let slot = counts.entry(level).or_insert(0);
            let automatic = Vec2::new(56. + *slot as f32 * (if e.graph_stages {mission_preview::WIDTH}else{NODE_SIZE.x}+34.), offsets[&level]);
            *slot += 1;
            let position = e
                .workspace
                .positions
                .get(&e.layout_key(id))
                .copied()
                .map(Vec2::from)
                .unwrap_or(automatic);
            CanvasNode {
                key: if e.graph_stages {
                    Key::Task(id)
                } else {
                    Key::Mission(id)
                },
                compact: false,
                id,
                row: representatives.get(&id).copied(),
                position,
                extent:extents[&id],
            }
        })
        .collect();
    if e.graph_stages {
        let mut candidates: Vec<_> = representatives.values().copied().collect();
        candidates.sort_unstable();
        let order = mission_graph::ordered_stages(rows, &candidates);
        nodes.sort_by_key(|node| {
            node.row
                .and_then(|row| order.iter().position(|r| *r == row))
                .unwrap_or(usize::MAX)
        });
    }
    let edges = edges
        .into_iter()
        .map(|link| CanvasEdge {
            from: if e.graph_stages {
                Key::Task(link.from)
            } else {
                Key::Mission(link.from)
            },
            to: if e.graph_stages {
                Key::Task(link.to)
            } else {
                Key::Mission(link.to)
            },
            link,
        })
        .collect();
    let mut scene = Scene { nodes, edges };
    if e.graph_stages {
        add_neighbors(e, &mut scene);
    }
    scene
}

fn add_neighbors(e: &XdtEditor, scene: &mut Scene) {
    let rows = e.mission_rows();
    let Some(group) = e
        .mission_row()
        .and_then(|r| rows[r]["m_iHMissionID"].as_i64())
    else {
        return;
    };
    let first = mission_dependencies::first_rows(rows);
    let Some(&entry) = first.get(&group) else {
        return;
    };
    let links = mission_dependencies::opening_edges(rows);
    let incoming: Vec<_> = links.iter().filter(|link| link.to == group).collect();
    let outgoing: Vec<_> = links.iter().filter(|link| link.from == group).collect();
    let min = scene
        .nodes
        .iter()
        .fold(Vec2::splat(f32::INFINITY), |v, n| v.min(n.position));
    let max = scene
        .nodes
        .iter()
        .fold(Vec2::splat(f32::NEG_INFINITY), |v, n| {
            v.max(n.position + n.size())
        });
    let terminals: Vec<_> = scene
        .nodes
        .iter()
        .filter(|n| {
            n.row
                .is_some_and(|r| rows[r]["m_iSUOutgoingTask"].as_i64() == Some(0))
        })
        .map(|n| n.key)
        .collect();
    for (before, links) in [(true, &incoming), (false, &outgoing)] {
        let ids: BTreeSet<_> = links
            .iter()
            .map(|link| if before { link.from } else { link.to })
            .collect();
        for (slot, id) in ids.iter().enumerate() {
            let mut node = CanvasNode {
                key: Key::Mission(*id),
                compact: true,
                id: *id,
                row: first.get(id).copied(),
                extent:Vec2::new(NODE_SIZE.x,112.),
                position: Vec2::new(
                    (min.x + max.x - NODE_SIZE.x) * 0.5
                        + (slot as f32 - (ids.len() - 1) as f32 * 0.5) * 328.,
                    if before { min.y - 178. } else { max.y + 76. },
                ),
            };
            if let Some(position) = e.workspace.positions.get(&node.layout_key(e)) {
                node.position = Vec2::from(*position);
            }
            if !scene.nodes.iter().any(|n| n.key == node.key) {
                scene.nodes.push(node);
            }
        }
        for link in links {
            if before {
                scene.edges.push(CanvasEdge {
                    from: Key::Mission(link.from),
                    to: Key::Task(rows[entry]["m_iHTaskID"].as_i64().unwrap()),
                    link: (*link).clone(),
                });
            } else {
                for &terminal in &terminals {
                    scene.edges.push(CanvasEdge {
                        from: terminal,
                        to: Key::Mission(link.to),
                        link: (*link).clone(),
                    });
                }
            }
        }
    }
}

pub(super) fn draw_neighbor(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    node: &CanvasNode,
    at: Vec2,
) {
    use mission_skin::{BLUE, TEXT};
    let scale = e.graph_zoom;
    let group = e
        .mission_row()
        .and_then(|r| e.mission_rows()[r]["m_iHMissionID"].as_i64());
    let before = mission_dependencies::opening_edges(e.mission_rows())
        .iter()
        .any(|edge| edge.from == node.id && Some(edge.to) == group);
    let mut card = p.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(at.x),
            top: px(at.y),
            width: px(node.size().x * scale),
            height: px(node.size().y * scale),
            padding: UiRect::all(px(10. * scale)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(5)),
            flex_direction: FlexDirection::Column,
            row_gap: px(5. * scale),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.035, 0.085, 0.15)),
        BorderColor::all(Color::srgb(0.14, 0.35, 0.53)),
    ));
    if let Some(row) = node.row {
        card.insert((mission_canvas::Hit::Neighbor(row), Button));
    }
    card.with_children(|p| {
        view::label(
            p,
            f,
            format!(
                "{} ({})",
                presentation::tr(
                    l,
                    lang,
                    if before {
                        "mission.requires_mission"
                    } else {
                        "mission.unlocks_mission"
                    },
                    if before {
                        "Required mission"
                    } else {
                        "Unlocks mission"
                    },
                ),
                node.id
            ),
            (12. * scale).max(10.),
            BLUE,
        );
        let title = node
            .row
            .map(|r| e.record_name(e.mission_workspace_table().unwrap(), r))
            .unwrap_or_else(|| presentation::tr(l, lang, "graph.missing", "Missing record"));
        view::label(p, f, title, (16. * scale).max(11.), TEXT);
    });
}
