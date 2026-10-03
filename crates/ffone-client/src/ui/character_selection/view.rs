use super::*;

pub(super) fn spawn_character_selection_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_assets: Option<Res<CharacterSelectionAssets>>,
) {
    if existing_assets.is_some() {
        return;
    }
    let assets = CharacterSelectionAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands.spawn((
        AudioPlayer::new(assets.music.clone()),
        PlaybackSettings::LOOP
            .paused()
            .with_volume(Volume::Linear(0.5)),
        SelectionMusic,
    ));
    let base_camera = commands
        .spawn((
            Name::new("Character selection base UI camera"),
            Camera2d,
            Camera {
                order: GAMEPLAY_UI_CAMERA_ORDER,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            SelectionBaseCamera,
        ))
        .id();
    let modal_camera = commands
        .spawn((
            Name::new("Character selection modal UI camera"),
            Camera2d,
            Camera {
                order: CHARACTER_SELECTION_MODAL_CAMERA_ORDER,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            SelectionModalCamera,
        ))
        .id();
    let portrait_overlay_camera = commands
        .spawn((
            Name::new("Character selection portrait foreground UI camera"),
            Camera2d,
            Camera {
                order: CHARACTER_SELECTION_PORTRAIT_OVERLAY_CAMERA_ORDER,
                clear_color: ClearColorConfig::None,
                ..default()
            },
            SelectionPortraitOverlayCamera,
        ))
        .id();

    let mut delete_modal = None;
    commands
        .spawn((
            absolute_node(LegacySelectionRect::new(
                0.0,
                0.0,
                CHARACTER_SELECTION_BASELINE_WIDTH,
                CHARACTER_SELECTION_BASELINE_HEIGHT,
            )),
            Visibility::Hidden,
            ZIndex(CHARACTER_SELECTION_BASE_INTERACTION_Z_INDEX),
            UiTargetCamera(base_camera),
            NativeCharacterSelectionRoot,
            crate::ui::shared::controller::ControllerUiScope::CharacterSelection,
        ))
        .with_children(|root| {
            for index in 0..CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT {
                root.spawn((
                    absolute_node(LegacySelectionRect::new(
                        index as f32 * CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH,
                        0.0,
                        CHARACTER_SELECTION_BACKGROUND_FRAME_WIDTH,
                        CHARACTER_SELECTION_BACKGROUND_FRAME_HEIGHT,
                    )),
                    UiTransform::default(),
                    image_node(assets.background(CharacterLocationBackground::Future, index)),
                    ZIndex(0),
                    SelectionBackgroundFrame(index),
                ));
            }
            root.spawn((
                absolute_node(LegacySelectionRect::new(
                    0.0,
                    0.0,
                    CHARACTER_SELECTION_REFERENCE_WIDTH,
                    CHARACTER_SELECTION_REFERENCE_HEIGHT,
                )),
                UiTransform::default(),
                image_node(assets.chrome.clone()),
                ZIndex(1),
                SelectionChrome,
            ));
            root.spawn((
                absolute_node(LegacySelectionRect::new(0.0, 0.0, 600.0, 600.0)),
                UiTransform::default(),
                ZIndex(2),
                SelectionAvatarGroup,
            ))
            .with_children(|avatar| {
                avatar.spawn((
                    absolute_node(LegacySelectionRect::new(
                        0.0,
                        0.0,
                        CHARACTER_SELECTION_PREVIEW_WIDTH,
                        CHARACTER_SELECTION_PREVIEW_HEIGHT,
                    )),
                    BackgroundColor(Color::NONE),
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                    SelectionPreviewArea,
                ))
                .with_child((
                    Text::new("Loading native player preview..."),
                    LocalizedText::new(
                        "status.player_preview.loading",
                        "Loading native player preview...",
                    ),
                    character_selection_text_node(
                        LegacySelectionRect::new(105.0, 275.0, 340.0, 30.0),
                        CharacterSelectionTextStyle::AvatarName,
                    ),
                    character_selection_text_font(
                        &assets,
                        CharacterSelectionTextStyle::AvatarName,
                    ),
                    character_selection_text_color(CharacterSelectionTextStyle::AvatarName),
                    character_selection_text_layout(CharacterSelectionTextStyle::AvatarName),
                    CharacterSelectionTextStyle::AvatarName,
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                    SelectionPreviewLoading,
                ));
                avatar.spawn((
                    Button,
                    absolute_node(LegacySelectionRect::new(50.0, 480.0, 66.0, 110.0)),
                    legacy_sliced_button_image(assets.rotate_left.clone()),
                    SelectionRotateLeft,
                ));
                avatar.spawn((
                    Button,
                    absolute_node(LegacySelectionRect::new(440.0, 480.0, 66.0, 110.0)),
                    legacy_sliced_button_image(assets.rotate_right.clone()),
                    SelectionRotateRight,
                ));
                avatar
                    .spawn(character_selection_text_node(
                        LegacySelectionRect::new(0.0, 23.0, 550.0, 15.0),
                        CharacterSelectionTextStyle::AvatarName,
                    ))
                    .with_child((
                        Text::new(""),
                        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                        character_selection_text_font(
                            &assets,
                            CharacterSelectionTextStyle::AvatarName,
                        ),
                        character_selection_text_color(CharacterSelectionTextStyle::AvatarName),
                        character_selection_text_layout(CharacterSelectionTextStyle::AvatarName),
                        CharacterSelectionTextStyle::AvatarName,
                        SelectionAvatarName,
                    ));
            });
            root.spawn((
                absolute_node(LegacySelectionRect::new(0.0, 0.0, 478.0, 600.0)),
                UiTransform::default(),
                ZIndex(3),
                SelectionPanel,
            ))
            .with_children(|panel| {
                panel
                    .spawn(character_selection_text_node(
                        LegacySelectionRect::new(134.0, 2.0, 217.0, 25.0),
                        CharacterSelectionTextStyle::Transparent2,
                    ))
                    .with_child((
                        Text::new("SELECT A CHARACTER"),
                        LocalizedText::new("ui.character_select.title", "SELECT A CHARACTER"),
                        character_selection_text_font(
                            &assets,
                            CharacterSelectionTextStyle::Transparent2,
                        ),
                        character_selection_text_color(CharacterSelectionTextStyle::Transparent2),
                        character_selection_text_layout(CharacterSelectionTextStyle::Transparent2),
                        CharacterSelectionTextStyle::Transparent2,
                    ));

                for index in 0..4 {
                    panel.spawn((
                        Button,
                        absolute_node(SLOT_BUTTON_RECTS[index]),
                        legacy_sliced_button_image(assets.slot_empty.clone()),
                        SelectionSlotButton(index),
                        crate::ui::shared::controller::ControllerUiDefault,
                    ));
                    let empty_label_font = character_selection_text_font(
                        &assets,
                        CharacterSelectionTextStyle::Transparent3,
                    );
                    // Clean paint order: the EMPTY/subscription label follows
                    // the slot background and precedes the portrait disks.
                    panel.spawn((
                        character_selection_text_node(
                            LegacySelectionRect::new(165.0, SLOT_EMPTY_Y[index], 172.0, 16.0),
                            CharacterSelectionTextStyle::Transparent3,
                        ),
                        Text::new("EMPTY"),
                        LocalizedText::new("ui.character_select.empty", "EMPTY"),
                        empty_label_font.clone(),
                        UiTextAutoFit::new(172.0, 16.0, &empty_label_font),
                        character_selection_text_color(CharacterSelectionTextStyle::Transparent3),
                        character_selection_text_layout(CharacterSelectionTextStyle::Transparent3),
                        CharacterSelectionTextStyle::Transparent3,
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                        SelectionSlotEmptyLabel(index),
                    ));
                    panel.spawn((
                        absolute_node(LegacySelectionRect::new(
                            26.0,
                            SLOT_AVATAR_Y[index],
                            63.0,
                            85.0,
                        )),
                        BackgroundColor(Color::NONE),
                        Visibility::Hidden,
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                        SelectionSlotPortraitArea,
                    ));
                    panel.spawn((
                        absolute_node(LegacySelectionRect::new(
                            23.0,
                            SLOT_DISK_BACK_Y[index],
                            69.0,
                            23.0,
                        )),
                        legacy_sliced_button_image(assets.disk_back.clone()),
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                        SelectionSlotDiskBack(index),
                    ));
                    panel
                        .spawn((
                            character_selection_text_node(
                                LegacySelectionRect::new(85.0, SLOT_NAME_Y[index], 370.0, 16.0),
                                CharacterSelectionTextStyle::CharNameUp,
                            ),
                            FocusPolicy::Pass,
                            Pickable::IGNORE,
                        ))
                        .with_child((
                            Text::new(""),
                            LocalizedText::new("ui.content.passthrough", "{text}")
                                .with_arg("text", ""),
                            character_selection_text_font(
                                &assets,
                                CharacterSelectionTextStyle::CharNameUp,
                            ),
                            character_selection_text_color(CharacterSelectionTextStyle::CharNameUp),
                            character_selection_text_layout(
                                CharacterSelectionTextStyle::CharNameUp,
                            ),
                            CharacterSelectionTextStyle::CharNameUp,
                            FocusPolicy::Pass,
                            Pickable::IGNORE,
                            SelectionSlotName(index),
                        ));
                    panel
                        .spawn((
                            character_selection_text_node(
                                LegacySelectionRect::new(98.0, SLOT_LEVEL_Y[index], 97.0, 16.0),
                                CharacterSelectionTextStyle::CharLevelUp,
                            ),
                            FocusPolicy::Pass,
                            Pickable::IGNORE,
                        ))
                        .with_child((
                            Text::new(""),
                            LocalizedText::new("ui.character_select.level", "LEVEL {level}")
                                .with_arg("level", ""),
                            character_selection_text_font(
                                &assets,
                                CharacterSelectionTextStyle::CharLevelUp,
                            ),
                            character_selection_text_color(
                                CharacterSelectionTextStyle::CharLevelUp,
                            ),
                            character_selection_text_layout(
                                CharacterSelectionTextStyle::CharLevelUp,
                            ),
                            CharacterSelectionTextStyle::CharLevelUp,
                            FocusPolicy::Pass,
                            Pickable::IGNORE,
                            SelectionSlotLevel(index),
                        ));
                    panel
                        .spawn((
                            character_selection_text_node(
                                LegacySelectionRect::new(98.0, SLOT_LOCATION_Y[index], 250.0, 16.0),
                                CharacterSelectionTextStyle::CharLevelUp,
                            ),
                            FocusPolicy::Pass,
                            Pickable::IGNORE,
                        ))
                        .with_child((
                            Text::new(""),
                            LocalizedText::new("ui.content.passthrough", "{text}")
                                .with_arg("text", ""),
                            character_selection_text_font(
                                &assets,
                                CharacterSelectionTextStyle::CharLevelUp,
                            ),
                            character_selection_text_color(
                                CharacterSelectionTextStyle::CharLevelUp,
                            ),
                            character_selection_text_layout(
                                CharacterSelectionTextStyle::CharLevelUp,
                            ),
                            CharacterSelectionTextStyle::CharLevelUp,
                            FocusPolicy::Pass,
                            Pickable::IGNORE,
                            SelectionSlotLocation(index),
                        ));
                }

                spawn_small_legacy_button(
                    panel,
                    LegacySelectionRect::new(244.0, 449.0, 188.0, 25.0),
                    "CREATE CHARACTER",
                    "ui.character_select.create",
                    &assets,
                    CharacterSelectionTextStyle::CreateButton,
                    assets.blue_button.clone(),
                    SelectionCreate,
                    SelectionCreateText,
                );
                panel
                    .spawn((
                        Button,
                        character_selection_text_node(
                            LegacySelectionRect::new(83.0, 503.0, 325.0, 57.0),
                            CharacterSelectionTextStyle::EnterGame,
                        ),
                        image_node(assets.enter.clone()),
                        SelectionEnter,
                    ))
                    .with_child((
                        Text::new("ENTER THE GAME"),
                        LocalizedText::new("ui.character_select.enter", "ENTER THE GAME"),
                        character_selection_text_font(
                            &assets,
                            CharacterSelectionTextStyle::EnterGame,
                        ),
                        character_selection_text_color(CharacterSelectionTextStyle::EnterGame),
                        character_selection_text_layout(CharacterSelectionTextStyle::EnterGame),
                        CharacterSelectionTextStyle::EnterGame,
                        FocusPolicy::Pass,
                        Pickable::IGNORE,
                        SelectionEnterText,
                    ));
                panel.spawn((
                    Button,
                    absolute_node(LegacySelectionRect::new(225.0, 567.0, 86.0, 30.0)),
                    image_node(assets.music_toggle_off.clone()),
                    SelectionMusicToggle,
                ));
            });
            root.spawn((
                Button,
                character_selection_text_node(
                    LegacySelectionRect::new(20.0, 643.0, 90.0, 23.0),
                    CharacterSelectionTextStyle::QuitButton,
                ),
                UiTransform::default(),
                legacy_sliced_button_image(assets.red_button.clone()),
                SelectionQuit,
                ZIndex(4),
            ))
            .with_child((
                Text::new("QUIT"),
                LocalizedText::new("ui.common.quit", "QUIT"),
                character_selection_text_font(&assets, CharacterSelectionTextStyle::QuitButton),
                character_selection_text_color(CharacterSelectionTextStyle::QuitButton),
                character_selection_text_layout(CharacterSelectionTextStyle::QuitButton),
                CharacterSelectionTextStyle::QuitButton,
                FocusPolicy::Pass,
                Pickable::IGNORE,
                SelectionQuitText,
            ));
            root.spawn((
                Button,
                absolute_node(LegacySelectionRect::new(1219.0, 646.0, 35.0, 31.0)),
                UiTransform::default(),
                image_node(assets.fullscreen.clone()),
                SelectionFullscreen,
                ZIndex(4),
            ));
            delete_modal = Some(
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0.0),
                        top: px(0.0),
                        width: percent(100.0),
                        height: percent(100.0),
                        ..default()
                    },
                    tinted_image_node(
                        assets.delete_backdrop.clone(),
                        Color::srgba(0.0, 0.0, 0.0, 0.75),
                    ),
                    Visibility::Hidden,
                    ZIndex(CHARACTER_SELECTION_MODAL_INTERACTION_Z_INDEX),
                    UiTargetCamera(modal_camera),
                    SelectionDeleteModal,
                ))
                .with_children(|modal| {
                    modal
                        .spawn((
                            absolute_node(LegacySelectionRect::new(
                                CHARACTER_SELECTION_BASELINE_WIDTH * 0.5 - 263.0,
                                CHARACTER_SELECTION_BASELINE_HEIGHT * 0.5 - 82.0,
                                526.0,
                                164.0,
                            )),
                            UiTransform::default(),
                            image_node(assets.delete_window.clone()),
                            SelectionDeletePanel,
                        ))
                        .with_children(|panel| {
                            panel
                                .spawn(character_selection_text_node(
                                    LegacySelectionRect::new(176.0, 15.0, 173.0, 17.0),
                                    CharacterSelectionTextStyle::Transparent3,
                                ))
                                .with_child((
                                    Text::new("DELETE CHARACTER"),
                                    LocalizedText::new(
                                        "ui.character_select.delete",
                                        "DELETE CHARACTER",
                                    ),
                                    character_selection_text_font(
                                        &assets,
                                        CharacterSelectionTextStyle::Transparent3,
                                    ),
                                    character_selection_text_color(
                                        CharacterSelectionTextStyle::Transparent3,
                                    ),
                                    character_selection_text_layout(
                                        CharacterSelectionTextStyle::Transparent3,
                                    ),
                                    CharacterSelectionTextStyle::Transparent3,
                                    FocusPolicy::Pass,
                                    Pickable::IGNORE,
                                ));
                            panel
                                .spawn(character_selection_text_node(
                                    LegacySelectionRect::new(118.0, 36.0, 288.0, 17.0),
                                    CharacterSelectionTextStyle::DeleteText,
                                ))
                                .with_child((
                                    Text::new("Please enter your character's first name to delete"),
                                    LocalizedText::new(
                                        "ui.character_select.delete_prompt",
                                        "Please enter your character's first name to delete",
                                    ),
                                    character_selection_text_font(
                                        &assets,
                                        CharacterSelectionTextStyle::DeleteText,
                                    ),
                                    character_selection_text_color(
                                        CharacterSelectionTextStyle::DeleteText,
                                    ),
                                    character_selection_text_layout(
                                        CharacterSelectionTextStyle::DeleteText,
                                    ),
                                    CharacterSelectionTextStyle::DeleteText,
                                    FocusPolicy::Pass,
                                    Pickable::IGNORE,
                                ));
                            panel
                                .spawn(character_selection_text_node(
                                    LegacySelectionRect::new(75.0, 73.0, 378.0, 24.0),
                                    CharacterSelectionTextStyle::DeleteText,
                                ))
                                .with_child((
                                    Text::new(""),
                                    LocalizedText::new("ui.content.passthrough", "{text}")
                                        .with_arg("text", ""),
                                    character_selection_text_font(
                                        &assets,
                                        CharacterSelectionTextStyle::DeleteText,
                                    ),
                                    character_selection_text_color(
                                        CharacterSelectionTextStyle::DeleteText,
                                    ),
                                    character_selection_text_layout(
                                        CharacterSelectionTextStyle::DeleteText,
                                    ),
                                    CharacterSelectionTextStyle::DeleteText,
                                    FocusPolicy::Pass,
                                    Pickable::IGNORE,
                                    SelectionDeleteFieldText,
                                ));
                            panel
                                .spawn((
                                    Button,
                                    character_selection_text_node(
                                        LegacySelectionRect::new(40.0, 125.0, 152.0, 27.0),
                                        CharacterSelectionTextStyle::Cancel,
                                    ),
                                    legacy_sliced_button_image(assets.cancel_normal.clone()),
                                    SelectionDeleteCancel,
                                ))
                                .with_child((
                                    Text::new("CANCEL"),
                                    LocalizedText::new("ui.common.cancel", "CANCEL"),
                                    character_selection_text_font(
                                        &assets,
                                        CharacterSelectionTextStyle::Cancel,
                                    ),
                                    character_selection_text_color(
                                        CharacterSelectionTextStyle::Cancel,
                                    ),
                                    character_selection_text_layout(
                                        CharacterSelectionTextStyle::Cancel,
                                    ),
                                    CharacterSelectionTextStyle::Cancel,
                                    FocusPolicy::Pass,
                                    Pickable::IGNORE,
                                    SelectionDeleteCancelText,
                                ));
                            panel
                                .spawn((
                                    Button,
                                    character_selection_text_node(
                                        LegacySelectionRect::new(335.0, 125.0, 152.0, 27.0),
                                        CharacterSelectionTextStyle::QuitButton,
                                    ),
                                    legacy_sliced_button_image(assets.red_button.clone()),
                                    SelectionDeleteConfirm,
                                ))
                                .with_child((
                                    Text::new("DELETE"),
                                    LocalizedText::new("ui.common.delete", "DELETE"),
                                    character_selection_text_font(
                                        &assets,
                                        CharacterSelectionTextStyle::QuitButton,
                                    ),
                                    character_selection_text_color(
                                        CharacterSelectionTextStyle::QuitButton,
                                    ),
                                    character_selection_text_layout(
                                        CharacterSelectionTextStyle::QuitButton,
                                    ),
                                    CharacterSelectionTextStyle::QuitButton,
                                    FocusPolicy::Pass,
                                    Pickable::IGNORE,
                                    SelectionDeleteConfirmText,
                                ));
                        });
                })
                .id(),
            );
        });
    commands
        .spawn((
            absolute_node(LegacySelectionRect::new(0.0, 0.0, 478.0, 600.0)),
            UiTransform::default(),
            Visibility::Hidden,
            UiTargetCamera(portrait_overlay_camera),
            ZIndex(CHARACTER_SELECTION_PORTRAIT_OVERLAY_INTERACTION_Z_INDEX),
            FocusPolicy::Pass,
            Pickable::IGNORE,
            SelectionPortraitOverlayRoot,
            crate::ui::shared::controller::ControllerUiScope::CharacterSelection,
        ))
        .with_children(|overlay| {
            for index in 0..4 {
                overlay.spawn((
                    absolute_node(LegacySelectionRect::new(
                        23.0,
                        SLOT_DISK_FRONT_Y[index],
                        69.0,
                        23.0,
                    )),
                    legacy_sliced_button_image(assets.disk_front.clone()),
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                    SelectionSlotDiskFront(index),
                ));
                overlay.spawn((
                    absolute_node(LegacySelectionRect::new(
                        42.0,
                        SLOT_LOCK_Y[index],
                        32.0,
                        36.0,
                    )),
                    legacy_sliced_button_image(assets.lock.clone()),
                    Visibility::Hidden,
                    FocusPolicy::Pass,
                    Pickable::IGNORE,
                    SelectionSlotLock(index),
                ));
            }
            // Clean IMGUI paint order: every portrait foreground and lock is
            // drawn before the bottom controls. Keeping DELETE last in this
            // same post-portrait pass prevents either 3D previews or their
            // decorations from painting over the button.
            spawn_small_legacy_button(
                overlay,
                LegacySelectionRect::new(47.0, 449.0, 188.0, 25.0),
                "DELETE CHARACTER",
                "ui.character_select.delete",
                &assets,
                CharacterSelectionTextStyle::QuitButton,
                assets.red_button.clone(),
                SelectionDelete,
                SelectionDeleteText,
            );
        });
    // `UiTargetCamera` is honored only on root UI nodes in Bevy 0.17.
    // Build the dialog with the selection hierarchy for convenient authored
    // coordinates, then detach its complete subtree into the modal camera.
    commands
        .entity(delete_modal.expect("selection delete modal must be spawned"))
        .remove::<ChildOf>();
}

pub(super) fn spawn_small_legacy_button<M: Component, L: Component>(
    parent: &mut ChildSpawnerCommands,
    rect: LegacySelectionRect,
    label: &'static str,
    localization_key: &'static str,
    assets: &CharacterSelectionAssets,
    style: CharacterSelectionTextStyle,
    background: Handle<Image>,
    marker: M,
    label_marker: L,
) {
    parent
        .spawn((
            Button,
            character_selection_text_node(rect, style),
            legacy_sliced_button_image(background),
            marker,
        ))
        .with_child((
            Text::new(label),
            LocalizedText::new(localization_key, label),
            character_selection_text_font(assets, style),
            character_selection_text_color(style),
            character_selection_text_layout(style),
            style,
            FocusPolicy::Pass,
            Pickable::IGNORE,
            label_marker,
        ));
}
