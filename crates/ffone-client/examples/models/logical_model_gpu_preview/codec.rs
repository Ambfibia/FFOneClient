use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CameraFrame {
    pub(super) eye: Vec3,
    pub(super) target: Vec3,
    pub(super) near: f32,
    pub(super) far: f32,
}

impl CameraFrame {
    pub(super) fn for_bounds(bounds: Bounds3, view: PreviewCameraView) -> Option<Self> {
        let target = bounds.center();
        let radius = bounds.half_extents().length().max(0.05);
        let half_vertical_fov = 22.5_f32.to_radians();
        let distance = radius / half_vertical_fov.sin() * 1.22;
        let eye = target + view.direction() * distance;
        let near = (distance - radius * 1.8).max(0.01);
        let far = (distance + radius * 5.0).max(50.0);
        (target.is_finite()
            && eye.is_finite()
            && near.is_finite()
            && far.is_finite()
            && near > 0.0
            && far > near)
            .then_some(Self {
                eye,
                target,
                near,
                far,
            })
    }
}
