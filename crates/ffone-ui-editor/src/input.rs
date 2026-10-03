use super::*;

pub(super) fn load_form_textures(
    ctx: &egui::Context,
    document: &UiLayoutDocument,
    inventory_override: Option<&Path>,
) -> BTreeMap<String, egui::TextureHandle> {
    let mut textures = BTreeMap::new();
    for form in &document.forms {
        let path = if form.id == "inventory" {
            inventory_override
                .map(Path::to_path_buf)
                .or_else(|| form.background.as_deref().map(PathBuf::from))
        } else {
            form.background.as_deref().map(PathBuf::from)
        };
        if let Some(path) = path
            && let Ok(texture) = load_texture(ctx, &path)
        {
            textures.insert(form.id.clone(), texture);
        }
    }
    textures
}

pub(super) fn load_element_textures(
    ctx: &egui::Context,
    document: &UiLayoutDocument,
) -> BTreeMap<String, egui::TextureHandle> {
    let mut textures = BTreeMap::new();
    for element in &document.elements {
        let Some(image) = element
            .visual
            .as_ref()
            .and_then(|visual| visual.image.as_ref())
        else {
            continue;
        };
        if textures.contains_key(&image.path) {
            continue;
        }
        let path = Path::new(&image.path);
        if let Ok(texture) = load_texture(ctx, path) {
            textures.insert(image.path.clone(), texture);
        }
    }
    textures
}
