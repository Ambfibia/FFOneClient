use super::*;

#[derive(Default)]
pub(in super::super) struct TutorialCameraPoseMemory {
    pub(super) mode: Option<CameraMode>,
    pub(super) translation: Option<Vec3>,
    pub(super) target: Option<Vec3>,
    pub(super) authored_target: Option<Vec3>,
    pub(super) authored_current_target: Option<Vec3>,
    pub(super) stored_start: Option<Vec3>,
    pub(super) frozen_target: Option<Vec3>,
    pub(super) applied_freeze_target_revision: u64,
    pub(super) applied_look_at_revision: u64,
}
