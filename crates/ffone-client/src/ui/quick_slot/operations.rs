use super::*;

#[must_use]
pub const fn quick_slot_component_enabled(
    config: QuickSlotUiConfig,
    preview: QuickSlotParityPreview,
) -> bool {
    config.localized_mode.draws_component() || preview.enabled
}

/// Resolves clean `CnGuiChat` group placement, nested `cnQuickSlot` groups,
/// and bottom-left UI scaling into absolute Bevy UI rectangles.
#[must_use]
pub fn quick_slot_ui_view(
    viewport_width: u32,
    viewport_height: u32,
    config: QuickSlotUiConfig,
    preview: QuickSlotParityPreview,
    model: &QuickSlotUiModel,
    static_assets_ready: bool,
) -> QuickSlotUiView {
    let mut view = QuickSlotUiView::default();
    let height = viewport_height as f32;
    view.scale = config.effective_ui_scale(height);
    if viewport_width == 0
        || viewport_height == 0
        || !static_assets_ready
        || !model.ready_for_play
        || !quick_slot_component_enabled(config, preview)
    {
        return view;
    }

    let group_left = config.chat_window_style.quick_slot_group_left();
    let group_top = height - QUICK_SLOT_CHAT_WINDOW_HEIGHT;
    let scale_rect = |rect: QuickSlotUiRect| rect.scaled_around_bottom_left(height, view.scale);
    view.visible = true;
    view.background = Some(scale_rect(QuickSlotUiRect::new(
        group_left + QUICK_SLOT_BACKGROUND_LOCAL_LEFT,
        group_top + QUICK_SLOT_BACKGROUND_LOCAL_TOP,
        QUICK_SLOT_BACKGROUND_WIDTH,
        QUICK_SLOT_BACKGROUND_HEIGHT,
    )));
    view.slots = array::from_fn(|logical_slot| {
        let entry = &model.slots[logical_slot];
        let frame = scale_rect(QuickSlotUiRect::new(
            group_left + QUICK_SLOT_PANEL_LOCAL_LEFT + logical_slot as f32 * QUICK_SLOT_STRIDE,
            group_top + QUICK_SLOT_PANEL_LOCAL_TOP,
            QUICK_SLOT_SIZE,
            QUICK_SLOT_SIZE,
        ));
        let occupied = entry.has_item_id() && entry.has_semantic_icon();
        let fraction = if occupied {
            entry.cooldown_fraction()
        } else {
            0.0
        };
        let cooldown = (fraction > 0.0).then(|| {
            scale_rect(QuickSlotUiRect::new(
                group_left
                    + QUICK_SLOT_PANEL_LOCAL_LEFT
                    + logical_slot as f32 * QUICK_SLOT_STRIDE
                    + QUICK_SLOT_COOLDOWN_INSET,
                group_top
                    + QUICK_SLOT_PANEL_LOCAL_TOP
                    + QUICK_SLOT_COOLDOWN_INSET
                    + QUICK_SLOT_COOLDOWN_SIZE * (1.0 - fraction),
                QUICK_SLOT_COOLDOWN_SIZE,
                QUICK_SLOT_COOLDOWN_SIZE * fraction,
            ))
        });
        QuickSlotView {
            logical_slot,
            frame,
            visual: if occupied {
                QuickSlotVisual::Occupied
            } else {
                QuickSlotVisual::Empty
            },
            pointer_enabled: occupied && !entry.inventory_empty && model.clicks_enabled,
            icon_visible: occupied,
            cooldown,
        }
    });
    view
}

pub(super) fn has_content_hash_suffix(path: &str) -> bool {
    let Some(stem) = path
        .rsplit('/')
        .next()
        .and_then(|name| name.strip_suffix(".png"))
    else {
        return false;
    };
    let bytes = stem.as_bytes();
    bytes.len() >= 18
        && &bytes[bytes.len() - 18..bytes.len() - 16] == b"--"
        && bytes[bytes.len() - 16..].iter().all(u8::is_ascii_hexdigit)
}

pub(super) fn bind_quick_slot_ui(
    asset_server: Res<AssetServer>,
    assets: Res<QuickSlotUiRuntimeAssets>,
    config: Res<QuickSlotUiConfig>,
    preview: Res<QuickSlotParityPreview>,
    model: Res<QuickSlotUiModel>,
    mission_ui: Option<Res<MissionUiModel>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<(&mut Node, &mut Visibility), With<QuickSlotUiRoot>>,
    mut elements: Query<(&QuickSlotUiElement, &mut Node, &mut ImageNode), Without<QuickSlotUiRoot>>,
) {
    let Ok(window) = windows.single() else {
        for (_, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let Some(static_assets) = assets.0.as_ref() else {
        for (mut node, mut visibility) in &mut roots {
            node.width = px(window.width().max(0.0));
            node.height = px(window.height().max(0.0));
            *visibility = Visibility::Hidden;
        }
        return;
    };

    let mut render_model = model.clone();
    let icon_handles: [Option<Handle<Image>>; QUICK_SLOT_COUNT] = array::from_fn(|slot| {
        let entry = &mut render_model.slots[slot];
        let handle = entry
            .icon_path
            .as_deref()
            .filter(|path| is_semantic_png_path(path))
            .map(|path| asset_server.load::<Image>(path.to_owned()));
        let loaded = handle.as_ref().is_some_and(|handle| {
            matches!(asset_server.load_state(handle.id()), LoadState::Loaded)
        });
        if !loaded {
            entry.icon_path = None;
        }
        handle.filter(|_| loaded)
    });
    let width = window.width().max(0.0) as u32;
    let height = window.height().max(0.0) as u32;
    let view = quick_slot_ui_view(
        width,
        height,
        *config,
        *preview,
        &render_model,
        static_assets.all_loaded(&asset_server),
    );

    for (mut node, mut visibility) in &mut roots {
        node.width = px(width);
        node.height = px(height);
        *visibility = if gameplay_chrome_visible(view.visible, mission_ui.as_deref()) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (element, mut node, mut image) in &mut elements {
        match *element {
            QuickSlotUiElement::Background => {
                bind_optional_rect(&mut node, view.background);
                image.image = static_assets.background.clone();
            }
            QuickSlotUiElement::SlotFrame(slot) => {
                let slot_view = view.slots[slot];
                bind_optional_rect(&mut node, view.visible.then_some(slot_view.frame));
                image.image = match slot_view.visual {
                    QuickSlotVisual::Empty => static_assets.empty_style.clone(),
                    QuickSlotVisual::Occupied => static_assets.occupied_style.clone(),
                };
            }
            QuickSlotUiElement::SlotIcon(slot) => {
                let slot_view = view.slots[slot];
                bind_optional_rect(
                    &mut node,
                    (view.visible && slot_view.icon_visible).then_some(slot_view.frame),
                );
                image.image = icon_handles[slot].clone().unwrap_or_default();
            }
            QuickSlotUiElement::Cooldown(slot) => {
                bind_optional_rect(&mut node, view.slots[slot].cooldown);
                image.image = static_assets.cooldown.clone();
            }
        }
    }
}
