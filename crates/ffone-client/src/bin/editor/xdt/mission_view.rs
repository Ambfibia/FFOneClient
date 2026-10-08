//! Mission authoring inside native game journal panels.
use super::*;
use mission_inline::{action, row, stack};
use mission_skin::{BLUE, CYAN, DARK, MUTED, MissionSkin, TEXT, heading};
use mission_workspace::Command;
use view::{button, field, label};

pub(super) fn draw(
    commands: &mut Commands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    skin: &MissionSkin,
    width: f32,
    height: f32,
    scroll: Vec2,
    catalog_scroll: Vec2,
) {
    let tr = |key: &str, en: &str| presentation::tr(l, lang, key, en);
    let table = e.mission_workspace_table().unwrap();
    let selected = e.mission_row().filter(|r| *r < e.mission_rows().len());
    commands
        .spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                top: px(EDITOR_HEADER_HEIGHT),
                bottom: px(0),
                width: percent(100),
                padding: UiRect::axes(px(12), px(10)),
                overflow: Overflow::clip(),
                row_gap: px(10),
                ..stack()
            },
            ZIndex(200),
            BackgroundColor(DARK),
        ))
        .with_children(|root| {
            toolbar(root, f, e, l, lang, width);
            mission_server::destination(root, f, e, l, lang);
            root.spawn(Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_height: px(0),
                min_width: px(0),
                column_gap: px(7),
                ..default()
            })
            .with_children(|body| {
                if !e.workspace.catalog_hidden {
                    body.spawn((
                        panel(e.workspace.catalog_width.clamp(205., width * 0.23)),
                        BackgroundColor(Color::srgb(0.04, 0.065, 0.11)),
                        BorderColor::all(Color::srgb(0.09, 0.15, 0.23)),
                    ))
                    .with_children(|p| catalog(p, f, e, l, lang, skin, catalog_scroll));
                    divider(body, mission_canvas::Hit::CatalogDivider);
                }
                body.spawn(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(220),
                    ..stack()
                })
                .with_children(|center| {
                    center.spawn(row()).with_children(|p| {
                        tab(
                            p,
                            f,
                            Action::GraphMode(true),
                            tr("mission.stage_scheme", "Stage flow"),
                            e.graph_stages,
                        );
                        tab(
                            p,
                            f,
                            Action::GraphMode(false),
                            tr("graph.missions", "Mission chains"),
                            !e.graph_stages,
                        );
                    });
                    if let Some(r) = selected {
                        let value = &e.mission_rows()[r];
                        let count = e
                            .mission_rows()
                            .iter()
                            .filter(|v| v["m_iHMissionID"] == value["m_iHMissionID"])
                            .count();
                        center
                            .spawn((
                                Node {
                                    height: px(72),
                                    min_height: px(72),
                                    padding: UiRect::all(px(12)),
                                    column_gap: px(12),
                                    align_items: AlignItems::Center,
                                    overflow: Overflow::clip(),
                                    border: UiRect::left(px(3)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.045, 0.09, 0.16)),
                                BorderColor::all(BLUE),
                            ))
                            .with_children(|p| {
                                skin.portrait_badge(p, value, 44.);
                                p.spawn(Node {
                                    flex_grow: 1.,
                                    min_width: px(0),
                                    overflow: Overflow::clip(),
                                    ..stack()
                                })
                                .with_children(|p| {
                                    label(p, f, e.record_name(table, r), 20., TEXT);
                                    label(
                                        p,
                                        f,
                                        format!(
                                            "{} ({})   /   {}: {count}",
                                            tr("graph.mission_id", "Mission ID"),
                                            value["m_iHMissionID"],
                                            tr("mission.stages_count", "Stages")
                                        ),
                                        12.,
                                        Color::srgb(0.67, 0.77, 0.9),
                                    );
                                });
                            });
                    }
                    if let Some((_, field)) = &e.workspace.pending_link {
                        label(
                            center,
                            f,
                            tr(
                                if field == mission_graph::REQUIRE { "mission.connect_mission_help" } else { "mission.connect_help" },
                                if field == mission_graph::REQUIRE { "Choose the mission to unlock. Esc cancels." } else { "Choose a target stage or create a new one. Esc cancels." },
                            ),
                            13.,
                            CYAN,
                        );
                    }
                    mission_canvas::draw(center, f, e, l, lang, skin);
                    center.spawn(row()).with_children(|p| {
                        button(
                            p,
                            f,
                            Action::Mission(Command::Zoom(-0.1)),
                            "ui.editor.xdt.graph.minus",
                            "−",
                            false,
                            32.,
                        );
                        mission_canvas::zoom_readout(p, f, e.graph_zoom);
                        button(
                            p,
                            f,
                            Action::Mission(Command::Zoom(0.1)),
                            "ui.editor.xdt.graph.plus",
                            "+",
                            false,
                            32.,
                        );
                        button(
                            p,
                            f,
                            Action::Mission(Command::Fit),
                            "ui.editor.xdt.mission.fit",
                            "Fit all",
                            false,
                            95.,
                        );
                        button(
                            p,
                            f,
                            Action::Mission(Command::Arrange),
                            "ui.editor.xdt.mission.arrange",
                            "Auto layout",
                            false,
                            130.,
                        );
                    });
                    if !e.graph_stages {
                        label(center, f, tr("mission.requirements_help", "All incoming missions must be completed. Other opening conditions still apply."), 12., MUTED);
                        button(
                            center,
                            f,
                            Action::GraphAdd(mission_graph::REQUIRE.into()),
                            "ui.editor.xdt.graph.add",
                            "+ Prerequisite",
                            false,
                            0.,
                        );
                    }
                    if e.workspace.selected_edge.is_some() {
                        center.spawn(row()).with_children(|p| {
                            button(
                                p,
                                f,
                                Action::Mission(Command::RemoveEdge),
                                "ui.editor.xdt.mission.remove_edge",
                                "Remove link",
                                false,
                                135.,
                            );
                            if e.workspace.selected_edge.as_ref().is_some_and(|(_, field, _)| field != mission_graph::REQUIRE) {
                                button(
                                    p,
                                    f,
                                    Action::Mission(Command::InsertStage),
                                    "ui.editor.xdt.mission.insert",
                                    "Insert stage",
                                    false,
                                    135.,
                                );
                            }
                        });
                    }
                });
                divider(body, mission_canvas::Hit::InspectorDivider);
                body.spawn((
                    panel(e.workspace.inspector_width.clamp(310., width * 0.32)),
                    BackgroundColor(Color::srgb(0.04, 0.065, 0.11)),
                    BorderColor::all(Color::srgb(0.09, 0.15, 0.23)),
                ))
                .with_children(|p| {
                    heading(
                        p,
                        f,
                        tr("mission.properties", "STAGE PROPERTIES"),
                        14.,
                        CYAN,
                    );
                    if let Some(r) = selected {
                        label(p, f, mission_canvas::stage_title(e, table, r), 18., TEXT);
                        label(
                            p,
                            f,
                            format!(
                                "{} ({})",
                                tr("mission.stage_id", "Stage ID"),
                                e.mission_rows()[r]["m_iHTaskID"]
                            ),
                            12.,
                            MUTED,
                        );
                    }
                    p.spawn((
                        ScrollRegion(2),
                        ScrollPosition(scroll),
                        Node {
                            flex_grow: 1.,
                            overflow: Overflow::scroll_y(),
                            padding: UiRect::right(px(4)),
                            ..stack()
                        },
                    ))
                    .with_children(|p| {
                        if e.draft.is_some() {
                            mission_inline::draft(p, f, e, l, lang);
                        } else if let Some(r) = selected {
                            if e.focus == Some(Focus::Row) {
                                presentation::edit_panel(p, f, e, l, lang);
                            } else {
                                inspector(p, f, e, l, lang, r);
                            }
                        } else {
                            label(
                                p,
                                f,
                                tr("select_help", "Select a record to edit its parameters."),
                                15.,
                                MUTED,
                            );
                        }
                    });
                });
            });
            root.spawn(Node {
                min_height: px(20),
                flex_shrink: 0.,
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            })
            .with_children(|p| {
                let dirty = e.dirty();
                p.spawn((
                    Node {
                        width: px(6),
                        height: px(6),
                        border_radius: BorderRadius::all(px(3)),
                        flex_shrink: 0.,
                        ..default()
                    },
                    BackgroundColor(if dirty { BLUE } else { CYAN }),
                ));
                label(
                    p,
                    f,
                    if e.status.is_empty() {
                        format!(
                            "{}  ·  {}",
                            tr(
                                if dirty { "unsaved" } else if e.unpublished() { "mission.pending_work" } else { "saved" },
                                if dirty { "Unsaved changes" } else if e.unpublished() { "Saved · Not applied to game" } else { "Saved" }
                            ),
                            tr(
                                "mission.canvas_hint",
                                "Drag stage headers · Wheel to zoom · Right-drag to pan · Right-click for actions"
                            )
                        )
                    } else {
                        schema::status(&e.status, l, lang)
                    },
                    12.,
                    if e.status.is_empty() {
                        MUTED
                    } else {
                        Color::srgb(1., 0.69, 0.45)
                    },
                );
            });
        });
    mission_context::draw(commands, f, e, l, lang, width, height);
}
fn panel(width: f32) -> Node {
    Node {
        width: px(width),
        flex_shrink: 0.,
        padding: UiRect::all(px(12)),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(6)),
        overflow: Overflow::clip(),
        row_gap: px(10),
        ..stack()
    }
}
fn divider(p: &mut ChildSpawnerCommands, hit: mission_canvas::Hit) {
    p.spawn((
        hit,
        Node {
            width: px(4),
            flex_shrink: 0.,
            border_radius: BorderRadius::all(px(2)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.08, 0.16, 0.27)),
    ));
}
fn toolbar(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    width: f32,
) {
    p.spawn(row()).with_children(|p| {
        if width >= 1400. {
            heading(
                p,
                f,
                presentation::tr(l, lang, "mission.workspace", "MISSION STUDIO"),
                18.,
                BLUE,
            );
        }
        button(
            p,
            f,
            Action::Mission(Command::ToggleCatalog),
            "ui.editor.xdt.mission.catalog",
            "Missions",
            false,
            100.,
        );
        p.spawn(Node {
            flex_grow: 1.,
            ..default()
        });
        if e.draft.is_none() {
            for (action, key, title, width) in [
                (
                    Action::Mission(Command::NewMission),
                    "new_mission",
                    "+ New mission",
                    155.,
                ),
                (
                    Action::Mission(Command::NewStage),
                    "mission.add_stage",
                    "+ Stage",
                    112.,
                ),
                (Action::Undo, "undo", "Undo", 78.),
                (Action::Redo, "redo", "Redo", 78.),
                (
                    Action::Mission(Command::Validate),
                    "mission.validate",
                    "Validate",
                    105.,
                ),
                (Action::Save, "mission.save_short", "Save", 115.),
                (Action::Rewrite, "mission.rewrite", "Rewrite", 120.),
            ] {
                button(
                    p,
                    f,
                    action,
                    &format!("ui.editor.xdt.{key}"),
                    title,
                    false,
                    width,
                );
            }
        }
        if e.draft.is_none() {
            button(
                p,
                f,
                Action::Mission(Command::Tables),
                "ui.editor.xdt.mission.tables",
                "Tables",
                false,
                90.,
            );
        }
    });
}
fn tab(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    action: Action,
    title: String,
    selected: bool,
) {
    let fill = if selected {
        Color::srgb(0.06, 0.13, 0.23)
    } else {
        Color::NONE
    };
    p.spawn((
        Button,
        action,
        view::ButtonTint(fill),
        Node {
            flex_grow: 1.,
            flex_basis: px(0),
            min_width: px(0),
            height: px(36),
            padding: UiRect::axes(px(12), px(8)),
            border: UiRect::bottom(px(2)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(fill),
        BorderColor::all(if selected {
            BLUE
        } else {
            Color::srgb(0.10, 0.16, 0.25)
        }),
    ))
    .with_children(|p| label(p, f, title, 14., if selected { TEXT } else { MUTED }));
}
fn catalog(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    skin: &MissionSkin,
    scroll: Vec2,
) {
    let table = e.mission_workspace_table().unwrap();
    heading(
        p,
        f,
        presentation::tr(l, lang, "mission.library", "MISSION LIBRARY"),
        14.,
        CYAN,
    );
    if e.workspace.quick.is_none() {
        field(p, f, e, Focus::Search, 38.);
    }
    let query = e
        .workspace
        .creation
        .first()
        .map_or(e.search.as_str(), |frame| frame.search.as_str());
    let mut groups: BTreeMap<i64, Vec<usize>> = BTreeMap::new();
    for (r, row) in e.mission_rows().iter().enumerate() {
        let Some(id) = row["m_iHMissionID"].as_i64().filter(|id| *id > 0) else {
            continue;
        };
        if e.matches_search(table, r, query) {
            groups.entry(id).or_default().push(r);
        }
    }
    let mut groups: Vec<_> = groups.into_iter().collect();
    for (_, rows) in &mut groups {
        *rows = mission_graph::ordered_stages(e.mission_rows(), rows);
    }
    if !query.is_empty() {
        groups.sort_by_key(|(_, rows)| {
            rows.iter()
                .map(|r| e.search_rank(table, *r, query))
                .min()
                .unwrap_or(3)
        });
    }
    let offset = e
        .workspace
        .catalog_offset
        .min(groups.len().saturating_sub(1) / 60 * 60);
    p.spawn((
        ScrollRegion(6),
        ScrollPosition(scroll),
        Node {
            flex_grow: 1.,
            overflow: Overflow::scroll_y(),
            row_gap: px(8),
            ..stack()
        },
    ))
    .with_children(|p| {
        if groups.is_empty() {
            label(
                p,
                f,
                presentation::tr(l, lang, "mission.no_results", "No matching records"),
                14.,
                MUTED,
            );
        }
        for (id, rows) in groups.iter().skip(offset).take(60) {
            let open = e.workspace.expanded.contains(id) || !query.is_empty();
            let selected = rows.iter().any(|r| Some(*r) == e.mission_row());
            let fill = if selected {
                Color::srgb(0.055, 0.16, 0.29)
            } else {
                Color::srgb(0.035, 0.075, 0.14)
            };
            p.spawn((
                Button,
                Action::Mission(Command::ToggleMission(*id)),
                view::ButtonTint(fill),
                Node {
                    width: percent(100),
                    min_height: px(62),
                    flex_shrink: 0.,
                    padding: UiRect::all(px(8)),
                    column_gap: px(8),
                    border: UiRect::left(px(3)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(if selected {
                    CYAN
                } else {
                    Color::srgb(0.12, 0.23, 0.34)
                }),
                BackgroundColor(fill),
            ))
            .with_children(|p| {
                p.spawn((
                    ImageNode::new(skin.category(&e.mission_rows()[rows[0]])),
                    Node {
                        width: px(24),
                        height: px(24),
                        flex_shrink: 0.,
                        ..default()
                    },
                ));
                p.spawn(Node {
                    flex_grow: 1.,
                    min_width: px(0),
                    ..stack()
                })
                .with_children(|p| {
                    label(p, f, e.record_name(table, rows[0]), 15., TEXT);
                    label(
                        p,
                        f,
                        format!("{}   (ID: {id})", if open { "−" } else { "+" }),
                        12.,
                        MUTED,
                    );
                });
            });
            if open {
                for (number, r) in rows.iter().enumerate() {
                    let selected = Some(*r) == e.mission_row();
                    let fill = if selected {
                        Color::srgb(0.045, 0.13, 0.23)
                    } else {
                        Color::NONE
                    };
                    p.spawn((
                        Button,
                        Action::Mission(Command::Select(*r)),
                        view::ButtonTint(fill),
                        Node {
                            width: percent(100),
                            padding: UiRect::axes(px(10), px(8)),
                            border: UiRect::left(px(1)),
                            flex_shrink: 0.,
                            column_gap: px(10),
                            ..default()
                        },
                        BorderColor::all(if selected {
                            CYAN
                        } else {
                            Color::srgb(0.15, 0.25, 0.38)
                        }),
                        BackgroundColor(fill),
                    ))
                    .with_children(|p| {
                        p.spawn(Node {
                            width: px(20),
                            flex_shrink: 0.,
                            ..default()
                        })
                        .with_children(|p| {
                            label(
                                p,
                                f,
                                format!("{:02}", number + 1),
                                12.,
                                if selected { CYAN } else { MUTED },
                            )
                        });
                        p.spawn(Node {
                            flex_grow: 1.,
                            min_width: px(0),
                            ..stack()
                        })
                        .with_children(|p| {
                            label(
                                p,
                                f,
                                mission_canvas::stage_title(e, table, *r),
                                14.,
                                if selected { TEXT } else { MUTED },
                            );
                            label(
                                p,
                                f,
                                format!("({})", e.mission_rows()[*r]["m_iHTaskID"]),
                                11.,
                                MUTED,
                            );
                        });
                    });
                }
            }
        }
    });
    p.spawn(row()).with_children(|p| {
        button(
            p,
            f,
            Action::Mission(Command::CatalogPage(false)),
            "ui.editor.xdt.previous",
            "←",
            false,
            68.,
        );
        button(
            p,
            f,
            Action::Mission(Command::CatalogPage(true)),
            "ui.editor.xdt.next",
            "→",
            false,
            68.,
        );
        label(
            p,
            f,
            format!("{} / {}", offset / 60 + 1, groups.len().max(1).div_ceil(60)),
            12.,
            MUTED,
        );
    });
}
fn inspector(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    r: usize,
) {
    let tr = |key: &str, en: &str| presentation::tr(l, lang, key, en);
    field(p, f, e, Focus::Columns, 34.);
    p.spawn(row()).with_children(|p| {
        button(
            p,
            f,
            Action::Advanced,
            if e.advanced {
                "ui.editor.xdt.basic"
            } else {
                "ui.editor.xdt.advanced"
            },
            "All fields",
            false,
            152.,
        );
        button(
            p,
            f,
            Action::Mission(Command::Collapse),
            "ui.editor.xdt.mission.collapse",
            "Collapse",
            false,
            112.,
        );
    });
    let value = &e.rows()[r];
    for block in e.authoring_blocks(value) {
        if block.key == "failure" && !e.advanced && !block.configured {
            continue;
        }
        let query = e.column_search.to_lowercase();
        let fields: Vec<_> = block
            .fields
            .iter()
            .filter(|col| {
                query.is_empty()
                    || schema::field_name(col, l, lang)
                        .to_lowercase()
                        .contains(&query)
                    || col.to_lowercase().contains(&query)
            })
            .collect();
        if fields.is_empty() {
            continue;
        }
        let open = e.block_open(&block) || !query.is_empty();
        let summary = if block.key == "identity" {
            ["m_iHMissionType", "m_iHDifficultyType"]
                .iter()
                .filter_map(|key| {
                    schema::value_name(&e.tables[e.table].label, key, &value[*key], l, lang)
                })
                .collect::<Vec<_>>()
                .join(" · ")
        } else if let Some(col) = fields
            .iter()
            .find(|col| !authoring::neutral(&value[col.as_str()]))
        {
            mission_inline::value_label(e, col, &value[col.as_str()], l, lang)
        } else {
            tr("mission.none", "Not configured")
        };
        let title = if block.key == "failure" {
            tr("mission.failure_settings", "If the task is not completed")
        } else if block.key == "success" {
            tr("mission.on_completion", "After completion")
        } else {
            tr(&format!("block.{}", block.key), block.key)
        };
        p.spawn((
            Button,
            Action::Block(block.key.into(), !open),
            view::ButtonTint(Color::srgb(0.035, 0.08, 0.15)),
            Node {
                width: percent(100),
                flex_shrink: 0.,
                min_height: px(55),
                padding: UiRect::all(px(10)),
                border: UiRect::bottom(px(1)),
                ..stack()
            },
            BackgroundColor(Color::srgb(0.035, 0.08, 0.15)),
            BorderColor::all(Color::srgb(0.09, 0.18, 0.3)),
        ))
        .with_children(|p| {
            label(
                p,
                f,
                format!("{}  {title}", if open { "−" } else { "+" }),
                15.,
                if open { CYAN } else { TEXT },
            );
            let short: String = summary.chars().take(46).collect();
            label(
                p,
                f,
                if summary.chars().count() > 46 {
                    format!("{short}…")
                } else {
                    short
                },
                12.,
                MUTED,
            );
        });
        if open {
            for col in fields {
                if matches!(col.as_str(), "m_iHTaskID" | "m_iHMissionID") && !e.advanced {
                    label(
                        p,
                        f,
                        format!("{} ({})", schema::field_name(col, l, lang), value[col]),
                        13.,
                        MUTED,
                    );
                } else {
                    mission_inline::draw_field(p, f, e, l, lang, col, &value[col]);
                }
            }
        }
    }
    for (row, field, reason) in e
        .workspace
        .diagnostics
        .iter()
        .filter(|(row, _, _)| *row == r)
        .take(12)
    {
        let message = schema::status(&schema::task_error(&e.rows()[*row],
            format!("{field}: {reason}")), l, lang);
        let background = Color::srgb(0.035, 0.075, 0.14);
        p.spawn((Button, Action::Cell(*row, field.split('[').next().unwrap_or(field).into()),
            view::ButtonTint(background), BackgroundColor(background),
            Node {
                width: percent(100), min_width: px(0), min_height: px(30), flex_shrink: 0.,
                padding: UiRect::all(px(8)), border_radius: BorderRadius::all(px(4)),
                flex_direction: FlexDirection::Column, ..default()
            }))
            .with_children(|p| label(p, f, format!("! {message}"), 13., TEXT));
    }
    if e.advanced {
        action(p, f, Action::Raw, "raw", "Edit JSON");
        action(p, f, Action::Export, "export", "Export CSV");
        action(p, f, Action::Import, "import", "Import CSV");
    }
}
