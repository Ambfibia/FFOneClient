use super::*;

pub(super) fn spawn_selected_model(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<EditorCatalog>,
    mut state: ResMut<EditorState>,
    mut preview: ResMut<ModelPreview>,
    mut prepared: ResMut<PreparedAnimations>,
    mut orbit: ResMut<OrbitCamera>,
    mut rig_cache: ResMut<NativePlayerRigAssetCache>,
) {
    if preview.current_index == Some(state.selected) {
        return;
    }
    if let Some(root) = preview.root.take() {
        commands.entity(root).despawn();
    }
    let entry = &catalog.entries[state.selected];
    let root = commands
        .spawn((
            Name::new(format!("editor preview: {}", entry.semantic_id)),
            Transform::IDENTITY,
            Visibility::Inherited,
            PreviewRoot,
        ))
        .id();
    let gltf = (entry.kind != CatalogKind::Equipment && !entry.glb.is_empty())
        .then(|| asset_server.load::<Gltf>(entry.glb.clone()));
    if let Some(definition) = entry.hnpc_visual.as_ref() {
        if let Some(hnpc) = catalog.hnpc.as_ref() {
            match spawn_network_hnpc_visual_0104(
                &mut commands,
                &asset_server,
                &mut rig_cache,
                hnpc,
                root,
                definition,
                preview.generation.wrapping_add(1).max(1),
            ) {
                Ok(visual) => {
                    commands.entity(root).insert(visual);
                }
                Err(error) => {
                    commands
                        .entity(root)
                        .insert(LegacyMaterialMetadataError(error));
                }
            }
        }
    }
    match (&entry.kind, entry.npc_visual.as_ref()) {
        (CatalogKind::Npc, Some(definition)) => {
            let visual = spawn_network_npc_visual_0104(
                &mut commands,
                &asset_server,
                root,
                definition,
                format!("editor XDT NPC {}", definition.npc_type),
            );
            commands.entity(root).insert(visual);
        }
        (CatalogKind::Nano, _) if !entry.glb.is_empty() => {
            let scene = asset_server.load(GltfAssetLabel::Scene(0).from_asset(entry.glb.clone()));
            spawn_legacy_character_scene(
                &mut commands,
                root,
                scene,
                entry.logical_name.clone(),
                LegacyCharacterRootPolicy::Nano,
            );
        }
        _ => {}
    }
    preview.current_index = Some(state.selected);
    preview.generation = preview.generation.wrapping_add(1).max(1);
    preview.root = Some(root);
    preview.gltf = gltf;
    prepared.0 = None;
    state.choose_default_clip(&catalog);
    state.playback_revision = state.playback_revision.wrapping_add(1).max(1);
    if !preview.preserve_camera { orbit.reset_for(entry); }
    preview.preserve_camera = false;
}

pub(super) fn spawn_header(parent: &mut ChildSpawnerCommands, fonts: &EditorFonts) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(EDITOR_HEADER_HEIGHT),
                flex_shrink: 0.,
                flex_direction: FlexDirection::Column,
                padding: UiRect::horizontal(px(16)),
                border: UiRect::bottom(px(2)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.012, 0.035, 0.05)),
            BorderColor::all(Color::srgb(0.26, 0.49, 0.83)),
        ))
        .with_children(|header| {
            header
                .spawn(Node {
                    width: percent(100),
                    height: px(48),
                    align_items: AlignItems::Center,
                    column_gap: px(18),
                    ..default()
                })
                .with_children(|bar| {
                    bar.spawn(editor_text(
                        fonts,
                        "ui.editor.brand",
                        "FFONE // CLIENT EDITOR",
                        22.,
                        Color::srgb(0.36, 0.69, 1.0),
                        true,
                    ));
                    bar.spawn((
                        strings::EditorSectionTitle,
                        editor_text(
                            fonts,
                            "ui.editor.title",
                            "XDT CHARACTER LAB",
                            15.,
                            Color::srgb(0.58, 0.83, 0.92),
                            true,
                        ),
                    ));
                    bar.spawn(Node {
                        flex_grow: 1.,
                        ..default()
                    });
                    spawn_action_button(
                        bar,
                        fonts,
                        EditorAction::ResetCamera,
                        "ui.editor.camera.reset",
                        "RESET VIEW  [R]",
                        150.,
                    );
                    spawn_action_button_with_role(
                        bar,
                        fonts,
                        EditorAction::ToggleLanguage,
                        "ui.editor.language",
                        "RU / EN  [F9]",
                        125.,
                        DynamicTextRole::Language,
                    );
                });
            header
                .spawn(Node {
                    width: percent(100),
                    height: px(44),
                    column_gap: px(8),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|tabs| {
                    for (action, key, label, width) in [
                        (
                            EditorAction::Tab(CatalogKind::Npc),
                            "ui.editor.tab.npc",
                            "NPC",
                            90.,
                        ),
                        (
                            EditorAction::Tab(CatalogKind::Nano),
                            "ui.editor.tab.nano",
                            "NANO",
                            90.,
                        ),
                        (
                            EditorAction::Tab(CatalogKind::Equipment),
                            "ui.editor.tab.equipment",
                            "Equipment",
                            130.,
                        ),
                        (
                            EditorAction::Strings,
                            "ui.editor.strings.tab",
                            "Strings",
                            130.,
                        ),
                        (EditorAction::Xdt, "ui.editor.xdt.tab", "XDT tables", 150.),
                    ] {
                        spawn_action_button(tabs, fonts, action, key, label, width);
                    }
                });
        });
}

pub(super) fn spawn_catalog_panel(
    parent: &mut ChildSpawnerCommands,
    fonts: &EditorFonts,
    asset_server: &AssetServer,
    catalog: &EditorCatalog,
) {
    parent
        .spawn(panel_node(px(EDITOR_CATALOG_WIDTH), fonts.panel.clone()))
        .with_children(|panel| {
            panel.spawn(editor_text(
                fonts,
                "ui.editor.library.title",
                "XDT LIBRARY",
                18.0,
                Color::srgb(0.83, 0.96, 1.0),
                true,
            ));
            panel
                .spawn((
                    Button,
                    EditorAction::FocusSearch,
                    Node {
                        border_radius: BorderRadius::all(px(6)),
                        width: percent(100),
                        height: px(42),
                        min_height: px(42),
                        max_height: px(42),
                        flex_shrink: 0.0,
                        min_width: px(0),
                        overflow: Overflow::clip(),
                        align_items: AlignItems::Center,
                        padding: UiRect::horizontal(px(12)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.025, 0.075, 0.095, 0.96)),
                    BorderColor::all(Color::srgba(0.18, 0.72, 0.86, 0.5)),
                    sliced_image(fonts.textfield.clone(), OPTION_TEXT_FIELD_BORDER),
                ))
                .with_children(|search| {
                    search.spawn((
                        single_line_text(editor_text(
                            fonts,
                            "ui.editor.search.placeholder",
                            "Search by name or ID...",
                            14.0,
                            Color::srgb(0.61, 0.8, 0.86),
                            false,
                        )),
                        DynamicTextRole::Search,
                        Node {
                            min_width: px(0),
                            max_width: percent(100),
                            ..default()
                        },
                    ));
                });
            panel
                .spawn(Node {
                    width: percent(100),
                    height: px(30),
                    flex_shrink: 0.0,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        editor_text(
                            fonts,
                            "ui.editor.library.count",
                            "{shown} of {total}",
                            12.0,
                            Color::srgb(0.48, 0.73, 0.8),
                            false,
                        ),
                        DynamicTextRole::CatalogCount,
                    ));
                    spawn_action_button(
                        row,
                        fonts,
                        EditorAction::ClearSearch,
                        "ui.editor.search.clear",
                        "CLEAR",
                        68.0,
                    );
                });
            panel
                .spawn(Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    min_height: px(0),
                    column_gap: px(5),
                    ..default()
                })
                .with_children(|scroll_area| {
                    scroll_area
                        .spawn((
                            Node {
                                flex_basis: px(0),
                                flex_grow: 1.0,
                                min_width: px(0),
                                min_height: px(0),
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::scroll_y(),
                                row_gap: px(CATALOG_ROW_GAP),
                                ..default()
                            },
                            ScrollPosition::default(),
                            RelativeCursorPosition::default(),
                            CatalogScroll,
                        ))
                        .with_children(|list| {
                            for (index, entry) in catalog.entries.iter().enumerate() {
                                list.spawn((
                                    Button,
                                    EditorAction::CatalogSlot(index),
                                    CatalogSlot(index),
                                    Node {
                                        border_radius: BorderRadius::all(px(5)),
                                        display: if entry.kind == CatalogKind::Npc {
                                            Display::Flex
                                        } else {
                                            Display::None
                                        },
                                        width: percent(100),
                                        height: px(CATALOG_ROW_HEIGHT),
                                        min_height: px(CATALOG_ROW_HEIGHT),
                                        flex_shrink: 0.0,
                                        align_items: AlignItems::Center,
                                        padding: UiRect::axes(px(6), px(5)),
                                        column_gap: px(7),
                                        border: UiRect::all(px(1)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.026, 0.065, 0.08, 0.94)),
                                    BorderColor::all(Color::srgba(0.16, 0.4, 0.47, 0.64)),
                                    EditorButtonSkin,
                                ))
                                .with_children(|button| {
                                    if let Some(icon_path) = entry.icon_path.as_ref() {
                                        button.spawn((
                                            Node {
                                                width: px(44),
                                                height: px(44),
                                                min_width: px(44),
                                                ..default()
                                            },
                                            ImageNode {
                                                image: asset_server.load(icon_path.clone()),
                                                image_mode: NodeImageMode::Stretch,
                                                ..default()
                                            },
                                        ));
                                    } else {
                                        button.spawn((
                                            Node {
                                                width: px(44),
                                                height: px(44),
                                                min_width: px(44),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgba(0.0, 0.08, 0.12, 0.65)),
                                        ));
                                    }
                                    button.spawn((
                                        editor_text(
                                            fonts,
                                            "ui.editor.catalog.row",
                                            "{name}\nID {id} · {clips} clips",
                                            12.0,
                                            Color::srgb(0.81, 0.92, 0.95),
                                            false,
                                        ),
                                        DynamicTextRole::CatalogSlot(index),
                                        Node {
                                            min_width: px(0),
                                            flex_grow: 1.0,
                                            flex_basis: px(0),
                                            max_height: px(48),
                                            overflow: Overflow::clip(),
                                            ..default()
                                        },
                                        EditorButtonLabel,
                                    ));
                                });
                            }
                        });
                    scroll_area
                        .spawn((
                            Node {
                                border_radius: BorderRadius::all(px(6)),
                                width: px(12),
                                min_width: px(12),
                                height: percent(100),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.015, 0.045, 0.06)),
                            RelativeCursorPosition::default(),
                            CatalogScrollbar,
                        ))
                        .with_children(|track| {
                            track.spawn((
                                Node {
                                    border_radius: BorderRadius::all(px(6)),
                                    position_type: PositionType::Absolute,
                                    width: percent(100),
                                    height: px(30),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.22, 0.65, 0.75)),
                                CatalogScrollThumb,
                            ));
                        });
                });
        });
}

pub(super) fn spawn_viewport_overlay(parent: &mut ChildSpawnerCommands, fonts: &EditorFonts) {
    parent
        .spawn(Node {
            flex_grow: 1.0,
            min_width: px(0),
            flex_basis: px(0),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::axes(px(EDITOR_VIEWPORT_PADDING_X), px(EDITOR_VIEWPORT_PADDING_Y)),
            ..default()
        })
        .with_children(|viewport| {
            viewport
                .spawn((
                    Node {
                        border_radius: BorderRadius::all(px(5)),
                        width: percent(100),
                        height: px(EDITOR_VIEWPORT_TITLE_HEIGHT),
                        min_height: px(EDITOR_VIEWPORT_TITLE_HEIGHT),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(px(18), px(9)),
                        row_gap: px(2),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.008, 0.027, 0.038, 0.72)),
                    BorderColor::all(Color::srgba(0.24, 0.82, 0.94, 0.28)),
                ))
                .with_children(|title| {
                    title.spawn((
                        editor_text(
                            fonts,
                            "ui.editor.current.title",
                            "{name}",
                            23.0,
                            Color::srgb(0.66, 1.0, 0.18),
                            true,
                        ),
                        DynamicTextRole::CurrentTitle,
                    ));
                    title.spawn((
                        editor_text(
                            fonts,
                            "ui.editor.current.subtitle",
                            "{kind} · {logical}",
                            12.0,
                            Color::srgb(0.47, 0.77, 0.86),
                            false,
                        ),
                        DynamicTextRole::CurrentSubtitle,
                    ));
                });
            viewport.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            spawn_playback_panel(viewport, fonts);
            spawn_equipment_playback(viewport, fonts);
        });
}

pub(super) fn spawn_playback_panel(parent: &mut ChildSpawnerCommands, fonts: &EditorFonts) {
    parent
        .spawn((
            CharacterSection,
            Node {
                border_radius: BorderRadius::all(px(5)),
                width: percent(100),
                height: px(EDITOR_PLAYBACK_HEIGHT),
                min_height: px(EDITOR_PLAYBACK_HEIGHT),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(11)),
                row_gap: px(8),
                border: UiRect::all(px(1)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.012, 0.036, 0.049, 0.93)),
            BorderColor::all(Color::srgba(0.39, 0.94, 0.13, 0.4)),
        ))
        .with_children(|panel| {
            panel
                .spawn(Node {
                    width: percent(100),
                    min_height: px(30),
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    column_gap: px(5),
                    row_gap: px(4),
                    ..default()
                })
                .with_children(|controls| {
                    spawn_action_button(
                        controls,
                        fonts,
                        EditorAction::PreviousClip,
                        "ui.editor.playback.previous",
                        "◀",
                        42.0,
                    );
                    spawn_action_button_with_role(
                        controls,
                        fonts,
                        EditorAction::TogglePlayback,
                        "ui.editor.playback.pause",
                        "PAUSE  [SPACE]",
                        100.0,
                        DynamicTextRole::Playback,
                    );
                    spawn_action_button(
                        controls,
                        fonts,
                        EditorAction::NextClip,
                        "ui.editor.playback.next",
                        "▶",
                        42.0,
                    );
                    spawn_action_button(
                        controls,
                        fonts,
                        EditorAction::SpeedDown,
                        "ui.editor.speed.down",
                        "−",
                        36.0,
                    );
                    controls.spawn((
                        editor_text(
                            fonts,
                            "ui.editor.speed.value",
                            "{speed}×",
                            13.0,
                            Color::srgb(0.82, 0.95, 1.0),
                            true,
                        ),
                        DynamicTextRole::Speed,
                    ));
                    spawn_action_button(
                        controls,
                        fonts,
                        EditorAction::SpeedUp,
                        "ui.editor.speed.up",
                        "+",
                        36.0,
                    );
                    controls.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });
                    spawn_action_button_with_role(
                        controls,
                        fonts,
                        EditorAction::ToggleLoop,
                        "ui.editor.loop.on",
                        "LOOP: ON",
                        72.0,
                        DynamicTextRole::Loop,
                    );
                    spawn_action_button_with_role(
                        controls,
                        fonts,
                        EditorAction::ToggleTurntable,
                        "ui.editor.turntable.on",
                        "ROTATE: ON",
                        86.0,
                        DynamicTextRole::Turntable,
                    );
                });
            panel
                .spawn((
                    Node {
                        border_radius: BorderRadius::all(px(5)),
                        width: percent(100),
                        height: px(9),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.025, 0.075, 0.09)),
                    BorderColor::all(Color::srgba(0.2, 0.58, 0.65, 0.5)),
                ))
                .with_children(|track| {
                    track.spawn((
                        TimelineFill,
                        Node {
                            border_radius: BorderRadius::all(px(4)),
                            width: percent(0),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.48, 0.98, 0.12)),
                    ));
                });
            panel.spawn((
                editor_text(
                    fonts,
                    "ui.editor.timeline",
                    "{clip}   {time} / {duration}",
                    12.0,
                    Color::srgb(0.53, 0.77, 0.84),
                    false,
                ),
                DynamicTextRole::Timeline,
            ));
        });
}
