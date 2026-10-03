use super::*;

pub(super) fn draw(
    mut commands: Commands,
    fonts: Option<Res<EditorFonts>>,
    state: Res<EditorState>,
    mut editor: ResMut<StringEditor>,
    roots: Query<Entity, With<StringsRoot>>,
    mut shown: Local<(bool, u64)>,
) {
    if *shown == (state.strings_open, editor.revision) {
        return;
    }
    let Some(fonts) = fonts else {
        return;
    };
    for root in &roots {
        commands.entity(root).despawn();
    }
    *shown = (state.strings_open, editor.revision);
    if !state.strings_open {
        return;
    }
    if editor.filter_dirty || editor.filter_query != editor.tools.filter {
        editor.filtered = editor.keys();
        editing_tools::sort_rows(&mut editor);
        editor.row_indices = editor
            .filtered
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, key)| (key, i))
            .collect();
        editor.filter_query = editor.tools.filter.clone();
        editor.filter_dirty = false;
        editor.metrics_dirty = true;
    }
    if editor.metrics_dirty {
        let width = editor.viewport.x.max(900.);
        let mut starts = Vec::with_capacity(editor.filtered.len() + 1);
        starts.push(0.);
        for key in &editor.filtered {
            let height = estimated_height(&editor.en[key], &editor.draft[key], width);
            starts.push(starts.last().unwrap() + height);
        }
        editor.row_starts = starts;
        editor.metrics_dirty = false;
    }
    if let Some(key) = editor.edited_metric.take() {
        if let Some(index) = editor.row_indices.get(&key).copied() {
            let next = estimated_height(
                &editor.en[&key],
                &editor.draft[&key],
                editor.viewport.x.max(900.),
            );
            let previous = editor.row_starts[index + 1] - editor.row_starts[index];
            for start in &mut editor.row_starts[index + 1..] {
                *start += next - previous;
            }
        }
    }
    if editor.tools.pending_find {
        editor.tools.pending_find = false;
        editing_tools::find_next(&mut editor, false);
    }
    let total = editor.row_starts.last().copied().unwrap_or(0.);
    let height = editor.viewport.y.max(400.);
    editor.offset = editor.offset.min((total - height).max(0.));
    let range = visible_range(&editor.row_starts, editor.offset, height);
    commands
        .spawn((
            StringsRoot,
            Node {
                position_type: PositionType::Absolute,
                top: px(EDITOR_HEADER_HEIGHT),
                bottom: px(0),
                width: percent(100),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                overflow: Overflow::clip(),
                ..default()
            },
            ZIndex(200),
            BackgroundColor(Color::srgb(0.012, 0.035, 0.05)),
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: percent(100),
                column_gap: px(12),
                min_height: px(38),
                ..default()
            })
            .with_children(|bar| {
                editing_tools::field(bar, &fonts, &editor, 2);
                let (key, label) = editor.tools.filter_label();
                button(bar, &fonts, Action::FilterMode, key, label);
                button(
                    bar,
                    &fonts,
                    Action::Save,
                    "ui.editor.strings.save_simple",
                    "Save",
                );
            });
            if editor.tools.find_open {
                root.spawn(Node {
                    width: percent(100),
                    column_gap: px(12),
                    ..default()
                })
                .with_children(|bar| {
                    editing_tools::field(bar, &fonts, &editor, 0);
                    button(
                        bar,
                        &fonts,
                        Action::Find(true),
                        "ui.editor.strings.previous_match",
                        "Previous",
                    );
                    button(
                        bar,
                        &fonts,
                        Action::Find(false),
                        "ui.editor.strings.next_match",
                        "Next",
                    );
                    button(
                        bar,
                        &fonts,
                        Action::CloseReplace,
                        "ui.editor.strings.close_replace",
                        "Close",
                    );
                });
            }
            if editor.tools.replace_open {
                root.spawn(Node {
                    width: percent(100),
                    column_gap: px(12),
                    ..default()
                })
                .with_children(|bar| {
                    editing_tools::field(bar, &fonts, &editor, 1);
                    button(
                        bar,
                        &fonts,
                        Action::ReplaceOne,
                        "ui.editor.strings.replace_one",
                        "Replace",
                    );
                    button(
                        bar,
                        &fonts,
                        Action::ReplaceAll,
                        "ui.editor.strings.replace_visible",
                        "Replace all in filtered rows",
                    );
                    button(
                        bar,
                        &fonts,
                        Action::CloseReplace,
                        "ui.editor.strings.close_replace",
                        "Close",
                    );
                });
            }
            if editor.tools.replace_open {
                root.spawn(Node {
                    width: percent(100),
                    column_gap: px(10),
                    ..default()
                })
                .with_children(|bar| {
                    editing_tools::toggle(
                        bar,
                        &fonts,
                        Action::ReplaceCase,
                        "ui.editor.strings.case_sensitive",
                        "Match case",
                        editor.tools.case_sensitive,
                    );
                    editing_tools::toggle(
                        bar,
                        &fonts,
                        Action::ReplaceWord,
                        "ui.editor.strings.whole_word",
                        "Whole word",
                        editor.tools.whole_word,
                    );
                    let (key, label) = if editor.tools.replace_english {
                        ("ui.editor.strings.target_en", "Replace in: English")
                    } else {
                        ("ui.editor.strings.target_ru", "Replace in: Russian")
                    };
                    button(bar, &fonts, Action::ReplaceLocale, key, label);
                });
            }
            root.spawn(Node {
                width: percent(100),
                padding: UiRect::horizontal(px(12)),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|headers| {
                button(
                    headers,
                    &fonts,
                    Action::Sort(0),
                    "ui.editor.strings.order",
                    "#",
                );
                for (column, (key, fallback)) in [
                    ("ui.editor.strings.english", "English"),
                    ("ui.editor.strings.russian", "Russian"),
                ]
                .into_iter()
                .enumerate()
                {
                    headers
                        .spawn((
                            Button,
                            Action::Sort(column as u8 + 1),
                            Node {
                                flex_grow: 1.,
                                flex_basis: px(0),
                                ..default()
                            },
                        ))
                        .with_children(|p| {
                            p.spawn(editor_text(
                                &fonts,
                                key,
                                fallback,
                                16.,
                                Color::srgb(0.59, 0.77, 0.83),
                                false,
                            ));
                            if editor.tools.sort.0 == column as u8 + 1 {
                                value(
                                    p,
                                    &fonts,
                                    if editor.tools.sort.1 { " ↓" } else { " ↑" }.to_owned(),
                                    16.,
                                );
                            }
                        });
                }
            });
            root.spawn(Node {
                width: percent(100),
                flex_grow: 1.,
                min_height: px(0),
                column_gap: px(6),
                ..default()
            })
            .with_children(|body| {
                body.spawn((
                    TableViewport,
                    RelativeCursorPosition::default(),
                    Node {
                        flex_grow: 1.,
                        min_width: px(0),
                        height: percent(100),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                ))
                .with_children(|table| {
                    if editor.filtered.is_empty() {
                        table.spawn(editor_text(
                            &fonts,
                            "ui.editor.strings.empty",
                            "No matching text",
                            16.,
                            Color::WHITE,
                            false,
                        ));
                    }
                    for index in range {
                        let key = &editor.filtered[index];
                        let selected = editor.editing && editor.selected == *key;
                        let row_height = editor.row_starts[index + 1] - editor.row_starts[index];
                        table
                            .spawn((
                                exchange::RowHit(key.clone()),
                                RelativeCursorPosition::default(),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: px(0),
                                    top: px(editor.row_starts[index] - editor.offset),
                                    width: percent(100),
                                    height: px(row_height),
                                    border: UiRect::bottom(px(1)),
                                    ..default()
                                },
                                BackgroundColor(if editor.rows_selected.keys.contains(key) {
                                    Color::srgb(0.035, 0.15, 0.20)
                                } else if index % 2 == 0 {
                                    Color::srgb(0.024, 0.056, 0.072)
                                } else {
                                    Color::srgb(0.019, 0.045, 0.059)
                                }),
                                BorderColor::all(Color::srgb(0.08, 0.14, 0.17)),
                            ))
                            .with_children(|row| {
                                row.spawn((
                                    Button,
                                    Action::SelectRow(key.clone(), true),
                                    Node {
                                        width: px(28),
                                        flex_shrink: 0.,
                                        height: percent(100),
                                        padding: UiRect::top(px(17)),
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                ))
                                .with_children(|p| {
                                    p.spawn((
                                        Node {
                                            border_radius: BorderRadius::all(px(2)),
                                            width: px(14),
                                            height: px(14),
                                            border: UiRect::all(px(1)),
                                            ..default()
                                        },
                                        BorderColor::all(Color::srgb(0.3, 0.55, 0.61)),
                                        BackgroundColor(
                                            if editor.rows_selected.keys.contains(key) {
                                                Color::srgb(0.3, 0.75, 0.82)
                                            } else {
                                                Color::NONE
                                            },
                                        ),
                                    ));
                                });
                                for russian in [false, true] {
                                    let mut cell = row.spawn(Node {
                                        flex_grow: 1.,
                                        flex_basis: px(0),
                                        min_width: px(0),
                                        height: percent(100),
                                        padding: UiRect::all(px(12)),
                                        overflow: Overflow::clip(),
                                        border: if russian {
                                            UiRect::left(px(1))
                                        } else {
                                            UiRect::ZERO
                                        },
                                        ..default()
                                    });
                                    cell.insert((
                                        Button,
                                        Action::Edit(key.clone(), russian),
                                        BorderColor::all(Color::srgb(0.1, 0.18, 0.21)),
                                        BackgroundColor(if selected && editor.russian == russian {
                                            Color::srgb(0.045, 0.15, 0.18)
                                        } else {
                                            Color::NONE
                                        }),
                                    ));
                                    cell.with_children(|p| {
                                        let text = if russian {
                                            editor.draft[key].clone()
                                        } else {
                                            editor.en[key].clone()
                                        };
                                        let mut bundle = editor_text(
                                            &fonts,
                                            "ui.editor.strings.value",
                                            "{value}",
                                            16.,
                                            Color::srgb(0.84, 0.91, 0.94),
                                            false,
                                        );
                                        bundle.localized = bundle.localized.with_arg("value", text);
                                        let mut text_entity = p.spawn((
                                            CellMetric {
                                                key: key.clone(),
                                                revision: editor.revision,
                                            },
                                            bundle,
                                            Node {
                                                width: percent(100),
                                                min_width: px(0),
                                                ..default()
                                            },
                                        ));
                                        if let Some((found, language, start, end)) =
                                            &editor.tools.found
                                        {
                                            if found == key
                                                && *language == russian
                                                && !(selected && editor.russian == russian)
                                            {
                                                text_entity.with_children(|p| {
                                                    editing_tools::cell_selection(p, *start, *end)
                                                });
                                            }
                                        }
                                        if russian == editor.russian
                                            && selected
                                            && editor.cursor != editor.anchor
                                        {
                                            text_entity.with_children(|p| {
                                                editing_tools::cell_selection(
                                                    p,
                                                    editor.cursor,
                                                    editor.anchor,
                                                )
                                            });
                                        }
                                        if russian == editor.russian
                                            && selected
                                            && editor.cursor == editor.anchor
                                        {
                                            text_entity.with_children(|p| {
                                                editing_tools::caret(p, editor.cursor)
                                            });
                                        }
                                        {
                                            text_entity.insert((
                                                StringCell(key.clone(), russian),
                                                RelativeCursorPosition::default(),
                                            ));
                                        }
                                    });
                                }
                            });
                    }
                });
                body.spawn((
                    TableTrack,
                    RelativeCursorPosition::default(),
                    Node {
                        width: px(8),
                        height: percent(100),
                        flex_shrink: 0.,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.03, 0.07, 0.09)),
                ))
                .with_children(|track| {
                    let thumb = (height / total.max(height) * height).max(24.);
                    let top = editor.offset / (total - height).max(1.) * (height - thumb).max(0.);
                    track.spawn((
                        Node {
                            border_radius: BorderRadius::all(px(4)),
                            position_type: PositionType::Absolute,
                            top: px(top),
                            height: px(thumb),
                            width: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.19, 0.35, 0.4)),
                    ));
                });
            });
            root.spawn(Node {
                column_gap: px(10),
                min_height: px(22),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|footer| {
                let mut status = editor_text(
                    &fonts,
                    "ui.editor.strings.ready",
                    "Ready",
                    13.,
                    Color::srgb(0.5, 0.69, 0.74),
                    false,
                );
                status.localized = if editor
                    .status
                    .key
                    .rsplit('.')
                    .next()
                    .is_some_and(|name| name.starts_with("exchange_"))
                    || editor.status.key == "ui.editor.strings.error"
                    || editor.status.key == "ui.editor.strings.exported"
                    || editor.status.key == "ui.editor.strings.imported"
                    || editor.status.key == "ui.editor.strings.replaced"
                    || editor.status.key == "ui.editor.strings.no_match"
                    || editor.status.key == "ui.editor.strings.match_found"
                    || editor.status.key == "ui.editor.strings.file_busy"
                {
                    editor.status.clone()
                } else if editor.changed_at.is_some() {
                    LocalizedText::new("ui.editor.strings.saving", "Saving…")
                } else {
                    LocalizedText::new("ui.editor.strings.saved_simple", "All changes saved")
                };
                footer.spawn(status);
                if !editor.rows_selected.keys.is_empty() {
                    let mut selected = editor_text(
                        &fonts,
                        "ui.editor.strings.selected_count",
                        "Selected: {count} · Right-click to export or import",
                        13.,
                        Color::srgb(0.5, 0.8, 0.85),
                        false,
                    );
                    selected.localized = selected
                        .localized
                        .with_arg("count", editor.rows_selected.keys.len().to_string());
                    footer.spawn(selected);
                }
                if editor.status.key == "ui.editor.strings.error" {
                    button(
                        footer,
                        &fonts,
                        Action::Sync,
                        "ui.editor.strings.sync",
                        "Synchronize",
                    );
                    button(
                        footer,
                        &fonts,
                        Action::Discard,
                        if editor.discard_confirm {
                            "ui.editor.strings.confirm_discard"
                        } else {
                            "ui.editor.strings.discard"
                        },
                        if editor.discard_confirm {
                            "Confirm discard ALL drafts"
                        } else {
                            "Discard drafts"
                        },
                    );
                }
            });
            if let Some(position) = editor.context_menu {
                root.spawn((
                    Node {
                        border_radius: BorderRadius::all(px(5)),
                        position_type: PositionType::Absolute,
                        left: px(position.x),
                        top: px(position.y - EDITOR_HEADER_HEIGHT),
                        width: px(210),
                        padding: UiRect::all(px(6)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(4),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    GlobalZIndex(300),
                    BackgroundColor(Color::srgb(0.035, 0.085, 0.105)),
                    BorderColor::all(Color::srgb(0.2, 0.4, 0.47)),
                ))
                .with_children(|menu| {
                    button(
                        menu,
                        &fonts,
                        Action::ExportRows,
                        "ui.editor.strings.export",
                        "Export…",
                    );
                    button(
                        menu,
                        &fonts,
                        Action::ImportRows,
                        "ui.editor.strings.import",
                        "Import…",
                    );
                });
            }
        });
}
