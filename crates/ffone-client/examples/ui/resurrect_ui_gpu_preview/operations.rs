use super::*;

pub(super) fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mode: Res<PreviewMode>,
    mut model: ResMut<ResurrectUiModel>,
    mut context: ResMut<ResurrectUiContext>,
    mut outbox: ResMut<ResurrectUiOutbox>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    spawn_world_backdrop(&mut commands);

    *context = mode.context();
    model.enter(0, &mut outbox);
    outbox.clear();

    commands.insert_resource(PreviewAssets {
        images: RESURRECT_TEXTURE_CONTRACTS
            .iter()
            .map(|contract| PreviewImage {
                role: contract.role,
                handle: asset_server.load(contract.runtime_path),
            })
            .collect(),
        fonts: [
            asset_server.load(RESURRECT_BUTTON_FONT_PATH),
            asset_server.load(RESURRECT_BODY_FONT_PATH),
        ],
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    localization: Res<Localization>,
    language: Res<Language>,
    language_slug: Res<PreviewLanguage>,
    mode: Res<PreviewMode>,
    model: Res<ResurrectUiModel>,
    context: Res<ResurrectUiContext>,
    outbox: Res<ResurrectUiOutbox>,
    mut state: ResMut<PreviewState>,
    queries: CaptureQueries,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);

    let asset_failed = preview_assets.images.iter().any(|image| {
        matches!(
            asset_server.load_state(image.handle.id()),
            LoadState::Failed(_)
        )
    }) || preview_assets
        .fonts
        .iter()
        .any(|font| matches!(asset_server.load_state(font.id()), LoadState::Failed(_)));
    if asset_failed {
        eprintln!("ResurrectMode acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets.images.iter().all(|image| {
        matches!(
            asset_server.load_state(image.handle.id()),
            LoadState::Loaded
        )
    }) && preview_assets
        .fonts
        .iter()
        .all(|font| matches!(asset_server.load_state(font.id()), LoadState::Loaded));
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|image| images.get(&image.handle).is_some())
        && preview_assets
            .fonts
            .iter()
            .all(|font| fonts.get(font).is_some());

    let viewport = Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32);
    let ui_scale = clean_resurrect_ui_scale(CLIENT_AREA_HEIGHT as f32);
    let layout = resurrect_ui_layout(viewport, ui_scale);
    let mut matching_roots = queries
        .roots
        .iter()
        .filter(|(_, _, z_index)| z_index.0 == RESURRECT_UI_Z_INDEX);
    let root_exact = matching_roots.next().is_some_and(|(node, computed, _)| {
        node.display == Display::Flex
            && node_matches_rect(
                node,
                ResurrectUiRect::new(
                    0.0,
                    0.0,
                    CLIENT_AREA_WIDTH as f32,
                    CLIENT_AREA_HEIGHT as f32,
                ),
            )
            && computed.size() == viewport
    }) && matching_roots.next().is_none();

    let dialog_id = preview_assets.image(ResurrectTextureRole::Dialog).id();
    let mut dialogs = queries
        .image_nodes
        .iter()
        .filter(|(_, _, image, _, _)| image.image.id() == dialog_id);
    let dialog_exact = dialogs
        .next()
        .is_some_and(|(node, computed, _, transform, _)| {
            node_matches_rect(
                node,
                ResurrectUiRect::new(
                    layout.dialog_left,
                    layout.dialog_top,
                    layout.source_size.x,
                    layout.source_size.y,
                ),
            ) && transform.is_some_and(|transform| transform.scale == Vec2::ONE)
                && computed.size() == layout.source_size
        })
        && dialogs.next().is_none();

    let buttons_exact =
        button_contract_is_exact(*mode, &preview_assets, &queries.buttons, &queries.text_copy);
    let phoenix_exact =
        phoenix_visual_contract_is_exact(*mode, &preview_assets, &queries.image_nodes);
    let text_audit = audit_text_contract(
        *mode,
        &localization,
        &language,
        &language_slug.0,
        &preview_assets,
        &queries.all_texts,
        &queries.text_styles,
        &queries.computed_nodes,
    );
    let text_exact = text_audit.is_ok();
    if assets_loaded && text_exact && !state.text_audited {
        state.text_audited = true;
        println!("{}", text_audit.as_deref().expect("checked above"));
    }
    let model_exact = model.visible
        && !model.request_sent
        && model.elapsed_seconds == 0.0
        && model.countdown_seconds() == RESURRECT_TIMEOUT_SECONDS as i32
        && model.last_blocker.is_none()
        && outbox.is_empty();
    let context_exact = *context == mode.context()
        && match *mode {
            PreviewMode::PhoenixSelf => {
                context.choice_draw_order()
                    == [
                        Some(ResurrectChoice::NearestResurrectEm),
                        Some(ResurrectChoice::PhoenixSelf),
                        None,
                    ]
                    && context.topmost_choice_at(Vec2::new(60.0, 130.0))
                        == Some(ResurrectChoice::PhoenixSelf)
            }
            PreviewMode::GroupItem => {
                context.choice_draw_order()
                    == [
                        Some(ResurrectChoice::NearestResurrectEm),
                        Some(ResurrectChoice::PhoenixGroup),
                        Some(ResurrectChoice::UseItem),
                    ]
                    && !context.choice_is_visible(ResurrectChoice::PhoenixSelf)
                    && context.topmost_choice_at(Vec2::new(60.0, 130.0))
                        == Some(ResurrectChoice::UseItem)
            }
        };
    let boundary_exact = model.input_boundary(*context) == EXPECTED_INPUT_BOUNDARY;
    let ready = assets_loaded
        && cpu_assets_present
        && ui_scale == 1.0
        && root_exact
        && dialog_exact
        && buttons_exact
        && phoenix_exact
        && text_exact
        && model_exact
        && context_exact
        && boundary_exact;
    if ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }

    let warmed = state
        .ready_frame
        .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES_AFTER_LOAD)
        && state
            .ready_at
            .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if ready && warmed && !state.capture_issued {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        if let Err(error) = text_audit {
            eprintln!("ResurrectMode text audit: {error}");
        }
        eprintln!(
            "ResurrectMode {} capture timed out: assets={assets_loaded}/{cpu_assets_present} \
             scale={ui_scale} root={root_exact} dialog={dialog_exact} \
             buttons={buttons_exact} phoenix={phoenix_exact} text={text_exact} \
             model={model_exact} context={context_exact} boundary={boundary_exact} language={}",
            mode.cli_name(),
            language_slug.0
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

pub(super) fn phoenix_visual_contract_is_exact(
    mode: PreviewMode,
    assets: &PreviewAssets,
    image_nodes: &Query<
        (
            &Node,
            &ComputedNode,
            &ImageNode,
            Option<&UiTransform>,
            Option<&ZIndex>,
        ),
        Without<Button>,
    >,
) -> bool {
    let self_id = assets.image(ResurrectTextureRole::PhoenixSelf).id();
    let group_id = assets.image(ResurrectTextureRole::PhoenixGroup).id();
    let back_id = assets.image(ResurrectTextureRole::SkillIconBack).id();
    let self_visible = mode == PreviewMode::PhoenixSelf;
    let group_visible = mode == PreviewMode::GroupItem;

    let icon_contract = |target_id, should_be_visible| {
        let mut matches = image_nodes
            .iter()
            .filter(|(_, _, image, _, _)| image.image.id() == target_id);
        matches
            .next()
            .is_some_and(|(node, computed, _, _, z_index)| {
                node_matches_rect(node, RESURRECT_PHOENIX_SKILL_RECT)
                    && z_index.is_some_and(|z_index| z_index.0 == 1)
                    && node.display
                        == if should_be_visible {
                            Display::Flex
                        } else {
                            Display::None
                        }
                    && (!should_be_visible
                        || computed.size()
                            == Vec2::new(
                                RESURRECT_PHOENIX_SKILL_RECT.width,
                                RESURRECT_PHOENIX_SKILL_RECT.height,
                            ))
            })
            && matches.next().is_none()
    };

    let mut skill_backs = image_nodes
        .iter()
        .filter(|(_, _, image, _, _)| image.image.id() == back_id);
    let mut skill_back_count = 0;
    let mut visible_skill_backs = 0;
    let mut skill_backs_exact = true;
    for (node, computed, _, _, z_index) in &mut skill_backs {
        skill_back_count += 1;
        skill_backs_exact &= node_matches_rect(node, RESURRECT_PHOENIX_SKILL_BACK_RECT)
            && z_index.is_some_and(|z_index| z_index.0 == 1);
        if node.display == Display::Flex {
            visible_skill_backs += 1;
            skill_backs_exact &= computed.size()
                == Vec2::new(
                    RESURRECT_PHOENIX_SKILL_BACK_RECT.width,
                    RESURRECT_PHOENIX_SKILL_BACK_RECT.height,
                );
        } else {
            skill_backs_exact &= node.display == Display::None;
        }
    }

    icon_contract(self_id, self_visible)
        && icon_contract(group_id, group_visible)
        && skill_back_count == 2
        && visible_skill_backs == 1
        && skill_backs_exact
}
