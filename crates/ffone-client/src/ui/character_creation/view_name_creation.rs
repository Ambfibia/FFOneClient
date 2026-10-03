use super::*;

pub(super) fn spawn_color(
    parent: &mut ChildSpawnerCommands,
    assets: &CharacterCreationAssets,
    kind: CharacterCreationColorKind,
    index: u8,
    x: f32,
    y: f32,
    color: Color,
) {
    spawn_image(
        parent,
        LegacyCreationRect::new(x, y, 22.0, 21.0),
        assets,
        CC_COLOR_OUTLINE,
    );
    let control = match kind {
        CharacterCreationColorKind::Skin => CharacterCreationControl::Skin(index),
        CharacterCreationColorKind::Hair => CharacterCreationControl::HairColor(index),
        CharacterCreationColorKind::Eye => CharacterCreationControl::EyeColor(index),
    };
    parent.spawn((
        LegacyCreationRect::new(x, y, 22.0, 21.0).node(),
        image_node(assets.image(CC_COLOR_SELECTED)),
        Visibility::Hidden,
        FocusPolicy::Pass,
        Pickable::IGNORE,
        SelectedColor(kind, index),
    ));
    let mut inside = source_style_image_node(assets, CC_COLOR_INSIDE);
    inside.color = color;
    parent.spawn((
        Button,
        LegacyCreationRect::new(x, y, 22.0, 21.0).node(),
        inside,
        ColorSwatch(kind, index),
        CharacterCreationButton(control),
    ));
}

pub(super) fn spawn_name_creation(parent: &mut ChildSpawnerCommands, assets: &CharacterCreationAssets) {
    spawn_image(
        parent,
        LegacyCreationRect::new(0.0, 0.0, 672.0, 365.0),
        assets,
        CC_NAME_BG,
    );
    spawn_label(
        parent,
        LegacyCreationRect::new(5.0, 5.0, 250.0, 20.0),
        "ui.character_create.title",
        "CHARACTER CREATION",
        assets,
        CharacterCreationTextStyle::Label,
    );
    spawn_name_mode_button(
        parent,
        LegacyCreationRect::new(90.0, 33.0, 231.0, 26.0),
        assets,
        CharacterNameMode::Generated,
        "ui.character_create.generate_name",
        "GENERATE A NAME",
    );
    spawn_name_mode_button(
        parent,
        LegacyCreationRect::new(354.0, 31.0, 231.0, 26.0),
        assets,
        CharacterNameMode::Custom,
        "ui.character_create.custom_name",
        "CREATE YOUR OWN",
    );
    spawn_label(
        parent,
        LegacyCreationRect::new(220.0, 30.0, 231.0, 26.0),
        "ui.common.or",
        "-OR-",
        assets,
        CharacterCreationTextStyle::OrText,
    );

    parent
        .spawn((
            LegacyCreationRect::new(0.0, 0.0, 672.0, 316.0).node(),
            ModeGeneratedRoot,
        ))
        .with_children(|generated| {
            spawn_image(
                generated,
                LegacyCreationRect::new(0.0, 32.0, 671.0, 285.0),
                assets,
                CC_NAME_TAB_1,
            );
            spawn_label(
                generated,
                LegacyCreationRect::new(90.0, 31.0, 231.0, 26.0),
                "ui.character_create.generate_name",
                "GENERATE A NAME",
                assets,
                CharacterCreationTextStyle::TabText,
            );
            generated
                .spawn((
                    character_creation_text_node(
                        LegacyCreationRect::new(76.0, 65.0, 524.0, 30.0),
                        CharacterCreationTextStyle::NameDisplay,
                    ),
                    source_style_image_node(assets, CC_NAME_DISPLAY),
                ))
                .with_child((
                    Text::new(""),
                    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                    character_creation_text_font(assets, CharacterCreationTextStyle::NameDisplay),
                    character_creation_text_color(CharacterCreationTextStyle::NameDisplay),
                    character_creation_text_layout(CharacterCreationTextStyle::NameDisplay),
                    CharacterCreationTextStyle::NameDisplay,
                    GeneratedNameText,
                ));
            spawn_image(
                generated,
                LegacyCreationRect::new(119.0, 124.0, 435.0, 156.0),
                assets,
                CC_SCROLL_BG,
            );
            for (part, x) in [
                (CharacterNamePart::First, 120.0),
                (CharacterNamePart::Middle, 264.0),
                (CharacterNamePart::Last, 408.0),
            ] {
                spawn_image_button(
                    generated,
                    LegacyCreationRect::new(x, 112.0, 147.0, 16.0),
                    assets,
                    CC_SCROLL_UP,
                    CharacterCreationControl::NameScroll(part, -1),
                );
                spawn_image_button(
                    generated,
                    LegacyCreationRect::new(x, 276.0, 147.0, 16.0),
                    assets,
                    CC_SCROLL_DOWN,
                    CharacterCreationControl::NameScroll(part, 1),
                );
                for row in 0..5 {
                    generated
                        .spawn(character_creation_text_node(
                            LegacyCreationRect::new(x, 128.0 + row as f32 * 30.0, 146.0, 30.0),
                            CharacterCreationTextStyle::Transparent,
                        ))
                        .with_child((
                            Text::new(""),
                            LocalizedText::new("ui.content.passthrough", "{text}")
                                .with_arg("text", ""),
                            character_creation_text_font(
                                assets,
                                CharacterCreationTextStyle::Transparent,
                            ),
                            character_creation_text_color(CharacterCreationTextStyle::Transparent),
                            character_creation_text_layout(CharacterCreationTextStyle::Transparent),
                            CharacterCreationTextStyle::Transparent,
                            NameColumnText(part, row),
                        ));
                }
            }
            spawn_image(
                generated,
                LegacyCreationRect::new(121.0, 127.0, 432.0, 150.0),
                assets,
                CC_NAME_SHADE,
            );
            spawn_text_button(
                generated,
                LegacyCreationRect::new(567.0, 192.0, 92.0, 20.0),
                assets,
                "ui.character_create.random",
                "RANDOM",
                CharacterCreationControl::RandomName,
            );
            spawn_label(
                generated,
                LegacyCreationRect::new(200.0, 316.0, 270.0, 40.0),
                "ui.character_create.generated_name_rules",
                "Names must have at least two parts.\nSingle-word names will not be accepted.",
                assets,
                CharacterCreationTextStyle::Transparent4,
            );
        });

    parent
        .spawn((
            LegacyCreationRect::new(0.0, 0.0, 672.0, 316.0).node(),
            ModeCustomRoot,
        ))
        .with_children(|custom| {
            spawn_image(
                custom,
                LegacyCreationRect::new(0.0, 30.0, 671.0, 286.0),
                assets,
                CC_NAME_TAB_2,
            );
            spawn_label(
                custom,
                LegacyCreationRect::new(354.0, 31.0, 231.0, 26.0),
                "ui.character_create.custom_name",
                "CREATE YOUR OWN",
                assets,
                CharacterCreationTextStyle::TabText,
            );
            spawn_label(
                custom,
                LegacyCreationRect::new(168.0, 103.0, 341.0, 25.0),
                "ui.character_create.custom_name_question",
                "WANT TO CREATE YOUR OWN NAME?",
                assets,
                CharacterCreationTextStyle::CustomQuestion,
            );
            spawn_label(
                custom,
                LegacyCreationRect::new(142.0, 139.0, 396.0, 58.0),
                "ui.character_create.custom_name_help",
                "Type in the name you wish to use and\nclick 'CONTINUE'. Your custom name\nwill be submitted for review.",
                assets,
                CharacterCreationTextStyle::Transparent5,
            );
            custom
                .spawn((
                    Button,
                    character_creation_text_node(
                        LegacyCreationRect::new(146.0, 217.0, 383.0, 28.0),
                        CharacterCreationTextStyle::TextField,
                    ),
                    source_style_image_node(assets, CHARACTER_CREATION_TEXT_FIELD_PATH),
                    CharacterCreationButton(CharacterCreationControl::FocusCustomName),
                ))
                .with_child((
                    Text::new(""),
                    LocalizedText::new("ui.content.passthrough", "{text}")
                        .with_arg("text", ""),
                    character_creation_text_font(assets, CharacterCreationTextStyle::TextField),
                    character_creation_text_color(CharacterCreationTextStyle::TextField),
                    character_creation_text_layout(CharacterCreationTextStyle::TextField),
                    CharacterCreationTextStyle::TextField,
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                    CustomNameText,
                ));
            spawn_label(
                custom,
                LegacyCreationRect::new(100.0, 316.0, 420.0, 40.0),
                "ui.character_create.custom_name_rules",
                "Names must have at least two parts.\nCustom first names cannot exceed 8 characters.\nCustom last names cannot exceed 16 characters.",
                assets,
                CharacterCreationTextStyle::Transparent4,
            );
        });

    spawn_text_button(
        parent,
        LegacyCreationRect::new(9.0, 323.0, 68.0, 29.0),
        assets,
        "ui.common.exit",
        "EXIT",
        CharacterCreationControl::Exit,
    );
    spawn_text_button(
        parent,
        LegacyCreationRect::new(525.0, 322.0, 135.0, 29.0),
        assets,
        "ui.common.continue",
        "CONTINUE",
        CharacterCreationControl::ContinueName,
    );
}
