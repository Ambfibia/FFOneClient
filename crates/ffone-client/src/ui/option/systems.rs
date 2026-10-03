use super::*;

pub(super) fn option_ui_visibility_needs_update(
    model: Res<OptionUiModel>,
    gate: Res<OptionUiAssetGate>,
    roots: Query<(), Added<OptionUiRoot>>,
) -> bool {
    model.is_changed() || gate.is_changed() || !roots.is_empty()
}

pub(super) fn option_ui_bindings_need_update(
    model: Res<OptionUiModel>,
    gate: Res<OptionUiAssetGate>,
    assets: Option<Res<OptionUiAssets>>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    voice_language: Option<Res<VoiceLanguage>>,
    windows: Query<(), (With<PrimaryWindow>, Changed<Window>)>,
    roots: Query<(), Added<OptionUiRoot>>,
) -> bool {
    model.is_changed()
        || gate.is_changed()
        || assets.is_some_and(|assets| assets.is_changed())
        || localization.is_some_and(|localization| localization.is_changed())
        || language.is_some_and(|language| language.is_changed())
        || voice_language.is_some_and(|voice_language| voice_language.is_changed())
        || !windows.is_empty()
        || !roots.is_empty()
}

pub(super) fn option_ui_visuals_need_update(
    model: Res<OptionUiModel>,
    assets: Option<Res<OptionUiAssets>>,
    interactions: Query<(), Changed<Interaction>>,
    roots: Query<(), Added<OptionUiRoot>>,
) -> bool {
    model.is_changed()
        || assets.is_some_and(|assets| assets.is_changed())
        || !interactions.is_empty()
        || !roots.is_empty()
}

pub(super) fn tick_option_key_capture(time: Res<Time>, mut model: ResMut<OptionUiModel>) {
    if model.key_capture.is_none() {
        return;
    }
    model.tick_key_capture(time.delta_secs());
}

pub(super) fn update_tab_visuals(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    mut tabs: Query<(
        &OptionTabButton,
        &Interaction,
        &mut Node,
        &mut ImageNode,
        &mut ZIndex,
        &mut Pickable,
        &Children,
    )>,
    mut labels: Query<(&OptionTabLabel, &mut TextColor), Without<OptionTabButton>>,
) {
    for (tab, interaction, mut node, mut image, mut z_index, mut pickable, children) in &mut tabs {
        let selected = model.selected_tab == tab.0;
        let hovered = *interaction == Interaction::Hovered && model.chrome_enabled();
        let rect = if selected {
            tab.0.selected_rect()
        } else {
            tab.0.normal_rect()
        };
        node.left = px(rect.x);
        node.top = px(rect.y);
        node.width = px(rect.width);
        node.height = px(rect.height);
        let (justify_content, padding) = option_tab_content_style(tab.0, selected);
        node.justify_content = justify_content;
        node.align_items = AlignItems::Center;
        node.padding = padding;
        z_index.0 = if selected {
            OPTION_SELECTED_TAB_Z_INDEX
        } else {
            OPTION_NORMAL_TAB_Z_INDEX
        };
        image.image = assets.image(if selected {
            tab_selected_role(tab.0)
        } else if hovered {
            tab_hover_role(tab.0)
        } else {
            tab_normal_role(tab.0)
        });
        image.image_mode = option_tab_image_mode(tab.0, selected);
        for child in children.iter() {
            if let Ok((label, mut color)) = labels.get_mut(child)
                && label.0 == tab.0
            {
                color.0 = if hovered && !selected {
                    Color::srgb(0.0, 0.282, 0.478)
                } else {
                    Color::srgb(0.8, 1.0, 1.0)
                };
            }
        }
        *pickable = if model.chrome_enabled() {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
}

pub(super) fn update_dropdown_visuals(
    assets: Res<OptionUiAssets>,
    mut dropdown_buttons: Query<
        (&Interaction, &mut ImageNode, &Children),
        (
            With<OptionDropdownButton>,
            Without<OptionDropdownChoice>,
            Without<OptionDropdownChoiceBackground>,
        ),
    >,
    dropdown_choices: Query<
        (&Interaction, &Children),
        (With<OptionDropdownChoice>, Without<OptionDropdownButton>),
    >,
    mut choice_backgrounds: Query<
        &mut ImageNode,
        (
            With<OptionDropdownChoiceBackground>,
            Without<OptionDropdownButton>,
        ),
    >,
    mut value_labels: Query<
        &mut TextColor,
        (
            With<OptionDropdownValueLabel>,
            Without<OptionDropdownChoiceLabel>,
        ),
    >,
    mut choice_labels: Query<
        &mut TextColor,
        (
            With<OptionDropdownChoiceLabel>,
            Without<OptionDropdownValueLabel>,
        ),
    >,
) {
    for (interaction, mut image, children) in &mut dropdown_buttons {
        let hovered = *interaction == Interaction::Hovered;
        image.image = assets.image(if hovered {
            OptionTextureRole::PulldownHover
        } else {
            OptionTextureRole::Pulldown
        });
        for child in children.iter() {
            if let Ok(mut color) = value_labels.get_mut(child) {
                color.0 = if hovered {
                    Color::srgb(0.0, 0.384_313_73, 0.658_823_55)
                } else {
                    Color::srgb(0.0, 0.383_064_5, 0.641_129)
                };
            }
        }
    }
    for (interaction, children) in &dropdown_choices {
        let hovered = *interaction == Interaction::Hovered;
        for child in children.iter() {
            if let Ok(mut image) = choice_backgrounds.get_mut(child) {
                image.image = assets.image(OptionTextureRole::DropdownItemHover);
                image.color = if hovered {
                    Color::WHITE
                } else {
                    Color::srgba(1.0, 1.0, 1.0, 0.0)
                };
            }
            if let Ok(mut color) = choice_labels.get_mut(child) {
                color.0 = if hovered {
                    Color::srgb(0.0, 0.100_806_45, 0.487_903_24)
                } else {
                    Color::srgb(0.584_677_4, 1.0, 0.995_967_75)
                };
            }
        }
    }
}
