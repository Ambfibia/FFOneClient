//! Detailed, bounded stage cards using the game's journal, NanoCom and speech art.
use super::*;
use mission_canvas::Hit;
use mission_preview::{Channel, EVENT_HEIGHT, excerpt, phrase};
use mission_skin::{BLUE, CYAN, MUTED, TEXT};
use mission_workspace::Command;

pub(super) fn draw(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    skin: &mission_skin::MissionSkin,
    node: &mission_scene::CanvasNode,
    number: usize,
    at: Vec2,
) {
    let Some(row) = node.row else {
        return;
    };
    let value = &e.mission_rows()[row];
    let scale = e.graph_zoom;
    let collapsed = e.workspace.collapsed_stages.contains(&node.id);
    let selected = e.workspace.selected.contains(&row) || e.mission_row() == Some(row);
    let first = e
        .mission_rows()
        .iter()
        .position(|candidate| candidate["m_iHMissionID"] == value["m_iHMissionID"])
        == Some(row);
    p.spawn((
        Hit::Node(row),
        Button,
        Node {
            position_type: PositionType::Absolute,
            left: px(at.x),
            top: px(at.y),
            width: px(node.size().x * scale),
            height: px(node.size().y * scale),
            padding: UiRect::all(px(12. * scale)),
            border: UiRect::all(px(if selected { 2. } else { 1. })),
            flex_direction: FlexDirection::Column,
            row_gap: px(8. * scale),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.018, 0.033, 0.075)),
        BorderColor::all(if selected {
            CYAN
        } else {
            Color::srgb(0.12, 0.3, 0.47)
        }),
    ))
    .with_children(|p| {
        let mut header = format!("{:02}  ({})", number + 1, node.id);
        if first {
            header.push_str(&format!(
                " / {}",
                presentation::tr(l, lang, "mission.start", "Start")
            ));
        }
        if value["m_iSUOutgoingTask"].as_i64().unwrap_or(0) == 0 {
            header.push_str(&format!(" / {}", presentation::tr(l, lang, "mission.end", "End")));
        }
        p.spawn(Node {
            width: percent(100),
            height: px(24. * scale),
            flex_shrink: 0.,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|p| {
            p.spawn(Node {
                flex_grow: 1.,
                min_width: px(0),
                ..default()
            })
            .with_children(|p| {
                fixed_text(p, f, header, 20., 12., BLUE, scale);
            });
            p.spawn((
                Hit::Preview(row),
                Node {
                    width: px(126. * scale),
                    flex_shrink: 0.,
                    ..default()
                },
            ))
            .with_children(|p| {
                canvas_button(
                    p,
                    f,
                    Command::ToggleStage(node.id),
                    if collapsed {
                        "mission.expand_stage"
                    } else {
                        "mission.collapse_stage"
                    },
                    if collapsed {
                        "+ Expand"
                    } else {
                        "− Collapse"
                    },
                    24.,
                    scale,
                )
            });
        });
        p.spawn(Node {
            width: percent(100),
            column_gap: px(10. * scale),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|p| {
            skin.portrait_badge(p, value, 48. * scale);
            p.spawn(Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(4. * scale),
                ..default()
            })
            .with_children(|p| {
                let title_id = value["m_iHCurrentObjective"].as_i64().unwrap_or(0);
                let title = mission_preview::string(e, l, lang, title_id);
                p.spawn((Hit::Objective(row),Node{width:percent(100),..default()})).with_children(|p|fixed_text(p,f,excerpt(&title,96),46.,18.,TEXT,scale));
                if collapsed {
                    return;
                }
                let goals = mission_preview::goals(value);
                if goals.is_empty() {
                    fixed_text(
                        p,
                        f,
                        phrase(l, lang, "no_goal", "Objective conditions are not set", &[]),
                        38.,
                        13.,
                        MUTED,
                        scale,
                    );
                }
                for goal in &goals {
                    fixed_text(
                        p,
                        f,
                        mission_preview::goal_text(e, l, lang, goal),
                        mission_preview::goal_height(goal),
                        13.,
                        Color::srgb(0.72, 0.84, 0.95),
                        scale,
                    );
                }
            });
        });
        for event in mission_preview::events(value)
            .into_iter()
            .filter(|_| !collapsed)
        {
            p.spawn((
                Hit::Preview(row),
                Node {
                    width: percent(100),
                    height: px(EVENT_HEIGHT * scale),
                    flex_shrink: 0.,
                    column_gap: px(10. * scale),
                    align_items: AlignItems::Center,
                    ..default()
                },
            ))
            .with_children(|p| {
                if let Some(speaker) = mission_events::speaker(event.field) {
                    p.spawn((Button, Action::Mission(Command::InspectField(row, speaker.into())),
                        Node { flex_shrink: 0., ..default() }))
                        .with_children(|p| skin.speaker_badge(p, event.npc, value, 42. * scale));
                    let index = e.workspace.event_locales.get(&(row, event.field.into())).copied()
                        .unwrap_or(usize::from(lang.effective == "ru"));
                    p.spawn((Button, Action::Mission(Command::EventLocale(row, event.field.into(), 1-index)),
                        Node { width: px(30. * scale), flex_shrink: 0., ..default() }))
                        .with_children(|p| { fixed_text(p, f, if index == 0 { "EN" } else { "RU" }.into(),
                            22., 11., CYAN, scale); });
                } else {
                    skin.speaker_badge(p, event.npc, value, 42. * scale);
                }
                let (image, ink, accent) = match event.channel {
                    Channel::Journal => {
                        let mut image = skin.card();
                        image.color = Color::srgb(0.48, 0.75, 0.98);
                        (image, TEXT, CYAN)
                    }
                    Channel::Message | Channel::Email => (
                        sliced_image(skin.message.clone(), bevy::sprite::BorderRect::all(2.)),
                        TEXT,
                        BLUE,
                    ),
                    Channel::Bubble => (
                        sliced_image(skin.speech.clone(), bevy::sprite::BorderRect::all(6.)),
                        Color::srgb(0.12, 0.11, 0.04),
                        Color::srgb(0.32, 0.27, 0.05),
                    ),
                };
                p.spawn((
                    Button,
                    Action::Mission(if event.channel == Channel::Journal {
                        Command::InspectField(row, event.field.into())
                    } else if event.channel==Channel::Email {Command::EditEmail(row,event.field.into(),event.slot)} else { Command::EditEvent(row, event.field.into()) }),
                    image,
                    Node {
                        height: percent(100),
                        flex_grow: 1.,
                        flex_basis: px(0),
                        min_width: px(0),
                        padding: UiRect::all(px(8. * scale)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(3. * scale),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    fixed_text(
                        p,
                        f,
                        mission_preview::event_heading(l, lang, &event),
                        16.,
                        11.,
                        accent,
                        scale,
                    );
                    fixed_text(
                        p,
                        f,
                        excerpt(&mission_preview::npc_name(e, l, lang, event.npc), 57),
                        14.,
                        10.,
                        accent,
                        scale,
                    );
                    let mut preview_language = lang.clone();
                    if let Some(index) = e.workspace.event_locales.get(&(row, event.field.into())) {
                        preview_language.effective = if *index == 0 { "en" } else { "ru" }.into();
                    }
                    fixed_text(
                        p,
                        f,
                        excerpt(&mission_preview::event_text(e, l, &preview_language, &event), 86),
                        35.,
                        14.,
                        ink,
                        scale,
                    );
                });
            });
        }
        p.spawn(Node {
            flex_grow: 1.,
            min_height: px(0),
            ..default()
        });
        p.spawn(Node {
            width: percent(100),
            height: px(28. * scale),
            column_gap: px(5. * scale),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|p| {
            for (field, key, fallback) in [
                ("m_iSUOutgoingTask", "mission.next_stage", "Next stage →"),
                ("m_iFOutgoingTask", "mission.failure_short", "On failure"),
            ] {
                if field == "m_iFOutgoingTask" && !e.advanced {
                    continue;
                }
                p.spawn((
                    Hit::Port(row, field.into()),
                    Node {
                        width: percent(if e.advanced { 49. } else { 100. }),
                        min_width: px(0),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    canvas_button(
                        p,
                        f,
                        Command::Connect(row, field.into()),
                        key,
                        fallback,
                        28.,
                        scale,
                    )
                });
            }
        });
    });
}

fn fixed_text(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    text: String,
    height: f32,
    size: f32,
    color: Color,
    scale: f32,
) {
    p.spawn(Node {
        width: percent(100),
        height: px(height * scale),
        min_width: px(0),
        flex_shrink: 0.,
        overflow: Overflow::clip(),
        ..default()
    })
    .with_children(|p| {
        let mut bundle = editor_text(
            f,
            "ui.editor.xdt.value",
            "{value}",
            size * scale,
            color,
            false,
        );
        bundle.localized = bundle.localized.with_arg("value", text);
        p.spawn((
            bundle,
            bevy::text::LineHeight::Px(size * scale * 1.25),
            Node {
                width: percent(100),
                min_width: px(0),
                flex_shrink: 0.,
                ..default()
            },
        ));
    });
}

fn canvas_button(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    command: Command,
    key: &str,
    fallback: &str,
    height: f32,
    scale: f32,
) {
    let background = Color::srgb(0.03, 0.075, 0.15);
    let mut image = sliced_image(
        f.button.clone(),
        ffone_client::option_ui::OPTION_BIG_LABEL_BORDER,
    );
    image.color = Color::srgb(0.30, 0.46, 0.63);
    p.spawn((
        Button,
        Action::Mission(command),
        view::ButtonTint(background),
        BackgroundColor(background),
        image,
        Node {
            width: percent(100),
            height: px(height * scale),
            min_width: px(0),
            flex_shrink: 0.,
            padding: UiRect::horizontal(px(8. * scale)),
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
    ))
    .with_children(|p| {
        p.spawn(single_line_text(editor_text(
            f,
            format!("ui.editor.xdt.{key}"),
            fallback,
            12. * scale,
            TEXT,
            false,
        )));
    });
}
