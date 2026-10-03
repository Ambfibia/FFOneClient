//! UI artwork follows the text locale. Paths below ui/en are canonical; an
//! optional file at the same relative path below ui/ru supplies translated art.
use super::Language;
use crate::assets::AssetLocator;
use bevy::{prelude::*, ui::UiSystems};
use std::{collections::BTreeSet, path::Path};

#[derive(Default, Resource)]
struct UiImageCatalog {
    russian: BTreeSet<String>,
}

impl UiImageCatalog {
    fn open(root: &Path) -> std::io::Result<Self> {
        let mut catalog = Self::default();
        let ru = root.join("ui/ru");
        if !ru.exists() {
            return Ok(catalog);
        }
        let mut pending = vec![ru.clone()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(directory)? {
                let entry = entry?;
                let kind = entry.file_type()?;
                if kind.is_dir() {
                    pending.push(entry.path());
                } else if kind.is_file() && entry.path().extension().is_some_and(|e| e == "png") {
                    let relative = entry
                        .path()
                        .strip_prefix(&ru)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    if !root.join("ui/en").join(&relative).is_file() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!("Russian UI image has no English source: {relative}"),
                        ));
                    }
                    catalog.russian.insert(relative);
                }
            }
        }
        Ok(catalog)
    }

    fn resolve(&self, path: &str, language: &str) -> Option<String> {
        let relative = path
            .strip_prefix("ui/en/")
            .or_else(|| path.strip_prefix("ui/ru/"))?;
        let locale = if language == "ru" && self.russian.contains(relative) {
            "ru"
        } else {
            "en"
        };
        let target = format!("ui/{locale}/{relative}");
        (target != path).then_some(target)
    }
}

fn load_catalog(locator: Option<Res<AssetLocator>>, mut catalog: ResMut<UiImageCatalog>) {
    let Some(locator) = locator else { return };
    match UiImageCatalog::open(locator.root()) {
        Ok(loaded) => *catalog = loaded,
        Err(error) => error!("Failed to load localized UI artwork: {error}"),
    }
}

fn apply_images(
    language: Option<Res<Language>>,
    server: Option<Res<AssetServer>>,
    catalog: Res<UiImageCatalog>,
    mut images: Query<&mut ImageNode>,
) {
    let (Some(language), Some(server)) = (language, server) else {
        return;
    };
    for mut image in &mut images {
        if !language.is_changed() && !image.is_changed() {
            continue;
        }
        let Some(path) = image.image.path() else {
            continue;
        };
        let Some(target) = catalog.resolve(&path.path().to_string_lossy(), &language.effective)
        else {
            continue;
        };
        image.image = server.load(target);
    }
}

pub(super) fn register(app: &mut App) {
    app.init_resource::<UiImageCatalog>()
        .add_systems(Startup, load_catalog)
        // State/hover writers run in Update; apply the locale afterwards and
        // before image measurements and extraction. Sprite geometry is untouched.
        .add_systems(PostUpdate, apply_images.before(UiSystems::Prepare));
}

#[cfg(test)]
mod tests;
