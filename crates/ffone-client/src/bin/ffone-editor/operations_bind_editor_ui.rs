use super::*;

pub(super) fn bind_editor_ui(
    catalog: Res<EditorCatalog>,
    state: Res<EditorState>,
    language: Res<Language>,
    localization: Res<Localization>,
    runtime: Res<EditorRuntimeStatus>,
    equipment: Res<EquipmentLibrary>,
    player_preview: Res<ffone_client::player_preview::NativePlayerPreviewModel>,
    mut texts: Query<(&DynamicTextRole, &mut LocalizedText)>,
    mut catalog_slots: Query<(&CatalogSlot, &mut Node), Without<AnimationSlot>>,
    mut animation_slots: Query<(&AnimationSlot, &mut Node), Without<CatalogSlot>>,
    mut timeline: Single<
        &mut Node,
        (
            With<TimelineFill>,
            Without<CatalogSlot>,
            Without<AnimationSlot>,
        ),
    >,
) {
    if (state.xdt_open || state.strings_open)
        && !state.is_changed() && !language.is_changed() && !localization.is_changed()
        && !catalog.is_changed() && !equipment.is_changed()
    {
        return;
    }
    let entry = &catalog.entries[state.selected];
    let filtered = state.filtered(&catalog);
    let animation_pages = page_count(entry.animations.len(), ANIMATION_SLOTS);
    for (slot, mut node) in &mut catalog_slots {
        let display = if filtered.binary_search(&slot.0).is_ok() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (slot, mut node) in &mut animation_slots {
        let display = if entry
            .animations
            .get(state.animation_page * ANIMATION_SLOTS + slot.0)
            .is_some()
        {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display { node.display = display; }
    }
    let progress = if runtime.animation_duration > f32::EPSILON {
        (runtime.animation_time / runtime.animation_duration).fract()
    } else {
        0.0
    };
    let width = percent(progress * 100.0);
    if timeline.width != width {
        timeline.width = width;
    }

    for (role, mut localized) in &mut texts {
        let next = match *role {
            DynamicTextRole::Search => {
                if state.search.is_empty() {
                    LocalizedText::new("ui.editor.search.placeholder", "Search by name or ID...")
                } else {
                    LocalizedText::new("ui.editor.search.value", "{query}")
                        .with_arg("query", state.search.clone())
                }
            }
            DynamicTextRole::CatalogCount => {
                LocalizedText::new("ui.editor.library.count", "{shown} of {total}")
                    .with_arg("shown", filtered.len().to_string())
                    .with_arg(
                        "total",
                        match state.kind {
                            CatalogKind::Npc => catalog.npc_count,
                            CatalogKind::Nano => catalog.nano_count,
                            CatalogKind::Equipment => equipment.entries.len(),
                        }
                        .to_string(),
                    )
            }
            DynamicTextRole::CatalogSlot(slot) => {
                let Some(entry) = catalog.entries.get(slot) else {
                    continue;
                };
                let id = entry
                    .network_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "—".to_owned());
                if entry.kind == CatalogKind::Equipment {
                    LocalizedText::new("ui.editor.equipment.row", "{name}\n{id} · {category}")
                        .with_arg("name", editor_entry_name(entry, &localization, &language))
                        .with_arg("id", id)
                        .with_arg(
                            "category",
                            equipment.category_label(slot, &localization, &language),
                        )
                } else if entry.glb.is_empty() {
                    LocalizedText::new(
                        "ui.editor.catalog.row.no_visual",
                        "{name}\nID {id} · no ordinary native visual",
                    )
                    .with_arg("name", editor_entry_name(entry, &localization, &language))
                    .with_arg("id", id)
                } else if entry.animations.is_empty() {
                    LocalizedText::new(
                        "ui.editor.catalog.row.static",
                        "{name}\nID {id} · static model",
                    )
                    .with_arg("name", editor_entry_name(entry, &localization, &language))
                    .with_arg("id", id)
                } else {
                    LocalizedText::new("ui.editor.catalog.row", "{name}\nID {id} · {clips} clips")
                        .with_arg("name", editor_entry_name(entry, &localization, &language))
                        .with_arg("id", id)
                        .with_arg("clips", entry.animations.len().to_string())
                }
            }
            DynamicTextRole::CurrentTitle => {
                LocalizedText::new("ui.editor.current.title", "{name}")
                    .with_arg("name", editor_entry_name(entry, &localization, &language))
            }
            DynamicTextRole::CurrentSubtitle if entry.kind == CatalogKind::Equipment => {
                LocalizedText::new("ui.editor.equipment.subtitle", "{category} · {gender}")
                    .with_arg(
                        "category",
                        equipment.category_label(state.selected, &localization, &language),
                    )
                    .with_arg(
                        "gender",
                        localization.text(
                            &language,
                            &LocalizedText::new(
                                if state.equipment_female {
                                    "ui.editor.equipment.female"
                                } else {
                                    "ui.editor.equipment.male"
                                },
                                if state.equipment_female {
                                    "Female"
                                } else {
                                    "Male"
                                },
                            ),
                        ),
                    )
            }
            DynamicTextRole::CurrentSubtitle => {
                LocalizedText::new("ui.editor.current.subtitle", "{kind} · {logical}")
                    .with_arg("kind", entry.kind.title())
                    .with_arg("logical", entry.logical_name.clone())
            }
            DynamicTextRole::RuntimeStatus => {
                if entry.kind == CatalogKind::Equipment {
                    equipment.status_text(&player_preview)
                } else if entry.glb.is_empty() {
                    LocalizedText::new(
                        "ui.editor.status.no_visual",
                        "● XDT ROW HAS NO ORDINARY NATIVE VISUAL",
                    )
                } else if let Some(error) = runtime.error.as_ref() {
                    LocalizedText::new("ui.editor.status.error", "● ASSET ERROR: {error}")
                        .with_arg("error", error.clone())
                } else if runtime.ready && entry.animations.is_empty() {
                    LocalizedText::new(
                        "ui.editor.status.static_ready",
                        "● NATIVE STATIC ASSET READY",
                    )
                } else if runtime.ready {
                    LocalizedText::new("ui.editor.status.ready", "● NATIVE ASSET READY")
                } else {
                    LocalizedText::new("ui.editor.status.loading", "● LOADING NATIVE ASSET")
                }
            }
            DynamicTextRole::InspectorIdentity => LocalizedText::new(
                "ui.editor.inspector.identity",
                "Semantic ID\n{semantic}\n\nLogical root\n{logical}",
            )
            .with_arg("semantic", entry.semantic_id.clone())
            .with_arg("logical", entry.logical_name.clone()),
            DynamicTextRole::InspectorSource => LocalizedText::new(
                "ui.editor.inspector.source",
                "XDT row: {row}\nNetwork ID: {network}\nSource: {source}",
            )
            .with_arg(
                "row",
                entry
                    .table_index
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "—".to_owned()),
            )
            .with_arg(
                "network",
                entry
                    .network_id
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "—".to_owned()),
            )
            .with_arg(
                "source",
                if entry.native_extension {
                    "REGISTRY"
                } else {
                    "XDT + REGISTRY"
                },
            ),
            DynamicTextRole::InspectorGeometry => LocalizedText::new(
                "ui.editor.inspector.geometry",
                "Scale: {scale}\nHeight: {height}\nStyle: {style}\nLevel / set: {level}",
            )
            .with_arg(
                "scale",
                entry
                    .scale
                    .map(|value| format!("{value:.3}"))
                    .unwrap_or_else(|| "—".to_owned()),
            )
            .with_arg(
                "height",
                entry
                    .height_server_units
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "—".to_owned()),
            )
            .with_arg(
                "style",
                entry
                    .style
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "—".to_owned()),
            )
            .with_arg(
                "level",
                entry
                    .level
                    .or(entry.team_or_set)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "—".to_owned()),
            ),
            DynamicTextRole::InspectorTextures => LocalizedText::new(
                "ui.editor.inspector.textures",
                "Main texture: {main}\nSub texture: {sub}",
            )
            .with_arg(
                "main",
                entry.texture_main.clone().unwrap_or_else(|| "—".to_owned()),
            )
            .with_arg(
                "sub",
                entry.texture_sub.clone().unwrap_or_else(|| "—".to_owned()),
            ),
            DynamicTextRole::AnimationPage => {
                LocalizedText::new("ui.editor.page.value", "{page} / {pages}")
                    .with_arg("page", (state.animation_page + 1).to_string())
                    .with_arg("pages", animation_pages.to_string())
            }
            DynamicTextRole::AnimationSlot(slot) => {
                let Some(clip) = entry
                    .animations
                    .get(state.animation_page * ANIMATION_SLOTS + slot)
                else {
                    continue;
                };
                LocalizedText::new("ui.editor.animation.clip", "{clip}")
                    .with_arg("clip", clip.clone())
            }
            DynamicTextRole::Playback => {
                if state.pose_mode != EditorPoseMode::Clip || state.paused {
                    LocalizedText::new("ui.editor.playback.play", "PLAY  [SPACE]")
                } else {
                    LocalizedText::new("ui.editor.playback.pause", "PAUSE  [SPACE]")
                }
            }
            DynamicTextRole::Loop => {
                if state.looping {
                    LocalizedText::new("ui.editor.loop.on", "LOOP: ON")
                } else {
                    LocalizedText::new("ui.editor.loop.off", "LOOP: OFF")
                }
            }
            DynamicTextRole::Speed => LocalizedText::new("ui.editor.speed.value", "{speed}×")
                .with_arg("speed", format!("{:.2}", state.speed)),
            DynamicTextRole::Turntable => {
                if state.turntable {
                    LocalizedText::new("ui.editor.turntable.on", "ROTATE: ON")
                } else {
                    LocalizedText::new("ui.editor.turntable.off", "ROTATE: OFF")
                }
            }
            DynamicTextRole::Timeline => {
                let clip = entry
                    .animations
                    .get(state.clip_index)
                    .map(String::as_str)
                    .unwrap_or("—");
                match state.pose_mode {
                    EditorPoseMode::TPose => {
                        LocalizedText::new("ui.editor.timeline.tpose", "T-POSE · NATIVE BIND RIG")
                    }
                    EditorPoseMode::Default => LocalizedText::new(
                        "ui.editor.timeline.default",
                        "DEFAULT POSE · {clip} · FRAME 0",
                    )
                    .with_arg("clip", clip),
                    EditorPoseMode::Clip => {
                        LocalizedText::new("ui.editor.timeline", "{clip}   {time} / {duration}")
                            .with_arg("clip", clip)
                            .with_arg("time", format!("{:.2}s", runtime.animation_time))
                            .with_arg("duration", format!("{:.2}s", runtime.animation_duration))
                    }
                }
            }
            DynamicTextRole::Language => {
                LocalizedText::new("ui.editor.language.value", "{language}  [F9]")
                    .with_arg("language", language.effective.to_uppercase())
            }
        };
        if *localized != next {
            *localized = next;
        }
    }
}

pub(super) fn style_editor_buttons(
    fonts: Res<EditorFonts>,
    state: Res<EditorState>,
    mut buttons: Query<(
        &Interaction,
        &EditorAction,
        &Children,
        &mut BackgroundColor,
        &mut BorderColor,
        Option<&EditorButtonSkin>,
        Option<&mut ImageNode>,
    )>,
    mut labels: Query<&mut TextColor, With<EditorButtonLabel>>,
) {
    for (interaction, action, children, mut background, mut border, button_skin, image) in
        &mut buttons
    {
        let selected = match *action {
            EditorAction::Strings => state.strings_open,
            EditorAction::Xdt => state.xdt_open,
            EditorAction::Tab(kind) => !state.strings_open && !state.xdt_open && state.kind == kind,
            EditorAction::EquipmentGender(female) => state.equipment_female == female,
            EditorAction::EquipmentCategory(category) => state.equipment_category == category,
            EditorAction::ToggleDetails => state.details_open,
            EditorAction::CatalogSlot(index) => index == state.selected,
            EditorAction::DefaultPose => state.pose_mode == EditorPoseMode::Default,
            EditorAction::TPose => state.pose_mode == EditorPoseMode::TPose,
            EditorAction::AnimationSlot(slot) => {
                state.pose_mode == EditorPoseMode::Clip
                    && state.animation_page * ANIMATION_SLOTS + slot == state.clip_index
            }
            EditorAction::FocusSearch => state.search_focused,
            _ => false,
        };
        let (fill, line) = if selected && matches!(action, EditorAction::Xdt) {
            (Color::srgb(0.055, 0.21, 0.36), Color::srgb(0.28, 0.77, 1.0))
        } else if selected {
            (
                Color::srgba(0.08, 0.23, 0.42, 0.96),
                Color::srgb(0.36, 0.8, 1.0),
            )
        } else if *interaction == Interaction::Pressed {
            (
                Color::srgba(0.08, 0.44, 0.53, 0.98),
                Color::srgb(0.35, 0.95, 1.0),
            )
        } else if *interaction == Interaction::Hovered {
            (
                Color::srgba(0.045, 0.19, 0.23, 0.98),
                Color::srgb(0.28, 0.82, 0.9),
            )
        } else {
            (
                Color::srgba(0.026, 0.073, 0.089, 0.96),
                Color::srgba(0.18, 0.53, 0.61, 0.62),
            )
        };
        if background.0 != fill {
            background.0 = fill;
        }
        let next_border = BorderColor::all(line);
        if *border != next_border {
            *border = next_border;
        }
        if button_skin.is_some() {
            let hovered = *interaction == Interaction::Hovered;
            if let Some(mut image) = image {
                let next_image = if hovered {
                    fonts.button_hover.clone()
                } else {
                    fonts.button.clone()
                };
                if image.image != next_image { image.image = next_image; }
            }
            let label_color = if hovered {
                Color::srgb(0.9, 0.98, 1.0)
            } else if selected {
                Color::srgb(0.81, 0.93, 1.0)
            } else {
                Color::srgb(0.82, 0.95, 0.98)
            };
            for child in children.iter() {
                if let Ok(mut color) = labels.get_mut(child) {
                    if color.0 != label_color {
                        color.0 = label_color;
                    }
                }
            }
        }
    }
}
