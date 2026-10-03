use super::*;

#[derive(Clone, Debug, Resource)]
pub struct OptionUiModel {
    pub visible: bool,
    pub selected_tab: OptionTab,
    /// Clean `bApply`: true after opening or applying, false after edits.
    pub clean: bool,
    pub opening_options: OptionSettings,
    pub persisted_options: OptionSettings,
    pub draft_options: OptionSettings,
    pub persisted_input: InputSettings,
    pub draft_input: InputSettings,
    pub modal: OptionModalState,
    pub popup: Option<OptionSystemPopup>,
    pub key_capture: Option<KeyCaptureState>,
    pub controls_scroll: f32,
    pub buddy_slots: Vec<OptionBuddySlot>,
    pub selected_blocked_slot: Option<usize>,
    pub blocked_scroll_row: usize,
    pub(super) next_click_clip: u8,
}

impl Default for OptionUiModel {
    fn default() -> Self {
        let options = OptionSettings::default();
        let input = InputSettings::default();
        Self {
            visible: false,
            selected_tab: OptionTab::Graphics,
            clean: true,
            opening_options: options.clone(),
            persisted_options: options.clone(),
            draft_options: options,
            persisted_input: input.clone(),
            draft_input: input,
            modal: OptionModalState::default(),
            popup: None,
            key_capture: None,
            controls_scroll: 0.0,
            buddy_slots: vec![OptionBuddySlot::default(); OPTION_BLOCKED_SLOT_CAPACITY],
            selected_blocked_slot: None,
            blocked_scroll_row: 0,
            next_click_clip: 0,
        }
    }
}

impl OptionUiModel {
    /// Tear down only the live OptionMode session. The production owner keeps
    /// committed settings across World transitions and supplies them again on
    /// the next [`Self::open`], matching the clean `cnOption`/mode split.
    pub fn reset_runtime_session(&mut self) {
        self.visible = false;
        self.clean = true;
        self.modal = OptionModalState::default();
        self.popup = None;
        self.key_capture = None;
        self.controls_scroll = 0.0;
        self.selected_blocked_slot = None;
        self.blocked_scroll_row = 0;
    }

    pub fn open(
        &mut self,
        persisted_options: OptionSettings,
        persisted_input: InputSettings,
        route: OptionOpenAudioRoute,
        outbox: &mut OptionUiOutbox,
    ) {
        self.visible = true;
        self.selected_tab = OptionTab::Graphics;
        self.clean = true;
        self.opening_options = persisted_options.clone();
        self.persisted_options = persisted_options.clone();
        self.draft_options = persisted_options;
        self.persisted_input = persisted_input.clone();
        self.draft_input = persisted_input;
        self.modal = OptionModalState::default();
        self.popup = None;
        self.key_capture = None;
        self.controls_scroll = 0.0;
        self.selected_blocked_slot = None;
        self.blocked_scroll_row = 0;
        if route.main_game_transition {
            outbox.audio(OptionUiAudioCue::OpenScreen);
        }
        if route.inventory_transition {
            outbox.audio(OptionUiAudioCue::OpenScreen);
        }
    }

    pub fn select_tab(&mut self, tab: OptionTab) -> bool {
        if self.modal.disables_all() {
            return false;
        }
        self.selected_tab = tab;
        true
    }

    #[must_use]
    pub const fn chrome_enabled(&self) -> bool {
        !self.modal.disables_all()
    }

    #[must_use]
    pub fn page_body_enabled(&self, tab: OptionTab) -> bool {
        !self.modal.disables_all() && !(tab == OptionTab::Controls && self.key_capture.is_some())
    }

    pub fn begin_key_capture(&mut self, action: LegacyOptionAction) -> bool {
        if self.modal.disables_all() {
            return false;
        }
        self.key_capture = Some(KeyCaptureState::new(action));
        true
    }

    pub fn tick_key_capture(&mut self, delta_seconds: f32) {
        let Some(capture) = self.key_capture.as_mut() else {
            return;
        };
        if delta_seconds.is_finite() && delta_seconds > 0.0 {
            capture.remaining_seconds -= delta_seconds;
        }
        if capture.remaining_seconds <= 0.0 {
            self.key_capture = None;
        }
    }

    pub fn open_dropdown(&mut self, kind: OptionDropdownKind) -> bool {
        if self.modal.system_popup || self.modal.help || self.key_capture.is_some() {
            return false;
        }
        self.modal.resolution_dropdown = false;
        self.modal.detail_dropdown = false;
        self.modal.shadow_dropdown = false;
        self.modal.texture_dropdown = false;
        self.modal.pad_dropdown = false;
        self.modal.translation_dropdown = false;
        self.modal.voice_dropdown = false;
        match kind {
            OptionDropdownKind::Resolution => self.modal.resolution_dropdown = true,
            OptionDropdownKind::Detail => self.modal.detail_dropdown = true,
            OptionDropdownKind::Shadow => self.modal.shadow_dropdown = true,
            OptionDropdownKind::Texture => self.modal.texture_dropdown = true,
            OptionDropdownKind::Pad => self.modal.pad_dropdown = true,
            OptionDropdownKind::Translation => self.modal.translation_dropdown = true,
            OptionDropdownKind::Voice => self.modal.voice_dropdown = true,
        }
        true
    }

    pub fn close_dropdowns(&mut self) {
        self.modal.resolution_dropdown = false;
        self.modal.detail_dropdown = false;
        self.modal.shadow_dropdown = false;
        self.modal.texture_dropdown = false;
        self.modal.pad_dropdown = false;
        self.modal.translation_dropdown = false;
        self.modal.voice_dropdown = false;
    }

    pub fn set_resolution(&mut self, width: u32, height: u32, windowed: bool) -> bool {
        if !self.modal.resolution_dropdown || width == 0 || height == 0 {
            return false;
        }
        let graphics = &mut self.draft_options.graphics;
        let changed =
            graphics.width != width || graphics.height != height || graphics.windowed != windowed;
        graphics.width = width;
        graphics.height = height;
        graphics.windowed = windowed;
        self.close_dropdowns();
        if changed {
            self.clean = false;
        }
        changed
    }

    pub fn set_graphics_detail(&mut self, detail: GraphicsDetail) -> bool {
        if !self.modal.detail_dropdown || detail == GraphicsDetail::Custom {
            return false;
        }
        let changed = self.draft_options.graphics.detail != detail;
        self.draft_options.graphics.apply_preset(detail);
        self.close_dropdowns();
        if changed {
            self.clean = false;
        }
        changed
    }

    pub fn set_shadow_quality(&mut self, shadow: ShadowQuality) -> bool {
        if !self.modal.shadow_dropdown {
            return false;
        }
        let changed = self.draft_options.graphics.shadow != shadow;
        self.draft_options.graphics.shadow = shadow;
        self.draft_options.graphics.detail = GraphicsDetail::Custom;
        self.close_dropdowns();
        if changed {
            self.clean = false;
        }
        changed
    }

    pub fn set_texture_quality(&mut self, texture: TextureQuality) -> bool {
        if !self.modal.texture_dropdown {
            return false;
        }
        let changed = self.draft_options.graphics.texture != texture;
        self.draft_options.graphics.texture = texture;
        self.draft_options.graphics.detail = GraphicsDetail::Custom;
        self.close_dropdowns();
        if changed {
            self.clean = false;
        }
        changed
    }

    pub fn set_graphics_toggle(&mut self, toggle: OptionGraphicsToggle, value: bool) -> bool {
        if !self.page_body_enabled(OptionTab::Graphics) {
            return false;
        }
        let graphics = &mut self.draft_options.graphics;
        let target = match toggle {
            OptionGraphicsToggle::ToonOutline => &mut graphics.toon_shading,
            OptionGraphicsToggle::Glow => &mut graphics.glow,
            OptionGraphicsToggle::AnisotropicFiltering => &mut graphics.anisotropic_filtering,
            OptionGraphicsToggle::SoftVegetation => &mut graphics.soft_vegetation,
            OptionGraphicsToggle::ObjectFading => &mut graphics.fade,
        };
        if *target == value {
            return false;
        }
        *target = value;
        graphics.detail = GraphicsDetail::Custom;
        self.clean = false;
        true
    }

    pub fn set_visibility_step(&mut self, step: u8) -> bool {
        if !self.page_body_enabled(OptionTab::Graphics) {
            return false;
        }
        let value = f32::from(step.min(10)) / 10.0;
        if (self.draft_options.graphics.visibility - value).abs() <= f32::EPSILON {
            return false;
        }
        self.draft_options.graphics.visibility = value;
        self.draft_options.graphics.detail = GraphicsDetail::Custom;
        self.clean = false;
        true
    }

    pub fn set_particle_level(&mut self, value: u8) -> bool {
        if !self.page_body_enabled(OptionTab::Graphics) {
            return false;
        }
        let value = value.min(3);
        if self.draft_options.graphics.particle_level == value {
            return false;
        }
        self.draft_options.graphics.particle_level = value;
        self.draft_options.graphics.detail = GraphicsDetail::Custom;
        self.clean = false;
        true
    }

    pub fn restore_graphics_defaults(&mut self) -> bool {
        if !self.page_body_enabled(OptionTab::Graphics) {
            return false;
        }
        let defaults = GraphicsSettings::default();
        if self.draft_options.graphics == defaults {
            return false;
        }
        self.draft_options.graphics = defaults;
        self.clean = false;
        true
    }

    pub fn set_sound_enabled(
        &mut self,
        channel: OptionSoundChannel,
        enabled: bool,
        outbox: &mut OptionUiOutbox,
    ) -> bool {
        if !self.page_body_enabled(OptionTab::Graphics) {
            return false;
        }
        let target = sound_channel_mut(&mut self.draft_options.sound, channel);
        if target.enabled == enabled {
            return false;
        }
        target.enabled = enabled;
        self.commit_immediate_sound(outbox);
        true
    }

    pub fn set_sound_volume_step(
        &mut self,
        channel: OptionSoundChannel,
        step: u8,
        outbox: &mut OptionUiOutbox,
    ) -> bool {
        if !self.page_body_enabled(OptionTab::Graphics) {
            return false;
        }
        let value = f32::from(step.min(10)) / 10.0;
        let target = sound_channel_mut(&mut self.draft_options.sound, channel);
        if (target.volume - value).abs() <= f32::EPSILON {
            return false;
        }
        target.volume = value;
        self.commit_immediate_sound(outbox);
        true
    }

    pub fn restore_sound_defaults(&mut self, outbox: &mut OptionUiOutbox) -> bool {
        if !self.page_body_enabled(OptionTab::Graphics) {
            return false;
        }
        let defaults = SoundSettings::default();
        if self.draft_options.sound == defaults {
            return false;
        }
        self.draft_options.sound = defaults;
        self.commit_immediate_sound(outbox);
        true
    }

    pub(super) fn commit_immediate_sound(&mut self, outbox: &mut OptionUiOutbox) {
        self.persisted_options.sound = self.draft_options.sound.clone();
        self.clean = false;
        outbox.action(OptionUiAction::ApplySoundImmediately(
            self.draft_options.sound.clone(),
        ));
    }

    #[must_use]
    pub fn display_value(&self, element: OptionDisplayElement) -> bool {
        let display = &self.draft_options.display;
        match element {
            OptionDisplayElement::MyName => display.my_name,
            OptionDisplayElement::OtherPlayerNames => display.other_name,
            OptionDisplayElement::GroupMemberNames => display.group_name,
            OptionDisplayElement::NpcNames => display.npc_name,
            OptionDisplayElement::MonsterNames => display.monster_name,
            OptionDisplayElement::DamageNumbers => display.floating_numbers,
            OptionDisplayElement::BalloonChat => display.balloon,
            OptionDisplayElement::ScaleUiElements => display.scale_ui,
            OptionDisplayElement::OldChat => !display.new_chat,
            OptionDisplayElement::ComputressHints => display.game_hint,
            OptionDisplayElement::NpcMessagesInChat => display.npc_messages_in_chat,
            OptionDisplayElement::CombatInChat => display.combat_in_chat,
            OptionDisplayElement::AnimatedNanocom => display.animated_nanos,
        }
    }

    pub fn set_display_value(&mut self, element: OptionDisplayElement, value: bool) -> bool {
        if !self.page_body_enabled(OptionTab::GameUi) {
            return false;
        }
        if self.display_value(element) == value {
            return false;
        }
        let display = &mut self.draft_options.display;
        match element {
            OptionDisplayElement::MyName => display.my_name = value,
            OptionDisplayElement::OtherPlayerNames => display.other_name = value,
            OptionDisplayElement::GroupMemberNames => display.group_name = value,
            OptionDisplayElement::NpcNames => display.npc_name = value,
            OptionDisplayElement::MonsterNames => display.monster_name = value,
            OptionDisplayElement::DamageNumbers => display.floating_numbers = value,
            OptionDisplayElement::BalloonChat => display.balloon = value,
            OptionDisplayElement::ScaleUiElements => display.scale_ui = value,
            OptionDisplayElement::OldChat => display.new_chat = !value,
            OptionDisplayElement::ComputressHints => display.game_hint = value,
            OptionDisplayElement::NpcMessagesInChat => display.npc_messages_in_chat = value,
            OptionDisplayElement::CombatInChat => display.combat_in_chat = value,
            OptionDisplayElement::AnimatedNanocom => display.animated_nanos = value,
        }
        self.clean = false;
        if element == OptionDisplayElement::OldChat {
            self.popup = Some(OptionSystemPopup::OldChatRestartRequired);
            self.modal.system_popup = true;
        }
        true
    }

    pub fn restore_display_defaults(&mut self) -> bool {
        if !self.page_body_enabled(OptionTab::GameUi) {
            return false;
        }
        let defaults = DisplaySettings::default();
        if self.draft_options.display == defaults {
            return false;
        }
        self.draft_options.display = defaults;
        self.clean = false;
        true
    }

    #[must_use]
    pub fn text_color_value(&self, channel: OptionTextColorChannel) -> u8 {
        match channel {
            OptionTextColorChannel::General => self.draft_options.text_colors.general,
            OptionTextColorChannel::Group => self.draft_options.text_colors.group,
            OptionTextColorChannel::Buddy => self.draft_options.text_colors.buddy,
        }
    }

    pub fn set_text_color(&mut self, channel: OptionTextColorChannel, index: u8) -> bool {
        if !self.page_body_enabled(OptionTab::GameUi)
            || usize::from(index) >= OPTION_CHAT_PALETTE_RGB.len()
        {
            return false;
        }
        let target = match channel {
            OptionTextColorChannel::General => &mut self.draft_options.text_colors.general,
            OptionTextColorChannel::Group => &mut self.draft_options.text_colors.group,
            OptionTextColorChannel::Buddy => &mut self.draft_options.text_colors.buddy,
        };
        if *target == index {
            return false;
        }
        *target = index;
        self.clean = false;
        true
    }

    pub fn restore_text_color_defaults(&mut self) -> bool {
        if !self.page_body_enabled(OptionTab::GameUi) {
            return false;
        }
        let defaults = TextColorSettings::default();
        if self.draft_options.text_colors == defaults {
            return false;
        }
        self.draft_options.text_colors = defaults;
        self.clean = false;
        true
    }

    pub fn acknowledge_popup(&mut self) -> bool {
        if self.popup.is_none() {
            return false;
        }
        self.popup = None;
        self.modal.system_popup = false;
        true
    }

    pub fn restore_input_controls(&mut self) -> bool {
        if !self.page_body_enabled(OptionTab::Controls) {
            return false;
        }
        if !self.draft_input.invert_y
            && !self.draft_input.pad_invert_y
            && (self.draft_input.pad_camera_sensitivity - 5.0).abs() <= f32::EPSILON
            && (self.draft_input.camera_sensitivity - 5.0).abs() <= f32::EPSILON
        {
            return false;
        }
        self.draft_input.invert_y = false;
        self.draft_input.pad_invert_y = false;
        self.draft_input.pad_camera_sensitivity = 5.0;
        self.draft_input.camera_sensitivity = 5.0;
        self.clean = false;
        true
    }

    pub fn set_invert_y(&mut self, value: bool) -> bool {
        if !self.page_body_enabled(OptionTab::Controls) || self.draft_input.invert_y == value {
            return false;
        }
        self.draft_input.invert_y = value;
        self.clean = false;
        true
    }

    pub fn set_camera_sensitivity(&mut self, value: u8) -> bool {
        if !self.page_body_enabled(OptionTab::Controls) {
            return false;
        }
        let value = f32::from(value.clamp(1, 10));
        if (self.draft_input.camera_sensitivity - value).abs() <= f32::EPSILON {
            return false;
        }
        self.draft_input.camera_sensitivity = value;
        self.clean = false;
        true
    }

    pub fn begin_binding_capture(
        &mut self,
        action: LegacyOptionAction,
        slot: OptionInputMappingSlot,
    ) -> bool {
        if self.modal.disables_all() || self.selected_tab != OptionTab::Controls {
            return false;
        }
        self.key_capture = Some(KeyCaptureState::with_slot(action, slot));
        true
    }

    pub fn submit_captured_binding(&mut self, binding: LegacyInputBinding) -> bool {
        let Some(capture) = self.key_capture.take() else {
            return false;
        };
        let duplicate = binding != LegacyInputBinding::Unbound
            && self.draft_input.mappings.iter().any(|row| {
                row.action != capture.action
                    && (row.primary == binding || row.alternate == binding)
            });
        if duplicate {
            self.popup = Some(OptionSystemPopup::DuplicateInputBinding);
            self.modal.system_popup = true;
            return false;
        }
        let Some(row) = self
            .draft_input
            .mappings
            .iter_mut()
            .find(|row| row.action == capture.action)
        else {
            return false;
        };
        let target = binding_for_slot_mut(row, capture.slot);
        if *target == binding {
            return false;
        }
        *target = binding;
        self.clean = false;
        true
    }

    pub fn restore_key_mapping_defaults(&mut self) -> bool {
        if !self.page_body_enabled(OptionTab::Controls) {
            return false;
        }
        let defaults = InputSettings::default();
        let changed = self.draft_input.pad_profile != defaults.pad_profile
            || self.draft_input.mappings != defaults.mappings;
        if !changed {
            return false;
        }
        self.draft_input.pad_profile = defaults.pad_profile;
        self.draft_input.mappings = defaults.mappings;
        self.clean = false;
        true
    }

    pub fn set_pad_profile(&mut self, profile: LegacyPadProfile) -> bool {
        if !self.modal.pad_dropdown {
            return false;
        }
        let previous_profile = self.draft_input.pad_profile;
        let previous_pad = self
            .draft_input
            .mappings
            .iter()
            .map(|row| row.pad)
            .collect::<Vec<_>>();
        self.draft_input.pad_profile = profile;
        reset_pad_profile_bindings(&mut self.draft_input.mappings, profile);
        let changed = previous_profile != profile
            || self
                .draft_input
                .mappings
                .iter()
                .map(|row| row.pad)
                .ne(previous_pad);
        self.close_dropdowns();
        if changed {
            self.clean = false;
        }
        changed
    }

    pub fn scroll_controls(&mut self, delta: f32) -> bool {
        if !self.page_body_enabled(OptionTab::Controls) || !delta.is_finite() {
            return false;
        }
        let next = (self.controls_scroll + delta).clamp(0.0, OPTION_CONTROL_SCROLL_MAX);
        if (next - self.controls_scroll).abs() <= f32::EPSILON {
            return false;
        }
        self.controls_scroll = next;
        true
    }

    pub fn set_social_request(&mut self, request: SocialRequestKind, enabled: bool) -> bool {
        if !self.page_body_enabled(OptionTab::Social) {
            return false;
        }
        let target = match request {
            SocialRequestKind::Group => &mut self.draft_options.social.allow_group_invites,
            SocialRequestKind::Buddy => &mut self.draft_options.social.allow_buddy_requests,
            SocialRequestKind::Trade => &mut self.draft_options.social.allow_trade_requests,
        };
        if *target == enabled {
            return false;
        }
        *target = enabled;
        self.clean = false;
        true
    }

    pub fn restore_social_defaults(&mut self) -> bool {
        if !self.page_body_enabled(OptionTab::Social) {
            return false;
        }
        let defaults = SocialRequestSettings::default();
        if self.draft_options.social == defaults {
            return false;
        }
        self.draft_options.social = defaults;
        self.clean = false;
        true
    }

    pub fn mark_options_edited(&mut self) {
        self.clean = false;
    }

    pub fn mark_input_edited(&mut self) {
        self.clean = false;
    }

    #[must_use]
    pub fn blocked_projection(&self) -> Vec<BlockedPlayerRow> {
        project_blocked_players(&self.buddy_slots)
    }

    pub fn select_blocked_slot(&mut self, slot: usize) -> bool {
        if self.blocked_projection().iter().any(|row| row.slot == slot) {
            self.selected_blocked_slot = Some(slot);
            true
        } else {
            false
        }
    }

    pub fn scroll_blocked_rows(&mut self, delta_rows: isize) -> bool {
        if !self.page_body_enabled(OptionTab::Social) {
            return false;
        }
        let maximum = self
            .blocked_projection()
            .len()
            .saturating_sub(OPTION_BLOCKED_VISIBLE_ROWS);
        let next = self
            .blocked_scroll_row
            .saturating_add_signed(delta_rows)
            .min(maximum);
        if next == self.blocked_scroll_row {
            return false;
        }
        self.blocked_scroll_row = next;
        true
    }

    pub fn remove_selected_buddy(&mut self, outbox: &mut OptionUiOutbox) -> bool {
        if !self.page_body_enabled(OptionTab::Social) {
            return false;
        }
        let Some(slot) = self.selected_blocked_slot else {
            return false;
        };
        let Some(buddy) = self.buddy_slots.get(slot) else {
            self.selected_blocked_slot = None;
            return false;
        };
        if buddy.pc_uid == 0 || !buddy.blocked {
            self.selected_blocked_slot = None;
            return false;
        }
        outbox.action(OptionUiAction::RemoveBuddy {
            slot,
            pc_uid: buddy.pc_uid,
        });
        true
    }

    pub fn apply(&mut self, outbox: &mut OptionUiOutbox) -> bool {
        if !self.chrome_enabled() || self.clean {
            return false;
        }
        self.persist_current(outbox);
        self.clean = true;
        // Intentionally do not refresh `opening_options`.
        outbox.audio(OptionUiAudioCue::ActionSuccess);
        self.emit_button_click(outbox);
        true
    }

    pub fn save_and_exit(
        &mut self,
        route: OptionCloseAudioRoute,
        outbox: &mut OptionUiOutbox,
    ) -> bool {
        if !self.chrome_enabled() {
            return false;
        }
        self.persist_current(outbox);
        self.clean = true;
        self.exit_with_audio(route, outbox);
        outbox.audio(OptionUiAudioCue::ActionSuccess);
        self.emit_button_click(outbox);
        true
    }

    pub fn cancel(
        &mut self,
        trigger: OptionCloseTrigger,
        route: OptionCloseAudioRoute,
        outbox: &mut OptionUiOutbox,
    ) -> bool {
        if !self.chrome_enabled() {
            return false;
        }
        if self.clean {
            self.persist_current(outbox);
        } else {
            self.draft_options = self.opening_options.clone();
            self.persisted_options = self.opening_options.clone();
            self.draft_input = self.persisted_input.clone();
            outbox.action(OptionUiAction::PersistOptions(self.opening_options.clone()));
        }
        self.exit_with_audio(route, outbox);
        if self.clean {
            outbox.audio(OptionUiAudioCue::ActionSuccess);
        }
        if trigger == OptionCloseTrigger::CloseButton {
            self.emit_button_click(outbox);
        }
        true
    }

    pub(super) fn persist_current(&mut self, outbox: &mut OptionUiOutbox) {
        self.persisted_input = self.draft_input.clone();
        self.persisted_options = self.draft_options.clone();
        outbox.action(OptionUiAction::PersistInput(self.persisted_input.clone()));
        outbox.action(OptionUiAction::PersistOptions(
            self.persisted_options.clone(),
        ));
    }

    pub(super) fn exit_with_audio(&mut self, route: OptionCloseAudioRoute, outbox: &mut OptionUiOutbox) {
        self.visible = false;
        if route.main_game_transition {
            outbox.audio(OptionUiAudioCue::CloseScreen);
        }
        if route.inventory_transition {
            outbox.audio(OptionUiAudioCue::CloseScreen);
        }
        outbox.action(OptionUiAction::Closed);
    }

    pub(super) fn emit_button_click(&mut self, outbox: &mut OptionUiOutbox) {
        outbox.audio(OptionUiAudioCue::ButtonClick {
            clip_index: self.next_click_clip,
            gain: 1.0,
        });
        self.next_click_clip = (self.next_click_clip + 1) % 5;
    }
}
