use super::*;

pub(super) fn spawn_tutorial_voice_subtitle_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let chalet: Handle<Font> = asset_server.load(TUTORIAL_VOICE_SUBTITLE_CHALET_FONT_PATH);
    let jeffe: Handle<Font> = asset_server.load(TUTORIAL_VOICE_SUBTITLE_JEFFE_FONT_PATH);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(TUTORIAL_VOICE_SUBTITLE_Z_INDEX),
            TutorialVoiceSubtitleUiElement::Root,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: px(0),
                    height: px(0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                Visibility::Hidden,
                Pickable::IGNORE,
                TutorialVoiceSubtitleUiElement::Area,
            ))
            .with_children(|area| {
                area.spawn((
                    gui_layout_label_node(TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE, 1.0),
                    Text::new(""),
                    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                    gui_text_font(jeffe, TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE, 1.0),
                    TextColor(TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE.color()),
                    gui_text_layout(TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE),
                    Visibility::Inherited,
                    Pickable::IGNORE,
                    TutorialVoiceSubtitleUiElement::Speaker,
                ));
                area.spawn((
                    gui_layout_label_node(TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE, 1.0),
                    Text::new(""),
                    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                    gui_text_font(
                        chalet.clone(),
                        TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE,
                        1.0,
                    ),
                    TextColor(TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE.color()),
                    gui_text_layout(TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE),
                    Visibility::Inherited,
                    Pickable::IGNORE,
                    TutorialVoiceSubtitleUiElement::Dialogue,
                ));
            });
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    width: percent(100),
                    height: px(30),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    padding: TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE
                        .padding
                        .scaled_ui_rect(1.0),
                    overflow: TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE.overflow(),
                    ..default()
                },
                Visibility::Hidden,
                Pickable::IGNORE,
                TutorialVoiceSubtitleUiElement::SkipLabel,
            ))
            .with_child((
                Text::new(TUTORIAL_SKIP_LABEL),
                LocalizedText::new("ui.tutorial.skip", TUTORIAL_SKIP_LABEL),
                gui_text_font(chalet, TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE, 1.0),
                TextColor(TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE.color()),
                gui_text_layout(TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE),
                Pickable::IGNORE,
                TutorialVoiceSubtitleUiElement::SkipLabelText,
            ));
        });
}
