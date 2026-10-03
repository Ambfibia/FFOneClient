//! Appearance controls and bounded native preview layout.
use super::*;

fn tr(l: &Localization, lang: &Language, key: &str) -> String {
    l.text(
        lang,
        &LocalizedText::new(format!("ui.editor.hnpc.{key}"), key),
    )
}
fn label(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    text: impl Into<String>,
    size: f32,
    color: Color,
) {
    let text = text.into();
    let mut b = editor_text(f, "ui.editor.xdt.value", "{value}", size, color, false);
    b.localized = b.localized.with_arg("value", text);
    p.spawn((
        b,
        Node {
            width: percent(100),
            min_width: px(0),
            flex_shrink: 0.,
            ..default()
        },
    ));
}
fn button(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    action: Action,
    text: String,
    selected: bool,
    width: f32,
) {
    let height = if matches!(
        action,
        Action::Part(_) | Action::Picker(_) | Action::Save(_)
    ) {
        52.
    } else {
        32.
    };
    p.spawn((
        Button,
        action,
        Node {
            width: if width == 0. { percent(100) } else { px(width) },
            min_height: px(height),
            height: px(height),
            flex_shrink: 0.,
            padding: UiRect::all(px(7)),
            border_radius: BorderRadius::all(px(5)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(if selected {
            Color::srgb(0.1, 0.32, 0.33)
        } else {
            Color::srgb(0.08, 0.13, 0.17)
        }),
    ))
    .with_children(|p| label(p, f, text, 15., Color::srgb(0.86, 0.94, 0.95)));
}
fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        min_height: px(0),
        ..default()
    }
}
pub(super) fn draw(
    mut commands: Commands,
    e: Res<HnpcEditor>,
    state: Res<EditorState>,
    f: Option<Res<EditorFonts>>,
    l: Res<Localization>,
    lang: Res<Language>,
    roots: Query<Entity, With<Root>>,
    mut shown: Local<(bool, u64, u32, u32)>,
    scrolls: Query<&ScrollPosition, With<Scroll>>,
    window: Single<&Window>,
) {
    let visible = e.open && !state.xdt_open && !state.strings_open;
    let next = (
        visible,
        e.revision,
        window.width() as u32,
        window.height() as u32,
    );
    if *shown == next && !lang.is_changed() {
        return;
    }
    *shown = next;
    let page_size = if window.height() < 800. { 2 } else { 6 };
    let scroll = scrolls.iter().next().map(|s| s.0).unwrap_or(Vec2::ZERO);
    for root in &roots {
        commands.entity(root).despawn();
    }
    if !visible {
        return;
    }
    let Some(f) = f else {
        return;
    };
    let f = &f;
    let l = &l;
    let lang = &lang;
    commands
        .spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                top: px(EDITOR_HEADER_HEIGHT),
                left: px(0),
                width: px(EDITOR_CATALOG_WIDTH + 28.),
                bottom: px(0),
                padding: UiRect::all(px(14)),
                overflow: Overflow::clip(),
                ..column()
            },
            ZIndex(250),
            BackgroundColor(Color::srgb(0.035, 0.065, 0.085)),
        ))
        .with_children(|p| {
            label(p, f, tr(l, lang, "title"), 22., Color::WHITE);
            button(p, f, Action::Back, tr(l, lang, "return"), false, 0.);
            p.spawn((
                Scroll,
                RelativeCursorPosition::default(),
                ScrollPosition(scroll),
                Node {
                    flex_grow: 1.,
                    overflow: Overflow::scroll_y(),
                    ..column()
                },
            ))
            .with_children(|p| {
                p.spawn(Node {
                    column_gap: px(5),
                    ..default()
                })
                .with_children(|p| {
                    for gender in ["male", "female"] {
                        button(
                            p,
                            f,
                            Action::Gender(gender.into()),
                            tr(l, lang, gender),
                            e.draft["gender"].as_str() == Some(gender),
                            125.,
                        );
                    }
                });
                for (field, count) in [("height", 5), ("shape", 3)] {
                    label(p, f, tr(l, lang, field), 16., Color::WHITE);
                    p.spawn(Node {
                        column_gap: px(4),
                        ..default()
                    })
                    .with_children(|p| {
                        for n in 0..count {
                            button(
                                p,
                                f,
                                Action::Select(field.into(), n),
                                format!("{}", n + 1),
                                e.draft[field].as_i64() == Some(n),
                                42.,
                            );
                        }
                    });
                }
                for (field, palette) in [("skinColor", "skin"), ("hairColor", "hair")] {
                    label(p, f, tr(l, lang, field), 16., Color::WHITE);
                    p.spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        row_gap: px(4),
                        column_gap: px(4),
                        ..default()
                    })
                    .with_children(|p| {
                        p.spawn((
                            Button,
                            Action::Select(field.into(), -1),
                            Node {
                                width: px(23),
                                height: px(23),
                                border: UiRect::all(px(
                                    if e.draft[field].as_i64().is_some_and(|v| v < 0 || v >= 20) {
                                        3.
                                    } else {
                                        1.
                                    },
                                )),
                                border_radius: BorderRadius::all(px(4)),
                                ..default()
                            },
                            BorderColor::all(Color::srgb(0.4, 0.75, 0.8)),
                            BackgroundColor(Color::WHITE),
                        ));
                        for (index, color) in e.palette[palette]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .enumerate()
                        {
                            let selected = e.draft[field].as_u64() == Some(index as u64);
                            p.spawn((
                                Button,
                                Action::Select(field.into(), index as i64),
                                Node {
                                    width: px(23),
                                    height: px(23),
                                    border: UiRect::all(px(if selected { 3. } else { 1. })),
                                    border_radius: BorderRadius::all(px(4)),
                                    ..default()
                                },
                                BorderColor::all(if selected {
                                    Color::WHITE
                                } else {
                                    Color::srgb(0.2, 0.25, 0.3)
                                }),
                                BackgroundColor(Color::LinearRgba(LinearRgba::new(
                                    color[0].as_f64().unwrap_or(0.) as f32,
                                    color[1].as_f64().unwrap_or(0.) as f32,
                                    color[2].as_f64().unwrap_or(0.) as f32,
                                    1.,
                                ))),
                            ));
                        }
                    });
                }
                for kind in [
                    "face",
                    "hair",
                    "shirt",
                    "pants",
                    "shoes",
                    "hat",
                    "glasses",
                    "back",
                    "rightWeapon",
                ] {
                    let part = e.draft["parts"].as_array().and_then(|parts| {
                        parts
                            .iter()
                            .find(|part| part["kind"].as_str() == Some(kind))
                    });
                    let caption = part
                        .and_then(|part| e.variants.iter().find(|v| v.value == *part))
                        .map(|v| v.caption.clone())
                        .unwrap_or_else(|| tr(l, lang, "none"));
                    label(p, f, tr(l, lang, kind), 15., Color::srgb(0.65, 0.8, 0.83));
                    button(
                        p,
                        f,
                        Action::Picker(kind.into()),
                        caption,
                        e.picker.as_deref() == Some(kind),
                        0.,
                    );
                }
            });
        });
    commands
        .spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                top: px(EDITOR_HEADER_HEIGHT),
                right: px(0),
                width: px(EDITOR_INSPECTOR_WIDTH + 28.),
                bottom: px(0),
                padding: UiRect::all(px(14)),
                overflow: Overflow::clip(),
                ..column()
            },
            ZIndex(250),
            BackgroundColor(Color::srgb(0.035, 0.065, 0.085)),
        ))
        .with_children(|p| {
            label(
                p,
                f,
                tr(l, lang, "preview_help"),
                15.,
                Color::srgb(0.65, 0.8, 0.83),
            );
            p.spawn(Node {
                column_gap: px(6),
                ..default()
            })
            .with_children(|p| {
                button(p, f, Action::Undo, tr(l, lang, "undo"), false, 130.);
                button(p, f, Action::Redo, tr(l, lang, "redo"), false, 130.);
            });
            if let Some(kind) = &e.picker {
                label(p, f, tr(l, lang, kind), 20., Color::WHITE);
                p.spawn((
                    Button,
                    Action::Search,
                    Node {
                        height: px(34),
                        min_height: px(34),
                        width: percent(100),
                        flex_shrink: 0.,
                        overflow: Overflow::clip(),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(5)),
                        ..default()
                    },
                    BorderColor::all(if e.filter_focused {
                        Color::srgb(0.4, 0.85, 0.8)
                    } else {
                        Color::srgb(0.18, 0.27, 0.3)
                    }),
                    BackgroundColor(Color::srgb(0.025, 0.04, 0.06)),
                ))
                .with_children(|p| {
                    let mut text = single_line_text(editor_text(
                        f,
                        "ui.editor.xdt.value",
                        "{value}",
                        15.,
                        Color::WHITE,
                        false,
                    ));
                    text.localized = text.localized.with_arg(
                        "value",
                        if e.filter.is_empty() && !e.filter_focused {
                            tr(l, lang, "search")
                        } else {
                            e.filter.clone()
                        },
                    );
                    p.spawn((
                        text,
                        SearchText,
                        ffone_client::text_edit::EditVisual {
                            edit: e.filter_edit.clone(),
                            active: e.filter_focused,
                            inset: 8.,
                            ..default()
                        },
                        Node {
                            height: px(32),
                            padding: UiRect::horizontal(px(8)),
                            flex_shrink: 0.,
                            ..default()
                        },
                    ))
                    .with_children(ffone_client::text_edit::spawn_decorations);
                });
                let choices = e.choices(kind);
                let start = e.page.min(choices.len().saturating_sub(1) / page_size) * page_size;
                for index in choices.iter().skip(start).take(page_size) {
                    button(
                        p,
                        f,
                        Action::Part(*index),
                        e.variants[*index].caption.clone(),
                        e.draft["parts"]
                            .as_array()
                            .is_some_and(|parts| parts.contains(&e.variants[*index].value)),
                        0.,
                    );
                }
                p.spawn(Node {
                    column_gap: px(6),
                    ..default()
                })
                .with_children(|p| {
                    button(p, f, Action::Page(false), "←".into(), false, 45.);
                    p.spawn(Node {
                        width: px(160),
                        flex_shrink: 0.,
                        ..default()
                    })
                    .with_children(|p| {
                        label(
                            p,
                            f,
                            format!(
                                "{} / {}",
                                start / page_size + 1,
                                choices.len().div_ceil(page_size).max(1)
                            ),
                            14.,
                            Color::WHITE,
                        )
                    });
                    button(p, f, Action::Page(true), "→".into(), false, 45.);
                });
                if matches!(kind.as_str(), "hat" | "glasses" | "back" | "rightWeapon") {
                    button(
                        p,
                        f,
                        Action::Remove(kind.clone()),
                        tr(l, lang, "remove"),
                        false,
                        0.,
                    );
                }
            }
            p.spawn(Node {
                flex_grow: 1.,
                ..default()
            });
            if !e.status.is_empty() {
                label(p, f, &e.status, 14., Color::srgb(1., 0.62, 0.4));
            }
            label(
                p,
                f,
                format!(
                    "{} {} · {} {}",
                    tr(l, lang, "appearance_id"),
                    e.source,
                    tr(l, lang, "users"),
                    e.users
                ),
                14.,
                Color::srgb(0.65, 0.8, 0.83),
            );
            label(
                p,
                f,
                tr(l, lang, "save_help"),
                14.,
                Color::srgb(0.65, 0.8, 0.83),
            );
            button(
                p,
                f,
                Action::Save(false),
                tr(l, lang, "save_private"),
                true,
                0.,
            );
            button(
                p,
                f,
                Action::Save(true),
                tr(l, lang, "save_shared"),
                false,
                0.,
            );
        });
}
