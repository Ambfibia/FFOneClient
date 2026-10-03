use super::*;

impl Plugin for ResurrectUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<ResurrectUiModel>()
            .init_resource::<ResurrectUiContext>()
            .init_resource::<ResurrectUiText>()
            .init_resource::<ResurrectUiOutbox>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_resurrect_ui,
            )
            .configure_sets(
                Update,
                (
                    ResurrectUiSet::Domain,
                    ResurrectUiSet::Interaction,
                    ResurrectUiSet::Bind,
                    ResurrectUiSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (advance_resurrect_timer.in_set(ResurrectUiSet::Domain))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (handle_resurrect_interactions.in_set(ResurrectUiSet::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    sync_resurrect_visibility,
                    update_resurrect_layout,
                    update_resurrect_content.before(LocalizationSet::Apply),
                )
                    .chain()
                    .in_set(ResurrectUiSet::Bind))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (update_resurrect_button_visuals.in_set(ResurrectUiSet::Visuals))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

pub(super) fn advance_resurrect_timer(
    time: Res<Time>,
    context: Res<ResurrectUiContext>,
    mut model: ResMut<ResurrectUiModel>,
    mut outbox: ResMut<ResurrectUiOutbox>,
) {
    model.advance(time.delta_secs(), *context, &mut outbox);
}

pub(super) fn handle_resurrect_interactions(
    buttons: Query<(&Interaction, &ResurrectUiButton), Changed<Interaction>>,
    context: Res<ResurrectUiContext>,
    mut model: ResMut<ResurrectUiModel>,
    mut outbox: ResMut<ResurrectUiOutbox>,
) {
    for (interaction, button) in &buttons {
        if *interaction == Interaction::Pressed
            && model
                .activate_choice(button.choice, *context, &mut outbox)
                .is_ok()
        {
            break;
        }
    }
}

pub(super) fn sync_resurrect_visibility(
    model: Res<ResurrectUiModel>,
    mut roots: Query<&mut Node, With<ResurrectUiRoot>>,
) {
    for mut node in &mut roots {
        node.display = if model.visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(super) fn update_resurrect_layout(
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<ResurrectUiModel>,
    mut roots: Query<&mut Node, With<ResurrectUiRoot>>,
    mut dialogs: Query<
        (&mut Node, &mut UiTransform),
        (With<ResurrectDialog>, Without<ResurrectUiRoot>),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let layout = resurrect_ui_layout(viewport, model.effective_ui_scale(viewport.y));
    for mut root in &mut roots {
        root.width = px(layout.viewport.x);
        root.height = px(layout.viewport.y);
    }
    for (mut node, mut transform) in &mut dialogs {
        node.left = px(layout.dialog_left);
        node.top = px(layout.dialog_top);
        node.width = px(layout.source_size.x);
        node.height = px(layout.source_size.y);
        transform.scale = Vec2::splat(layout.scale);
    }
}

pub(super) fn update_resurrect_content(
    model: Res<ResurrectUiModel>,
    context: Res<ResurrectUiContext>,
    text_resource: Res<ResurrectUiText>,
    mut text_nodes: Query<(&ResurrectTextNode, &mut LocalizedText)>,
    mut button_labels: Query<
        (&ResurrectUiButtonLabel, &mut LocalizedText),
        Without<ResurrectTextNode>,
    >,
    mut buttons: Query<(&ResurrectUiButton, &mut Node, &mut Pickable)>,
    mut phoenix_visuals: Query<
        (&ResurrectPhoenixVisual, &mut Node),
        (Without<ResurrectUiButton>, Without<ResurrectTextNode>),
    >,
) {
    for (role, mut localized) in &mut text_nodes {
        let next_localized = match role.0 {
            ResurrectTextRole::Title => text_resource.title_localized(),
            ResurrectTextRole::Question => text_resource.question_localized(),
            ResurrectTextRole::Countdown => {
                text_resource.countdown_localized(model.countdown_seconds())
            }
        };
        if *localized != next_localized {
            *localized = next_localized;
        }
    }
    for (label, mut localized) in &mut button_labels {
        let next_localized = text_resource.choice_localized(label.choice);
        if *localized != next_localized {
            *localized = next_localized;
        }
    }

    let enabled = model.controls_enabled(*context);
    for (button, mut node, mut pickable) in &mut buttons {
        let visible = model.visible && context.choice_is_visible(button.choice);
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *pickable = if visible && enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }

    let phoenix_choice = context.phoenix_choice();
    let show_phoenix = model.visible && context.clean_controls_gate_open();
    for (visual, mut node) in &mut phoenix_visuals {
        node.display = if show_phoenix && phoenix_choice == Some(visual.choice) {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(super) fn update_resurrect_button_visuals(
    model: Res<ResurrectUiModel>,
    context: Res<ResurrectUiContext>,
    assets: Res<ResurrectUiAssets>,
    mut buttons: Query<(&Interaction, &ResurrectUiButton, &mut ImageNode, &Children)>,
    mut labels: Query<(&ResurrectUiButtonLabel, &mut TextColor)>,
) {
    let enabled = model.controls_enabled(*context);
    for (interaction, button, mut image, children) in &mut buttons {
        let hovered = enabled
            && context.choice_is_visible(button.choice)
            && matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
        image.image = if hovered {
            assets.button_hover.clone()
        } else {
            assets.button_normal.clone()
        };
        let color = array_color(if hovered {
            RESURRECT_BUTTON_HOVER_COLOR
        } else {
            RESURRECT_BUTTON_NORMAL_COLOR
        });
        for child in children.iter() {
            if let Ok((label, mut text_color)) = labels.get_mut(child) {
                if label.choice == button.choice {
                    text_color.0 = color;
                }
            }
        }
    }
}
