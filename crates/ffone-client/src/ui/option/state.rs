use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OptionModalState {
    pub system_popup: bool,
    pub resolution_dropdown: bool,
    pub detail_dropdown: bool,
    pub texture_dropdown: bool,
    pub shadow_dropdown: bool,
    pub help: bool,
    pub pad_dropdown: bool,
    pub translation_dropdown: bool,
    pub voice_dropdown: bool,
}

impl OptionModalState {
    #[must_use]
    pub const fn disables_all(self) -> bool {
        self.system_popup
            || self.resolution_dropdown
            || self.detail_dropdown
            || self.texture_dropdown
            || self.shadow_dropdown
            || self.help
            || self.pad_dropdown
            || self.translation_dropdown
            || self.voice_dropdown
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyCaptureState {
    pub action: LegacyOptionAction,
    pub slot: OptionInputMappingSlot,
    pub remaining_seconds: f32,
}

impl KeyCaptureState {
    #[must_use]
    pub const fn new(action: LegacyOptionAction) -> Self {
        Self {
            action,
            slot: OptionInputMappingSlot::Primary,
            remaining_seconds: OPTION_KEY_CAPTURE_TIMEOUT_SECONDS,
        }
    }

    #[must_use]
    pub const fn with_slot(action: LegacyOptionAction, slot: OptionInputMappingSlot) -> Self {
        Self {
            action,
            slot,
            remaining_seconds: OPTION_KEY_CAPTURE_TIMEOUT_SECONDS,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionTextColorSelection {
    pub(super) channel: OptionTextColorChannel,
    pub(super) index: u8,
}

pub(super) fn sliced_image_mode(border: BorderRect) -> NodeImageMode {
    NodeImageMode::Sliced(TextureSlicer {
        border,
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    })
}

pub(super) fn option_tab_image_mode(tab: OptionTab, selected: bool) -> NodeImageMode {
    if tab == OptionTab::Graphics && !selected {
        sliced_image_mode(OPTION_GRAPHICS_TAB_BORDER)
    } else {
        NodeImageMode::Stretch
    }
}

pub(super) const fn tab_selected_role(tab: OptionTab) -> OptionTextureRole {
    match tab {
        OptionTab::Graphics => OptionTextureRole::GraphicsSelected,
        OptionTab::GameUi => OptionTextureRole::GameUiSelected,
        OptionTab::Social => OptionTextureRole::SocialSelected,
        OptionTab::Controls => OptionTextureRole::ControlsSelected,
    }
}
