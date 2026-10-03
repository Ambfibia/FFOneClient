use super::*;

#[must_use]
pub const fn legacy_raw_fifth_tab_reset_visible(raw_tab: i32) -> bool {
    raw_tab == 4
}

pub(super) fn binding_for_slot(
    row: &LegacyInputBindingSet,
    slot: OptionInputMappingSlot,
) -> LegacyInputBinding {
    match slot {
        OptionInputMappingSlot::Primary => row.primary,
        OptionInputMappingSlot::Alternate => row.alternate,
        OptionInputMappingSlot::Pad => row.pad,
    }
}

pub(super) fn binding_for_slot_mut(
    row: &mut LegacyInputBindingSet,
    slot: OptionInputMappingSlot,
) -> &mut LegacyInputBinding {
    match slot {
        OptionInputMappingSlot::Primary => &mut row.primary,
        OptionInputMappingSlot::Alternate => &mut row.alternate,
        OptionInputMappingSlot::Pad => &mut row.pad,
    }
}

pub(super) fn reset_pad_profile_bindings(mappings: &mut [LegacyInputBindingSet], profile: LegacyPadProfile) {
    for row in mappings.iter_mut() {
        row.pad = LegacyInputBinding::Unbound;
    }
    let set = |mappings: &mut [LegacyInputBindingSet],
               action: LegacyOptionAction,
               binding: LegacyInputBinding| {
        if let Some(row) = mappings.iter_mut().find(|row| row.action == action) {
            row.pad = binding;
        }
    };
    let positive = |axis| LegacyInputBinding::PadAxis {
        axis,
        direction: LegacyAxisDirection::Positive,
    };
    let negative = |axis| LegacyInputBinding::PadAxis {
        axis,
        direction: LegacyAxisDirection::Negative,
    };
    set(mappings, LegacyOptionAction::Up, negative(4));
    set(mappings, LegacyOptionAction::Down, positive(4));
    set(mappings, LegacyOptionAction::Left, negative(3));
    set(mappings, LegacyOptionAction::Right, positive(3));

    let (a, b, x, back, start, camera_y_axis, camera_x_axis) = match profile {
        LegacyPadProfile::Xbox360 => (0, 1, 2, 6, 7, 7, 6),
        LegacyPadProfile::RumblePad2 => (1, 2, 0, 8, 9, 6, 5),
        LegacyPadProfile::PlayStation2 => (1, 2, 0, 8, 9, 5, 6),
    };
    for (action, button) in [
        (LegacyOptionAction::Jump, a),
        (LegacyOptionAction::Nano1, x),
        (LegacyOptionAction::Nano2, 3),
        (LegacyOptionAction::Nano3, b),
        (LegacyOptionAction::Fire1, 17),
        (LegacyOptionAction::Fire2, 16),
        (LegacyOptionAction::FreeCamera, 4),
        (LegacyOptionAction::NanoCharge, 5),
        (LegacyOptionAction::Menu, start),
        (LegacyOptionAction::Inventory, back),
        (LegacyOptionAction::VehicleToggle, 10),
        (LegacyOptionAction::AutoRun, 11),
        (LegacyOptionAction::ZoomIn, 12),
        (LegacyOptionAction::ZoomOut, 13),
        (LegacyOptionAction::Journal, 14),
        (LegacyOptionAction::WeaponChange, 15),
    ] {
        set(mappings, action, LegacyInputBinding::PadButton(button));
    }
    set(mappings, LegacyOptionAction::CameraUp, positive(camera_y_axis));
    set(mappings, LegacyOptionAction::CameraDown, negative(camera_y_axis));
    set(mappings, LegacyOptionAction::CameraLeft, negative(camera_x_axis));
    set(mappings, LegacyOptionAction::CameraRight, positive(camera_x_axis));
}

#[must_use]
pub fn legacy_binding_label(binding: LegacyInputBinding) -> String {
    match binding {
        LegacyInputBinding::Unbound => String::new(),
        LegacyInputBinding::Key(key) => format!("{key:?}"),
        LegacyInputBinding::Axis { axis, direction } => {
            let suffix = match direction {
                LegacyAxisDirection::Negative => "-",
                LegacyAxisDirection::Positive => "+",
            };
            format!("{axis:?} {suffix}")
        }
        LegacyInputBinding::PadButton(button) => format!("JoystickButton{button}"),
        LegacyInputBinding::PadAxis { axis, direction } => {
            let suffix = match direction {
                LegacyAxisDirection::Negative => "-",
                LegacyAxisDirection::Positive => "+",
            };
            format!("Joy {axis} {suffix}")
        }
    }
}

pub(super) fn option_ui_is_visible(model: Res<OptionUiModel>) -> bool {
    model.visible
}

pub(super) fn option_passthrough_text(value: impl Into<String>) -> LocalizedText {
    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
}

pub(super) fn sliced_image(image: Handle<Image>, border: BorderRect) -> ImageNode {
    ImageNode {
        image,
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: sliced_image_mode(border),
        ..default()
    }
}

pub(super) fn option_tab_image(image: Handle<Image>, tab: OptionTab, selected: bool) -> ImageNode {
    ImageNode {
        image,
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: option_tab_image_mode(tab, selected),
        ..default()
    }
}

pub(super) fn option_tab_content_style(tab: OptionTab, selected: bool) -> (JustifyContent, UiRect) {
    match (tab, selected) {
        (OptionTab::Graphics, false) => (
            JustifyContent::Start,
            UiRect {
                left: px(OPTION_GRAPHICS_TAB_TEXT_INSET),
                top: px(-8.0),
                ..default()
            },
        ),
        (OptionTab::Graphics, true) => (
            JustifyContent::Start,
            UiRect {
                left: px(OPTION_GRAPHICS_TAB_TEXT_INSET),
                top: px(-10.0),
                ..default()
            },
        ),
        (OptionTab::GameUi, false) => (JustifyContent::Center, UiRect::default()),
        (OptionTab::GameUi, true) => (
            JustifyContent::Center,
            UiRect {
                top: px(-10.0),
                ..default()
            },
        ),
        (OptionTab::Social, false) => (
            JustifyContent::Center,
            UiRect {
                top: px(-3.0),
                ..default()
            },
        ),
        (OptionTab::Social, true) => (
            JustifyContent::Center,
            UiRect {
                left: px(-4.0),
                top: px(-12.0),
                ..default()
            },
        ),
        (OptionTab::Controls, false) => (
            JustifyContent::End,
            UiRect {
                right: px(24.0),
                top: px(-11.0),
                ..default()
            },
        ),
        (OptionTab::Controls, true) => (
            JustifyContent::End,
            UiRect {
                right: px(24.0),
                top: px(3.0),
                ..default()
            },
        ),
    }
}

pub(super) const fn tab_normal_role(tab: OptionTab) -> OptionTextureRole {
    match tab {
        OptionTab::Graphics => OptionTextureRole::Graphics,
        OptionTab::GameUi => OptionTextureRole::GameUi,
        OptionTab::Social => OptionTextureRole::Social,
        OptionTab::Controls => OptionTextureRole::Controls,
    }
}

pub(super) const fn tab_label(tab: OptionTab) -> &'static str {
    match tab {
        OptionTab::Graphics => "GRAPHICS & SOUND",
        OptionTab::GameUi => "GAME UI",
        OptionTab::Social => "SOCIAL",
        OptionTab::Controls => "CONTROLS",
    }
}

pub const fn legacy_physical_key(key: KeyCode) -> Option<LegacyPhysicalKey> {
    match key {
        KeyCode::KeyW => Some(LegacyPhysicalKey::W),
        KeyCode::KeyS => Some(LegacyPhysicalKey::S),
        KeyCode::KeyA => Some(LegacyPhysicalKey::A),
        KeyCode::KeyD => Some(LegacyPhysicalKey::D),
        KeyCode::Backquote => Some(LegacyPhysicalKey::BackQuote),
        KeyCode::Space => Some(LegacyPhysicalKey::Space),
        KeyCode::KeyZ => Some(LegacyPhysicalKey::Z),
        KeyCode::KeyX => Some(LegacyPhysicalKey::X),
        KeyCode::Digit1 => Some(LegacyPhysicalKey::Digit1),
        KeyCode::Digit2 => Some(LegacyPhysicalKey::Digit2),
        KeyCode::Digit3 => Some(LegacyPhysicalKey::Digit3),
        KeyCode::Tab => Some(LegacyPhysicalKey::Tab),
        KeyCode::KeyR => Some(LegacyPhysicalKey::R),
        KeyCode::KeyC => Some(LegacyPhysicalKey::C),
        KeyCode::KeyI => Some(LegacyPhysicalKey::I),
        KeyCode::KeyN => Some(LegacyPhysicalKey::N),
        KeyCode::KeyJ => Some(LegacyPhysicalKey::J),
        KeyCode::KeyP => Some(LegacyPhysicalKey::P),
        KeyCode::KeyH => Some(LegacyPhysicalKey::H),
        KeyCode::Home => Some(LegacyPhysicalKey::Home),
        KeyCode::Enter => Some(LegacyPhysicalKey::Enter),
        KeyCode::KeyM => Some(LegacyPhysicalKey::M),
        KeyCode::Quote => Some(LegacyPhysicalKey::Quote),
        KeyCode::KeyQ => Some(LegacyPhysicalKey::Q),
        KeyCode::KeyE => Some(LegacyPhysicalKey::E),
        KeyCode::ControlLeft => Some(LegacyPhysicalKey::ControlLeft),
        KeyCode::KeyF => Some(LegacyPhysicalKey::F),
        KeyCode::KeyV => Some(LegacyPhysicalKey::V),
        KeyCode::ArrowUp => Some(LegacyPhysicalKey::ArrowUp),
        KeyCode::ArrowDown => Some(LegacyPhysicalKey::ArrowDown),
        KeyCode::ArrowLeft => Some(LegacyPhysicalKey::ArrowLeft),
        KeyCode::ArrowRight => Some(LegacyPhysicalKey::ArrowRight),
        KeyCode::Numpad1 => Some(LegacyPhysicalKey::Numpad1),
        KeyCode::Numpad2 => Some(LegacyPhysicalKey::Numpad2),
        KeyCode::Numpad3 => Some(LegacyPhysicalKey::Numpad3),
        KeyCode::NumpadEnter => Some(LegacyPhysicalKey::NumpadEnter),
        KeyCode::ControlRight => Some(LegacyPhysicalKey::ControlRight),
        _ => None,
    }
}

pub(super) fn bind_option_visibility(
    model: Res<OptionUiModel>,
    gate: Res<OptionUiAssetGate>,
    mut roots: Query<(&mut Visibility, &mut Pickable), With<OptionUiRoot>>,
) {
    let visible = model.visible && gate.ready && !gate.failed;
    for (mut visibility, mut pickable) in &mut roots {
        let desired_visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        let desired_pickable = if visible {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
        if *visibility != desired_visibility {
            *visibility = desired_visibility;
        }
        if *pickable != desired_pickable {
            *pickable = desired_pickable;
        }
    }
}

pub(super) fn bind_option_pages(
    model: Res<OptionUiModel>,
    mut pages: Query<(&OptionPage, &mut Node, &mut Pickable)>,
    mut language_extensions: Query<
        &mut Node,
        (With<OptionLanguageExtensionRoot>, Without<OptionPage>),
    >,
) {
    for (page, mut node, mut pickable) in &mut pages {
        let selected = model.selected_tab == page.0;
        node.display = if selected {
            Display::Flex
        } else {
            Display::None
        };
        *pickable = if selected {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
    for mut node in &mut language_extensions {
        node.display = if model.selected_tab == OptionTab::Social {
            Display::Flex
        } else {
            Display::None
        };
    }
}

pub(super) fn bind_option_radio_labels(
    model: Res<OptionUiModel>,
    mut labels: Query<(&OptionRadioLabel, &mut TextColor)>,
) {
    for (label, mut color) in &mut labels {
        let selected = match label.0 {
            OptionRadioIndicator::Graphics { toggle, value } => {
                let current = match toggle {
                    OptionGraphicsToggle::ToonOutline => model.draft_options.graphics.toon_shading,
                    OptionGraphicsToggle::Glow => model.draft_options.graphics.glow,
                    OptionGraphicsToggle::AnisotropicFiltering => {
                        model.draft_options.graphics.anisotropic_filtering
                    }
                    OptionGraphicsToggle::SoftVegetation => {
                        model.draft_options.graphics.soft_vegetation
                    }
                    OptionGraphicsToggle::ObjectFading => model.draft_options.graphics.fade,
                };
                current == value
            }
            OptionRadioIndicator::Sound { channel, value } => {
                sound_channel(&model.draft_options.sound, channel).enabled == value
            }
            OptionRadioIndicator::Display { element, value } => {
                model.display_value(element) == value
            }
            OptionRadioIndicator::InvertY(value) => model.draft_input.invert_y == value,
            OptionRadioIndicator::PadInvertY(value) => model.draft_input.pad_invert_y == value,
            OptionRadioIndicator::Social { request, value } => {
                let current = match request {
                    SocialRequestKind::Group => model.draft_options.social.allow_group_invites,
                    SocialRequestKind::Buddy => model.draft_options.social.allow_buddy_requests,
                    SocialRequestKind::Trade => model.draft_options.social.allow_trade_requests,
                };
                current == value
            }
        };
        color.0 = if selected {
            Color::WHITE
        } else {
            Color::srgb(0.116_935_484, 0.665_322_6, 0.9)
        };
    }
}

pub(super) fn bind_graphics_controls(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    mut queries: ParamSet<(
        Query<(&OptionGraphicsToggleButton, &mut ImageNode, &mut Pickable)>,
        Query<(&OptionGraphicsSliderButton, &mut Pickable)>,
        Query<(&OptionSoundToggleButton, &mut ImageNode, &mut Pickable)>,
        Query<(&OptionSoundSliderButton, &mut Pickable)>,
        Query<(&OptionGraphicsSliderThumb, &mut Node)>,
    )>,
) {
    let enabled =
        model.selected_tab == OptionTab::Graphics && model.page_body_enabled(OptionTab::Graphics);
    for (button, mut image, mut pickable) in &mut queries.p0() {
        let current = match button.toggle {
            OptionGraphicsToggle::ToonOutline => model.draft_options.graphics.toon_shading,
            OptionGraphicsToggle::Glow => model.draft_options.graphics.glow,
            OptionGraphicsToggle::AnisotropicFiltering => {
                model.draft_options.graphics.anisotropic_filtering
            }
            OptionGraphicsToggle::SoftVegetation => model.draft_options.graphics.soft_vegetation,
            OptionGraphicsToggle::ObjectFading => model.draft_options.graphics.fade,
        };
        image.image = assets.image(if current == button.value {
            OptionTextureRole::RadioChecked
        } else {
            OptionTextureRole::RadioEmpty
        });
        *pickable = pickable_for(enabled);
    }
    for (_, mut pickable) in &mut queries.p1() {
        *pickable = pickable_for(enabled);
    }
    for (button, mut image, mut pickable) in &mut queries.p2() {
        let current = sound_channel(&model.draft_options.sound, button.channel).enabled;
        image.image = assets.image(if current == button.value {
            OptionTextureRole::RadioChecked
        } else {
            OptionTextureRole::RadioEmpty
        });
        *pickable = pickable_for(enabled);
    }
    for (_, mut pickable) in &mut queries.p3() {
        *pickable = pickable_for(enabled);
    }
    for (thumb, mut node) in &mut queries.p4() {
        let (track_x, fraction) = match thumb {
            OptionGraphicsSliderThumb::Visibility => (
                230.0,
                model.draft_options.graphics.visibility.clamp(0.0, 1.0),
            ),
            OptionGraphicsSliderThumb::Particles => (
                230.0,
                f32::from(model.draft_options.graphics.particle_level.min(3)) / 3.0,
            ),
            OptionGraphicsSliderThumb::Sound(channel) => (
                723.0,
                sound_channel(&model.draft_options.sound, *channel)
                    .volume
                    .clamp(0.0, 1.0),
            ),
        };
        node.left = px(option_slider_thumb_left(track_x, fraction));
    }
}

pub(super) fn bind_option_dropdowns(
    model: Res<OptionUiModel>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    voice_language: Option<Res<VoiceLanguage>>,
    mut labels: Query<(&OptionDropdownValueLabel, &mut LocalizedText)>,
    mut queries: ParamSet<(
        Query<(&OptionDropdownPanel, &mut Node, &mut Pickable)>,
        Query<(&OptionDropdownButton, &mut Pickable, &mut ZIndex)>,
        Query<(&OptionDropdownChoice, &mut Pickable)>,
    )>,
) {
    for (panel, mut node, mut pickable) in &mut queries.p0() {
        let open = dropdown_open(&model.modal, panel.0);
        node.display = if open { Display::Flex } else { Display::None };
        *pickable = pickable_for(open);
    }
    for (label, mut localized_component) in &mut labels {
        *localized_component = dropdown_value_localized(
            &model,
            label.0,
            localization.as_deref(),
            language.as_deref(),
            voice_language.as_deref(),
        );
    }
    for (button, mut pickable, mut z_index) in &mut queries.p1() {
        let selected_page = match button.0 {
            OptionDropdownKind::Pad => model.selected_tab == OptionTab::Controls,
            OptionDropdownKind::Translation | OptionDropdownKind::Voice => {
                model.selected_tab == OptionTab::Social
            }
            _ => model.selected_tab == OptionTab::Graphics,
        };
        *pickable = pickable_for(selected_page && !model.modal.disables_all());
        z_index.set_if_neq(ZIndex(if dropdown_open(&model.modal, button.0) {
            OPTION_DROPDOWN_OPEN_BUTTON_Z_INDEX
        } else {
            OPTION_DROPDOWN_BUTTON_Z_INDEX
        }));
    }
    for (choice, mut pickable) in &mut queries.p2() {
        let kind = match choice {
            OptionDropdownChoice::Resolution { .. } => OptionDropdownKind::Resolution,
            OptionDropdownChoice::Detail(_) => OptionDropdownKind::Detail,
            OptionDropdownChoice::Shadow(_) => OptionDropdownKind::Shadow,
            OptionDropdownChoice::Texture(_) => OptionDropdownKind::Texture,
            OptionDropdownChoice::Pad(_) => OptionDropdownKind::Pad,
            OptionDropdownChoice::Translation(_) => OptionDropdownKind::Translation,
            OptionDropdownChoice::Voice(_) => OptionDropdownKind::Voice,
        };
        *pickable = pickable_for(dropdown_open(&model.modal, kind));
    }
}

pub(super) fn bind_game_ui_controls(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    mut queries: ParamSet<(
        Query<(&OptionDisplayFlagButton, &mut ImageNode, &mut Pickable)>,
        Query<(&OptionTextColorButton, &mut Pickable)>,
        Query<(&OptionTextColorSelection, &mut Visibility)>,
    )>,
) {
    let enabled =
        model.selected_tab == OptionTab::GameUi && model.page_body_enabled(OptionTab::GameUi);
    for (button, mut image, mut pickable) in &mut queries.p0() {
        image.image = assets.image(if model.display_value(button.element) == button.value {
            OptionTextureRole::RadioChecked
        } else {
            OptionTextureRole::RadioEmpty
        });
        *pickable = pickable_for(enabled);
    }
    for (_, mut pickable) in &mut queries.p1() {
        *pickable = pickable_for(enabled);
    }
    for (selection, mut visibility) in &mut queries.p2() {
        *visibility = if model.text_color_value(selection.channel) == selection.index {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

pub(super) fn bind_controls_page(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    mut queries: ParamSet<(
        Query<(&OptionInvertYButton, &mut ImageNode, &mut Pickable)>,
        Query<(&OptionSensitivityButton, &mut Pickable)>,
        Query<(&OptionMappingButton, &mut Pickable)>,
        Query<(&OptionControlsScrollButton, &mut Pickable)>,
    )>,
    mut sensitivity_thumbs: Query<
        &mut Node,
        (
            With<OptionSensitivityThumb>,
            Without<OptionControlsScrollContent>,
            Without<OptionControlsScrollThumb>,
        ),
    >,
    mut controls_thumbs: Query<
        &mut Node,
        (
            With<OptionControlsScrollThumb>,
            Without<OptionSensitivityThumb>,
            Without<OptionControlsScrollContent>,
        ),
    >,
    mut content: Query<
        &mut Node,
        (
            With<OptionControlsScrollContent>,
            Without<OptionSensitivityThumb>,
            Without<OptionControlsScrollThumb>,
        ),
    >,
    mut labels: Query<(&OptionMappingLabel, &mut LocalizedText), Without<OptionKeyCapturePrompt>>,
    mut prompts: Query<
        &mut LocalizedText,
        (With<OptionKeyCapturePrompt>, Without<OptionMappingLabel>),
    >,
) {
    let enabled =
        model.selected_tab == OptionTab::Controls && model.page_body_enabled(OptionTab::Controls);
    for (button, mut image, mut pickable) in &mut queries.p0() {
        image.image = assets.image(if model.draft_input.invert_y == button.0 {
            OptionTextureRole::RadioChecked
        } else {
            OptionTextureRole::RadioEmpty
        });
        *pickable = pickable_for(enabled);
    }
    let sensitivity = model
        .draft_input
        .camera_sensitivity
        .clamp(1.0, 10.0)
        .round() as u8;
    for (_, mut pickable) in &mut queries.p1() {
        *pickable = pickable_for(enabled);
    }
    let sensitivity_fraction = f32::from(sensitivity.saturating_sub(1)) / 9.0;
    for mut node in &mut sensitivity_thumbs {
        node.left = px(option_slider_thumb_left(680.0, sensitivity_fraction));
    }
    for (_, mut pickable) in &mut queries.p2() {
        *pickable = pickable_for(enabled);
    }
    for (button, mut pickable) in &mut queries.p3() {
        let available = match button {
            OptionControlsScrollButton::Up => model.controls_scroll > 0.0,
            OptionControlsScrollButton::Down => model.controls_scroll < OPTION_CONTROL_SCROLL_MAX,
        };
        *pickable = pickable_for(enabled && available);
    }
    let scroll_ratio = if OPTION_CONTROL_SCROLL_MAX <= 0.0 {
        0.0
    } else {
        model.controls_scroll / OPTION_CONTROL_SCROLL_MAX
    };
    for mut node in &mut controls_thumbs {
        node.top = px(245.0 + scroll_ratio.clamp(0.0, 1.0) * 176.0);
    }
    for mut node in &mut content {
        node.top = px(-model.controls_scroll);
    }
    for (label, mut localized_component) in &mut labels {
        let value = model
            .draft_input
            .mappings
            .iter()
            .find(|row| row.action == label.action)
            .map(|row| legacy_binding_label(binding_for_slot(row, label.slot)))
            .unwrap_or_default();
        *localized_component = option_passthrough_text(value);
    }
    let prompt = model.key_capture.map_or_else(
        || option_passthrough_text(""),
        |capture| {
            LocalizedText::new(
                "ui.option.controls.capture_prompt",
                "Select the key or button to map to {action} ({seconds})",
            )
            .with_arg(
                "action",
                resolve_option_source(
                    localization.as_deref(),
                    language.as_deref(),
                    capture.action.label(),
                ),
            )
            .with_arg(
                "seconds",
                format!("{:.0}", capture.remaining_seconds.ceil()),
            )
        },
    );
    for mut localized_component in &mut prompts {
        *localized_component = prompt.clone();
    }
}

pub(super) fn bind_system_popup(
    model: Res<OptionUiModel>,
    mut roots: Query<(&mut Node, &mut Pickable), With<OptionSystemPopupRoot>>,
    mut labels: Query<&mut LocalizedText, With<OptionSystemPopupLabel>>,
) {
    let visible = model.popup.is_some();
    for (mut node, mut pickable) in &mut roots {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        *pickable = pickable_for(visible);
    }
    let message = model.popup.map_or("", OptionSystemPopup::message);
    for mut localized_component in &mut labels {
        *localized_component =
            option_localized_text(message).unwrap_or_else(|| option_passthrough_text(message));
    }
}

pub(super) fn option_slider_thumb_left(track_x: f32, fraction: f32) -> f32 {
    track_x + fraction.clamp(0.0, 1.0) * (OPTION_SLIDER_TRACK_WIDTH - OPTION_SLIDER_THUMB_WIDTH)
}

pub(super) fn pickable_for(enabled: bool) -> Pickable {
    if enabled {
        Pickable::default()
    } else {
        Pickable::IGNORE
    }
}

pub(super) const fn dropdown_open(modal: &OptionModalState, kind: OptionDropdownKind) -> bool {
    match kind {
        OptionDropdownKind::Resolution => modal.resolution_dropdown,
        OptionDropdownKind::Detail => modal.detail_dropdown,
        OptionDropdownKind::Shadow => modal.shadow_dropdown,
        OptionDropdownKind::Texture => modal.texture_dropdown,
        OptionDropdownKind::Pad => modal.pad_dropdown,
        OptionDropdownKind::Translation => modal.translation_dropdown,
        OptionDropdownKind::Voice => modal.voice_dropdown,
    }
}

pub(super) fn bind_social_flags(
    model: Res<OptionUiModel>,
    assets: Res<OptionUiAssets>,
    mut flags: Query<(&OptionSocialFlagButton, &mut ImageNode, &mut Pickable)>,
) {
    let enabled =
        model.selected_tab == OptionTab::Social && model.page_body_enabled(OptionTab::Social);
    for (button, mut image, mut pickable) in &mut flags {
        let current = match button.request {
            SocialRequestKind::Group => model.draft_options.social.allow_group_invites,
            SocialRequestKind::Buddy => model.draft_options.social.allow_buddy_requests,
            SocialRequestKind::Trade => model.draft_options.social.allow_trade_requests,
        };
        image.image = assets.image(if button.value == current {
            OptionTextureRole::RadioChecked
        } else {
            OptionTextureRole::RadioEmpty
        });
        *pickable = if enabled {
            Pickable::default()
        } else {
            Pickable::IGNORE
        };
    }
}
