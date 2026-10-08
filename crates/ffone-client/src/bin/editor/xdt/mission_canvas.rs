//! Native UI canvas. Layout coordinates never alter task order or game data.
use super::*;
pub(super) use mission_scene::scene;
use mission_workspace::Command;
use view::{button, label};

pub(super) const NODE_SIZE: Vec2 = Vec2::new(300., 174.);
#[derive(Component)]
pub(super) struct CanvasLayer(pub u64);
#[derive(Component)]
pub(super) struct ZoomReadout;
pub(super) fn zoom_readout(p: &mut ChildSpawnerCommands, f: &EditorFonts, zoom: f32) {
    p.spawn((
        Button,
        Action::Mission(Command::ResetZoom),
        Node {
            width: px(58),
            height: px(34),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    ))
    .with_children(|p| {
        let mut text = editor_text(
            f,
            "ui.editor.xdt.value",
            "{value}",
            14.,
            mission_skin::MUTED,
            false,
        );
        text.localized = text
            .localized
            .with_arg("value", format!("{:.0}%", zoom * 100.));
        p.spawn((ZoomReadout, text));
    });
}
#[derive(Component, Clone)]
pub(super) enum Hit {
    Canvas,
    Node(usize),
    Objective(usize),
    Neighbor(usize),
    Port(usize, String),
    Edge(usize, String, usize),
    CatalogDivider,
    InspectorDivider,
    Field(String),
    Preview(usize),
}
pub(super) fn stage_title(e: &XdtEditor, table: usize, row: usize) -> String {
    let value = e
        .document
        .pointer(&e.tables[table].pointer)
        .and_then(|v| v.get(row));
    let text = value.and_then(|v| v["m_iHCurrentObjective"].as_u64());
    text.and_then(|i| {
        relations::rule(&e.tables[table].label, "m_iHCurrentObjective")
            .and_then(|(name, _)| e.tables.iter().position(|t| t.label == name))
            .map(|t| e.record_name(t, i as usize))
    })
    .filter(|s| !s.trim().is_empty())
    .unwrap_or_else(|| e.record_name(table, row))
}

pub(super) fn draw(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    skin: &mission_skin::MissionSkin,
) {
    p.spawn((
        Hit::Canvas,
        Node {
            flex_grow: 1.,
            flex_basis: px(0),
            min_height: px(100),
            width: percent(100),
            overflow: Overflow::clip(),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.025, 0.043, 0.078)),
        BorderColor::all(Color::srgb(0.085, 0.16, 0.25)),
    ))
    .with_children(|p| {
        p.spawn((
            CanvasLayer(e.workspace.canvas_revision),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|p| contents(p, f, e, l, lang, skin));
    });
}

// A gesture redraws only this small layer, retaining the catalog and inspector.
pub(super) fn redraw(
    mut commands: Commands,
    e: Res<XdtEditor>,
    f: Option<Res<EditorFonts>>,
    l: Res<Localization>,
    lang: Res<Language>,
    skin: Res<mission_skin::MissionSkin>,
    mut layers: Query<(Entity, &mut CanvasLayer, &Children)>,
    mut zoom: Query<&mut LocalizedText, With<ZoomReadout>>,
    mut shown_zoom: Local<f32>,
) {
    let Some(f) = f else {
        return;
    };
    if *shown_zoom != e.graph_zoom {
        *shown_zoom = e.graph_zoom;
        for mut text in &mut zoom {
            *text = LocalizedText::new("ui.editor.xdt.value", "{value}")
                .with_arg("value", format!("{:.0}%", e.graph_zoom * 100.));
        }
    }
    for (entity, mut layer, children) in &mut layers {
        if layer.0 == e.workspace.canvas_revision {
            continue;
        }
        layer.0 = e.workspace.canvas_revision;
        for child in children.iter() {
            commands.entity(child).despawn();
        }
        let started = std::time::Instant::now();
        commands
            .entity(entity)
            .with_children(|p| contents(p, &f, &e, &l, &lang, &skin));
        if std::env::var_os("FFONE_MISSION_PROFILE").is_some() {
            eprintln!("mission canvas draw {:?}", started.elapsed());
        }
    }
}

fn route(a: Vec2, b: Vec2, scale: f32) -> Vec<Vec2> {
    let mut points = vec![a];
    if b.y > a.y + 20. {
        let mid = (a.y + b.y) * 0.5;
        points.extend([Vec2::new(a.x, mid), Vec2::new(b.x, mid)]);
    } else {
        let x = a.x.max(b.x) + NODE_SIZE.x * scale * 0.7;
        points.extend([
            Vec2::new(a.x, a.y + 20.),
            Vec2::new(x, a.y + 20.),
            Vec2::new(x, b.y - 28.),
            Vec2::new(b.x, b.y - 28.),
        ]);
    }
    points.push(b);
    points
}
fn segment(
    p: &mut ChildSpawnerCommands,
    a: Vec2,
    b: Vec2,
    thickness: f32,
    color: Color,
    edge: &mission_graph::Edge,
) {
    let delta = b - a;
    let length = delta.length();
    if length < 0.01 {
        return;
    }
    let center = (a + b) * 0.5;
    p.spawn((
        Hit::Edge(edge.row, edge.field.clone(), edge.slot),
        Button,
        Action::Mission(Command::Edge(edge.row, edge.field.clone(), edge.slot)),
        Node {
            position_type: PositionType::Absolute,
            left: px(center.x - length * 0.5),
            top: px(center.y - 6.),
            width: px(length),
            height: px(12),
            align_items: AlignItems::Center,
            ..default()
        },
        UiTransform::from_rotation(Rot2::radians(delta.y.atan2(delta.x))),
        BackgroundColor(Color::NONE),
    ))
    .with_children(|p| {
        p.spawn((
            Node {
                width: percent(100),
                height: px(thickness),
                ..default()
            },
            BackgroundColor(color),
        ));
    });
}
fn arrow_arms(tip: Vec2) -> [Vec2; 2] {
    [tip + Vec2::new(-6., -9.), tip + Vec2::new(6., -9.)]
}

fn contents(
    canvas: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    skin: &mission_skin::MissionSkin,
) {
    use mission_skin::{BLUE, CYAN, MUTED, TEXT};
    let scene = scene(e);
    if scene.nodes.is_empty() {
        canvas
            .spawn(Node {
                position_type: PositionType::Absolute,
                left: percent(10),
                top: percent(45),
                width: percent(80),
                ..default()
            })
            .with_children(|p| {
                label(
                    p,
                    f,
                    presentation::tr(
                        l,
                        lang,
                        "select_help",
                        "Select a record to edit its parameters.",
                    ),
                    16.,
                    MUTED,
                )
            });
        return;
    }
    let table = e.mission_workspace_table().unwrap();
    let scale = e.graph_zoom;
    let tr = |key: &str, en: &str| presentation::tr(l, lang, key, en);
    if let Some((a, b)) = e.workspace.marquee {
        let bounds = Rect::from_corners(a, b);
        canvas.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(bounds.min.x),
                top: px(bounds.min.y),
                width: px(bounds.width()),
                height: px(bounds.height()),
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(CYAN),
            BackgroundColor(Color::srgba(0.15, 0.5, 0.8, 0.12)),
            ZIndex(10),
        ));
    }
    let positions: BTreeMap<_, _> = scene
        .nodes
        .iter()
        .map(|n| (n.key, (n.position * scale + e.workspace.pan, n.size())))
        .collect();
    for drawn in &scene.edges {
        let edge = &drawn.link;
        let Some(a) = positions.get(&drawn.from) else {
            continue;
        };
        let Some(b) = positions.get(&drawn.to) else {
            continue;
        };
        let failure = edge.field == "m_iFOutgoingTask";
        let a = a.0
            + Vec2::new(
                a.1.x
                    * if edge.field == mission_graph::REQUIRE {
                        0.5
                    } else if e.advanced {
                        if failure { 0.75 } else { 0.25 }
                    } else if failure {
                        0.9
                    } else {
                        0.5
                    },
                a.1.y,
            ) * scale;
        let b = b.0 + Vec2::new(b.1.x * 0.5, 0.) * scale - Vec2::Y * 3.;
        let color = if failure {
            Color::srgb(0.9, 0.55, 0.3)
        } else {
            Color::srgb(0.2, 0.66, 0.86)
        };
        let selected = e
            .workspace
            .selected_edge
            .as_ref()
            .is_some_and(|(r, f, s)| *r == edge.row && *f == edge.field && *s == edge.slot);
        let thickness = if selected { 3. } else { 2. };
        let color = if selected { CYAN } else { color };
        for pair in route(a, b, scale).windows(2) {
            segment(canvas, pair[0], pair[1], thickness, color, edge);
        }
        for arm in arrow_arms(b) {
            segment(canvas, arm, b, thickness, color, edge);
        }
    }
    for (number, node) in scene.nodes.iter().enumerate() {
        let at = positions[&node.key].0;
        if node.compact {
            mission_scene::draw_neighbor(canvas, f, e, l, lang, node, at);
            continue;
        }
        if e.graph_stages && node.row.is_some() {
            mission_stage_card::draw(canvas,f,e,l,lang,skin,node,number,at);
            continue;
        }
        let selected = node
            .row
            .is_some_and(|r| e.workspace.selected.contains(&r) || e.mission_row() == Some(r));
        let mut image = skin.card();
        image.color = if selected {
            Color::srgb(0.24, 0.42, 0.65)
        } else {
            Color::srgb(0.15, 0.25, 0.4)
        };
        let mut card = canvas.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(at.x),
                top: px(at.y),
                width: px(NODE_SIZE.x * scale),
                height: px(NODE_SIZE.y * scale),
                border: UiRect::all(px(if selected { 2. } else { 1. })),
                padding: UiRect::all(px(10. * scale)),
                flex_direction: FlexDirection::Column,
                row_gap: px(6. * scale),
                overflow: Overflow::clip(),
                ..default()
            },
            image,
            BackgroundColor(if node.row.is_none() {
                Color::srgb(0.24, 0.07, 0.1)
            } else {
                Color::srgb(0.025, 0.065, 0.13)
            }),
            BorderColor::all(if selected {
                CYAN
            } else {
                Color::srgb(0.11, 0.27, 0.41)
            }),
        ));
        if let Some(row) = node.row {
            card.insert((Hit::Node(row), Button));
        }
        card.with_children(|p| {
            let first = node.row.is_some_and(|r| {
                e.graph_stages
                    && e.mission_rows()
                        .iter()
                        .position(|v| v["m_iHMissionID"] == e.mission_rows()[r]["m_iHMissionID"])
                        == Some(r)
            });
            p.spawn(Node {
                min_height: px(20. * scale),
                flex_shrink: 0.,
                ..default()
            })
            .with_children(|p| {
                label(
                    p,
                    f,
                    format!(
                        "{:02}   ({}){}",
                        number + 1,
                        node.id,
                        if first {
                            format!("  /  {}", tr("mission.start", "Start"))
                        } else {
                            String::new()
                        }
                    ),
                    (12. * scale).max(10.),
                    if first { BLUE } else { CYAN },
                )
            });
            let title = node
                .row
                .map(|r| {
                    if e.graph_stages {
                        stage_title(e, table, r)
                    } else {
                        e.record_name(table, r)
                    }
                })
                .unwrap_or_else(|| tr("graph.missing", "Missing record"));
            p.spawn(Node {
                flex_grow: 1.,
                min_height: px(0),
                column_gap: px(10. * scale),
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|p| {
                if scale >= 0.85 {
                    if let Some(row) = node.row {
                        skin.portrait_badge(p, &e.mission_rows()[row], 44. * scale);
                    }
                }
                p.spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|p| label(p, f, title, (17. * scale).max(11.), TEXT));
            });
            if let Some(row) = node.row.filter(|_| e.graph_stages) {
                let value = &e.mission_rows()[row];
                if scale >= 0.75 {
                    let kind = schema::value_name(
                        &e.tables[table].label,
                        "m_iHTaskType",
                        &value["m_iHTaskType"],
                        l,
                        lang,
                    )
                    .unwrap_or_else(|| display(&value["m_iHTaskType"]));
                    label(p, f, kind, (12. * scale).max(10.), MUTED);
                }
                p.spawn(Node {
                    width: percent(100),
                    column_gap: px(5),
                    flex_shrink: 0.,
                    ..default()
                })
                .with_children(|ports| {
                    ports
                        .spawn((
                            Hit::Port(row, "m_iSUOutgoingTask".into()),
                            Node {
                                width: percent(if e.advanced { 50. } else { 100. }),
                                min_width: px(0),
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            button(
                                p,
                                f,
                                Action::Mission(Command::Connect(row, "m_iSUOutgoingTask".into())),
                                if scale < 0.85 || e.advanced {
                                    "ui.editor.xdt.mission.next_short"
                                } else {
                                    "ui.editor.xdt.mission.next_stage"
                                },
                                "Next stage →",
                                false,
                                0.,
                            )
                        });
                    if e.advanced {
                        ports
                            .spawn((
                                Hit::Port(row, "m_iFOutgoingTask".into()),
                                Node {
                                    width: percent(48),
                                    min_width: px(0),
                                    ..default()
                                },
                            ))
                            .with_children(|p| {
                                button(
                                    p,
                                    f,
                                    Action::Mission(Command::Connect(
                                        row,
                                        "m_iFOutgoingTask".into(),
                                    )),
                                    "ui.editor.xdt.mission.failure_short",
                                    "On failure",
                                    false,
                                    0.,
                                )
                            });
                    }
                });
            } else if let Some(row) = node.row {
                p.spawn((
                    Hit::Port(row, mission_graph::REQUIRE.into()),
                    Node {
                        width: percent(100),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    button(
                        p,
                        f,
                        Action::Mission(Command::Connect(row, mission_graph::REQUIRE.into())),
                        "ui.editor.xdt.mission.link_mission",
                        "Unlock mission →",
                        false,
                        0.,
                    )
                });
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arrow_tip_stays_on_terminal_segment_at_every_zoom_and_direction() {
        for scale in [0.5, 0.85, 1., 1.5] {
            for target in [
                Vec2::new(240., 400.),
                Vec2::new(90., 0.),
                Vec2::new(240., 100.),
            ] {
                let tip = target * scale;
                let points = route(Vec2::new(180., 174.) * scale, tip, scale);
                assert_eq!(points.last(), Some(&tip));
                let last = points[points.len() - 2];
                assert_eq!(last.x, tip.x);
                assert!(last.y < tip.y);
                let arms = arrow_arms(tip);
                assert_eq!((arms[0].x + arms[1].x) * 0.5, tip.x);
                assert!(arms.iter().all(|a| a.y < tip.y));
            }
        }
    }
}
