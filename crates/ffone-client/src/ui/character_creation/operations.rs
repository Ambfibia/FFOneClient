use super::*;

pub(super) fn compose_legacy_last_name(middle: &str, last: &str) -> String {
    let middle = if middle == " " { "" } else { middle };
    let last = if last == " " { "" } else { last };
    let mut suffix = last.to_owned();
    if let Some(first) = suffix.get_mut(0..1) {
        if middle.contains(' ') || middle.is_empty() {
            first.make_ascii_uppercase();
        } else {
            first.make_ascii_lowercase();
        }
    }
    format!("{middle}{suffix}")
}

pub(super) fn character_creation_assets_active(
    phase: Res<State<crate::ui_startup::NativeUiStartupPhase>>,
) -> bool {
    matches!(
        phase.get(),
        crate::ui_startup::NativeUiStartupPhase::CharacterSelection
            | crate::ui_startup::NativeUiStartupPhase::CharacterCreation
    )
}

pub(super) fn image_node(handle: Handle<Image>) -> ImageNode {
    ImageNode {
        image: handle,
        image_mode: NodeImageMode::Stretch,
        ..default()
    }
}

pub(super) fn sliced_image_node(handle: Handle<Image>, border: BorderRect) -> ImageNode {
    ImageNode {
        image: handle,
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border,
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}

pub(super) fn source_style_image_node(assets: &CharacterCreationAssets, path: &'static str) -> ImageNode {
    let handle = assets.image(path);
    let mut image = match source_style_border(path) {
        Some(border) => sliced_image_node(handle, border),
        None => image_node(handle),
    };
    image.visual_box = bevy::ui::VisualBox::BorderBox;
    image
}

pub(super) fn character_creation_text_font(
    assets: &CharacterCreationAssets,
    style: CharacterCreationTextStyle,
) -> (TextFont, LineHeight) {
    let spec = style.spec();
    let font = match spec.font_role {
        CharacterCreationFontRole::Jeffe => assets.display_font.clone(),
        CharacterCreationFontRole::Chalet => assets.font.clone(),
    };
    (
        TextFont {
            font: (font).into(),
            font_size: (spec.font_size).into(),
            ..default()
        },
        LineHeight::Px(spec.line_height),
    )
}

pub(super) fn character_creation_text_color(style: CharacterCreationTextStyle) -> TextColor {
    let [red, green, blue, alpha] = style.spec().normal_color;
    TextColor(Color::srgba(red, green, blue, alpha))
}

pub(super) fn character_creation_text_node(
    rect: LegacyCreationRect,
    style: CharacterCreationTextStyle,
) -> Node {
    let spec = style.spec();
    let mut node = LegacyCreationRect::new(
        rect.x + spec.content_offset[0],
        rect.y + spec.content_offset[1] + spec.y_offset,
        rect.width,
        rect.height,
    )
    .node();
    node.align_items = AlignItems::Center;
    node.justify_content = match spec.anchor {
        CharacterCreationTextAnchor::MiddleLeft => JustifyContent::Start,
        CharacterCreationTextAnchor::MiddleCenter => JustifyContent::Center,
    };
    if style != CharacterCreationTextStyle::Toggle {
        let [left, right, top, bottom] = spec.padding;
        node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
    }
    if spec.clip {
        node.overflow = Overflow::clip();
    }
    node
}

pub(super) fn bind_character_creation_visibility(
    model: Res<CharacterCreationUiModel>,
    mut visibility: ParamSet<(
        Query<&mut Visibility, With<AppearanceRoot>>,
        Query<&mut Visibility, With<NameRoot>>,
        Query<&mut Visibility, With<ModeGeneratedRoot>>,
        Query<&mut Visibility, With<ModeCustomRoot>>,
        Query<(&CreationNameModeButton, &mut Visibility)>,
        Query<&mut Visibility, With<CreationPreviewControlsRoot>>,
    )>,
) {
    if let Ok(mut node_visibility) = visibility.p0().single_mut() {
        *node_visibility = if model.screen == CharacterCreationScreen::Appearance {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut node_visibility) = visibility.p1().single_mut() {
        *node_visibility = if model.screen == CharacterCreationScreen::Name {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut node_visibility) = visibility.p2().single_mut() {
        *node_visibility = if model.name_mode == CharacterNameMode::Generated {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut node_visibility) = visibility.p3().single_mut() {
        *node_visibility = if model.name_mode == CharacterNameMode::Custom {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (button, mut button_visibility) in &mut visibility.p4() {
        // The selected tab is part of TabArea1/TabArea2 in the legacy skin;
        // only the inactive tab is an actual `GUI.Button`.
        *button_visibility = if model.name_mode == button.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    if let Ok(mut controls_visibility) = visibility.p5().single_mut() {
        *controls_visibility =
            if model.visible && model.screen == CharacterCreationScreen::Appearance {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
    }
}

pub(super) fn bind_character_creation_controls(
    model: Res<CharacterCreationUiModel>,
    assets: Res<CharacterCreationAssets>,
    mut buttons: Query<(&CharacterCreationButton, &Interaction, &mut ImageNode)>,
) {
    for (button, interaction, mut image) in &mut buttons {
        if let CharacterCreationControl::Gender(gender) = button.0 {
            image.image = match (model.appearance.gender == gender, *interaction) {
                (true, Interaction::None) => assets.image(CC_CHECKED),
                (true, Interaction::Hovered | Interaction::Pressed) => {
                    assets.image(CC_CHECKED_OVER)
                }
                (false, Interaction::Hovered) => assets.image(CC_CHECK_OVER),
                (false, Interaction::None | Interaction::Pressed) => assets.image(CC_CHECK_NORMAL),
            };
            continue;
        }
        if button.0 == CharacterCreationControl::ToggleFullscreen {
            image.image = match (model.fullscreen, *interaction) {
                (true, Interaction::Hovered | Interaction::Pressed) => {
                    assets.image(CC_WINDOWED_OVER)
                }
                (true, Interaction::None) => assets.image(CC_WINDOWED),
                (false, Interaction::Hovered | Interaction::Pressed) => {
                    assets.image(CC_FULLSCREEN_OVER)
                }
                (false, Interaction::None) => assets.image(CC_FULLSCREEN),
            };
            continue;
        }
        let Some((normal, hover, active)) = source_button_states(button.0) else {
            continue;
        };
        image.image = match *interaction {
            Interaction::None => assets.image(normal),
            Interaction::Hovered => assets.image(hover),
            Interaction::Pressed => assets.image(active),
        };
    }
}

pub(super) fn bind_character_creation_clothing_icons(
    model: Res<CharacterCreationUiModel>,
    asset_server: Res<AssetServer>,
    mut icons: Query<(&CreationClothingIcon, &mut ImageNode)>,
) {
    if !model.is_changed() {
        return;
    }
    for (icon, mut image) in &mut icons {
        image.image = model.starter_icon_paths[icon.row][icon.column]
            .as_ref()
            .map(|path| asset_server.load(path.clone()))
            .unwrap_or_default();
    }
}

pub(super) fn bind_character_creation_text(
    model: Res<CharacterCreationUiModel>,
    names: Res<CharacterNameLists>,
    mut texts: ParamSet<(
        Query<&mut LocalizedText, With<AppearanceHeightText>>,
        Query<&mut LocalizedText, With<AppearanceBodyText>>,
        Query<&mut LocalizedText, With<AppearanceHairText>>,
        Query<&mut LocalizedText, With<AppearanceFaceText>>,
        Query<&mut LocalizedText, With<CustomNameText>>,
        Query<&mut LocalizedText, With<GeneratedNameText>>,
        Query<(&NameColumnText, &mut LocalizedText)>,
    )>,
) {
    if let Ok(mut localized_component) = texts.p0().single_mut() {
        let keys = [
            "ui.character_create.height.shortest",
            "ui.character_create.height.short",
            "ui.character_create.height.medium",
            "ui.character_create.height.tall",
            "ui.character_create.height.tallest",
        ];
        let localized = LocalizedText::new(
            keys[usize::from(model.appearance.height).min(keys.len() - 1)],
            model.appearance.height_label(),
        );
        *localized_component = localized;
    }
    if let Ok(mut localized_component) = texts.p1().single_mut() {
        let keys = [
            "ui.character_create.body.heavy",
            "ui.character_create.body.medium",
            "ui.character_create.body.light",
        ];
        let localized = LocalizedText::new(
            keys[usize::from(model.appearance.body).min(keys.len() - 1)],
            model.appearance.body_label(),
        );
        *localized_component = localized;
    }
    if let Ok(mut localized_component) = texts.p2().single_mut() {
        let localized = indexed_appearance_label(
            &model.hair_label,
            "HAIR ",
            "ui.character_create.hair_variant",
            "HAIR {index}",
            model.appearance.gender,
            model.appearance.hair,
            "hair",
        );
        *localized_component = localized;
    }
    if let Ok(mut localized_component) = texts.p3().single_mut() {
        let localized = indexed_appearance_label(
            &model.face_label,
            "FACE ",
            "ui.character_create.face_variant",
            "FACE {index}",
            model.appearance.gender,
            model.appearance.face,
            "face",
        );
        *localized_component = localized;
    }
    if let Ok(mut localized_component) = texts.p4().single_mut() {
        let value = if model.custom_name.is_empty() && model.custom_name_focused {
            "|".to_owned()
        } else if model.custom_name_focused {
            format!("{}|", model.custom_name)
        } else {
            model.custom_name.clone()
        };
        let localized =
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &value);
        *localized_component = localized;
    }
    if let Ok(mut localized_component) = texts.p5().single_mut() {
        let value = model
            .generated_name(&names)
            .map(|name| format!("{} {}", name.first, name.last).to_uppercase())
            .unwrap_or_default();
        let localized =
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &value);
        *localized_component = localized;
    }
    for (marker, mut localized_component) in &mut texts.p6() {
        let Some(indices) = model.visible_name_indices(&names, marker.0) else {
            let localized =
                LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", "");
            *localized_component = localized;
            continue;
        };
        let index = indices[marker.1];
        let source = names
            .list(marker.0)
            .get(index)
            .map_or_else(String::new, |value| {
                if value == " " {
                    "<BLANK>".to_owned()
                } else {
                    // The legacy bitmap JEFFE font renders its lowercase name
                    // glyphs as small uppercase forms.
                    value.to_uppercase()
                }
            });
        let localized = if source == "<BLANK>" {
            LocalizedText::new("ui.character_create.name.blank_option", "<BLANK>")
        } else {
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &source)
        };
        *localized_component = localized;
    }
}

pub(super) fn indexed_appearance_label(
    source: &str,
    prefix: &str,
    key: &'static str,
    fallback: &'static str,
    gender: CharacterGender,
    selector: u8,
    category: &str,
) -> LocalizedText {
    if let Some(index) = source.strip_prefix(prefix).filter(|index| {
        !index.is_empty() && index.chars().all(|character| character.is_ascii_digit())
    }) {
        LocalizedText::new(key, fallback).with_arg("index", index)
    } else {
        let gender = match gender {
            CharacterGender::Boy => "male",
            CharacterGender::Girl => "female",
        };
        LocalizedText::new(
            format!("content.appearance.{gender}.{category}.{selector}.name"),
            source,
        )
    }
}

pub(super) fn bind_character_creation_colors(
    model: Res<CharacterCreationUiModel>,
    mut selected: Query<(&SelectedColor, &mut Visibility)>,
    mut swatches: Query<(&ColorSwatch, &mut ImageNode)>,
    mut pages: Query<(&CharacterCreationButton, &mut Node)>,
) {
    if !model.is_changed() {
        return;
    }
    for (marker, mut visibility) in &mut selected {
        let code = marker.1 + marker.0.offset(&model) + 1;
        let active = match marker.0 {
            CharacterCreationColorKind::Skin => model.appearance.skin_color == code,
            CharacterCreationColorKind::Hair => model.appearance.hair_color == code,
            CharacterCreationColorKind::Eye => model.appearance.eye_color == code,
        };
        let next = if active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != next {
            *visibility = next;
        }
    }
    for (swatch, mut image) in &mut swatches {
        if let Some(&color) = model.color_palettes[swatch.0.index()]
            .get(usize::from(swatch.1 + swatch.0.offset(&model)))
        {
            if image.color != color {
                image.color = color;
            }
        }
    }
    for (button, mut node) in &mut pages {
        if let CharacterCreationControl::ColorPage(kind, _) = button.0 {
            let next = if model.color_palettes[kind.index()].len() > usize::from(kind.page_size()) {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != next {
                node.display = next;
            }
        }
    }
}

pub(super) fn randomize_appearance(
    model: &mut CharacterCreationUiModel,
    random: &mut CharacterCreationRandom,
) {
    let counts = model.option_counts;
    let gender = model.appearance.gender;
    // The initial roll uses the first skin page. The upper bound of the
    // source integer Random.Range is exclusive, including later full rolls.
    let skin_limit = if model.randomize_on_appearance_open {
        model.color_palettes[0].len().min(12)
    } else {
        model.color_palettes[0].len()
    };
    model.appearance.skin_color = random.inclusive(1, skin_limit.saturating_sub(1).max(1) as u8);
    model.appearance.hair_color = random.inclusive(1, model.color_palettes[1].len() as u8 - 1);
    model.appearance.eye_color = random.inclusive(1, model.color_palettes[2].len() as u8);
    model.color_pages = [
        (model.appearance.skin_color - 1) / 12,
        (model.appearance.hair_color - 1) / 18,
        (model.appearance.eye_color - 1) / 5,
    ];
    model.appearance.body = random.inclusive(0, 2);
    model.appearance.height = random.inclusive(0, 4);
    for field in [
        AppearanceField::Hair,
        AppearanceField::Face,
        AppearanceField::Shirt,
        AppearanceField::Pants,
        AppearanceField::Shoes,
    ] {
        *model.appearance.selector_mut(field) =
            random.inclusive(2, counts.count(gender, field) + 1);
    }
}

pub(super) fn control_on_current_screen(
    control: CharacterCreationControl,
    model: &CharacterCreationUiModel,
) -> bool {
    match control {
        CharacterCreationControl::Exit | CharacterCreationControl::ToggleFullscreen => true,
        CharacterCreationControl::NameMode(_) | CharacterCreationControl::ContinueName => {
            model.screen == CharacterCreationScreen::Name
        }
        CharacterCreationControl::RandomName | CharacterCreationControl::NameScroll(_, _) => {
            model.screen == CharacterCreationScreen::Name
                && model.name_mode == CharacterNameMode::Generated
        }
        CharacterCreationControl::FocusCustomName => {
            model.screen == CharacterCreationScreen::Name
                && model.name_mode == CharacterNameMode::Custom
        }
        _ => model.screen == CharacterCreationScreen::Appearance,
    }
}

pub(super) fn repeat_character_creation_camera_controls(
    time: Res<Time>,
    model: Res<CharacterCreationUiModel>,
    mut outbox: ResMut<CharacterCreationUiOutbox>,
    buttons: Query<(&CharacterCreationButton, &Interaction)>,
) {
    if !model.visible || model.screen != CharacterCreationScreen::Appearance {
        return;
    }
    let delta = time.delta();
    if delta.is_zero() {
        return;
    }
    for (button, interaction) in &buttons {
        let CharacterCreationControl::Camera(action) = button.0 else {
            continue;
        };
        if *interaction == Interaction::Pressed {
            outbox
                .actions
                .push_back(CharacterCreationUiAction::Camera { action, delta });
        }
    }
}

pub(super) fn edit_custom_character_name(
    keys: Option<MessageReader<KeyboardInput>>,
    mut model: ResMut<CharacterCreationUiModel>,
) {
    if !model.visible
        || model.screen != CharacterCreationScreen::Name
        || model.name_mode != CharacterNameMode::Custom
        || !model.custom_name_focused
    {
        return;
    }
    let Some(mut keys) = keys else {
        return;
    };
    for key in keys.read() {
        if key.state != ButtonState::Pressed {
            continue;
        }
        if key.key_code == KeyCode::Backspace {
            model.custom_name.pop();
            model.validation_error = None;
            continue;
        }
        let Some(produced) = key.text.as_deref() else {
            continue;
        };
        for character in produced.chars() {
            if character == '\n' || character == '\r' || character.is_control() {
                continue;
            }
            if model.custom_name.chars().count() >= 26 {
                break;
            }
            model.custom_name.push(character);
            model.validation_error = None;
        }
    }
}

pub(super) fn control_character_creation_music(
    model: Res<CharacterCreationUiModel>,
    mix: Res<RetrobutionAudioMix>,
    phase: Option<Res<State<crate::ui_startup::NativeUiStartupPhase>>>,
    mut sources: Query<(&mut PlaybackSettings, Option<&mut AudioSink>), With<CreationMusic>>,
) {
    // Audio must keep running outside CharacterCreationStartupSet so leaving
    // the screen can silence its persistent source, even before a sink loads.
    let active = model.visible
        && phase.as_ref().is_none_or(|phase| {
            matches!(
                phase.get(),
                crate::ui_startup::NativeUiStartupPhase::CharacterCreation
                    | crate::ui_startup::NativeUiStartupPhase::Gameplay
            )
        });
    // The original PlayLoopSound routes this UI loop through SFXSound, just
    // like the buttons. Master gain is applied separately by GlobalVolume.
    let volume = Volume::Linear(0.7 * mix.effects);
    for (mut settings, sink) in &mut sources {
        let leaving = !active && !settings.paused;
        settings.paused = !active || !model.music_enabled;
        settings.volume = volume;
        if let Some(mut sink) = sink {
            sink.set_volume(volume);
            if settings.paused {
                sink.pause();
                if leaving {
                    let _ = sink.try_seek(Duration::ZERO);
                }
            } else {
                sink.play();
            }
        }
    }
}
