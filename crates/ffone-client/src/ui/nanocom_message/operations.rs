use super::*;

#[must_use]
pub const fn nanocom_gui_style(role: NanocomGuiStyleRole) -> NanocomGuiStyleEvidence {
    match role {
        NanocomGuiStyleRole::BigFont14 => NanocomGuiStyleEvidence {
            skin_path_id: NANOCOM_HUD_SKIN_PATH_ID,
            style_name: "BIGFont14",
            source_font_name: "JEFFE___14",
            source_font_path_id: NANOCOM_JEFFE_14_FONT_PATH_ID,
            replacement_font_path: NANOCOM_JEFFE_FONT_PATH,
            replacement_font_size: NANOCOM_JEFFE_14_FONT_SIZE,
            source_line_height: NANOCOM_JEFFE_14_LINE_HEIGHT,
            alignment: 3,
            word_wrap: true,
            text_clipping: 1,
            padding: NanocomGuiInsets::ZERO,
            margin: NanocomGuiInsets::ZERO,
            content_offset: [0.0, 0.0],
            replacement_y_offset: 0.0,
        },
        NanocomGuiStyleRole::MessageText => NanocomGuiStyleEvidence {
            skin_path_id: NANOCOM_HUD_SKIN_PATH_ID,
            style_name: "messagetext",
            source_font_name: "ChaletBook-Regular Small",
            source_font_path_id: NANOCOM_CHALET_SMALL_FONT_PATH_ID,
            replacement_font_path: NANOCOM_CHALET_FONT_PATH,
            replacement_font_size: NANOCOM_CHALET_SMALL_FONT_SIZE,
            source_line_height: NANOCOM_CHALET_SMALL_LINE_HEIGHT,
            alignment: 0,
            word_wrap: true,
            text_clipping: 0,
            padding: NanocomGuiInsets::new(10.0, 4.0, 6.0, 6.0),
            margin: NanocomGuiInsets::new(4.0, 4.0, 4.0, 4.0),
            content_offset: [0.0, 0.0],
            replacement_y_offset: 0.0,
        },
        NanocomGuiStyleRole::MessageTitle => NanocomGuiStyleEvidence {
            skin_path_id: NANOCOM_HUD_SKIN_PATH_ID,
            style_name: "MessageTitle",
            source_font_name: "JEFFE___13",
            source_font_path_id: NANOCOM_MESSAGE_TITLE_FONT_PATH_ID,
            replacement_font_path: NANOCOM_JEFFE_FONT_PATH,
            replacement_font_size: NANOCOM_MESSAGE_TITLE_FONT_SIZE,
            source_line_height: NANOCOM_MESSAGE_TITLE_LINE_HEIGHT,
            alignment: 3,
            word_wrap: true,
            text_clipping: 1,
            padding: NanocomGuiInsets::ZERO,
            margin: NanocomGuiInsets::ZERO,
            content_offset: [0.0, 0.0],
            replacement_y_offset: 0.0,
        },
        NanocomGuiStyleRole::CenterBox2 => NanocomGuiStyleEvidence {
            skin_path_id: NANOCOM_HUD_SKIN_PATH_ID,
            style_name: "centerbox2",
            source_font_name: "ChaletBook-Regular Small",
            source_font_path_id: NANOCOM_CHALET_SMALL_FONT_PATH_ID,
            replacement_font_path: NANOCOM_CHALET_FONT_PATH,
            replacement_font_size: NANOCOM_CHALET_SMALL_FONT_SIZE,
            source_line_height: NANOCOM_CHALET_SMALL_LINE_HEIGHT,
            alignment: 1,
            word_wrap: true,
            text_clipping: 1,
            padding: NanocomGuiInsets::new(10.0, 4.0, 6.0, 6.0),
            margin: NanocomGuiInsets::new(4.0, 4.0, 4.0, 4.0),
            content_offset: [0.0, 0.0],
            replacement_y_offset: 0.0,
        },
        NanocomGuiStyleRole::Button => NanocomGuiStyleEvidence {
            skin_path_id: NANOCOM_HUD_SKIN_PATH_ID,
            style_name: "button",
            source_font_name: "JEFFE___14",
            source_font_path_id: NANOCOM_JEFFE_14_FONT_PATH_ID,
            replacement_font_path: NANOCOM_JEFFE_FONT_PATH,
            replacement_font_size: NANOCOM_JEFFE_14_FONT_SIZE,
            source_line_height: NANOCOM_JEFFE_14_LINE_HEIGHT,
            alignment: 4,
            word_wrap: false,
            text_clipping: 1,
            padding: NanocomGuiInsets::new(10.0, 3.0, 6.0, 6.0),
            margin: NanocomGuiInsets::new(4.0, 4.0, 4.0, 4.0),
            content_offset: [0.0, 0.0],
            replacement_y_offset: 0.0,
        },
        NanocomGuiStyleRole::RedButton => NanocomGuiStyleEvidence {
            skin_path_id: NANOCOM_HUD_SKIN_PATH_ID,
            style_name: "RedButton",
            source_font_name: "JEFFE___14",
            source_font_path_id: NANOCOM_JEFFE_14_FONT_PATH_ID,
            replacement_font_path: NANOCOM_JEFFE_FONT_PATH,
            replacement_font_size: NANOCOM_JEFFE_14_FONT_SIZE,
            source_line_height: NANOCOM_JEFFE_14_LINE_HEIGHT,
            alignment: 4,
            word_wrap: true,
            text_clipping: 1,
            padding: NanocomGuiInsets::new(10.0, 4.0, 6.0, 6.0),
            margin: NanocomGuiInsets::new(4.0, 4.0, 4.0, 4.0),
            content_offset: [0.0, 0.0],
            replacement_y_offset: 0.0,
        },
    }
}

pub(super) fn nanocom_comm_out_true_name(owner: &str, request_id: u64) -> String {
    let mut parts = owner.split('_');
    let first = parts.next().unwrap_or(owner);
    let second = parts.next();
    let third = parts.next();
    let base = if third.is_some() {
        format!("{first}_{}", second.unwrap_or_default())
    } else {
        owner.to_owned()
    };
    let take = request_id % 3 + 1;
    format!("{base}_CommOut0{take}")
}

pub(super) fn nanocom_passthrough(copy: impl Into<String>) -> LocalizedText {
    LocalizedText::new(NANOCOM_CONTENT_LOCALIZATION_KEY, "{text}").with_arg("text", copy)
}

#[must_use]
pub fn compact_buddy_body(body: &str, remaining_seconds: f32) -> String {
    format!(
        "{body}\n\nPress Enter to accept or decline.\n{} seconds remaining. ",
        rounded_seconds(remaining_seconds)
    )
}

#[must_use]
pub fn expanded_buddy_expiration(remaining_seconds: f32) -> String {
    format!(
        "Buddy will expire :{} seconds",
        rounded_seconds(remaining_seconds)
    )
}

pub(super) fn rounded_seconds(remaining_seconds: f32) -> String {
    format!("{:.0}", remaining_seconds.max(0.0))
}

pub(super) fn bind_nanocom_message_ui(
    windows: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    model: Res<NanocomMessageUiModel>,
    mut previous_binding: Local<Option<NanocomBindingKey>>,
    mut elements: Query<(
        &NanocomMessageElement,
        &mut Node,
        Option<&mut UiTransform>,
        Option<&mut Visibility>,
        Option<&mut ImageNode>,
        Option<&mut LocalizedText>,
    )>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let viewport = Vec2::new(window.width(), window.height());
    let scale = model.effective_ui_scale(viewport.y);
    let compact_layout = nanocom_compact_layout(viewport, model.reveal_parameter(), scale);
    let expanded_layout = nanocom_expanded_layout(viewport, scale);
    let active = model.active();
    let binding_key = NanocomBindingKey {
        viewport: [viewport.x.to_bits(), viewport.y.to_bits()],
        scale: scale.to_bits(),
        reveal: model.reveal_parameter().to_bits(),
        active_request: active.map(|active| active.request.clone()),
        rounded_remaining_seconds: active.map(|active| rounded_seconds(active.remaining_seconds)),
        compact_visible: model.compact_visible(),
        expanded_visible: model.expanded_visible(),
    };
    if previous_binding.as_ref() == Some(&binding_key) {
        return;
    }
    *previous_binding = Some(binding_key);

    for (element, mut node, transform, visibility, image, localized) in &mut elements {
        match element {
            NanocomMessageElement::CompactPanel => {
                node.left = px(compact_layout.node.x);
                node.top = px(compact_layout.node.y);
                node.width = px(compact_layout.node.width);
                node.height = px(compact_layout.node.height);
                node.overflow = Overflow::clip();
                if let Some(mut transform) = transform {
                    transform.scale = Vec2::splat(compact_layout.scale);
                }
                if let Some(mut visibility) = visibility {
                    *visibility = if model.compact_visible() {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            NanocomMessageElement::CompactFrame => {
                if let Some(active) = active {
                    *node = active.request.compact_frame_rect().node();
                }
                if let (Some(active), Some(mut image)) = (active, image) {
                    image.image = asset_server.load(active.request.compact_frame_path.clone());
                }
            }
            NanocomMessageElement::CompactIcon => {
                let icon_path = active.and_then(|active| active.request.compact_icon_path.as_ref());
                if let Some(active) = active {
                    *node = active.request.compact_icon_rect().node();
                }
                if let Some(mut visibility) = visibility {
                    *visibility = if icon_path.is_some() {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
                if let (Some(path), Some(mut image)) = (icon_path, image) {
                    image.image = asset_server.load(path.clone());
                }
            }
            NanocomMessageElement::CompactTitle => {
                if let Some(mut localized) = localized {
                    *localized = active
                        .map(|active| active.request.compact_title_localized())
                        .unwrap_or_else(|| nanocom_passthrough(""));
                }
            }
            NanocomMessageElement::CompactBody => {
                if let Some(mut localized) = localized {
                    *localized = active
                        .map(|active| {
                            active
                                .request
                                .compact_body_localized(active.remaining_seconds)
                        })
                        .unwrap_or_else(|| nanocom_passthrough(""));
                }
            }
            NanocomMessageElement::ExpandedRoot => {
                if let Some(mut visibility) = visibility {
                    *visibility = if model.expanded_visible() {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    };
                }
            }
            NanocomMessageElement::ExpandedDialog => {
                node.left = px(expanded_layout.node.x);
                node.top = px(expanded_layout.node.y);
                if let Some(mut transform) = transform {
                    transform.scale = Vec2::splat(expanded_layout.scale);
                }
            }
            NanocomMessageElement::ExpandedIcon => {
                let icon_path = active.and_then(|active| active.request.compact_icon_path.as_ref());
                if let Some(mut visibility) = visibility {
                    *visibility = if icon_path.is_some() {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
                if let (Some(path), Some(mut image)) = (icon_path, image) {
                    image.image = asset_server.load(path.clone());
                }
            }
            NanocomMessageElement::ExpandedTitle => {
                if let Some(mut localized) = localized {
                    *localized = active
                        .map(|active| active.request.expanded_title_localized())
                        .unwrap_or_else(|| nanocom_passthrough(""));
                }
            }
            NanocomMessageElement::ExpandedBody => {
                if let Some(mut localized) = localized {
                    *localized = active
                        .map(|active| active.request.expanded_body_localized())
                        .unwrap_or_else(|| nanocom_passthrough(""));
                }
            }
            NanocomMessageElement::ExpandedExpiration => {
                if let Some(mut localized) = localized {
                    *localized = active
                        .map(|active| {
                            active
                                .request
                                .expanded_expiration_localized(active.remaining_seconds)
                        })
                        .unwrap_or_else(|| nanocom_passthrough(""));
                }
            }
        }
    }
}

pub(super) fn play_nanocom_message_sounds(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Option<Res<NativeAudioCatalog>>,
    voice_language: Option<Res<VoiceLanguage>>,
    mut model: ResMut<NanocomMessageUiModel>,
    mut sources: Query<(&mut PlaybackSettings, Option<&AudioSink>), With<NanocomMessageAudio>>,
) {
    let paused = model.scene_event_active();
    for (mut settings, sink) in &mut sources {
        if settings.paused != paused {
            settings.paused = paused;
        }
        if let Some(sink) = sink {
            if paused && !sink.is_paused() {
                sink.pause();
            } else if !paused && sink.is_paused() {
                sink.play();
            }
        }
    }
    if model.scene_event_active() {
        return;
    }
    while let Some(sound) = model.pop_sound() {
        let path = match &sound {
            NanocomMessageSound::SlideIn => Some(NANOCOM_SLIDE_IN_PATH),
            NanocomMessageSound::SlideOut => Some(NANOCOM_SLIDE_OUT_PATH),
            NanocomMessageSound::NanoCreationComplete => Some(NANOCOM_NANO_CREATION_COMPLETE_PATH),
            NanocomMessageSound::Yes => Some(NANOCOM_YES_PATH),
            NanocomMessageSound::No => Some(NANOCOM_NO_PATH),
            NanocomMessageSound::Voice(_) => None,
        };
        if let Some(path) = path {
            commands.spawn((
                NanocomMessageAudio,
                crate::audio_channel::GameplayAudioChannel::ui_sfx(),
                AudioPlayer::new(asset_server.load(path)),
                PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.7)),
            ));
            continue;
        }
        let NanocomMessageSound::Voice(true_name) = sound else {
            unreachable!()
        };
        let (Some(catalog), Some(voice_language)) = (catalog.as_deref(), voice_language.as_deref())
        else {
            warn!("NanoCom voice {true_name:?} has no semantic audio authority");
            continue;
        };
        let Some((resolved_true_name, path)) =
            resolve_nanocom_voice(&catalog, &voice_language.effective, &true_name)
        else {
            warn!(
                "NanoCom voice {true_name:?} has no semantic voice asset or available CommOut sibling"
            );
            continue;
        };
        commands.spawn((
            NanocomMessageAudio,
            Name::new(format!("NanoCom voice {resolved_true_name}")),
            LocalizedVoice::by_true_name(resolved_true_name),
            crate::audio_channel::GameplayAudioChannel::new(NativeAudioCategory::Voice, 1.0),
            AudioPlayer::new(asset_server.load(path)),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(1.0)),
        ));
    }
}

pub fn cleanup_nanocom_message_ui(world: &mut World) {
    world.resource_mut::<NanocomMessageUiModel>().clear();
    world.resource_mut::<NanocomMessageUiOutbox>().clear();
}
