use super::*;

pub(super) fn array_color(value: [f32; 4]) -> Color {
    Color::srgba(value[0], value[1], value[2], value[3])
}
