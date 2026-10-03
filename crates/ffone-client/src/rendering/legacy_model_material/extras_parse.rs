//! Cached parsing of glTF material extras.

use super::metadata::{
    PendingLegacyModelMaterial, PendingLegacyStaticWorldMaterial, PendingLegacyWaterMaterial,
};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum ParsedLegacyMaterialExtras {
    Model(PendingLegacyModelMaterial),
    Water(PendingLegacyWaterMaterial),
    StaticWorld(PendingLegacyStaticWorldMaterial),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct LegacyMaterialExtrasParseKey {
    pub(super) digest: [u8; 32],
    pub(super) material_name_bytes: usize,
    pub(super) extras_bytes: usize,
}

#[derive(Debug, Default, Resource)]
pub(super) struct LegacyMaterialExtrasParseCache {
    pub(super) entries: HashMap<LegacyMaterialExtrasParseKey, Vec<CachedLegacyMaterialExtrasParse>>,
    #[cfg(test)]
    pub(super) misses: usize,
}

#[derive(Debug)]
pub(super) struct CachedLegacyMaterialExtrasParse {
    pub(super) material_name: Option<Box<str>>,
    pub(super) extras: Box<str>,
    pub(super) parsed: ParsedLegacyMaterialExtras,
}

impl LegacyMaterialExtrasParseCache {
    pub(super) fn parse(&mut self, material_name: Option<&str>, extras: &str) -> ParsedLegacyMaterialExtras {
        let key = legacy_material_extras_parse_key(material_name, extras);
        if let Some(entries) = self.entries.get(&key)
            && let Some(cached) = entries.iter().find(|cached| {
                cached.material_name.as_deref() == material_name && cached.extras.as_ref() == extras
            })
        {
            return cached.parsed.clone();
        }
        let parsed = parse_legacy_material_extras(material_name, extras);
        self.entries
            .entry(key)
            .or_default()
            .push(CachedLegacyMaterialExtrasParse {
                material_name: material_name.map(Into::into),
                extras: extras.into(),
                parsed: parsed.clone(),
            });
        #[cfg(test)]
        {
            self.misses += 1;
        }
        parsed
    }
}

pub(super) fn legacy_material_extras_parse_key(
    material_name: Option<&str>,
    extras: &str,
) -> LegacyMaterialExtrasParseKey {
    let mut hasher = blake3::Hasher::new();
    match material_name {
        Some(name) => {
            hasher.update(&[1]);
            hasher.update(&(name.len() as u64).to_le_bytes());
            hasher.update(name.as_bytes());
        }
        None => {
            hasher.update(&[0]);
            hasher.update(&0_u64.to_le_bytes());
        }
    }
    hasher.update(&(extras.len() as u64).to_le_bytes());
    hasher.update(extras.as_bytes());
    LegacyMaterialExtrasParseKey {
        digest: *hasher.finalize().as_bytes(),
        material_name_bytes: material_name.map_or(0, str::len),
        extras_bytes: extras.len(),
    }
}

pub(super) fn parse_legacy_material_extras(
    material_name: Option<&str>,
    extras: &str,
) -> ParsedLegacyMaterialExtras {
    match PendingLegacyModelMaterial::from_gltf_extras(material_name, extras) {
        Ok(pending) => ParsedLegacyMaterialExtras::Model(pending),
        Err(strict_error) => {
            match PendingLegacyWaterMaterial::from_gltf_extras(material_name, extras) {
                Ok(pending) => ParsedLegacyMaterialExtras::Water(pending),
                Err(water_error) => {
                    match PendingLegacyStaticWorldMaterial::from_gltf_extras(material_name, extras)
                    {
                        Ok(pending) => ParsedLegacyMaterialExtras::StaticWorld(pending),
                        Err(compatibility_error) => ParsedLegacyMaterialExtras::Error(format!(
                            "{strict_error}; water compatibility: {water_error}; static-world compatibility: {compatibility_error}"
                        )),
                    }
                }
            }
        }
    }
}
