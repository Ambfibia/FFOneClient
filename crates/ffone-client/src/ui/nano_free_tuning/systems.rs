use super::*;

pub(super) fn sync_nano_free_tuning_content(
    model: Res<NanoFreeTuningModel>,
    asset_server: Res<AssetServer>,
    mut texts: Query<(&NanoFreeTuningText, &mut LocalizedText)>,
    mut icons: Query<(&NanoFreeTuningPowerIcon, &mut ImageNode)>,
) {
    let Some(content) = model.content() else {
        return;
    };
    for (role, mut localized) in &mut texts {
        let copy = match role.0 {
            NanoFreeTuningTextRole::AcquiredPrefix => NANO_FREE_TUNING_COPY_ACQUIRED.to_owned(),
            NanoFreeTuningTextRole::NanoName => content.nano_name.clone(),
            NanoFreeTuningTextRole::Bang => NANO_FREE_TUNING_COPY_BANG.to_owned(),
            NanoFreeTuningTextRole::PowerName(index) => content.powers[index].name.clone(),
            NanoFreeTuningTextRole::PowerType(index) => content.powers[index].power_type.clone(),
            NanoFreeTuningTextRole::PowerDescription(index) => {
                content.powers[index].description.clone()
            }
        };
        *localized = match role.0 {
            NanoFreeTuningTextRole::NanoName => LocalizedText::new(
                format!("content.nano.{}.acquired_name", content.nano_id),
                format!(" {copy} "),
            ),
            NanoFreeTuningTextRole::PowerName(index)
            | NanoFreeTuningTextRole::PowerType(index)
            | NanoFreeTuningTextRole::PowerDescription(index) => {
                let field = match role.0 {
                    NanoFreeTuningTextRole::PowerName(_) => "name",
                    NanoFreeTuningTextRole::PowerType(_) => "type_label",
                    _ => "description",
                };
                LocalizedText::new(
                    format!(
                        "content.nano_tune.{}.{field}",
                        content.powers[index].tune_id
                    ),
                    &copy,
                )
            }
            _ => nano_free_tuning_localized_text(role.0, &copy),
        };
    }
    for (icon, mut image) in &mut icons {
        image.image = asset_server.load(content.powers[icon.0].icon_path.clone());
        image.image_mode = NodeImageMode::Stretch;
    }
}
