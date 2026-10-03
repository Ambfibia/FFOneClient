use super::*;

pub(super) fn project_icons(
    ids: &[i32],
    projection: IconProjection,
    viewport_width: u32,
    scale: f32,
    model: &SkillBuffUiModel,
    catalog: &SkillBuffUiCatalog,
) -> Vec<SkillBuffIconView> {
    ids.iter()
        .enumerate()
        .filter_map(|(index, buff_id)| {
            let definition = catalog.definition(*buff_id)?;
            let rect = icon_rect(projection, viewport_width, ids.len(), index, scale);
            let (icon_number, icon_path, cash_time) = match projection {
                IconProjection::Cash => (
                    definition.cash_icon_number,
                    definition.cash_icon_path.clone(),
                    Some(format_cash_time(model.cash_remaining_ms(*buff_id))),
                ),
                IconProjection::Local | IconProjection::Target => {
                    (definition.icon_number, definition.icon_path.clone(), None)
                }
            };
            Some(SkillBuffIconView {
                buff_id: *buff_id,
                icon_number,
                icon_path,
                rect,
                cash_time,
            })
        })
        .collect()
}
