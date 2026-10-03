use super::*;

pub(super) fn refresh_ui_text_fit_regions(
    regions: Query<Ref<Node>>,
    mut texts: Query<(
        &UiTextFitRegion,
        &mut UiTextAutoFit,
        &mut TextFont,
        &mut LineHeight,
    )>,
) {
    for (region, mut fit, mut font, mut line_height) in &mut texts {
        let Ok(node) = regions.get(region.0) else {
            continue;
        };
        if !node.is_changed() {
            continue;
        }
        let Some(mut bounds) = fixed_text_region(&node) else {
            continue;
        };
        bounds.y = fit_region_height(bounds.y, fit.max_font_size, fit.max_line_height);
        if Vec2::new(fit.max_width, fit.max_height) != bounds {
            fit.max_width = bounds.x;
            fit.max_height = bounds.y;
            font.font_size = fit.max_font_size.into();
            *line_height = fit.max_line_height;
        }
    }
}
