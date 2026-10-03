use super::*;

pub(super) fn current_page_id(catalog: &GameGuideCatalog, model: &GameGuideUiModel) -> Option<usize> {
    let range = catalog.topics.get(model.selected_main_topic)?;
    let page_id = range.start.checked_add(model.selected_sub_topic)?;
    (page_id <= range.end).then_some(page_id)
}

pub(super) fn close_game_guide(
    model: &mut GameGuideUiModel,
    audio: &mut Option<ResMut<GameplayAudioRuntime>>,
) {
    model.close();
    if let Some(audio) = audio.as_mut() {
        audio.queue_gameplay_ui_sound("Close_Screen");
    }
}

pub(super) fn handle_game_guide_controls(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    controls: Query<(&Interaction, &GameGuideControl), Changed<Interaction>>,
    stacks: Query<&ComputedNode, With<GameGuideContentStack>>,
    catalog: Res<GameGuideCatalog>,
    mut model: ResMut<GameGuideUiModel>,
    mut audio: Option<ResMut<GameplayAudioRuntime>>,
) {
    if !model.visible {
        return;
    }
    if keyboard
        .as_ref()
        .is_some_and(|keyboard| keyboard.just_pressed(KeyCode::Escape))
    {
        close_game_guide(&mut model, &mut audio);
        return;
    }
    let content_height = stacks
        .single()
        .map_or(CONTENT_VIEW_HEIGHT, |node| node.size().y);
    let max_scroll = (content_height - CONTENT_VIEW_HEIGHT).max(0.0);
    if let Some(scroll) = mouse_scroll.as_ref() {
        let axis = scroll.delta.y.clamp(-1.0, 1.0);
        if axis != 0.0 {
            model.content_scroll_y =
                (model.content_scroll_y - axis * LEGACY_SCROLL_STEP).clamp(0.0, max_scroll);
        }
    }
    for (interaction, control) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *control {
            GameGuideControl::Main(main) if main < catalog.topics.len() => {
                model.selected_main_topic = main;
                model.selected_sub_topic = 0;
                model.content_scroll_y = 0.0;
            }
            GameGuideControl::Sub(sub) => {
                let range = &catalog.topics[model.selected_main_topic];
                if range.start + sub <= range.end {
                    model.selected_sub_topic = sub;
                    model.content_scroll_y = 0.0;
                }
            }
            GameGuideControl::Previous => step_page(&catalog, &mut model, -1),
            GameGuideControl::Next => step_page(&catalog, &mut model, 1),
            GameGuideControl::ScrollUp => {
                model.content_scroll_y = (model.content_scroll_y - LEGACY_SCROLL_STEP).max(0.0)
            }
            GameGuideControl::ScrollDown => {
                model.content_scroll_y =
                    (model.content_scroll_y + LEGACY_SCROLL_STEP).min(max_scroll)
            }
            GameGuideControl::Close => close_game_guide(&mut model, &mut audio),
            GameGuideControl::Main(_) => {}
        }
        break;
    }
}

pub(super) fn step_page(catalog: &GameGuideCatalog, model: &mut GameGuideUiModel, delta: isize) {
    let Some(range) = catalog.topics.get(model.selected_main_topic) else {
        return;
    };
    let page = range.start + model.selected_sub_topic;
    let first = catalog.topics.get(1).map_or(page, |range| range.start);
    let last = catalog.topics.last().map_or(first, |range| range.end);
    let next = if delta < 0 {
        if page == first { last } else { page - 1 }
    } else if page == last {
        first
    } else {
        page + 1
    };
    if let Some((main, range)) = catalog
        .topics
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, range)| (range.start..=range.end).contains(&next))
    {
        model.selected_main_topic = main;
        model.selected_sub_topic = next - range.start;
        model.content_scroll_y = 0.0;
    }
}

pub(super) fn page_string_key(string_id: usize) -> String {
    format!("content.tabledata.help.help_page_string.{string_id}.str_name")
}

pub(super) fn rebuild_game_guide_content(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    assets: Res<GameGuideAssets>,
    catalog: Res<GameGuideCatalog>,
    model: Res<GameGuideUiModel>,
    mut stacks: Query<(Entity, &mut GameGuideContentStack)>,
) {
    if !model.visible {
        return;
    }
    let Some(page_id) = current_page_id(&catalog, &model) else {
        return;
    };
    let Ok((entity, mut rendered)) = stacks.single_mut() else {
        return;
    };
    if rendered.page_id == page_id {
        return;
    }
    rendered.page_id = page_id;
    let Some(page) = catalog.pages.get(page_id) else {
        return;
    };
    let entries = catalog
        .content
        .get(page.start..=page.end)
        .unwrap_or_default()
        .to_vec();

    commands
        .entity(entity)
        .despawn_related::<Children>()
        .with_children(|stack| {
            for entry in entries {
                let fallback = catalog
                    .page_names
                    .get(entry.string_id)
                    .cloned()
                    .unwrap_or_default();
                match entry.kind {
                    3 => {
                        stack
                            .spawn((
                                Node {
                                    width: px(GAME_GUIDE_CONTENT_RECT.width),
                                    height: px(22.0),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                                ImageNode {
                                    image: assets.title_bar.clone(),
                                    image_mode: NodeImageMode::Stretch,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|row| {
                                spawn_text(
                                    row,
                                    GuideUiRect::new(8.0, 0.0, 782.0, 22.0),
                                    page_string_key(entry.string_id),
                                    fallback,
                                    assets.chalet_font.clone(),
                                    14.0,
                                    14.08,
                                    LABEL_CYAN,
                                    Justify::Left,
                                );
                            });
                    }
                    4 => {
                        stack.spawn((
                            Node {
                                width: px(GAME_GUIDE_CONTENT_RECT.width),
                                height: px(20.0),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            Text::new(fallback.clone()),
                            LocalizedText::new(page_string_key(entry.string_id), fallback),
                            (
                                TextFont {
                                    font: (assets.chalet_font.clone()).into(),
                                    font_size: (14.0).into(),
                                    ..default()
                                },
                                LineHeight::Px(14.08),
                            ),
                            TextColor(Color::BLACK),
                            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                            Pickable::IGNORE,
                        ));
                    }
                    1 => {
                        let justify = if entry.size == 4 {
                            Justify::Center
                        } else {
                            Justify::Left
                        };
                        stack.spawn((
                            Node {
                                width: px(GAME_GUIDE_CONTENT_RECT.width),
                                flex_shrink: 0.0,
                                ..default()
                            },
                            Text::new(fallback.clone()),
                            LocalizedText::new(page_string_key(entry.string_id), fallback),
                            (
                                TextFont {
                                    font: (assets.chalet_font.clone()).into(),
                                    font_size: (12.0).into(),
                                    ..default()
                                },
                                LineHeight::Px(12.07),
                            ),
                            TextColor(BODY_BLUE),
                            TextLayout::new(justify, LineBreak::WordBoundary),
                            Pickable::IGNORE,
                        ));
                    }
                    2 => {
                        let filename = catalog
                            .page_comments
                            .get(entry.string_id)
                            .map(String::as_str)
                            .unwrap_or_default();
                        stack
                            .spawn((
                                Node {
                                    width: px(GAME_GUIDE_CONTENT_RECT.width),
                                    height: px(GAME_GUIDE_SCREENSHOT_RECT.height),
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|row| {
                                if let Some(path) = screenshot_asset_path(filename) {
                                    row.spawn((
                                        rect_node(GuideUiRect::new(
                                            GAME_GUIDE_SCREENSHOT_RECT.x
                                                - GAME_GUIDE_CONTENT_RECT.x,
                                            0.0,
                                            GAME_GUIDE_SCREENSHOT_RECT.width,
                                            GAME_GUIDE_SCREENSHOT_RECT.height,
                                        )),
                                        ImageNode {
                                            image: asset_server.load(path),
                                            image_mode: NodeImageMode::Stretch,
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ));
                                }
                            });
                    }
                    _ => {}
                }
            }
        });
}
