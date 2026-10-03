use super::*;
#[derive(Component)]
pub(super) struct Canvas;
#[derive(Component)]
pub(super) struct Root;
#[derive(Component)]
pub(super) enum CameraValue {
    Roll,
    Fov,
}
fn value(p: &mut ChildSpawnerCommands, f: &EditorFonts, kind: CameraValue) {
    let b = editor_text(
        f,
        "ui.editor.xdt.value",
        "{value}",
        14.,
        Color::srgb(0.8, 0.9, 0.95),
        false,
    );
    p.spawn((b, kind));
}
fn tr(l: &Localization, lang: &Language, key: &str) -> String {
    l.text(
        lang,
        &LocalizedText::new(format!("ui.editor.icons.{key}"), key),
    )
}
fn text(p: &mut ChildSpawnerCommands, f: &EditorFonts, value: String, size: f32) {
    let mut b = editor_text(
        f,
        "ui.editor.xdt.value",
        "{value}",
        size,
        Color::srgb(0.8, 0.9, 0.95),
        false,
    );
    b.localized = b.localized.with_arg("value", value);
    p.spawn(b);
}
fn button(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    action: Action,
    value: String,
    selected: bool,
) {
    p.spawn((
        Button,
        action,
        Node {
            height: px(31),
            min_width: px(70),
            padding: UiRect::horizontal(px(10)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        BackgroundColor(if selected {
            Color::srgb(0.08, 0.35, 0.43)
        } else {
            Color::srgb(0.08, 0.13, 0.17)
        }),
    ))
    .with_children(|p| text(p, f, value, 13.));
}
fn row(p: &mut ChildSpawnerCommands, children: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node {
        column_gap: px(6),
        row_gap: px(6),
        flex_wrap: FlexWrap::Wrap,
        ..default()
    })
    .with_children(children);
}
pub(super) fn draw(
    mut commands: Commands,
    icons: Res<IconGenerator>,
    state: Res<EditorState>,
    catalog: Res<EditorCatalog>,
    library: Res<EquipmentLibrary>,
    fonts: Option<Res<EditorFonts>>,
    localization: Res<Localization>,
    language: Res<Language>,
    roots: Query<Entity, With<Root>>,
    mut stamp: Local<Option<String>>,
    mut values: Query<(&CameraValue, &mut LocalizedText)>,
) {
    if !icons.active {
        for root in &roots {
            commands.entity(root).despawn();
        }
        *stamp = None;
        return;
    }
    let Some(f) = fonts.as_ref() else { return };
    for (kind, mut label) in &mut values {
        let value = match kind {
            CameraValue::Roll => format!(
                "{}: {:.1}°",
                tr(&localization, &language, "roll"),
                icons.roll
            ),
            CameraValue::Fov => format!("{:.1}°", icons.fov),
        };
        let next = LocalizedText::new("ui.editor.xdt.value", "{value}").with_arg("value", value);
        if *label != next {
            *label = next;
        }
    }
    let next = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        state.selected,
        state.equipment_female,
        language.effective,
        serde_json::to_string(&icons.layers).unwrap(),
        icons.status,
        icons.scale,
        icons.level_text,
        icons.level_focused,
        icons.level_edit.cursor,
        icons.level_edit.anchor
    );
    if stamp.as_ref() == Some(&next) {
        return;
    }
    *stamp = Some(next);
    for root in &roots {
        commands.entity(root).despawn();
    }
    let l = &localization;
    let lang = &language;
    let layers = &icons.layers;
    commands
        .spawn((
            Root,
            Interaction::default(),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(600),
            BackgroundColor(Color::srgb(0.015, 0.025, 0.04)),
        ))
        .with_children(|screen| {
            screen
                .spawn(Node {
                    width: px(970),
                    padding: UiRect::all(px(22)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(15),
                    ..default()
                })
                .with_children(|p| {
                    row(p, |p| {
                        text(p, f, format!("{} · 128 × 128", tr(l, lang, "title")), 24.);
                        button(p, f, Action::Close, tr(l, lang, "close"), false);
                    });
                    text(
                        p,
                        f,
                        editor_entry_name(&catalog.entries[state.selected], l, lang),
                        17.,
                    );
                    row(p, |p| {
                        p.spawn(Node {
                            width: px(400),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(12),
                            ..default()
                        })
                        .with_children(|p| {
                            p.spawn((
                                Canvas,
                                RelativeCursorPosition::default(),
                                Node {
                                    width: px(384),
                                    height: px(384),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    border: UiRect::all(px(1)),
                                    ..default()
                                },
                                BorderColor::all(Color::srgb(0.2, 0.4, 0.45)),
                                BackgroundColor(Color::srgb(0.16, 0.19, 0.23)),
                            ))
                            .with_children(|p| {
                                p.spawn((
                                    ImageNode::new(icons.preview.clone()),
                                    Node {
                                        width: px(128 * i32::from(icons.scale)),
                                        height: px(128 * i32::from(icons.scale)),
                                        flex_shrink: 0.,
                                        ..default()
                                    },
                                ));
                            });
                            text(p, f, tr(l, lang, "mouse"), 13.);
                            row(p, |p| {
                                for scale in 1..=3 {
                                    button(
                                        p,
                                        f,
                                        Action::Scale(scale),
                                        format!("{scale}×"),
                                        icons.scale == scale,
                                    );
                                }
                            });
                        });
                        p.spawn(Node {
                            width: px(500),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(12),
                            ..default()
                        })
                        .with_children(|p| {
                            text(p, f, tr(l, lang, "camera"), 17.);
                            row(p, |p| {
                                for (action, key) in [
                                    (Action::Front, "front"),
                                    (Action::Back, "back"),
                                    (Action::Left, "left"),
                                    (Action::Right, "right"),
                                    (Action::Top, "top"),
                                    (Action::Fit, "fit"),
                                    (Action::Restore, "restore"),
                                ] {
                                    button(p, f, action, tr(l, lang, key), false);
                                }
                            });
                            row(p, |p| {
                                button(p, f, Action::Roll(-5.), "−5°".into(), false);
                                value(p, f, CameraValue::Roll);
                                button(p, f, Action::Roll(5.), "+5°".into(), false);
                                button(p, f, Action::Fov(-5.), "FOV −".into(), false);
                                value(p, f, CameraValue::Fov);
                                button(p, f, Action::Fov(5.), "FOV +".into(), false);
                            });
                            if state.kind == CatalogKind::Equipment {
                                let gender = library
                                    .entries
                                    .get(&state.selected)
                                    .map_or(0, |item| item.required_gender);
                                text(p, f, tr(l, lang, "single"), 14.);
                                row(p, |p| {
                                    if gender == 0 || gender == 1 {
                                        button(
                                            p,
                                            f,
                                            Action::Gender(false),
                                            tr(l, lang, "male"),
                                            !state.equipment_female,
                                        );
                                    }
                                    if gender == 0 || gender == 2 {
                                        button(
                                            p,
                                            f,
                                            Action::Gender(true),
                                            tr(l, lang, "female"),
                                            state.equipment_female,
                                        );
                                    }
                                });
                            }
                            if layers.nano {
                                text(p, f, tr(l, lang, "layers"), 17.);
                                row(p, |p| {
                                    for (a, key, on) in [
                                        (Action::Background, "background", layers.background),
                                        (Action::Outline, "outline", layers.outline),
                                        (Action::Locked, "locked", layers.locked),
                                        (Action::Badge, "badge", layers.badge),
                                        (Action::Affinity, "affinity", layers.affinity),
                                    ] {
                                        button(
                                            p,
                                            f,
                                            a,
                                            format!(
                                                "{} {}",
                                                if on { "●" } else { "○" },
                                                tr(l, lang, key)
                                            ),
                                            on,
                                        );
                                    }
                                });
                                row(p, |p| {
                                    button(
                                        p,
                                        f,
                                        Action::BadgeKind,
                                        tr(
                                            l,
                                            lang,
                                            ["source", "level", "world", "item"]
                                                [layers.badge_kind as usize],
                                        ),
                                        false,
                                    );
                                    if layers.badge_kind == 1 {
                                        button(p, f, Action::Level(-1), "−".into(), false);
                                        p.spawn((
                                            Button,
                                            Action::LevelInput,
                                            Node {
                                                width: px(70),
                                                height: px(31),
                                                padding: UiRect::horizontal(px(10)),
                                                align_items: AlignItems::Center,
                                                overflow: Overflow::clip(),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.08, 0.14, 0.19)),
                                            ffone_client::text_edit::EditVisual {
                                                edit: icons.level_edit.clone(),
                                                active: icons.level_focused,
                                                inset: 10.,
                                                scroll: 0.,
                                                shaped_text: icons.level_text.clone(),
                                            },
                                        ))
                                        .with_children(
                                            |p| {
                                                text(
                                                    p,
                                                    f,
                                                    if icons.level_focused {
                                                        icons.level_text.clone()
                                                    } else {
                                                        layers.level.to_string()
                                                    },
                                                    16.,
                                                );
                                                ffone_client::text_edit::spawn_decorations(p);
                                            },
                                        );
                                        button(p, f, Action::Level(1), "+".into(), false);
                                    }
                                });
                                text(p, f, tr(l, lang, "source_help"), 12.);
                            }
                            row(p, |p| {
                                button(p, f, Action::Export, tr(l, lang, "export"), true);
                                if state.kind == CatalogKind::Equipment
                                    && library
                                        .entries
                                        .get(&state.selected)
                                        .is_some_and(|item| item.required_gender == 0)
                                {
                                    button(p, f, Action::Both, tr(l, lang, "both"), false);
                                }
                                button(p, f, Action::Folder, tr(l, lang, "folder"), false);
                            });
                            text(p, f, tr(l, lang, "export_help"), 12.);
                        });
                    });
                    text(
                        p,
                        f,
                        if icons.status.is_empty() {
                            readable_path(&icons.output)
                        } else {
                            icons
                                .status
                                .strip_prefix("\\\\?\\")
                                .unwrap_or(&icons.status)
                                .to_owned()
                        },
                        12.,
                    );
                });
        });
}
