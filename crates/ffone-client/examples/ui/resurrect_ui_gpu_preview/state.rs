use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub(super) enum PreviewMode {
    PhoenixSelf,
    GroupItem,
}

impl PreviewMode {
    pub(super) const fn cli_name(self) -> &'static str {
        match self {
            Self::PhoenixSelf => "self",
            Self::GroupItem => "group-item",
        }
    }

    pub(super) fn default_output(self, language: &str) -> PathBuf {
        PathBuf::from(format!(
            "target/ui-parity/resurrect-{}-{language}-1264x681.png",
            self.cli_name()
        ))
    }

    pub(super) const fn context(self) -> ResurrectUiContext {
        ResurrectUiContext {
            // This freezes the clean start-time rewrite at exactly 60 seconds
            // while leaving the independently owned mouse controls enabled.
            ready_for_play: false,
            player_available: true,
            system_popup_active: false,
            skill_icon_back_available: true,
            phoenix_group_icon_available: true,
            phoenix_self_icon_available: true,
            phoenix_group_skill: matches!(self, Self::GroupItem),
            // Supplying this in GroupItem proves clean Group-before-Self
            // priority instead of rendering an impossible simultaneous pair.
            phoenix_self_skill_bit_16: true,
            resurrection_item_slot: match self {
                Self::PhoenixSelf => None,
                Self::GroupItem => Some(12),
            },
            nearest_xcom_index: Some(17),
        }
    }

    pub(super) const fn hover_z_index(self) -> i32 {
        match self {
            Self::PhoenixSelf => 0,
            Self::GroupItem => 2,
        }
    }
}

#[derive(Resource)]
pub(super) struct PreviewState {
    pub(super) frames: u32,
    pub(super) ready_frame: Option<u32>,
    pub(super) ready_at: Option<Instant>,
    pub(super) capture_issued: bool,
    pub(super) capture_saved: bool,
    pub(super) capture_failed: bool,
    pub(super) text_audited: bool,
    pub(super) started_at: Instant,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            frames: 0,
            ready_frame: None,
            ready_at: None,
            capture_issued: false,
            capture_saved: false,
            capture_failed: false,
            text_audited: false,
            started_at: Instant::now(),
        }
    }
}
