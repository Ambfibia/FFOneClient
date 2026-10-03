use super::*;

/// Legacy `BillboardNode.mode`. The original client either rotated the node to
/// face the camera completely, or kept it upright and yawed only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BillboardMode {
    Camera,
    Up,
    RigidCamera,
    Center,
    RigidCenter,
}

impl BillboardMode {
    pub(super) fn from_legacy(mode: i64) -> Self {
        match mode {
            0 => Self::Camera,
            1 => Self::Up,
            2 => Self::RigidCamera,
            3 => Self::Center,
            _ => Self::RigidCenter,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct WorldSwitchState {
    pub on: bool,
}
