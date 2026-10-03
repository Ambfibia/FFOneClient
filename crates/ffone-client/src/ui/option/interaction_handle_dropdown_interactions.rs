use super::*;

pub(super) fn handle_option_scroll_wheel(
    mouse_scroll: Option<Res<AccumulatedMouseScroll>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut model: ResMut<OptionUiModel>,
) {
    if !model.visible || !model.page_body_enabled(model.selected_tab) {
        return;
    }
    let axis = mouse_scroll.as_ref().map_or(0.0, |scroll| scroll.delta.y);
    if !axis.is_finite() || axis == 0.0 {
        return;
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let layout = option_ui_layout(
        Vec2::new(window.width(), window.height()),
        model.draft_options.display.scale_ui && window.resolution.scale_factor_override().is_none(),
    );
    if layout.scale.x <= f32::EPSILON || layout.scale.y <= f32::EPSILON {
        return;
    }
    let point = Vec2::new(
        (cursor.x - layout.visual.x) / layout.scale.x,
        (cursor.y - layout.visual.y) / layout.scale.y,
    );
    let contains = |rect: OptionUiRect| {
        point.x >= rect.x
            && point.x < rect.x + rect.width
            && point.y >= rect.y
            && point.y < rect.y + rect.height
    };
    match model.selected_tab {
        OptionTab::Controls => {
            let viewport = OptionUiRect::new(
                OPTION_PAGE_RECT.x + OPTION_KEYMAP_CONTENT_RECT.x,
                OPTION_PAGE_RECT.y + OPTION_KEYMAP_CONTENT_RECT.y,
                OPTION_KEYMAP_CONTENT_RECT.width,
                OPTION_KEYMAP_CONTENT_RECT.height,
            );
            if contains(viewport) {
                model.scroll_controls(-axis * 20.0);
            }
        }
        OptionTab::Social => {
            let viewport = OptionUiRect::new(
                OPTION_PAGE_RECT.x + OPTION_SOCIAL_BLOCKED_LIST_RECT.x + 5.0,
                OPTION_PAGE_RECT.y + OPTION_SOCIAL_BLOCKED_LIST_RECT.y + 5.0,
                OPTION_SOCIAL_BLOCKED_LIST_RECT.width - 10.0,
                OPTION_SOCIAL_BLOCKED_LIST_RECT.height - 10.0,
            );
            if contains(viewport) {
                let rows = axis.abs().ceil().max(1.0) as isize;
                model.scroll_blocked_rows(if axis > 0.0 { -rows } else { rows });
            }
        }
        OptionTab::Graphics | OptionTab::GameUi => {}
    }
}

pub(super) fn handle_dropdown_interactions(
    buttons: Query<(&Interaction, &OptionDropdownButton), Changed<Interaction>>,
    choices: Query<(&Interaction, &OptionDropdownChoice), Changed<Interaction>>,
    mut model: ResMut<OptionUiModel>,
    mut outbox: ResMut<OptionUiOutbox>,
    localization: Option<Res<Localization>>,
    mut language: Option<ResMut<Language>>,
    mut voice_language: Option<ResMut<VoiceLanguage>>,
) {
    for (interaction, button) in &buttons {
        if matches!(
            button.0,
            OptionDropdownKind::Translation | OptionDropdownKind::Voice
        ) && (!model.visible || model.selected_tab != OptionTab::Social)
        {
            continue;
        }
        if *interaction == Interaction::Pressed && model.open_dropdown(button.0) {
            model.emit_button_click(&mut outbox);
        }
    }
    for (interaction, choice) in &choices {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let locale_kind = match choice {
            OptionDropdownChoice::Translation(_) => Some(OptionDropdownKind::Translation),
            OptionDropdownChoice::Voice(_) => Some(OptionDropdownKind::Voice),
            _ => None,
        };
        if let Some(kind) = locale_kind {
            if !model.visible
                || model.selected_tab != OptionTab::Social
                || model.modal.system_popup
                || model.modal.help
                || model.key_capture.is_some()
                || !dropdown_open(&model.modal, kind)
            {
                continue;
            }
        }
        let changed = match choice {
            OptionDropdownChoice::Resolution {
                width,
                height,
                windowed,
            } => model.set_resolution(*width, *height, *windowed),
            OptionDropdownChoice::Detail(detail) => model.set_graphics_detail(*detail),
            OptionDropdownChoice::Shadow(shadow) => model.set_shadow_quality(*shadow),
            OptionDropdownChoice::Texture(texture) => model.set_texture_quality(*texture),
            OptionDropdownChoice::Pad(profile) => model.set_pad_profile(*profile),
            OptionDropdownChoice::Translation(locale) => {
                model.close_dropdowns();
                if let (Some(localization), Some(language)) =
                    (localization.as_deref(), language.as_deref_mut())
                {
                    let changed = language.effective != *locale;
                    localization.select(language, locale);
                    changed
                } else {
                    false
                }
            }
            OptionDropdownChoice::Voice(locale) => {
                model.close_dropdowns();
                if let Some(voice_language) = voice_language.as_deref_mut() {
                    let changed = voice_language.effective != *locale;
                    voice_language.select(locale);
                    changed
                } else {
                    false
                }
            }
        };
        if changed {
            model.emit_button_click(&mut outbox);
        }
    }
}

pub(super) fn handle_system_popup_interactions(
    buttons: Query<&Interaction, (Changed<Interaction>, With<OptionSystemPopupOkButton>)>,
    mut model: ResMut<OptionUiModel>,
    mut outbox: ResMut<OptionUiOutbox>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed && model.acknowledge_popup() {
            model.emit_button_click(&mut outbox);
        }
    }
}

pub(super) fn update_chrome_button_visuals(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    mut buttons: Query<(
        &OptionChromeButton,
        &Interaction,
        &mut ImageNode,
        &mut Pickable,
    )>,
) {
    for (button, interaction, mut image, mut pickable) in &mut buttons {
        let enabled =
            model.chrome_enabled() && !(*button == OptionChromeButton::Apply && model.clean);
        let hover = *interaction == Interaction::Hovered && enabled;
        image.image = match button {
            OptionChromeButton::Close => assets.image(if hover {
                OptionTextureRole::CloseHover
            } else {
                OptionTextureRole::Close
            }),
            OptionChromeButton::Apply | OptionChromeButton::Save => assets.image(if hover {
                OptionTextureRole::ButtonHover
            } else {
                OptionTextureRole::Button
            }),
        };
        image.color = if enabled {
            Color::WHITE
        } else {
            Color::srgba(0.55, 0.55, 0.55, 0.9)
        };
        *pickable = pickable_for(enabled);
    }
}

pub(super) fn update_page_button_visuals(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    mut buttons: Query<
        (&Interaction, &mut ImageNode),
        (
            With<OptionPageButtonVisual>,
            Without<OptionChromeButton>,
            Without<OptionSocialDefaultsButton>,
            Without<OptionUnignoreButton>,
        ),
    >,
) {
    let modal_choice_active = model.modal.resolution_dropdown
        || model.modal.detail_dropdown
        || model.modal.shadow_dropdown
        || model.modal.texture_dropdown
        || model.modal.pad_dropdown
        || model.modal.system_popup;
    for (interaction, mut image) in &mut buttons {
        image.image = assets.image(
            if *interaction == Interaction::Hovered
                && (model.chrome_enabled() || modal_choice_active)
            {
                OptionTextureRole::ButtonHover
            } else {
                OptionTextureRole::Button
            },
        );
    }
}

pub(super) fn update_option_button_labels(
    buttons: Query<(&Interaction, &Children), With<Button>>,
    mut labels: Query<&mut TextColor, With<OptionButtonLabel>>,
) {
    for (interaction, children) in &buttons {
        let color = match interaction {
            Interaction::Hovered => Color::srgb(0.0, 0.278_431_4, 0.478_431_37),
            Interaction::Pressed => Color::WHITE,
            Interaction::None => Color::srgb(0.9, 0.9, 0.9),
        };
        for child in children.iter() {
            if let Ok(mut text_color) = labels.get_mut(child) {
                text_color.0 = color;
            }
        }
    }
}

pub(super) fn update_key_button_labels(
    model: Res<OptionUiModel>,
    buttons: Query<(
        &Interaction,
        &Children,
        Option<&OptionChromeButton>,
        Option<&OptionMappingButton>,
    )>,
    mut labels: Query<&mut TextColor, With<OptionKeyButtonLabel>>,
) {
    for (interaction, children, chrome, mapping) in &buttons {
        let enabled = if let Some(chrome) = chrome {
            model.chrome_enabled() && !(*chrome == OptionChromeButton::Apply && model.clean)
        } else if mapping.is_some() {
            model.selected_tab == OptionTab::Controls
                && model.page_body_enabled(OptionTab::Controls)
        } else {
            true
        };
        let color = if !enabled {
            Color::srgba(0.55, 0.55, 0.55, 0.9)
        } else if *interaction == Interaction::Hovered {
            Color::srgb(0.229_838_71, 0.463_709_68, 1.0)
        } else {
            Color::WHITE
        };
        for child in children.iter() {
            if let Ok(mut text_color) = labels.get_mut(child) {
                text_color.0 = color;
            }
        }
    }
}

pub(super) fn update_social_button_visuals(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    mut defaults: Query<
        (&Interaction, &mut ImageNode, &mut Pickable),
        (
            With<OptionSocialDefaultsButton>,
            Without<OptionUnignoreButton>,
        ),
    >,
    mut unignore: Query<
        (&Interaction, &mut ImageNode, &mut Pickable),
        (
            With<OptionUnignoreButton>,
            Without<OptionSocialDefaultsButton>,
        ),
    >,
) {
    let body_enabled =
        model.selected_tab == OptionTab::Social && model.page_body_enabled(OptionTab::Social);
    for (interaction, mut image, mut pickable) in &mut defaults {
        image.image = assets.image(if *interaction == Interaction::Hovered && body_enabled {
            OptionTextureRole::ButtonHover
        } else {
            OptionTextureRole::Button
        });
        *pickable = if body_enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
    let unignore_enabled = body_enabled;
    for (interaction, mut image, mut pickable) in &mut unignore {
        image.image = assets.image(
            if *interaction == Interaction::Hovered && unignore_enabled {
                OptionTextureRole::ButtonHover
            } else {
                OptionTextureRole::Button
            },
        );
        image.color = Color::WHITE;
        *pickable = if unignore_enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
}
