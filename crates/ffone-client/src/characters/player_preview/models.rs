use super::*;

#[derive(Resource)]
pub struct NativePlayerPreviewModel {
    pub visible: bool,
    pub stage: NativePlayerPreviewStage,
    pub yaw_degrees: f32,
    /// Direct source `cnSimpleCharRenderCamera.Distance`, in native metres.
    /// Keeping distance-space here avoids reciprocal zoom factors drifting
    /// away from the serialized 1..=3 character-creation range.
    pub camera_distance: f32,
    pub status: NativePlayerPreviewStatus,
    pub loading_detail: Option<String>,
    pub(super) look: Option<NativePlayerLook>,
    pub(super) revision: u64,
}

impl Default for NativePlayerPreviewModel {
    fn default() -> Self {
        Self {
            visible: false,
            stage: NativePlayerPreviewStage::Selection,
            yaw_degrees: 0.0,
            camera_distance: NATIVE_PLAYER_SELECTION_CAMERA_DISTANCE,
            status: NativePlayerPreviewStatus::Empty,
            loading_detail: None,
            look: None,
            revision: 0,
        }
    }
}

impl NativePlayerPreviewModel {
    pub fn look(&self) -> Option<&NativePlayerLook> {
        self.look.as_ref()
    }

    pub fn set_look(&mut self, look: NativePlayerLook) -> Result<(), String> {
        look.validate()?;
        if self.look.as_ref() != Some(&look) {
            self.look = Some(look);
            self.revision = self.revision.wrapping_add(1).max(1);
            self.status = NativePlayerPreviewStatus::Loading;
            self.loading_detail = None;
        }
        Ok(())
    }

    pub fn clear_look(&mut self) {
        if self.look.take().is_some() {
            self.revision = self.revision.wrapping_add(1).max(1);
        }
        self.status = NativePlayerPreviewStatus::Empty;
        self.loading_detail = None;
    }
}
