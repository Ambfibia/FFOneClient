use super::*;

#[allow(dead_code)]
pub(super) fn array<'a>(value: &'a Value, key: &str) -> Result<&'a Vec<Value>, String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("table has no {key} array"))
}

#[allow(dead_code)]
pub(super) fn integer(value: &Value, key: &str) -> Option<i64> {
    value.get(key).and_then(Value::as_i64)
}

#[allow(dead_code)]
pub(super) fn number(value: &Value, key: &str) -> Option<f32> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .map(|value| value as f32)
}

#[allow(dead_code)]
pub(super) fn native_string<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && !value.eq_ignore_ascii_case("null"))
}

#[allow(dead_code)]
pub(super) fn humanize_name(logical_name: &str) -> String {
    let source = logical_name
        .strip_prefix("npc_")
        .or_else(|| logical_name.strip_prefix("nano_"))
        .unwrap_or(logical_name);
    source
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn capture_editor_preview(
    mut commands: Commands,
    mut capture: ResMut<EditorCapture>,
    status: Res<EditorRuntimeStatus>,
    catalog: Res<EditorCatalog>,
    state: Res<EditorState>,
    equipment: Res<EquipmentLibrary>,
    player: Res<ffone_client::player_preview::NativePlayerPreviewModel>,
    search_nodes: Query<(&EditorAction, &ComputedNode)>,
    string_editor: Res<strings::StringEditor>,
    xdt_editor: Res<xdt::XdtEditor>,
    icons: Res<icon_generator::IconGenerator>,
) {
    let Some(output) = capture.output.clone() else {
        return;
    };
    if capture.issued {
        return;
    }
    capture.frames += 1;
    let is_equipment = state.kind == CatalogKind::Equipment;
    let ready = if icons.active && !icons.has_subject {
        false
    } else if state.xdt_open || state.strings_open || state.world_open.is_some() {
        true
    } else if is_equipment {
        matches!(
            player.status,
            ffone_client::player_preview::NativePlayerPreviewStatus::ReadyAnimated { .. }
        )
    } else {
        status.ready
    };
    let error = if is_equipment {
        equipment.capture_error(&player)
    } else {
        status.error.clone()
    };
    if ready {
        capture.ready_frames += 1;
    }
    if capture.ready_frames < 45 && capture.frames < 900 && error.is_none() {
        return;
    }
    capture.issued = true;
    let report = serde_json::json!({
        "strings": state.strings_open.then(|| string_editor.capture_report()),
        "xdt": state.xdt_open.then(|| xdt_editor.capture_report()),
        "npc": catalog.entries[state.selected].network_id,
        "ready": ready, "error": error, "frames": capture.frames,
        "equipment": is_equipment, "female": state.equipment_female,
        "search_size": search_nodes.iter().find(|(action, _)| **action == EditorAction::FocusSearch).map(|(_, node)| [node.size().x, node.size().y]),
        "glb": catalog.entries[state.selected].glb,
        "hnpc": catalog.entries[state.selected].hnpc_visual.is_some(),
    });
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).expect("capture output directory");
    }
    fs::write(
        output.with_extension("json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .expect("write capture status");
    commands.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
            event
                .image
                .clone()
                .try_into_dynamic()
                .expect("capture image")
                .save(&output)
                .expect("save capture");
            exit.write(AppExit::Success);
        },
    );
}

pub(super) fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        // Production MainCamera uses the clean client's no-MSAA Good profile.
        Msaa::Off,
        Projection::Perspective(PerspectiveProjection {
            fov: 45_f32.to_radians(),
            near: 0.15,
            far: 250.0,
            ..default()
        }),
        Transform::from_xyz(0.0, 1.4, -4.2).looking_at(Vec3::Y, Vec3::Y),
        PreviewCamera,
    ));
    commands.spawn((
        // Keep the same neutral production world light used by FFOneClient.
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub(super) fn preview_viewport_logical_rect(window_size: Vec2) -> Option<(Vec2, Vec2)> {
    let position = Vec2::new(
        EDITOR_BODY_PADDING_X + EDITOR_CATALOG_WIDTH + EDITOR_BODY_GAP + EDITOR_VIEWPORT_PADDING_X,
        EDITOR_HEADER_HEIGHT
            + EDITOR_BODY_PADDING_Y
            + EDITOR_VIEWPORT_PADDING_Y
            + EDITOR_VIEWPORT_TITLE_HEIGHT
            + EDITOR_VIEWPORT_CONTENT_GAP,
    );
    let right_inset = EDITOR_BODY_PADDING_X
        + EDITOR_INSPECTOR_WIDTH
        + EDITOR_BODY_GAP
        + EDITOR_VIEWPORT_PADDING_X;
    let bottom_inset = EDITOR_BODY_PADDING_Y
        + EDITOR_VIEWPORT_PADDING_Y
        + EDITOR_PLAYBACK_HEIGHT
        + EDITOR_VIEWPORT_CONTENT_GAP;
    let size = Vec2::new(
        window_size.x - position.x - right_inset,
        window_size.y - position.y - bottom_inset,
    );
    (size.x >= 1.0 && size.y >= 1.0).then_some((position, size))
}

pub(super) fn animate_turntable(
    time: Res<Time>,
    state: Res<EditorState>,
    mut roots: Query<&mut Transform, With<PreviewRoot>>,
) {
    if !state.turntable {
        return;
    }
    for mut transform in &mut roots {
        transform.rotate_y(time.delta_secs() * 0.28);
    }
}

pub(super) fn handle_editor_buttons(
    interactions: Query<(&Interaction, &EditorAction), Changed<Interaction>>,
    catalog: Res<EditorCatalog>,
    localization: Res<Localization>,
    mut language: ResMut<Language>,
    mut state: ResMut<EditorState>,
    mut orbit: ResMut<OrbitCamera>,
    mut preview: ResMut<ModelPreview>,
    icons: Res<icon_generator::IconGenerator>,
    mut xdt: ResMut<xdt::XdtEditor>,
    mut world: ResMut<world_editor::WorldEditor>,
    mut inspector_scroll:Query<&mut ScrollPosition,With<InspectorScroll>>,
) {
    if icons.active { return; }
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            EditorAction::World2d | EditorAction::World3d => {
                world.request_open();
                state.world_open = Some(*action == EditorAction::World3d);
                state.xdt_open = false;
                state.strings_open = false;
                state.missions_open = false;
                state.search_focused = false;
            }
            EditorAction::Missions => {
                state.world_open = None;
                state.xdt_open = true;
                state.strings_open = false;
                state.missions_open = true;
                state.search_focused = false;
                xdt.open_missions();
            }
            EditorAction::Strings => {
                state.world_open = None;
                state.missions_open = false;
                state.strings_open = true;
                state.xdt_open = false;
                state.search_focused = false;
            }
            EditorAction::Xdt => {
                state.world_open = None;
                state.missions_open = false;
                xdt.open_tables();
                state.xdt_open = true;
                state.strings_open = false;
                state.search_focused = false;
            }
            EditorAction::NewNpcTemplate => {
                state.world_open = None;
                state.missions_open = false;
                xdt.create_npc_from_catalog();
                state.xdt_open = true;
                state.strings_open = false;
                state.search_focused = false;
            }
            EditorAction::Tab(kind) => {
                state.world_open = None;
                state.missions_open = false;
                state.strings_open = false;
                state.xdt_open = false;
                let previous_kind = state.kind;
                let previous = (state.selected, state.search.clone());
                state.viewer_tabs.insert(previous_kind, previous);
                state.kind = kind;
                state.search_focused = false;
                if let Some((index, search)) = state.viewer_tabs.get(&kind).cloned() {
                    state.search = search;
                    state.select(&catalog, index);
                } else {
                    state.search.clear();
                    if let Some(index) = state.filtered(&catalog).first().copied() {
                        state.select(&catalog, index);
                    }
                }
                state.reveal_selection = true;
            }
            EditorAction::ToggleDetails => state.details_open = !state.details_open,
            EditorAction::NpcInspector(tab) => {
                if xdt.finish_npc_field(){state.npc_inspector=tab;state.search_focused=false;for mut scroll in &mut inspector_scroll{scroll.0=Vec2::ZERO;}}
            },
            EditorAction::IconGenerator => {},
            EditorAction::EquipmentGender(_)
            | EditorAction::EquipmentCategory(_)
            | EditorAction::ResetOutfit => {}
            EditorAction::FocusSearch => state.search_focused = true,
            EditorAction::ClearSearch => {
                state.search.clear();
                state.search_focused = true;
            }
            EditorAction::CatalogSlot(index) => {
                if state.npc_editing()&&!xdt.finish_npc_field(){continue;}
                if catalog.entries.get(index).is_some() {
                    state.select(&catalog, index);
                    state.search_focused = false;
                }
            }
            EditorAction::DefaultPose => {
                state.activate_default_pose(&catalog);
                // Respawn restores every authored node before sampling frame zero.
                preview.current_index = None;
            }
            EditorAction::TPose => {
                state.activate_t_pose();
                // AnimationPlayer::stop_all does not restore transforms already sampled.
                preview.current_index = None;
            }
            EditorAction::AnimationSlot(slot) => {
                let clip_index = state.animation_page * ANIMATION_SLOTS + slot;
                state.select_clip(&catalog, clip_index);
            }
            EditorAction::AnimationPreviousPage => {
                state.animation_page = state.animation_page.saturating_sub(1);
            }
            EditorAction::AnimationNextPage => {
                let pages = page_count(
                    catalog.entries[state.selected].animations.len(),
                    ANIMATION_SLOTS,
                );
                state.animation_page = (state.animation_page + 1).min(pages.saturating_sub(1));
            }
            EditorAction::PreviousClip => select_relative_clip(&catalog, &mut state, -1),
            EditorAction::NextClip => select_relative_clip(&catalog, &mut state, 1),
            EditorAction::TogglePlayback => state.toggle_playback(&catalog),
            EditorAction::ToggleLoop => state.looping = !state.looping,
            EditorAction::SpeedDown => state.speed = (state.speed - 0.25).max(0.25),
            EditorAction::SpeedUp => state.speed = (state.speed + 0.25).min(2.0),
            EditorAction::ResetCamera => orbit.reset_for(&catalog.entries[state.selected]),
            EditorAction::ToggleTurntable => state.turntable = !state.turntable,
            EditorAction::ToggleLanguage => {
                let next = if language.effective == "ru" {
                    "en"
                } else {
                    "ru"
                };
                localization.select(&mut language, next);
            }
        }
    }
}

pub(super) fn handle_search_keyboard(
    mut keys: MessageReader<KeyboardInput>,
    mut state: ResMut<EditorState>,
    appearance: Res<hnpc::HnpcEditor>,
    icons: Res<icon_generator::IconGenerator>,
) {
    if icons.active || appearance.active() || state.strings_open || state.xdt_open || state.world_open.is_some() || !state.search_focused {
        keys.clear();
        return;
    }
    for key in keys.read() {
        if key.state != ButtonState::Pressed {
            continue;
        }
        match key.key_code {
            KeyCode::Escape | KeyCode::Enter | KeyCode::NumpadEnter => {
                state.search_focused = false;
            }
            KeyCode::Backspace => {
                state.search.pop();
            }
            _ => {
                let produced = key.text.as_deref().or_else(|| match &key.logical_key {
                    Key::Character(value) => Some(value.as_str()),
                    _ => None,
                });
                if let Some(produced) = produced.filter(|text| {
                    !text.chars().any(char::is_control) && state.search.chars().count() < 64
                }) {
                    state.search.push_str(produced);
                }
            }
        }
    }
}

pub(super) fn handle_editor_shortcuts(
    keyboard: Res<ButtonInput<KeyCode>>,
    catalog: Res<EditorCatalog>,
    mut preview: ResMut<ModelPreview>,
    mut state: ResMut<EditorState>,
    mut orbit: ResMut<OrbitCamera>,
    appearance: Res<hnpc::HnpcEditor>,
    icons: Res<icon_generator::IconGenerator>,
) {
    if icons.active || appearance.active() || state.strings_open || state.xdt_open || state.world_open.is_some() || state.npc_editing() {
        return;
    }
    let control = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if control && keyboard.just_pressed(KeyCode::KeyF) {
        state.search_focused = true;
        return;
    }
    // Navigation remains available while typing a search, without changing the query.
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        state.select_relative_entry(&catalog, true);
    } else if keyboard.just_pressed(KeyCode::ArrowUp) {
        state.select_relative_entry(&catalog, false);
    }
    if state.search_focused {
        return;
    }
    if keyboard.just_pressed(KeyCode::KeyD) {
        state.activate_default_pose(&catalog);
        preview.current_index = None;
    }
    if keyboard.just_pressed(KeyCode::KeyT) {
        state.activate_t_pose();
        preview.current_index = None;
    }
    if keyboard.just_pressed(KeyCode::Space) {
        state.toggle_playback(&catalog);
    }
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        select_relative_clip(&catalog, &mut state, -1);
    }
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        select_relative_clip(&catalog, &mut state, 1);
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        orbit.reset_for(&catalog.entries[state.selected]);
    }
}

pub(super) fn page_count(count: usize, page_size: usize) -> usize {
    count.max(1).div_ceil(page_size)
}

pub(super) fn single_line_text(mut text: EditorTextBundle) -> EditorTextBundle {
    text.layout = TextLayout::new(Justify::Left, LineBreak::NoWrap);
    text
}

pub(super) fn editor_text(
    fonts: &EditorFonts,
    key: impl Into<String>,
    fallback: impl Into<String>,
    size: f32,
    color: Color,
    display: bool,
) -> EditorTextBundle {
    let fallback = fallback.into();
    EditorTextBundle {
        text: Text::new(fallback.clone()),
        font: TextFont {
            font: (if display {
                fonts.display.clone()
            } else {
                fonts.body.clone()
            })
            .into(),
            font_size: (size).into(),
            ..default()
        },
        color: TextColor(color),
        layout: TextLayout::default().with_justify(Justify::Left),
        localized: LocalizedText::new(key, fallback),
    }
}

pub(super) fn setup_editor_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<EditorCatalog>,
) {
    let fonts = EditorFonts {
        body: asset_server.load("fonts/chaletbook-regular.ttf"),
        display: asset_server.load("fonts/jeffe.otf"),
        button: asset_server.load("ui/en/option/button.png"),
        button_hover: asset_server.load("ui/en/option/button-hover.png"),
        panel: asset_server.load("ui/en/option/panel-side.png"),
        textfield: asset_server.load("ui/en/option/textfield.png"),
    };
    commands.insert_resource(fonts.clone());
    let ui_camera = commands
        .spawn((
            Name::new("FFOne editor UI camera"),
            Camera2d,
            // Both cameras share the window target. Matching sample counts
            // preserve the 3D viewport when the transparent UI is composited.
            Msaa::Off,
            Camera {
                order: EDITOR_UI_CAMERA_ORDER,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            IsDefaultUiCamera,
        ))
        .id();

    commands
        .spawn((
            Name::new("FFOne editor chrome"),
            icon_generator::EditorChrome,
            Node {
                left: px(0),
                top: px(0),
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            UiTargetCamera(ui_camera),
            ZIndex(100),
        ))
        .with_children(|root| {
            spawn_header(root, &fonts);
            root.spawn((
                strings::ModelEditorBody,
                Node {
                    width: percent(100),
                    flex_grow: 1.0,
                    min_height: px(0),
                    padding: UiRect::axes(px(EDITOR_BODY_PADDING_X), px(EDITOR_BODY_PADDING_Y)),
                    column_gap: px(EDITOR_BODY_GAP),
                    ..default()
                },
            ))
            .with_children(|body| {
                spawn_catalog_panel(body, &fonts, &asset_server, &catalog);
                spawn_viewport_overlay(body, &fonts);
                spawn_inspector_panel(body, &fonts);
            });
        });
}

pub(super) fn panel_node(width: Val, _image: Handle<Image>) -> impl Bundle {
    (
        Node {
            border_radius: BorderRadius::all(px(8)),
            width,
            min_width: width,
            max_width: width,
            flex_shrink: 0.0,
            height: percent(100),
            min_height: px(0),
            overflow: Overflow::clip(),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(14)),
            row_gap: px(10),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.018, 0.043, 0.058, 0.95)),
        BorderColor::all(Color::srgba(0.15, 0.78, 0.9, 0.34)),
    )
}

pub(super) fn editor_entry_name(
    entry: &EditorCatalogEntry,
    localization: &Localization,
    language: &Language,
) -> String {
    if entry.kind == CatalogKind::Npc
        && let Some(number) = entry.network_id
    {
        return localization.text(
            language,
            &LocalizedText::new(
                format!("content.npc.{number}.name"),
                entry.display_name.clone(),
            ),
        );
    }
    let localized = match entry.logical_name.as_str() {
        "nano_holonano" => LocalizedText::new("ui.editor.nano.holo", "Unstable Nano"),
        "nano_upgrade" => LocalizedText::new("ui.editor.nano.upgrade", "Upgrade"),
        "nano_ghostfreak" => LocalizedText::new("ui.editor.nano.ghostfreak", "Ghostfreak"),
        "nano_ben" => LocalizedText::new("ui.editor.nano.ben", "Ben Tennyson"),
        "nano_flapjack" => LocalizedText::new("ui.editor.nano.flapjack", "Flapjack"),
        "nano_johnnybravo" => LocalizedText::new("ui.editor.nano.johnny_bravo", "Johnny Bravo"),
        _ => return entry.display_name.clone(),
    };
    localization.text(language, &localized)
}
