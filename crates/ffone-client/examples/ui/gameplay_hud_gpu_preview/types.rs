use super::*;

#[derive(Resource)]
pub(super) struct PreviewOutput(pub(super) PathBuf);

#[derive(Resource)]
pub(super) struct PreviewPlayerLook(pub(super) NativePlayerLook);
