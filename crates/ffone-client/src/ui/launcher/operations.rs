use super::*;

pub(super) fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

#[must_use]
pub fn launcher_forward(pitch_degrees: f32, yaw_degrees: f32) -> Vec3 {
    let pitch = pitch_degrees.to_radians();
    let yaw = yaw_degrees.to_radians();
    crate::coordinates::unity_to_native_vector(Vec3::new(
        yaw.sin() * pitch.cos(),
        -pitch.sin(),
        yaw.cos() * pitch.cos(),
    ))
    .normalize_or_zero()
}

pub(super) fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

pub(super) fn absolute_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        ..default()
    }
}

pub(super) fn centered_label_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    }
}

pub(super) fn sliced_backdrop_image(image: Handle<Image>) -> ImageNode {
    ImageNode {
        image,
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border: LAUNCHER_UI_BACKDROP_BORDER,
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}
