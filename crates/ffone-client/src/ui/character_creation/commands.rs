use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterCreationCameraAction {
    RotateLeft,
    RotateRight,
    ZoomIn,
    ZoomOut,
}

impl CharacterCreationCameraAction {
    pub fn rotation_delta_degrees(self, delta: Duration) -> f32 {
        let seconds = delta.as_secs_f32();
        match self {
            // `CnCharCreationMode::RotateLeft` adds to Euler Y, while the
            // selection screen's similarly named mode subtracts from it.
            Self::RotateLeft => seconds * CHARACTER_CREATION_PREVIEW_ROTATION_SPEED_DEGREES,
            Self::RotateRight => -seconds * CHARACTER_CREATION_PREVIEW_ROTATION_SPEED_DEGREES,
            Self::ZoomIn | Self::ZoomOut => 0.0,
        }
    }

    pub fn distance_delta(self, delta: Duration) -> f32 {
        let seconds = delta.as_secs_f32();
        match self {
            Self::ZoomIn => -seconds * CHARACTER_CREATION_PREVIEW_ZOOM_SPEED,
            Self::ZoomOut => seconds * CHARACTER_CREATION_PREVIEW_ZOOM_SPEED,
            Self::RotateLeft | Self::RotateRight => 0.0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CharacterCreationUiAction {
    ExitToSelection,
    ToggleFullscreen,
    /// One native frame of the source `GUI.RepeatButton` camera action.
    ///
    /// A duration is carried instead of a fixed click step so rotation remains
    /// exactly 100 degrees/second and zoom remains 0.4 distance units/second
    /// regardless of the render frame rate.
    Camera {
        action: CharacterCreationCameraAction,
        delta: Duration,
    },
    AppearanceChanged(CharacterAppearance),
    RandomizeAppearance(CharacterAppearance),
    SubmitGeneratedName(GeneratedCharacterName),
    SubmitCustomName(CustomCharacterName),
    ConfirmAppearance(CharacterAppearance),
}
