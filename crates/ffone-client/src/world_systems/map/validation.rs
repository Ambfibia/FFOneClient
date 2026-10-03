use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldMapError {
    AlreadyOpen,
    TutorialLocked,
    NonFiniteInput,
    InvalidDeltaTime,
    InvalidViewState,
    ModeNotInteractive,
    DuplicatePresentNpcType(i32),
    AmbiguousNpcCatalog(i32),
    InvalidNpcMapIcon { npc_type: i32, map_icon: i32 },
    MissingPlayerForWaypoint,
}

pub(super) fn validate_motion(delta_x: f32, delta_y: f32, delta_seconds: f32) -> Result<(), WorldMapError> {
    if !delta_x.is_finite() || !delta_y.is_finite() {
        return Err(WorldMapError::NonFiniteInput);
    }
    if !delta_seconds.is_finite() || delta_seconds < 0.0 {
        return Err(WorldMapError::InvalidDeltaTime);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldMapPresentationError {
    NonFiniteHover,
    InvalidHoverMarker,
    InvalidMapView,
    InvalidMarker,
}
