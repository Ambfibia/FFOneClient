use super::*;

#[must_use]
pub fn skill_buff_ui_view(
    viewport_width: u32,
    model: &SkillBuffUiModel,
    catalog: &SkillBuffUiCatalog,
) -> SkillBuffUiView {
    if !model.visible || !catalog.is_ready() {
        return SkillBuffUiView::default();
    }
    let scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
        model.ui_scale
    } else {
        1.0
    };

    let local_ids = local_buff_ids(model.local_condition_bit_flag);
    let cash_ids = cash_buff_ids(model.cash_condition_bit_flag);
    let target_ids = model
        .target
        .map(|target| target_buff_ids(target.condition_bit_flag, model.nano_styles))
        .unwrap_or_default();

    SkillBuffUiView {
        visible: true,
        local: project_icons(
            &local_ids,
            IconProjection::Local,
            viewport_width,
            scale,
            model,
            catalog,
        ),
        cash: project_icons(
            &cash_ids,
            IconProjection::Cash,
            viewport_width,
            scale,
            model,
            catalog,
        ),
        target: project_icons(
            &target_ids,
            IconProjection::Target,
            viewport_width,
            scale,
            model,
            catalog,
        ),
    }
}

pub(super) fn local_buff_ids(condition: u32) -> Vec<i32> {
    let mut ids = enabled_ids(condition, &SKILL_BUFF_DEBUFFS);
    ids.extend(
        SKILL_BUFF_BUFFS
            .iter()
            .filter(|(flag, _)| condition & *flag != 0 && !STIM_FLAGS.contains(flag))
            .map(|(_, buff_id)| *buff_id),
    );
    ids
}

pub(super) fn cash_buff_ids(condition: u32) -> Vec<i32> {
    enabled_ids(condition, &SKILL_BUFF_BUFFS)
}

pub(super) fn target_buff_ids(condition: u32, nano_styles: [Option<u8>; 3]) -> Vec<i32> {
    let mut ids = enabled_ids(condition, &SKILL_BUFF_DEBUFFS);
    for (flag, regular_buff_id) in SKILL_BUFF_BUFFS {
        if condition & flag == 0 {
            continue;
        }
        let buff_id = if let Some(slot) = STIM_FLAGS.iter().position(|stim_flag| *stim_flag == flag)
        {
            // `sync_skill_buff_ui_context` represents a clean empty Nano slot
            // as style 0. `None` is reserved for a nonzero Nano ID that the
            // native content cannot resolve, so keep that corrupt input
            // fail-closed instead of inventing a style for it.
            let Some(style) = nano_styles[slot] else {
                continue;
            };
            21 + i32::from(style)
        } else {
            regular_buff_id
        };
        ids.push(buff_id);
    }
    ids
}

pub(super) fn enabled_ids(condition: u32, entries: &[(u32, i32)]) -> Vec<i32> {
    entries
        .iter()
        .filter(|(flag, _)| condition & *flag != 0)
        .map(|(_, buff_id)| *buff_id)
        .collect()
}

#[must_use]
pub fn format_cash_time(milliseconds: u64) -> String {
    let mut value = milliseconds / 1_000;
    let mut suffix = "s";
    if value >= 60 {
        value /= 60;
        suffix = "m";
    }
    if suffix == "m" && value >= 60 {
        value /= 60;
        suffix = "h";
    }
    if suffix == "h" && value >= 24 {
        // This is an exact clean-client bug: the hour-to-day conversion uses
        // 60 instead of 24.
        value /= 60;
        suffix = "d";
    }
    format!("{value}{suffix}")
}

pub(super) fn bind_skill_buff_ui(
    model: Res<SkillBuffUiModel>,
    mission_ui: Option<Res<MissionUiModel>>,
    catalog: Res<SkillBuffUiCatalog>,
    assets: Res<SkillBuffUiAssets>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<
        (&mut Node, &mut Visibility),
        (With<SkillBuffUiRoot>, Without<SkillBuffUiElement>),
    >,
    mut elements: Query<
        (
            &SkillBuffUiElement,
            &mut Node,
            Option<&mut ImageNode>,
            Option<(&mut TextFont, &mut LineHeight)>,
            Option<&mut LocalizedText>,
        ),
        Without<SkillBuffUiRoot>,
    >,
) {
    let Ok(window) = windows.single() else {
        for (_, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let width = window.width().max(0.0) as u32;
    let height = window.height().max(0.0);
    let view = skill_buff_ui_view(width, &model, &catalog);
    for (mut node, mut visibility) in &mut roots {
        node.width = px(width);
        node.height = px(height);
        *visibility = if gameplay_chrome_visible(view.visible, mission_ui.as_deref()) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    let scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
        model.ui_scale
    } else {
        1.0
    };
    for (element, mut node, image, font, localized) in &mut elements {
        match *element {
            SkillBuffUiElement::Slot(row, index) => {
                let icon = view.row(row).get(index);
                node.display = if icon.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                if let Some(icon) = icon {
                    bind_rect(&mut node, icon.rect);
                }
            }
            SkillBuffUiElement::Icon(row, index) => {
                if let Some(mut image) = image
                    && let Some(icon) = view.row(row).get(index)
                    && let Some(handle) = assets.icons.get(&icon.icon_path)
                {
                    image.image = handle.clone();
                }
            }
            SkillBuffUiElement::CashTime(_) => {
                let spec = SkillBuffTextStyle::HudLabel.spec();
                node.left = px(2.0 * scale);
                node.top = px((19.0 + spec.y_offset) * scale);
                node.width = px(SKILL_BUFF_ICON_SIZE * scale);
                node.height = px(SKILL_BUFF_ICON_SIZE * scale);
                node.overflow = Overflow::clip();
            }
            SkillBuffUiElement::CashText(index) => {
                let value = view
                    .cash
                    .get(index)
                    .and_then(|icon| icon.cash_time.as_deref())
                    .unwrap_or_default()
                    .to_owned();
                if let Some(mut localized) = localized {
                    *localized = skill_buff_cash_time_localized(value);
                }
                if let Some(mut font) = font {
                    let spec = SkillBuffTextStyle::HudLabel.spec();
                    font.0.font_size = (spec.font_size * scale).into();
                    *font.1 = LineHeight::Px(spec.line_height * scale);
                }
            }
        }
    }
}
