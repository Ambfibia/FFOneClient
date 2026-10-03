use super::*;

pub(super) fn sync_tutorial_voice_subtitle_ui(
    windows: Query<&Window, With<PrimaryWindow>>,
    state: Res<TutorialVoiceSubtitleState>,
    context: Res<TutorialVoiceSubtitleUiContext>,
    mut elements: Query<(
        &TutorialVoiceSubtitleUiElement,
        &mut Node,
        &mut Visibility,
        Option<&mut LocalizedText>,
        Option<(&mut TextFont, &mut LineHeight)>,
        Option<&mut TextColor>,
    )>,
) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let Ok(geometry) = TutorialVoiceSubtitleGeometry::for_viewport(window.width(), window.height())
    else {
        return;
    };
    let active = state.active();
    let visibility = tutorial_voice_subtitle_visibility(active.is_some(), &context);

    for (element, mut node, mut element_visibility, localized, font, text_color) in &mut elements {
        match element {
            TutorialVoiceSubtitleUiElement::Root => {}
            TutorialVoiceSubtitleUiElement::Area => {
                node.left = px(geometry.scaled_area.x);
                node.top = px(geometry.scaled_area.y);
                node.width = px(geometry.scaled_area.width);
                node.height = px(geometry.scaled_area.height);
                *element_visibility = if visibility.subtitle {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            TutorialVoiceSubtitleUiElement::Speaker => {
                apply_gui_layout_label_node(
                    &mut node,
                    TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE,
                    geometry.ui_scale,
                );
                if let (Some(mut font), Some(mut text_color)) = (font, text_color) {
                    apply_gui_text_style(
                        &mut font.0,
                        &mut font.1,
                        &mut text_color,
                        TUTORIAL_VOICE_SUBTITLE_CENTERBOX_STYLE,
                        geometry.ui_scale,
                    );
                }
                if let Some(mut localized) = localized {
                    let value = active
                        .map(|active| active.resolved.speaker.clone())
                        .unwrap_or_default();
                    *localized = LocalizedText::new("ui.content.passthrough", "{text}")
                        .with_arg("text", value);
                }
            }
            TutorialVoiceSubtitleUiElement::Dialogue => {
                apply_gui_layout_label_node(
                    &mut node,
                    TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE,
                    geometry.ui_scale,
                );
                if let (Some(mut font), Some(mut text_color)) = (font, text_color) {
                    apply_gui_text_style(
                        &mut font.0,
                        &mut font.1,
                        &mut text_color,
                        TUTORIAL_VOICE_SUBTITLE_SMALLFONT2_STYLE,
                        geometry.ui_scale,
                    );
                }
                if let Some(mut localized) = localized {
                    let value = active
                        .map(|active| active.resolved.dialogue.clone())
                        .unwrap_or_default();
                    *localized = LocalizedText::new("ui.content.passthrough", "{text}")
                        .with_arg("text", value);
                }
            }
            TutorialVoiceSubtitleUiElement::SkipLabel => {
                node.height = px(30.0 * geometry.ui_scale);
                node.padding = TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE
                    .padding
                    .scaled_ui_rect(geometry.ui_scale);
                node.overflow = TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE.overflow();
                *element_visibility = if visibility.skip_label {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
            TutorialVoiceSubtitleUiElement::SkipLabelText => {
                if let (Some(mut font), Some(mut text_color)) = (font, text_color) {
                    apply_gui_text_style(
                        &mut font.0,
                        &mut font.1,
                        &mut text_color,
                        TUTORIAL_VOICE_SUBTITLE_SMALLFONT_STYLE,
                        geometry.ui_scale,
                    );
                }
            }
        }
    }
}
