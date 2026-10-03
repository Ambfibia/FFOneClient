use super::*;

pub(super) fn install_document_fonts(
    ctx: &egui::Context,
    document: &UiLayoutDocument,
) -> BTreeMap<String, FontFamily> {
    let mut definitions = egui::FontDefinitions::default();
    let mut families = BTreeMap::new();
    for element in &document.elements {
        let Some(path) = element
            .visual
            .as_ref()
            .and_then(|visual| visual.text.as_ref())
            .and_then(|text| text.font.as_ref())
        else {
            continue;
        };
        if families.contains_key(path) {
            continue;
        }
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        let key = format!("ffui-font-{}", families.len());
        let family = FontFamily::Name(key.clone().into());
        definitions
            .font_data
            .insert(key.clone(), egui::FontData::from_owned(bytes).into());
        definitions
            .families
            .insert(family.clone(), vec![key.clone()]);
        families.insert(path.clone(), family);
    }
    ctx.set_fonts(definitions);
    families
}
