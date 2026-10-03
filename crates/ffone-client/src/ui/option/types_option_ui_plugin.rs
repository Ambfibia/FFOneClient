use super::*;

impl Plugin for OptionUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<OptionUiModel>()
            .init_resource::<OptionUiOutbox>()
            .init_resource::<OptionUiAudioRouting>()
            .init_resource::<OptionUiAssetGate>()
            .configure_sets(
                Update,
                (
                    OptionUiSet::AssetGate,
                    OptionUiSet::Interaction,
                    OptionUiSet::Bind,
                    OptionUiSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_option_ui)
            .add_systems(
                Update,
                (update_option_asset_gate.in_set(OptionUiSet::AssetGate))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    tick_option_key_capture,
                    capture_option_binding_input,
                    handle_option_interactions,
                    handle_graphics_interactions,
                    handle_game_ui_interactions,
                    handle_controls_interactions,
                    handle_pad_controls,
                    handle_option_scroll_wheel,
                    handle_dropdown_interactions,
                    handle_system_popup_interactions,
                )
                    .chain()
                    .in_set(OptionUiSet::Interaction)
                    .run_if(option_ui_is_visible))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (bind_option_visibility
                    .in_set(OptionUiSet::Bind)
                    .before(LocalizationSet::Apply)
                    .run_if(option_ui_visibility_needs_update))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    update_option_layout,
                    bind_option_pages,
                    bind_option_radio_labels,
                    bind_graphics_controls,
                    bind_option_dropdowns,
                    bind_game_ui_controls,
                    bind_controls_page,
                    bind_pad_controls,
                    bind_system_popup,
                    bind_social_flags,
                    bind_blocked_rows,
                )
                    .in_set(OptionUiSet::Bind)
                    .before(LocalizationSet::Apply)
                    .run_if(option_ui_is_visible)
                    .run_if(option_ui_bindings_need_update))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    update_tab_visuals,
                    update_chrome_button_visuals,
                    update_page_button_visuals,
                    update_option_button_labels,
                    update_key_button_labels,
                    update_dropdown_visuals,
                    update_social_button_visuals,
                )
                    .in_set(OptionUiSet::Visuals)
                    .run_if(option_ui_is_visible)
                    .run_if(option_ui_visuals_need_update))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum OptionTextAnchor {
    UpperLeft,
    UpperRight,
    MiddleLeft,
    MiddleRight,
}

impl OptionTextAnchor {
    pub(super) const fn align_items(self) -> AlignItems {
        match self {
            Self::UpperLeft | Self::UpperRight => AlignItems::FlexStart,
            Self::MiddleLeft | Self::MiddleRight => AlignItems::Center,
        }
    }

    pub(super) const fn justify_content(self) -> JustifyContent {
        match self {
            Self::UpperLeft | Self::MiddleLeft => JustifyContent::Start,
            Self::UpperRight | Self::MiddleRight => JustifyContent::End,
        }
    }

    pub(super) const fn justify(self) -> Justify {
        match self {
            Self::UpperLeft | Self::MiddleLeft => Justify::Left,
            Self::UpperRight | Self::MiddleRight => Justify::Right,
        }
    }
}
