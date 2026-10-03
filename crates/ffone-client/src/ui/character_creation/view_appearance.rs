use super::*;

pub(super) fn spawn_image(
    parent: &mut ChildSpawnerCommands,
    rect: LegacyCreationRect,
    assets: &CharacterCreationAssets,
    path: &'static str,
) {
    parent.spawn((
        rect.node(),
        source_style_image_node(assets, path),
        FocusPolicy::Pass,
        Pickable::IGNORE,
    ));
}

pub(super) fn spawn_label(
    parent: &mut ChildSpawnerCommands,
    rect: LegacyCreationRect,
    localization_key: &'static str,
    text: &'static str,
    assets: &CharacterCreationAssets,
    style: CharacterCreationTextStyle,
) -> Entity {
    parent
        .spawn((
            character_creation_text_node(rect, style),
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ))
        .with_child((
            Text::new(text),
            LocalizedText::new(localization_key, text),
            character_creation_text_font(assets, style),
            character_creation_text_color(style),
            character_creation_text_layout(style),
            style,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ))
        .id()
}

pub(super) fn spawn_image_button(
    parent: &mut ChildSpawnerCommands,
    rect: LegacyCreationRect,
    assets: &CharacterCreationAssets,
    path: &'static str,
    control: CharacterCreationControl,
) -> Entity {
    let mut button = parent
        .spawn((
            Button,
            rect.node(),
            source_style_image_node(assets, path),
            CharacterCreationButton(control),
        ));
    if matches!(control, CharacterCreationControl::Step(..) | CharacterCreationControl::ColorPage(..) | CharacterCreationControl::NameScroll(..)) {
        button.insert(crate::ui::shared::controller::ControllerUiRepeat);
    }
    button.id()
}

pub(super) fn spawn_name_mode_button(
    parent: &mut ChildSpawnerCommands,
    rect: LegacyCreationRect,
    assets: &CharacterCreationAssets,
    mode: CharacterNameMode,
    localization_key: &'static str,
    label: &'static str,
) {
    parent
        .spawn((
            Button,
            character_creation_text_node(rect, CharacterCreationTextStyle::TabButton),
            source_style_image_node(assets, CC_NAME_BUTTON),
            CharacterCreationButton(CharacterCreationControl::NameMode(mode)),
            CreationNameModeButton(mode),
        ))
        .with_child((
            Text::new(label),
            LocalizedText::new(localization_key, label),
            character_creation_text_font(assets, CharacterCreationTextStyle::TabButton),
            character_creation_text_color(CharacterCreationTextStyle::TabButton),
            character_creation_text_layout(CharacterCreationTextStyle::TabButton),
            CharacterCreationTextStyle::TabButton,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
}

pub(super) fn spawn_text_button(
    parent: &mut ChildSpawnerCommands,
    rect: LegacyCreationRect,
    assets: &CharacterCreationAssets,
    localization_key: &'static str,
    text: &'static str,
    control: CharacterCreationControl,
) {
    let path = if control == CharacterCreationControl::Exit {
        CC_RED_BUTTON
    } else {
        CC_BLUE_BUTTON
    };
    let style = match control {
        CharacterCreationControl::Exit => CharacterCreationTextStyle::ExitButton,
        CharacterCreationControl::RandomAppearance | CharacterCreationControl::RandomName => {
            CharacterCreationTextStyle::ButtonTabFont
        }
        _ => CharacterCreationTextStyle::Button,
    };
    parent
        .spawn((
            Button,
            character_creation_text_node(rect, style),
            source_style_image_node(assets, path),
            CharacterCreationButton(control),
        ))
        .with_child((
            Text::new(text),
            LocalizedText::new(localization_key, text),
            character_creation_text_font(assets, style),
            character_creation_text_color(style),
            character_creation_text_layout(style),
            style,
            FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
}

pub(super) fn spawn_preview_controls(parent: &mut ChildSpawnerCommands, assets: &CharacterCreationAssets) {
    spawn_image_button(
        parent,
        LegacyCreationRect::new(87.0, 431.0, 60.0, 100.0),
        assets,
        CC_ROTATE_LEFT,
        CharacterCreationControl::Camera(CharacterCreationCameraAction::RotateLeft),
    );
    spawn_image_button(
        parent,
        LegacyCreationRect::new(319.0, 431.0, 60.0, 100.0),
        assets,
        CC_ROTATE_RIGHT,
        CharacterCreationControl::Camera(CharacterCreationCameraAction::RotateRight),
    );
    spawn_image_button(
        parent,
        LegacyCreationRect::new(195.0, 498.0, 38.0, 38.0),
        assets,
        CC_ZOOM_IN,
        CharacterCreationControl::Camera(CharacterCreationCameraAction::ZoomIn),
    );
    spawn_image_button(
        parent,
        LegacyCreationRect::new(232.0, 498.0, 38.0, 38.0),
        assets,
        CC_ZOOM_OUT,
        CharacterCreationControl::Camera(CharacterCreationCameraAction::ZoomOut),
    );
    spawn_text_button(
        parent,
        LegacyCreationRect::new(287.0, 49.0, 90.0, 20.0),
        assets,
        "ui.character_create.random",
        "RANDOM",
        CharacterCreationControl::RandomAppearance,
    );
}

pub(super) fn spawn_character_creation_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_assets: Option<Res<CharacterCreationAssets>>,
) {
    if existing_assets.is_some() {
        return;
    }
    let assets = CharacterCreationAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands.spawn((
        crate::audio_channel::GameplayAudioChannel::ui_sfx(),
        AudioPlayer::new(assets.music.clone()),
        PlaybackSettings::LOOP
            .paused()
            .with_volume(Volume::Linear(0.5)),
        CreationMusic,
    ));
    let base_camera = commands
        .spawn((
            Name::new("Character creation base UI camera"),
            Camera2d,
            Camera {
                order: GAMEPLAY_UI_CAMERA_ORDER,
                clear_color: ClearColorConfig::None,
                is_active: false,
                ..default()
            },
            CharacterCreationBaseCamera,
        ))
        .id();
    let foreground_camera = commands
        .spawn((
            Name::new("Character creation foreground UI camera"),
            Camera2d,
            Camera {
                order: CHARACTER_CREATION_FOREGROUND_CAMERA_ORDER,
                clear_color: ClearColorConfig::None,
                is_active: false,
                ..default()
            },
            CharacterCreationForegroundCamera,
        ))
        .id();

    commands
        .spawn((
            LegacyCreationRect::new(0.0, 0.0, 1.0, 1.0).node(),
            Visibility::Hidden,
            NativeCharacterCreationRoot,
            ZIndex(100),
            UiTargetCamera(base_camera),
        ))
        .with_children(|root| {
            root.spawn((
                LegacyCreationRect::new(
                    0.0,
                    0.0,
                    CHARACTER_CREATION_REFERENCE_WIDTH,
                    CHARACTER_CREATION_REFERENCE_HEIGHT,
                )
                .node(),
                UiTransform::default(),
                image_node(assets.image(CHARACTER_CREATION_BACKGROUND_PATH)),
                CreationBackground,
            ));
            root.spawn((
                LegacyCreationRect::new(
                    0.0,
                    0.0,
                    CHARACTER_CREATION_APPEARANCE_WIDTH,
                    CHARACTER_CREATION_APPEARANCE_HEIGHT,
                )
                .node(),
                UiTransform::default(),
                AppearanceRoot,
            ))
            .with_children(|appearance| spawn_appearance(appearance, &assets));
            root.spawn((
                LegacyCreationRect::new(
                    0.0,
                    0.0,
                    CHARACTER_CREATION_NAME_WIDTH,
                    CHARACTER_CREATION_NAME_HEIGHT,
                )
                .node(),
                UiTransform::default(),
                NameRoot,
            ))
            .with_children(|name| spawn_name_creation(name, &assets));
            root.spawn((
                Button,
                LegacyCreationRect::new(0.0, 0.0, 35.0, 31.0).node(),
                UiTransform::default(),
                image_node(assets.image(CC_FULLSCREEN)),
                CharacterCreationButton(CharacterCreationControl::ToggleFullscreen),
                CreationFullscreen,
            ));
        });

    commands
        .spawn((
            LegacyCreationRect::new(
                0.0,
                0.0,
                CHARACTER_CREATION_APPEARANCE_WIDTH,
                CHARACTER_CREATION_APPEARANCE_HEIGHT,
            )
            .node(),
            UiTransform::default(),
            Visibility::Hidden,
            UiTargetCamera(foreground_camera),
            FocusPolicy::Pass,
            Pickable::IGNORE,
            CreationPreviewControlsRoot,
        ))
        .with_children(|controls| spawn_preview_controls(controls, &assets));
}

pub(super) fn spawn_appearance(parent: &mut ChildSpawnerCommands, assets: &CharacterCreationAssets) {
    spawn_image(
        parent,
        LegacyCreationRect::new(
            0.0,
            0.0,
            CHARACTER_CREATION_APPEARANCE_WIDTH,
            CHARACTER_CREATION_APPEARANCE_HEIGHT,
        ),
        assets,
        CCBG,
    );
    spawn_label(
        parent,
        LegacyCreationRect::new(10.0, 10.0, 240.0, 20.0),
        "ui.character_create.title",
        "CHARACTER CREATION",
        assets,
        CharacterCreationTextStyle::Transparent,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(77.0, 44.0, 307.0, 498.0),
        assets,
        CC_CHARACTER_DISPLAY,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(450.0, 1.0, 550.0, 637.0),
        assets,
        CC_RIGHT_BG,
    );

    // `DoCreationWindow` sets this runtime style to new Color(0,255,255),
    // which Unity clamps to pure cyan.
    spawn_label(
        parent,
        LegacyCreationRect::new(503.0, 15.0, 140.0, 15.0),
        "ui.character_create.step.body",
        "1. CHOOSE BODY",
        assets,
        CharacterCreationTextStyle::SectionLabel,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(495.0, 33.0, 466.0, 113.0),
        assets,
        CC_IN_12_BG,
    );
    spawn_gender(parent, assets, CharacterGender::Boy, 520.0, 44.0, "BOY");
    spawn_gender(parent, assets, CharacterGender::Girl, 603.0, 44.0, "GIRL");
    spawn_body_step(parent, assets, AppearanceField::Height, 521.0, 75.0);
    spawn_body_step(parent, assets, AppearanceField::Body, 521.0, 104.0);
    parent
        .spawn((
            character_creation_text_node(
                LegacyCreationRect::new(542.0, 75.0, 114.0, 23.0),
                CharacterCreationTextStyle::BodyText,
            ),
            source_style_image_node(assets, CC_BODY_DISPLAY),
        ))
        .with_child((
            Text::new("MEDIUM"),
            LocalizedText::new("ui.character_create.height.medium", "MEDIUM"),
            UiTextAutoFit::new(
                98.0,
                23.0,
                &character_creation_text_font(assets, CharacterCreationTextStyle::BodyText),
            ),
            character_creation_text_font(assets, CharacterCreationTextStyle::BodyText),
            character_creation_text_color(CharacterCreationTextStyle::BodyText),
            character_creation_text_layout(CharacterCreationTextStyle::BodyText),
            CharacterCreationTextStyle::BodyText,
            AppearanceHeightText,
        ));
    parent
        .spawn((
            character_creation_text_node(
                LegacyCreationRect::new(542.0, 104.0, 114.0, 23.0),
                CharacterCreationTextStyle::BodyText,
            ),
            source_style_image_node(assets, CC_BODY_DISPLAY),
        ))
        .with_child((
            Text::new("MEDIUM"),
            LocalizedText::new("ui.character_create.body.medium", "MEDIUM"),
            UiTextAutoFit::new(
                98.0,
                23.0,
                &character_creation_text_font(assets, CharacterCreationTextStyle::BodyText),
            ),
            character_creation_text_font(assets, CharacterCreationTextStyle::BodyText),
            character_creation_text_color(CharacterCreationTextStyle::BodyText),
            character_creation_text_layout(CharacterCreationTextStyle::BodyText),
            CharacterCreationTextStyle::BodyText,
            AppearanceBodyText,
        ));
    spawn_label(
        parent,
        LegacyCreationRect::new(813.0, 44.0, 41.0, 12.0),
        "ui.character_create.skin",
        "SKIN",
        assets,
        CharacterCreationTextStyle::SectionLabel,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(745.0, 60.0, 177.0, 65.0),
        assets,
        CC_NAME_DISPLAY,
    );
    for index in 0..12 {
        let x = 757.0 + f32::from(index % 6) * 26.0;
        let y = 67.0 + f32::from(index / 6) * 30.0;
        spawn_color(
            parent,
            assets,
            CharacterCreationColorKind::Skin,
            index,
            x,
            y,
            SKIN_COLORS[index as usize],
        );
    }

    spawn_label(
        parent,
        LegacyCreationRect::new(503.0, 159.0, 180.0, 15.0),
        "ui.character_create.step.features",
        "2. CHOOSE FEATURES",
        assets,
        CharacterCreationTextStyle::SectionLabel,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(495.0, 180.0, 466.0, 148.0),
        assets,
        CC_IN_12_BG,
    );
    spawn_body_step(parent, assets, AppearanceField::Hair, 535.0, 194.0);
    parent
        .spawn((
            character_creation_text_node(
                LegacyCreationRect::new(556.0, 194.0, 125.0, 23.0),
                CharacterCreationTextStyle::BodyText,
            ),
            source_style_image_node(assets, CC_BODY_DISPLAY),
        ))
        .with_child((
            Text::new("HAIR 2"),
            LocalizedText::new("ui.character_create.hair_variant", "HAIR {index}")
                .with_arg("index", "2"),
            UiTextAutoFit::new(
                109.0,
                23.0,
                &character_creation_text_font(assets, CharacterCreationTextStyle::BodyText),
            ),
            character_creation_text_font(assets, CharacterCreationTextStyle::BodyText),
            character_creation_text_color(CharacterCreationTextStyle::BodyText),
            character_creation_text_layout(CharacterCreationTextStyle::BodyText),
            CharacterCreationTextStyle::BodyText,
            AppearanceHairText,
        ));
    spawn_label(
        parent,
        LegacyCreationRect::new(810.0, 197.0, 48.0, 17.0),
        "ui.character_create.face_label",
        "FACE",
        assets,
        CharacterCreationTextStyle::SectionLabel,
    );
    spawn_body_step(parent, assets, AppearanceField::Face, 751.0, 220.0);
    parent
        .spawn((
            character_creation_text_node(
                LegacyCreationRect::new(772.0, 220.0, 130.0, 23.0),
                CharacterCreationTextStyle::BodyText,
            ),
            source_style_image_node(assets, CC_BODY_DISPLAY),
        ))
        .with_child((
            Text::new("FACE 2"),
            LocalizedText::new("ui.character_create.face_variant", "FACE {index}")
                .with_arg("index", "2"),
            UiTextAutoFit::new(
                114.0,
                23.0,
                &character_creation_text_font(assets, CharacterCreationTextStyle::BodyText),
            ),
            character_creation_text_font(assets, CharacterCreationTextStyle::BodyText),
            character_creation_text_color(CharacterCreationTextStyle::BodyText),
            character_creation_text_layout(CharacterCreationTextStyle::BodyText),
            CharacterCreationTextStyle::BodyText,
            AppearanceFaceText,
        ));
    spawn_image(
        parent,
        LegacyCreationRect::new(531.0, 226.0, 177.0, 89.0),
        assets,
        CC_NAME_DISPLAY,
    );
    for index in 0..18 {
        let x = 541.0 + f32::from(index % 6) * 27.0;
        let y = 234.0 + f32::from(index / 6) * 26.0;
        spawn_color(
            parent,
            assets,
            CharacterCreationColorKind::Hair,
            index,
            x,
            y,
            HAIR_COLORS[index as usize],
        );
    }
    spawn_label(
        parent,
        LegacyCreationRect::new(785.0, 259.0, 98.0, 18.0),
        "ui.character_create.eye_color",
        "EYE COLOR",
        assets,
        CharacterCreationTextStyle::SectionLabel,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(756.0, 279.0, 164.0, 31.0),
        assets,
        CC_NAME_DISPLAY,
    );
    for (index, color) in EYE_COLORS.iter().copied().enumerate() {
        spawn_color(
            parent,
            assets,
            CharacterCreationColorKind::Eye,
            index as u8,
            768.0 + index as f32 * 30.0,
            284.0,
            color,
        );
    }

    for (kind, x, y, width, height, delta) in [
        (
            CharacterCreationColorKind::Skin,
            726.0,
            61.0,
            23.0,
            64.0,
            -1,
        ),
        (CharacterCreationColorKind::Skin, 917.0, 61.0, 23.0, 64.0, 1),
        (
            CharacterCreationColorKind::Hair,
            512.0,
            227.0,
            23.0,
            88.0,
            -1,
        ),
        (
            CharacterCreationColorKind::Hair,
            704.0,
            227.0,
            23.0,
            88.0,
            1,
        ),
        (
            CharacterCreationColorKind::Eye,
            915.0,
            280.0,
            22.0,
            30.0,
            -1,
        ),
        (CharacterCreationColorKind::Eye, 739.0, 280.0, 22.0, 30.0, 1),
    ] {
        let control = CharacterCreationControl::ColorPage(kind, delta);
        let (normal, _, _) = source_button_states(control).expect("color pager artwork");
        spawn_image_button(
            parent,
            LegacyCreationRect::new(x, y, width, height),
            assets,
            normal,
            control,
        );
    }
    spawn_label(
        parent,
        LegacyCreationRect::new(503.0, 342.0, 170.0, 15.0),
        "ui.character_create.step.clothes",
        "3. CHOOSE CLOTHES",
        assets,
        CharacterCreationTextStyle::SectionLabel,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(495.0, 362.0, 466.0, 267.0),
        assets,
        CC_IN_3_BG,
    );
    spawn_image(
        parent,
        LegacyCreationRect::new(535.0, 377.0, 387.0, 239.0),
        assets,
        CC_CLOTHES_BG,
    );
    for (field, row) in [
        (AppearanceField::Shirt, 0),
        (AppearanceField::Pants, 1),
        (AppearanceField::Shoes, 2),
    ] {
        let y = 378.0 + row as f32 * 79.0;
        let left_path = match field {
            AppearanceField::Shirt => {
                "ui/en/character/creation/clothes/CCClothesLeftTopButtonNormal.png"
            }
            AppearanceField::Pants => {
                "ui/en/character/creation/clothes/CCClothesLeftButtonNormal.png"
            }
            AppearanceField::Shoes => {
                "ui/en/character/creation/clothes/CCClothesLeftBottomButtonNormal.png"
            }
            _ => unreachable!(),
        };
        let right_path = match field {
            AppearanceField::Shirt => {
                "ui/en/character/creation/clothes/CCClothesRightTopButtonNormal.png"
            }
            AppearanceField::Pants => {
                "ui/en/character/creation/clothes/CCClothesRightButtonNormal.png"
            }
            AppearanceField::Shoes => {
                "ui/en/character/creation/clothes/CCClothesRightBottomButtonNormal.png"
            }
            _ => unreachable!(),
        };
        spawn_image_button(
            parent,
            LegacyCreationRect::new(513.0, y, 23.0, 79.0),
            assets,
            left_path,
            CharacterCreationControl::Step(field, -1),
        );
        spawn_image_button(
            parent,
            LegacyCreationRect::new(920.0, y, 23.0, 79.0),
            assets,
            right_path,
            CharacterCreationControl::Step(field, 1),
        );
    }
    for (row, (field, y)) in [
        (AppearanceField::Shirt, 387.0),
        (AppearanceField::Pants, 467.0),
        (AppearanceField::Shoes, 545.0),
    ]
    .into_iter()
    .enumerate()
    {
        for column in 0..5 {
            // `CnGuiCharCreation`: the five-item carousel fades outwards.
            let alpha = [0.5, 0.75, 1.0, 0.75, 0.5][column];
            let mut icon = parent.spawn((
                LegacyCreationRect::new(546.0 + column as f32 * 75.0, y, 62.0, 62.0).node(),
                ImageNode {
                    color: Color::srgba(1.0, 1.0, 1.0, alpha),
                    ..default()
                },
                CreationClothingIcon { row, column },
            ));
            if column == 2 {
                icon.insert((FocusPolicy::Pass, Pickable::IGNORE));
            } else {
                icon.insert((
                    Button,
                    CharacterCreationButton(CharacterCreationControl::ClothingChoice(
                        field,
                        column as i8 - 2,
                    )),
                ));
            }
        }
    }
    spawn_text_button(
        parent,
        LegacyCreationRect::new(84.0, 550.0, 295.0, 40.0),
        assets,
        "ui.common.continue",
        "CONTINUE",
        CharacterCreationControl::ContinueAppearance,
    );
    spawn_text_button(
        parent,
        LegacyCreationRect::new(22.0, 600.0, 68.0, 28.0),
        assets,
        "ui.common.exit",
        "EXIT",
        CharacterCreationControl::Exit,
    );
}

pub(super) fn spawn_gender(
    parent: &mut ChildSpawnerCommands,
    assets: &CharacterCreationAssets,
    gender: CharacterGender,
    x: f32,
    y: f32,
    label: &'static str,
) {
    parent
        .spawn((
            Button,
            LegacyCreationRect::new(x, y, 72.0, 22.0).node(),
            sliced_image_node(
                assets.image(CC_CHECK_NORMAL),
                BorderRect {
                    min_inset: Vec2::new(30.0, 5.0),
                    max_inset: Vec2::new(0.0, 5.0),
                },
            ),
            CharacterCreationButton(CharacterCreationControl::Gender(gender)),
        ))
        .with_children(|toggle| {
            let mut content = LegacyCreationRect::new(0.0, 0.0, 102.0, 22.0).node();
            content.align_items = AlignItems::Center;
            content.justify_content = JustifyContent::Center;
            content.overflow = Overflow::visible();
            let font = character_creation_text_font(assets, CharacterCreationTextStyle::Toggle);
            toggle
                .spawn((content, FocusPolicy::Pass, Pickable::IGNORE))
                .with_child((
                    Text::new(label),
                    LocalizedText::new(
                        match gender {
                            CharacterGender::Boy => "ui.character_create.boy",
                            CharacterGender::Girl => "ui.character_create.girl",
                        },
                        label,
                    ),
                    font.clone(),
                    character_creation_text_color(CharacterCreationTextStyle::Toggle),
                    character_creation_text_layout(CharacterCreationTextStyle::Toggle),
                    // Clean uses a 72px Toggle hit rect with a 24px check and
                    // 48px label region. Localized copy may shrink inside that
                    // label region, but the source Rect and max metrics stay
                    // unchanged.
                    UiTextAutoFit::new(48.0, 22.0, &font),
                    CharacterCreationTextStyle::Toggle,
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                ));
        });
}

pub(super) fn spawn_body_step(
    parent: &mut ChildSpawnerCommands,
    assets: &CharacterCreationAssets,
    field: AppearanceField,
    x: f32,
    y: f32,
) {
    spawn_image_button(
        parent,
        LegacyCreationRect::new(x, y, 22.0, 22.0),
        assets,
        CC_BODY_RIGHT,
        CharacterCreationControl::Step(field, -1),
    );
    let right_x = match field {
        AppearanceField::Height | AppearanceField::Body => x + 134.0,
        AppearanceField::Hair => x + 145.0,
        AppearanceField::Face => x + 149.0,
        _ => x + 134.0,
    };
    spawn_image_button(
        parent,
        LegacyCreationRect::new(right_x, y, 22.0, 22.0),
        assets,
        CC_BODY_LEFT,
        CharacterCreationControl::Step(field, 1),
    );
}
