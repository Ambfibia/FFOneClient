//! MissionUiModel warp-away countdown and request state.

use super::super::warp_away::{WARP_AWAY_COOLDOWN_SECONDS, WARP_AWAY_DELAY_SECONDS};
use super::MissionUiModel;
use crate::gameplay_ui::{GameplayUiAction, GameplayUiOutbox};

impl MissionUiModel {
    #[must_use]
    pub fn warp_away_available(&self) -> bool {
        self.nanocom_menu_presented()
            && self.system_dialog.is_none()
            && self.warp_away_countdown_seconds.is_none()
            && !self.warp_away_request_pending
            && self.warp_away_cooldown_seconds <= 0.0
    }

    #[must_use]
    pub const fn warp_away_countdown_seconds(&self) -> Option<f32> {
        self.warp_away_countdown_seconds
    }

    #[must_use]
    pub fn warp_away_display_seconds(&self) -> Option<i32> {
        self.warp_away_countdown_seconds
            .map(|seconds| seconds.max(0.0).round() as i32)
    }

    #[must_use]
    pub const fn warp_away_cooldown_seconds(&self) -> f32 {
        self.warp_away_cooldown_seconds
    }

    #[must_use]
    pub const fn warp_away_request_pending(&self) -> bool {
        self.warp_away_request_pending
    }

    pub fn begin_warp_away(&mut self, outbox: &mut GameplayUiOutbox) -> bool {
        if !self.warp_away_available() {
            return false;
        }
        self.warp_away_countdown_seconds = Some(WARP_AWAY_DELAY_SECONDS);
        outbox.push(GameplayUiAction::WarpAwayStarted);
        true
    }

    pub fn advance_warp_away(&mut self, delta_seconds: f32, outbox: &mut GameplayUiOutbox) {
        let delta_seconds = delta_seconds.max(0.0);
        self.warp_away_cooldown_seconds =
            (self.warp_away_cooldown_seconds - delta_seconds).max(0.0);

        let Some(remaining) = self.warp_away_countdown_seconds.as_mut() else {
            return;
        };
        *remaining -= delta_seconds;
        if *remaining > 0.0 {
            return;
        }

        self.warp_away_countdown_seconds = None;
        self.warp_away_request_pending = true;
        self.warp_away_cooldown_seconds = WARP_AWAY_COOLDOWN_SECONDS;
        outbox.push(GameplayUiAction::RequestWarpAway);
    }

    /// Mirrors `CnGuiChat.ResetWarp`. The cooldown is deliberately retained;
    /// only a locally failed handoff rolls it back for a safe retry.
    pub fn cancel_warp_away(&mut self) -> bool {
        let countdown_active = self.warp_away_countdown_seconds.take().is_some();
        let request_active = std::mem::take(&mut self.warp_away_request_pending);
        countdown_active || request_active
    }

    pub fn reject_warp_away_request(&mut self) -> bool {
        let active = self.cancel_warp_away();
        if active {
            self.warp_away_cooldown_seconds = 0.0;
        }
        active
    }

    pub fn complete_warp_away(&mut self) -> bool {
        std::mem::take(&mut self.warp_away_request_pending)
    }
}
