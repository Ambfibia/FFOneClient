//! Source-honest runtime correlation for clean-Retrobution `cnGuideMode`.
//!
//! This module keeps the protocol's mentor and mentor-count values lossless.
//! In particular, it does not normalize OpenFusion's constant success count
//! into invented authoritative state. The server profile only interprets an
//! accepted, correlated reply into the clean client's first-change or
//! later-change follow-up intent.

use bevy::prelude::Resource;
use ffone_protocol::{
    PcChangeMentorFailure0104, PcChangeMentorRequest0104, PcChangeMentorSuccess0104, PcLoadData0104,
};

use crate::guide_ui::GuideMentor;

pub const GUIDE_COMPUTRESS_FUTURE_MENTOR_ID: i16 = 5;
pub const GUIDE_FIRST_CHANGE_WARP_NPC_TABLE_ID: i32 = 1_425;

/// Reply-semantics profile selected explicitly by the owning connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideServerProfile {
    /// Clean protocol-0104: reply count `1` is the first mentor change.
    Clean0104,
    /// OpenFusion protocol-0104: the server currently replies with count `1`
    /// for both first and later changes, so the pre-request mentor is used.
    OpenFusion0104,
}

/// A recognized raw mentor value. Only mentor IDs `1..=4` are selectable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideRawMentor {
    Selectable(GuideMentor),
    /// Clean Computress/future ownership sentinel (`5`).
    ComputressFuture,
}

impl GuideRawMentor {
    #[must_use]
    pub const fn from_raw(raw: i16) -> Option<Self> {
        match GuideMentor::from_wire_id(raw) {
            Some(mentor) => Some(Self::Selectable(mentor)),
            None if raw == GUIDE_COMPUTRESS_FUTURE_MENTOR_ID => Some(Self::ComputressFuture),
            None => None,
        }
    }

    #[must_use]
    pub const fn raw(self) -> i16 {
        match self {
            Self::Selectable(mentor) => mentor.wire_id(),
            Self::ComputressFuture => GUIDE_COMPUTRESS_FUTURE_MENTOR_ID,
        }
    }
}

/// Lossless authoritative mentor fields read from `sPCLoadData2CL` or an
/// accepted change-mentor success reply.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuideAuthoritativeState {
    raw_mentor: i16,
    raw_mentor_count: i16,
}

impl GuideAuthoritativeState {
    #[must_use]
    pub fn from_pc_load(load: &PcLoadData0104) -> Self {
        Self {
            raw_mentor: load.mentor(),
            raw_mentor_count: load.mentor_count(),
        }
    }

    #[must_use]
    pub const fn raw_mentor(self) -> i16 {
        self.raw_mentor
    }

    #[must_use]
    pub const fn raw_mentor_count(self) -> i16 {
        self.raw_mentor_count
    }

    #[must_use]
    pub const fn recognized_mentor(self) -> Option<GuideRawMentor> {
        GuideRawMentor::from_raw(self.raw_mentor)
    }
}

/// Exact `NpcIconMode.StartGuideChage` routing result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideNpcServiceRoute {
    GuideChanger {
        passed_mentor: GuideRawMentor,
    },
    PastWarp,
    /// Payment flag `0` opens the Upsell mode and must not open Guide mode.
    UpsellRequired,
}

/// Map the NPC TableData service category and login payment flag.
///
/// Categories `18..=22` pass mentor IDs `1..=5`. Category `23` opens the
/// Past Warp path only for the confirmed paid flag `1`; unsupported flags and
/// unrelated categories fail closed as `None`.
#[must_use]
pub const fn guide_npc_service_route(
    service_category: i32,
    payment_flag: i8,
) -> Option<GuideNpcServiceRoute> {
    if service_category >= 18 && service_category <= 22 {
        let passed_raw = (service_category - 18 + 1) as i16;
        return match GuideRawMentor::from_raw(passed_raw) {
            Some(passed_mentor) => Some(GuideNpcServiceRoute::GuideChanger { passed_mentor }),
            None => None,
        };
    }
    if service_category != 23 {
        return None;
    }
    match payment_flag {
        0 => Some(GuideNpcServiceRoute::UpsellRequired),
        1 => Some(GuideNpcServiceRoute::PastWarp),
        _ => None,
    }
}

/// Source-backed initialization decision before a `GuideUiModel` is opened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideInitIntent {
    /// A recognized owned mentor overrides the mentor passed by the NPC.
    ChangeExisting { current: GuideMentor },
    /// Owned sentinel `5` plus passed value `0` starts at the warp warning.
    InitialWarpWarning,
    /// Owned sentinel `5` plus a recognized nonzero passed value starts at
    /// mentor selection. The passed value is context, not a fabricated
    /// selected mentor; sentinel `5` therefore remains representable.
    InitialSelection { passed_mentor: GuideRawMentor },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideInitError {
    PcStateUnavailable,
    UnsupportedOwnedMentor { raw_mentor: i16 },
    UnsupportedPassedMentor { raw_mentor: i16 },
}

/// Reproduce `cnGuideMode.InitMode` without accepting unknown mentor values.
pub fn guide_init_intent(
    owned_raw_mentor: i16,
    passed_raw_mentor: i16,
) -> Result<GuideInitIntent, GuideInitError> {
    match GuideRawMentor::from_raw(owned_raw_mentor) {
        Some(GuideRawMentor::Selectable(current)) => {
            Ok(GuideInitIntent::ChangeExisting { current })
        }
        Some(GuideRawMentor::ComputressFuture) => {
            if passed_raw_mentor == 0 {
                return Ok(GuideInitIntent::InitialWarpWarning);
            }
            let passed_mentor = GuideRawMentor::from_raw(passed_raw_mentor).ok_or(
                GuideInitError::UnsupportedPassedMentor {
                    raw_mentor: passed_raw_mentor,
                },
            )?;
            Ok(GuideInitIntent::InitialSelection { passed_mentor })
        }
        None => Err(GuideInitError::UnsupportedOwnedMentor {
            raw_mentor: owned_raw_mentor,
        }),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuidePendingChange {
    pub mentor: GuideMentor,
    pub previous_raw_mentor: i16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideChangeKind {
    First,
    Later,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuidePostChangeIntent {
    WarpToNpc {
        npc_table_id: i32,
    },
    /// The shell owns the mission store and performs the clean event-12/30
    /// equivalent. This module intentionally does not mutate mission state.
    RefreshGuideMissions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuideChangeApplied {
    pub mentor: GuideMentor,
    pub raw_mentor_count: i16,
    pub fusion_matter: i32,
    pub intent: GuidePostChangeIntent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuideChangeFailed {
    pub mentor: GuideMentor,
    pub error_code: i32,
}

/// The raw accepted success reply retained separately for diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuideReplyDiagnostics {
    pub profile: GuideServerProfile,
    pub requested_mentor: GuideMentor,
    pub previous_raw_mentor: i16,
    pub reply_raw_mentor: i16,
    pub raw_reply_mentor_count: i16,
    pub interpreted_as: GuideChangeKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuideCorrelationError {
    PcStateUnavailable,
    RequestAlreadyPending,
    UnsupportedOwnedMentor {
        raw_mentor: i16,
    },
    AlreadyCurrentMentor {
        mentor: GuideMentor,
    },
    ReplyWithoutPendingRequest,
    UnsupportedReplyMentor {
        raw_mentor: i16,
    },
    ReplyMentorMismatch {
        expected: GuideMentor,
        actual: GuideMentor,
    },
    PreviousMentorChanged {
        expected_raw_mentor: i16,
        actual_raw_mentor: i16,
    },
}

/// Compact production state for PcLoad ownership, request correlation and
/// profile-specific success interpretation.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct GuideRuntime {
    profile: GuideServerProfile,
    authoritative: Option<GuideAuthoritativeState>,
    pending: Option<GuidePendingChange>,
    last_success_diagnostics: Option<GuideReplyDiagnostics>,
}

impl GuideRuntime {
    #[must_use]
    pub const fn new(profile: GuideServerProfile) -> Self {
        Self {
            profile,
            authoritative: None,
            pending: None,
            last_success_diagnostics: None,
        }
    }

    /// Start a fresh authoritative session from the lossless PcLoad fields.
    pub fn load_pc_state(&mut self, load: &PcLoadData0104) {
        self.authoritative = Some(GuideAuthoritativeState::from_pc_load(load));
        self.pending = None;
        self.last_success_diagnostics = None;
    }

    pub fn clear_pc_state(&mut self) {
        self.authoritative = None;
        self.pending = None;
        self.last_success_diagnostics = None;
    }

    #[must_use]
    pub const fn profile(&self) -> GuideServerProfile {
        self.profile
    }

    #[must_use]
    pub const fn authoritative(&self) -> Option<GuideAuthoritativeState> {
        self.authoritative
    }

    #[must_use]
    pub const fn pending(&self) -> Option<GuidePendingChange> {
        self.pending
    }

    #[must_use]
    pub const fn last_success_diagnostics(&self) -> Option<GuideReplyDiagnostics> {
        self.last_success_diagnostics
    }

    /// Cancel only the locally pending request after the transport itself
    /// becomes unavailable. Authoritative mentor/count state remains intact.
    pub fn cancel_pending_change(&mut self) {
        self.pending = None;
    }

    pub fn init_intent(&self, passed_raw_mentor: i16) -> Result<GuideInitIntent, GuideInitError> {
        let owned_raw_mentor = self
            .authoritative
            .ok_or(GuideInitError::PcStateUnavailable)?;
        guide_init_intent(owned_raw_mentor.raw_mentor, passed_raw_mentor)
    }

    /// Correlate a selectable request with the exact pre-request mentor.
    pub fn request_change(
        &mut self,
        mentor: GuideMentor,
    ) -> Result<PcChangeMentorRequest0104, GuideCorrelationError> {
        if self.pending.is_some() {
            return Err(GuideCorrelationError::RequestAlreadyPending);
        }
        let authoritative = self
            .authoritative
            .ok_or(GuideCorrelationError::PcStateUnavailable)?;
        match authoritative.recognized_mentor() {
            Some(GuideRawMentor::Selectable(current)) if current == mentor => {
                return Err(GuideCorrelationError::AlreadyCurrentMentor { mentor });
            }
            Some(GuideRawMentor::Selectable(_)) | Some(GuideRawMentor::ComputressFuture) => {}
            None => {
                return Err(GuideCorrelationError::UnsupportedOwnedMentor {
                    raw_mentor: authoritative.raw_mentor,
                });
            }
        }
        self.pending = Some(GuidePendingChange {
            mentor,
            previous_raw_mentor: authoritative.raw_mentor,
        });
        Ok(PcChangeMentorRequest0104 {
            mentor: mentor.wire_id(),
        })
    }

    /// Accept a success reply only after both request mentor and pre-request
    /// ownership still correlate. All rejection paths leave this runtime
    /// byte-for-byte unchanged.
    pub fn accept_success(
        &mut self,
        reply: PcChangeMentorSuccess0104,
    ) -> Result<GuideChangeApplied, GuideCorrelationError> {
        let (pending, authoritative, reply_mentor) = self.validate_reply(reply.mentor)?;
        let kind = match self.profile {
            GuideServerProfile::Clean0104 => {
                if reply.mentor_count == 1 {
                    GuideChangeKind::First
                } else {
                    GuideChangeKind::Later
                }
            }
            GuideServerProfile::OpenFusion0104 => {
                match GuideRawMentor::from_raw(pending.previous_raw_mentor) {
                    Some(GuideRawMentor::ComputressFuture) => GuideChangeKind::First,
                    Some(GuideRawMentor::Selectable(_)) => GuideChangeKind::Later,
                    None => {
                        return Err(GuideCorrelationError::UnsupportedOwnedMentor {
                            raw_mentor: pending.previous_raw_mentor,
                        });
                    }
                }
            }
        };

        let intent = match kind {
            GuideChangeKind::First => GuidePostChangeIntent::WarpToNpc {
                npc_table_id: GUIDE_FIRST_CHANGE_WARP_NPC_TABLE_ID,
            },
            GuideChangeKind::Later => GuidePostChangeIntent::RefreshGuideMissions,
        };
        let diagnostics = GuideReplyDiagnostics {
            profile: self.profile,
            requested_mentor: pending.mentor,
            previous_raw_mentor: pending.previous_raw_mentor,
            reply_raw_mentor: reply.mentor,
            raw_reply_mentor_count: reply.mentor_count,
            interpreted_as: kind,
        };

        // Commit only after every correlation and profile check succeeded.
        self.authoritative = Some(GuideAuthoritativeState {
            raw_mentor: reply.mentor,
            raw_mentor_count: reply.mentor_count,
        });
        self.pending = None;
        self.last_success_diagnostics = Some(diagnostics);

        debug_assert_eq!(reply_mentor, pending.mentor);
        debug_assert_eq!(authoritative.raw_mentor, diagnostics.previous_raw_mentor);
        Ok(GuideChangeApplied {
            mentor: reply_mentor,
            raw_mentor_count: reply.mentor_count,
            fusion_matter: reply.fusion_matter,
            intent,
        })
    }

    /// A correlated failure clears only the pending request. Mentor/count
    /// authority remains exactly as it was before the request.
    pub fn accept_failure(
        &mut self,
        reply: PcChangeMentorFailure0104,
    ) -> Result<GuideChangeFailed, GuideCorrelationError> {
        let (_, _, reply_mentor) = self.validate_reply(reply.mentor)?;
        self.pending = None;
        Ok(GuideChangeFailed {
            mentor: reply_mentor,
            error_code: reply.error_code,
        })
    }

    fn validate_reply(
        &self,
        raw_reply_mentor: i16,
    ) -> Result<(GuidePendingChange, GuideAuthoritativeState, GuideMentor), GuideCorrelationError>
    {
        let pending = self
            .pending
            .ok_or(GuideCorrelationError::ReplyWithoutPendingRequest)?;
        let authoritative = self
            .authoritative
            .ok_or(GuideCorrelationError::PcStateUnavailable)?;
        if authoritative.raw_mentor != pending.previous_raw_mentor {
            return Err(GuideCorrelationError::PreviousMentorChanged {
                expected_raw_mentor: pending.previous_raw_mentor,
                actual_raw_mentor: authoritative.raw_mentor,
            });
        }
        let reply_mentor = GuideMentor::from_wire_id(raw_reply_mentor).ok_or(
            GuideCorrelationError::UnsupportedReplyMentor {
                raw_mentor: raw_reply_mentor,
            },
        )?;
        if reply_mentor != pending.mentor {
            return Err(GuideCorrelationError::ReplyMentorMismatch {
                expected: pending.mentor,
                actual: reply_mentor,
            });
        }
        Ok((pending, authoritative, reply_mentor))
    }
}

#[cfg(test)]
mod tests;
