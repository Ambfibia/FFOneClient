//! Barber uses the character-creation skin and camera, with its own server-owned draft.
use super::*;
use crate::barber::{BarberField as Field, BarberModel, BarberPhase};
use crate::localization::{Language, Localization};

pub struct BarberUiPlugin;
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Backdrop;
#[derive(Component)]
struct BodyGroup;
#[derive(Component)]
struct FeaturesGroup;
#[derive(Component)]
struct Review;
#[derive(Component)]
struct Preview;
#[derive(Component)]
struct Value(Field);
#[derive(Component)]
struct Cost(usize);
#[derive(Component)]
struct Total;
#[derive(Component)]
struct ErrorText;
#[derive(Component, Clone, Copy)]
enum Control {
    Gender(i8),
    Step(Field, i32),
    Color(Field, usize),
    Page(usize, i32),
    Reset,
    Confirm,
    Close,
    Rotate(f32),
    Zoom(f32),
}
#[derive(Component)]
struct Visual(&'static str, &'static str);
#[derive(Component)]
struct Swatch(Field, usize);
#[derive(Component)]
struct SelectedSwatch(Field, usize);

impl Plugin for BarberUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<BarberModel>()
            .init_resource::<crate::barber::BarberInbox>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn)
            .add_systems(Update, (input, bind).chain().before(LocalizationSet::Apply));
    }
}

fn label(
    parent: &mut ChildSpawnerCommands,
    rect: LegacyCreationRect,
    value: LocalizedText,
    assets: &CharacterCreationAssets,
    marker: impl Bundle,
) {
    let font = character_creation_text_font(assets, CharacterCreationTextStyle::Transparent);
    parent
        .spawn((rect.node(), FocusPolicy::Pass, Pickable::IGNORE))
        .with_child((
            Text::new(value.fallback.clone()),
            value,
            font,
            TextColor(Color::WHITE),
            TextLayout::new(Justify::Left, LineBreak::WordBoundary),
            marker,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
}
fn button(
    parent: &mut ChildSpawnerCommands,
    rect: LegacyCreationRect,
    control: Control,
    normal: &'static str,
    over: &'static str,
    assets: &CharacterCreationAssets,
    caption: Option<LocalizedText>,
) {
    parent
        .spawn((
            Button,
            rect.node(),
            source_style_image_node(assets, normal),
            control,
            Visual(normal, over),
        ))
        .with_children(|b| {
            if let Some(caption) = caption {
                label(
                    b,
                    LegacyCreationRect::new(8.0, 8.0, rect.width - 16.0, rect.height - 10.0),
                    caption,
                    assets,
                    (),
                );
            }
        });
}
fn step(
    parent: &mut ChildSpawnerCommands,
    assets: &CharacterCreationAssets,
    field: Field,
    x: f32,
    y: f32,
    width: f32,
) {
    button(
        parent,
        LegacyCreationRect::new(x, y, 22., 22.),
        Control::Step(field, -1),
        CC_BODY_RIGHT,
        CC_BODY_RIGHT_OVER,
        assets,
        None,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(x + 21., y, width, 23.),
        assets,
        CC_BODY_DISPLAY,
    );
    label(
        parent,
        LegacyCreationRect::new(x + 25., y + 3., width - 8., 23.),
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
        assets,
        Value(field),
    );
    button(
        parent,
        LegacyCreationRect::new(x + width + 20., y, 22., 22.),
        Control::Step(field, 1),
        CC_BODY_LEFT,
        CC_BODY_LEFT_OVER,
        assets,
        None,
    );
}
fn palette(
    parent: &mut ChildSpawnerCommands,
    assets: &CharacterCreationAssets,
    field: Field,
    which: usize,
    count: usize,
    columns: usize,
    x: f32,
    y: f32,
    dx: f32,
    dy: f32,
) {
    for i in 0..count {
        let rect = LegacyCreationRect::new(
            x + (i % columns) as f32 * dx,
            y + (i / columns) as f32 * dy,
            22.,
            21.,
        );
        parent.spawn((
            rect.node(),
            source_style_image_node(assets, CC_COLOR_OUTLINE),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        parent.spawn((
            rect.node(),
            source_style_image_node(assets, CC_COLOR_SELECTED),
            SelectedSwatch(field, i),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
        parent.spawn((
            Button,
            rect.node(),
            source_style_image_node(assets, CC_COLOR_INSIDE),
            Control::Color(field, i),
            Swatch(field, i),
        ));
    }
    for (delta, px) in [(-1, x - 24.), (1, x + columns as f32 * dx)] {
        button(
            parent,
            LegacyCreationRect::new(px, y, 20., 22.),
            Control::Page(which, delta),
            if delta < 0 {
                CC_BODY_RIGHT
            } else {
                CC_BODY_LEFT
            },
            if delta < 0 {
                CC_BODY_RIGHT_OVER
            } else {
                CC_BODY_LEFT_OVER
            },
            assets,
            None,
        );
    }
}
fn spawn(mut commands: Commands, server: Res<AssetServer>) {
    let assets = CharacterCreationAssets::load(&server);
    commands.spawn((
        Backdrop,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            display: Display::None,
            ..default()
        },
        ImageNode::new(server.load("ui/en/shared/panelback.png")),
        GlobalZIndex(180),
        Interaction::None,
    ));
    commands
        .spawn((
            Root,
            Node {
                position_type: PositionType::Absolute,
                width: px(1036.),
                height: px(654.),
                display: Display::None,
                ..default()
            },
            UiTransform::default(),
            GlobalZIndex(181),
            Interaction::None,
        ))
        .with_children(|root| {
            label(
                root,
                LegacyCreationRect::new(75., 10., 340., 25.),
                LocalizedText::new("ui.barber.title", "Dr. Barber's Surgery and Haircut"),
                &assets,
                (),
            );
            spawn_image(
                root,
                LegacyCreationRect::new(77., 44., 307., 498.),
                &assets,
                CC_CHARACTER_DISPLAY,
            );
            root.spawn((
                Preview,
                LegacyCreationRect::new(77., 44., 307., 498.).node(),
                ImageNode::default(),
                FocusPolicy::Pass,
                Pickable::IGNORE,
            ));
            for (control, x, y, w, h, normal, over) in [
                (
                    Control::Rotate(-100.),
                    87.,
                    431.,
                    60.,
                    100.,
                    CC_ROTATE_LEFT,
                    CC_ROTATE_LEFT_OVER,
                ),
                (
                    Control::Rotate(100.),
                    319.,
                    431.,
                    60.,
                    100.,
                    CC_ROTATE_RIGHT,
                    CC_ROTATE_RIGHT_OVER,
                ),
                (
                    Control::Zoom(-0.4),
                    195.,
                    498.,
                    38.,
                    38.,
                    CC_ZOOM_IN,
                    CC_ZOOM_IN_OVER,
                ),
                (
                    Control::Zoom(0.4),
                    232.,
                    498.,
                    38.,
                    38.,
                    CC_ZOOM_OUT,
                    CC_ZOOM_OUT_OVER,
                ),
            ] {
                button(
                    root,
                    LegacyCreationRect::new(x, y, w, h),
                    control,
                    normal,
                    over,
                    &assets,
                    None,
                );
            }
            button(
                root,
                LegacyCreationRect::new(84., 550., 295., 40.),
                Control::Reset,
                CC_BLUE_BUTTON,
                CC_BLUE_BUTTON_OVER,
                &assets,
                Some(LocalizedText::new("ui.barber.reset", "RESET TO DEFAULT")),
            );
            // The menu skin's close art is already a shared native asset.
            root.spawn((
                Button,
                LegacyCreationRect::new(994., 10., 32., 32.).node(),
                ImageNode::new(server.load("ui/en/user-equip/close.png")),
                Control::Close,
            ));
            root.spawn((
                LegacyCreationRect::new(440., 0., 550., 652.).node(),
                FocusPolicy::Pass,
                Pickable::IGNORE,
            ))
            .with_children(|right| {
                spawn_image(
                    right,
                    LegacyCreationRect::new(0., 1., 550., 651.),
                    &assets,
                    CC_RIGHT_BG,
                );
                right
                    .spawn((
                        BodyGroup,
                        LegacyCreationRect::new(0., 0., 550., 144.).node(),
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                    ))
                    .with_children(|body| {
                        label(
                            body,
                            LegacyCreationRect::new(53., 15., 240., 18.),
                            LocalizedText::new("ui.barber.choose_body", "CHOOSE BODY"),
                            &assets,
                            (),
                        );
                        spawn_image(
                            body,
                            LegacyCreationRect::new(45., 33., 466., 113.),
                            &assets,
                            CC_IN_12_BG,
                        );
                        for (gender, x, key, caption) in [
                            (1, 70., "ui.character_create.boy", "BOY"),
                            (2, 153., "ui.character_create.girl", "GIRL"),
                        ] {
                            button(
                                body,
                                LegacyCreationRect::new(x, 44., 22., 22.),
                                Control::Gender(gender),
                                CC_CHECK_NORMAL,
                                CC_CHECK_OVER,
                                &assets,
                                None,
                            );
                            label(
                                body,
                                LegacyCreationRect::new(x + 25., 44., 55., 22.),
                                LocalizedText::new(key, caption),
                                &assets,
                                (),
                            );
                        }
                        step(body, &assets, Field::Height, 71., 75., 114.);
                        step(body, &assets, Field::Body, 71., 104., 114.);
                        label(
                            body,
                            LegacyCreationRect::new(335., 44., 150., 18.),
                            Field::Skin.label(),
                            &assets,
                            (),
                        );
                        palette(body, &assets, Field::Skin, 0, 12, 6, 308., 68., 26., 30.);
                    });
                right
                    .spawn((
                        FeaturesGroup,
                        LegacyCreationRect::new(0., 144., 550., 190.).node(),
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                    ))
                    .with_children(|features| {
                        label(
                            features,
                            LegacyCreationRect::new(53., 15., 240., 18.),
                            LocalizedText::new("ui.barber.choose_features", "CHOOSE FEATURES"),
                            &assets,
                            (),
                        );
                        spawn_image(
                            features,
                            LegacyCreationRect::new(45., 36., 466., 148.),
                            &assets,
                            CC_IN_12_BG,
                        );
                        step(features, &assets, Field::Hair, 85., 50., 125.);
                        label(
                            features,
                            LegacyCreationRect::new(335., 53., 150., 18.),
                            Field::Face.label(),
                            &assets,
                            (),
                        );
                        step(features, &assets, Field::Face, 301., 76., 130.);
                        palette(
                            features,
                            &assets,
                            Field::HairColor,
                            1,
                            18,
                            6,
                            92.,
                            91.,
                            27.,
                            26.,
                        );
                        label(
                            features,
                            LegacyCreationRect::new(330., 115., 160., 18.),
                            Field::Eye.label(),
                            &assets,
                            (),
                        );
                        palette(features, &assets, Field::Eye, 2, 5, 5, 319., 141., 30., 30.);
                    });
                right
                    .spawn((
                        Review,
                        LegacyCreationRect::new(53., 340., 440., 240.).node(),
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                    ))
                    .with_children(|review| {
                        label(
                            review,
                            LegacyCreationRect::new(0., 0., 300., 20.),
                            LocalizedText::new("ui.barber.review", "REVIEW CHANGES"),
                            &assets,
                            (),
                        );
                        for i in 0..7 {
                            label(
                                review,
                                LegacyCreationRect::new(0., 24. + i as f32 * 28., 430., 26.),
                                LocalizedText::new("ui.content.passthrough", "{text}")
                                    .with_arg("text", ""),
                                &assets,
                                Cost(i),
                            );
                        }
                        label(
                            review,
                            LegacyCreationRect::new(0., 220., 430., 26.),
                            LocalizedText::new("ui.barber.total", "TOTAL: {cost} Taros")
                                .with_arg("cost", "0"),
                            &assets,
                            Total,
                        );
                    });
                button(
                    right,
                    LegacyCreationRect::new(130., 595., 295., 40.),
                    Control::Confirm,
                    CC_BLUE_BUTTON,
                    CC_BLUE_BUTTON_OVER,
                    &assets,
                    Some(LocalizedText::new("ui.barber.continue", "CONTINUE")),
                );
            });
            label(
                root,
                LegacyCreationRect::new(77., 595., 330., 55.),
                LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                &assets,
                ErrorText,
            );
        });
}
fn input(
    time: Res<Time>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    controls: Query<(&Control, Ref<Interaction>)>,
    mut model: ResMut<BarberModel>,
    popup: Option<Res<crate::system_message_ui::SystemMessageUiModel>>,
) {
    if popup.as_ref().is_some_and(|p| p.is_popup()) {
        return;
    }
    if !model.active() {
        return;
    }
    if keys
        .as_ref()
        .is_some_and(|keys| keys.just_pressed(KeyCode::Escape))
        && model.phase != BarberPhase::Confirming
    {
        model.close_requested = true;
    }
    for (control, interaction) in &controls {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *control {
            Control::Rotate(delta) => {
                model.yaw += delta * time.delta_secs();
                continue;
            }
            Control::Zoom(delta) => {
                model.distance = (model.distance + delta * time.delta_secs()).clamp(1., 3.);
                continue;
            }
            _ => {}
        }
        if !interaction.is_changed() {
            continue;
        }
        match *control {
            Control::Close if model.phase != BarberPhase::Confirming => {
                model.close_requested = true
            }
            Control::Confirm if model.can_confirm() => model.confirm_requested = true,
            Control::Gender(gender) => {
                if model.draft.as_ref().is_some_and(|s| s.gender != gender) {
                    model.step(Field::Gender, 1);
                }
            }
            Control::Reset => model.reset(),
            Control::Step(field, delta) => model.step(field, delta),
            Control::Color(field, index) => {
                let which = match field {
                    Field::Skin => 0,
                    Field::HairColor => 1,
                    _ => 2,
                };
                let size = [12, 18, 5][which];
                let index = model.pages[which] * size + index;
                model.set_color(field, index);
            }
            Control::Page(which, delta) if model.phase == BarberPhase::Editing => {
                let pages = model.palettes[which]
                    .len()
                    .div_ceil([12, 18, 5][which])
                    .max(1);
                model.pages[which] =
                    (model.pages[which] as i32 + delta).rem_euclid(pages as i32) as usize;
            }
            _ => {}
        }
    }
}
#[allow(clippy::type_complexity)]
fn bind(
    model: Res<BarberModel>,
    windows: Query<Ref<Window>, With<PrimaryWindow>>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    server: Res<AssetServer>,
    preview: Option<Res<crate::player_preview::NativePlayerBarberPreviewImage>>,
    changed_controls: Query<(), (With<Control>, Changed<Interaction>)>,
    mut nodes: Query<
        (
            &mut Node,
            Option<&mut UiTransform>,
            Has<Root>,
            Has<Backdrop>,
            Has<BodyGroup>,
            Has<FeaturesGroup>,
            Has<Review>,
        ),
        Or<(
            With<Root>,
            With<Backdrop>,
            With<BodyGroup>,
            With<FeaturesGroup>,
            With<Review>,
        )>,
    >,
    mut images: Query<
        (
            &mut ImageNode,
            Option<&Control>,
            Option<&Interaction>,
            Option<&Visual>,
            Option<&Swatch>,
            Option<&SelectedSwatch>,
            Has<Preview>,
        ),
        Or<(
            With<Control>,
            With<Swatch>,
            With<SelectedSwatch>,
            With<Preview>,
        )>,
    >,
    mut texts: Query<
        (
            &mut LocalizedText,
            Option<&Value>,
            Option<&Cost>,
            Has<Total>,
            Has<ErrorText>,
        ),
        Or<(With<Value>, With<Cost>, With<Total>, With<ErrorText>)>,
    >,
) {
    if !model.is_changed()
        && !windows.iter().any(|w| w.is_changed())
        && changed_controls.is_empty()
        && !localization.as_ref().is_some_and(|r| r.is_changed())
        && !language.as_ref().is_some_and(|r| r.is_changed())
        && !preview.as_ref().is_some_and(|r| r.is_changed())
    {
        return;
    }
    let show = model.active();
    let offset = if model.body_allowed() { 144. } else { 0. };
    for (mut node, transform, root, back, body, features, review) in &mut nodes {
        if root || back {
            node.display = if show { Display::Flex } else { Display::None };
        }
        if root && let Ok(window) = windows.single() {
            let scale = (window.width() / 1036.).min(window.height() / 654.).min(1.);
            node.left = px((window.width() - 1036.) * 0.5);
            node.top = px((window.height() - 654.) * 0.5);
            if let Some(mut t) = transform {
                t.scale = Vec2::splat(scale);
            }
        }
        if body {
            node.display = if model.body_allowed() {
                Display::Flex
            } else {
                Display::None
            };
        }
        if features {
            node.top = px(offset);
        }
        if review {
            node.top = px(offset + 196.);
        }
    }
    if !show {
        return;
    }
    for (mut image, control, interaction, visual, swatch, selected_swatch, is_preview) in
        &mut images
    {
        if is_preview {
            if let Some(preview) = &preview {
                image.image = preview.0.clone();
            }
            continue;
        }
        if let Some(selected) = selected_swatch {
            let which = match selected.0 {
                Field::Skin => 0,
                Field::HairColor => 1,
                _ => 2,
            };
            let code = model.pages[which] * [12, 18, 5][which] + selected.1 + 1;
            image.color = if model
                .draft
                .as_ref()
                .is_some_and(|s| selected.0.value(s) as usize == code)
            {
                Color::WHITE
            } else {
                Color::NONE
            };
        } else if let Some(swatch) = swatch {
            let which = match swatch.0 {
                Field::Skin => 0,
                Field::HairColor => 1,
                _ => 2,
            };
            let index = model.pages[which] * [12, 18, 5][which] + swatch.1;
            image.color = model.palettes[which]
                .get(index)
                .copied()
                .unwrap_or(Color::NONE);
        } else if let Some(visual) = visual {
            image.image = server.load(if interaction.is_some_and(|i| *i != Interaction::None) {
                visual.1
            } else {
                visual.0
            });
        }
        if let Some(Control::Gender(gender)) = control {
            let checked = model
                .draft
                .as_ref()
                .is_some_and(|style| style.gender == *gender);
            image.image = server.load(
                match (
                    checked,
                    interaction.is_some_and(|i| *i != Interaction::None),
                ) {
                    (true, true) => CC_CHECKED_OVER,
                    (true, false) => CC_CHECKED,
                    (false, true) => CC_CHECK_OVER,
                    (false, false) => CC_CHECK_NORMAL,
                },
            );
        }
        if let Some(Control::Confirm) = control {
            image.color = if model.can_confirm() {
                Color::WHITE
            } else {
                Color::srgba(0.5, 0.5, 0.5, 1.)
            };
        }
    }
    let resolve = |value: &LocalizedText| match (&localization, &language) {
        (Some(l), Some(lang)) => l.text(lang, value),
        _ => value.fallback.clone(),
    };
    let charges = model.charges();
    for (mut text, value, cost, total, error) in &mut texts {
        let next = if let Some(value) = value {
            model.draft.as_ref().map(|draft| {
                let gender = usize::from(draft.gender == 2);
                let choice_label = |field: Field, code: i8, label: String| {
                    model
                        .appearance_keys
                        .get(&(draft.gender, field as u8, code))
                        .map_or(label.clone(), |key| {
                            resolve(&LocalizedText::new(key.clone(), label))
                        })
                };
                let value = match value.0 {
                    Field::Hair => model.hair[gender]
                        .iter()
                        .find(|v| v.0 == draft.hair_style)
                        .map(|v| choice_label(Field::Hair, v.0, v.1.clone()))
                        .unwrap_or_else(|| draft.hair_style.to_string()),
                    Field::Face => model.face[gender]
                        .iter()
                        .find(|v| v.0 == draft.face_style)
                        .map(|v| choice_label(Field::Face, v.0, v.1.clone()))
                        .unwrap_or_else(|| draft.face_style.to_string()),
                    Field::Gender => resolve(&LocalizedText::new(
                        if draft.gender == 1 {
                            "ui.character_create.boy"
                        } else {
                            "ui.character_create.girl"
                        },
                        if draft.gender == 1 { "BOY" } else { "GIRL" },
                    )),
                    Field::Height => {
                        let (key, label) = [
                            ("shortest", "SHORTEST"),
                            ("short", "SHORT"),
                            ("medium", "MEDIUM"),
                            ("tall", "TALL"),
                            ("tallest", "TALLEST"),
                        ][draft.height.clamp(0, 4) as usize];
                        resolve(&LocalizedText::new(
                            format!("ui.character_create.height.{key}"),
                            label,
                        ))
                    }
                    Field::Body => {
                        let (key, label) =
                            [("heavy", "HEAVY"), ("medium", "MEDIUM"), ("light", "LIGHT")]
                                [draft.body.clamp(0, 2) as usize];
                        resolve(&LocalizedText::new(
                            format!("ui.character_create.body.{key}"),
                            label,
                        ))
                    }
                    field => format!("{} {}", resolve(&field.label()), field.value(draft) + 1),
                };
                LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
            })
        } else if let Some(cost) = cost {
            Some(charges.get(cost.0).map_or_else(
                || LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                |(field, cost)| {
                    LocalizedText::new("ui.barber.charge", "{field}: {cost}")
                        .with_arg("field", resolve(&field.label()))
                        .with_arg("cost", cost.to_string())
                },
            ))
        } else if total {
            Some(
                LocalizedText::new("ui.barber.total", "TOTAL: {cost} Taros")
                    .with_arg("cost", model.cost().unwrap_or(0).to_string()),
            )
        } else if error {
            Some(model.error.clone().unwrap_or_else(|| {
                if model.phase == BarberPhase::Opening {
                    LocalizedText::new("ui.barber.waiting", "Waiting for the server…")
                } else {
                    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "")
                }
            }))
        } else {
            None
        };
        if let Some(next) = next {
            text.set_if_neq(next);
        }
    }
}

#[cfg(test)]
mod tests;
