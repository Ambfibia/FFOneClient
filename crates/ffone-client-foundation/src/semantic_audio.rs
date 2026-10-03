//! In-memory audio index built from TableData paths and available locale directories.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

use bevy::prelude::Resource;
use serde::Deserialize;

mod table_audio;

pub const CHARACTER_CREATION_AUDIO_RULE: &str = "character_creation";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum NativeAudioCategory {
    Music,
    Ambient,
    Voice,
    Sfx,
}

impl NativeAudioCategory {
    const fn directory(self) -> &'static str {
        match self {
            Self::Music => "music",
            Self::Ambient => "ambient",
            Self::Voice => "voice",
            Self::Sfx => "sfx",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAudioAsset {
    pub logical_key: String,
    pub aliases: Vec<String>,
    pub true_name: String,
    pub path: String,
    pub category: NativeAudioCategory,
    pub owner: String,
    pub classification_rule: String,
    pub classification_uncertain: bool,
    pub variant: Option<NativeAudioVariant>,
    pub locale_variants: BTreeMap<String, NativeAudioLocaleVariant>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAudioVariant {
    pub index: u32,
    pub total: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAudioLocaleVariant {
    pub locale: String,
    pub path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAudioCatalogDiagnostics {
    pub assets: usize,
    pub bytes: u64,
    pub music: usize,
    pub ambient: usize,
    pub voice: usize,
    pub sfx: usize,
    pub character_creation: usize,
    pub localized_files: usize,
    pub english_voice_files: usize,
    pub russian_voice_files: usize,
}

#[derive(Debug, Resource)]
pub struct NativeAudioCatalog {
    asset_root: PathBuf,
    assets: Vec<NativeAudioAsset>,
    by_path: BTreeMap<String, usize>,
    by_logical_key: BTreeMap<String, usize>,
    by_true_name: BTreeMap<String, Vec<usize>>,
    fallback_locale: String,
    locale_fallbacks: BTreeMap<String, String>,
    voice_locales: BTreeSet<String>,
    diagnostics: NativeAudioCatalogDiagnostics,
}

impl NativeAudioCatalog {
    pub fn open(asset_root: impl Into<PathBuf>, verify_hashes: bool) -> Result<Self, String> {
        let asset_root = asset_root.into();
        let _ = verify_hashes;
        let document = table_audio::load(&asset_root)?;
        Self::open_editable(asset_root, document)
    }

    fn open_editable(
        asset_root: PathBuf,
        document: EditableCatalogDocument,
    ) -> Result<Self, String> {
        validate_locale_id(&document.fallback_locale)?;
        for (locale, fallback) in &document.locale_fallbacks {
            validate_locale_id(locale)?;
            validate_locale_id(fallback)?;
        }

        let mut assets = Vec::with_capacity(document.assets.len());
        let mut by_path = BTreeMap::new();
        let mut by_logical_key = BTreeMap::new();
        let mut by_true_name = BTreeMap::<String, Vec<usize>>::new();
        let mut folded_paths = BTreeSet::new();
        let mut music = 0_usize;
        let mut ambient = 0_usize;
        let mut voice = 0_usize;
        let mut sfx = 0_usize;
        let mut character_creation = 0_usize;
        let mut localized_files = 0_usize;
        let mut base_bytes = 0_u64;

        validate_editable_voice_take_sets(&document)?;
        for source in document.assets {
            if source.logical_key.trim().is_empty()
                || source.true_name.trim().is_empty()
                || source.owner.trim().is_empty()
            {
                return Err(format!(
                    "editable audio identity is incomplete for {:?}",
                    source.logical_key
                ));
            }
            let index = assets.len();
            index_strict_logical_keys(
                &mut by_logical_key,
                &source.logical_key,
                &source.aliases,
                index,
                true,
            )?;
            let category: NativeAudioCategory = source.category.into();
            let mut locale_variants = BTreeMap::new();

            for file in &source.files {
                validate_editable_file(&source, category, file)?;
                let folded_path = folded_identity(&file.path);
                if !folded_paths.insert(folded_path) {
                    return Err(format!(
                        "case-insensitive duplicate editable audio path {:?}",
                        file.path
                    ));
                }
                let bytes = require_audio_file(&asset_root, &file.path)?;
                localized_files += 1;
                if category == NativeAudioCategory::Voice {
                    let locale = file.locale.as_deref().ok_or_else(|| {
                        format!("editable voice file has no locale: {:?}", file.path)
                    })?;
                    let locale = folded_identity(locale);
                    if locale_variants.contains_key(&locale) {
                        return Err(format!(
                            "duplicate editable voice locale {locale:?} for {:?}",
                            source.logical_key
                        ));
                    }
                    if locale == document.fallback_locale {
                        base_bytes = base_bytes
                            .checked_add(bytes)
                            .ok_or_else(|| "editable audio byte count overflow".to_owned())?;
                    }
                    locale_variants.insert(
                        locale.clone(),
                        NativeAudioLocaleVariant {
                            locale,
                            path: file.path.clone(),
                        },
                    );
                } else {
                    if source.files.len() != 1 || file.locale.is_some() {
                        return Err(format!(
                            "editable non-voice asset must have one locale-free file: {:?}",
                            source.logical_key
                        ));
                    }
                    base_bytes = base_bytes
                        .checked_add(bytes)
                        .ok_or_else(|| "editable audio byte count overflow".to_owned())?;
                }
            }

            match category {
                NativeAudioCategory::Music => music += 1,
                NativeAudioCategory::Ambient => ambient += 1,
                NativeAudioCategory::Voice => voice += 1,
                NativeAudioCategory::Sfx => sfx += 1,
            }
            let classification_rule = if source.scope.as_deref() == Some("character_creation") {
                character_creation += 1;
                CHARACTER_CREATION_AUDIO_RULE.to_owned()
            } else {
                "native-table-audio".to_owned()
            };
            let asset = NativeAudioAsset {
                logical_key: source.logical_key,
                aliases: source.aliases,
                true_name: source.true_name,
                path: source.canonical_path,
                category,
                owner: source.owner,
                classification_rule,
                classification_uncertain: false,
                variant: None,
                locale_variants,
            };
            // The declared base path remains an identity even when EN is absent.
            by_path.insert(asset.path.clone(), index);
            for file in source.files {
                by_path.insert(file.path, index);
            }
            by_true_name
                .entry(folded_identity(&asset.true_name))
                .or_default()
                .push(index);
            assets.push(asset);
        }

        let actual_counts = EditableCatalogCounts {
            assets: assets.len() as u64,
            music: music as u64,
            ambient: ambient as u64,
            voice: voice as u64,
            sfx: sfx as u64,
        };
        if document.counts != actual_counts {
            return Err(format!(
                "editable semantic audio counts disagree: catalog={:?}, actual={actual_counts:?}",
                document.counts
            ));
        }

        let mut voice_locales = collect_voice_locales(&assets);
        // English remains selectable even for a package containing only RU.
        voice_locales.insert("en".to_owned());
        let english_voice_files = count_voice_locale(&assets, "en");
        let russian_voice_files = count_voice_locale(&assets, "ru");
        Ok(Self {
            asset_root,
            assets,
            by_path,
            by_logical_key,
            by_true_name,
            fallback_locale: document.fallback_locale,
            locale_fallbacks: document.locale_fallbacks,
            voice_locales,
            diagnostics: NativeAudioCatalogDiagnostics {
                assets: actual_counts.assets as usize,
                bytes: base_bytes,
                music,
                ambient,
                voice,
                sfx,
                character_creation,
                localized_files,
                english_voice_files,
                russian_voice_files,
            },
        })
    }

    #[must_use]
    pub fn assets(&self) -> &[NativeAudioAsset] {
        &self.assets
    }

    #[must_use]
    pub fn diagnostics(&self) -> &NativeAudioCatalogDiagnostics {
        &self.diagnostics
    }

    #[must_use]
    pub fn by_path(&self, path: &str) -> Option<&NativeAudioAsset> {
        self.by_path
            .get(path)
            .and_then(|index| self.assets.get(*index))
    }

    /// Resolves either the editable physical path or its stable
    /// `audio/<logical-key>.ogg` compatibility route.
    #[must_use]
    pub fn by_legacy_path(&self, path: &str) -> Option<&NativeAudioAsset> {
        self.by_path(path).or_else(|| {
            let normalized = path.replace('\\', "/");
            let route = normalized.strip_prefix("audio/").unwrap_or(&normalized);
            let logical_key = route.strip_suffix(".ogg").unwrap_or(route);
            self.by_logical_key(logical_key)
        })
    }

    #[must_use]
    pub fn by_logical_key(&self, logical_key: &str) -> Option<&NativeAudioAsset> {
        self.by_logical_key
            .get(&folded_identity(logical_key))
            .and_then(|index| self.assets.get(*index))
    }

    #[must_use]
    pub fn by_true_name(&self, true_name: &str) -> Vec<&NativeAudioAsset> {
        self.by_true_name
            .get(&folded_identity(true_name))
            .into_iter()
            .flatten()
            .filter_map(|index| self.assets.get(*index))
            .collect()
    }

    /// Choose from all declared takes, independently of file availability or language.
    /// Language fallback is applied to this exact selected take afterwards.
    pub fn choose_voice_family(
        &self,
        true_name: &str,
        locale: &str,
        entropy: u32,
    ) -> Option<&NativeAudioAsset> {
        let (prefix, _) = table_audio::numbered(true_name)?;
        let prefix = folded_identity(prefix);
        let candidates: Vec<_> = self
            .by_true_name
            .range(prefix.clone()..)
            .take_while(|(name, _)| name.starts_with(&prefix))
            .filter(|(name, _)| table_audio::numbered(name).is_some_and(|(p, _)| p == prefix))
            .filter_map(|(_, indices)| {
                indices
                    .iter()
                    .map(|i| &self.assets[*i])
                    .find(|a| a.category == NativeAudioCategory::Voice)
            })
            .collect();
        let _ = locale;
        if candidates.is_empty() {
            None
        } else {
            Some(candidates[entropy as usize % candidates.len()])
        }
    }

    pub fn choose_owner_voice(
        &self,
        owner: &str,
        cue: &str,
        locale: &str,
        entropy: u32,
    ) -> Option<&NativeAudioAsset> {
        let asset = self.assets.iter().find(|a| {
            a.category == NativeAudioCategory::Voice
                && a.owner == owner
                && table_audio::numbered(&a.true_name)
                    .is_some_and(|(prefix, _)| prefix.to_ascii_lowercase().ends_with(cue))
        })?;
        self.choose_voice_family(&asset.true_name, locale, entropy)
    }

    /// Compatibility routes for legacy code-owned asset paths.
    ///
    /// The logical key remains stable when an editable v5 catalog changes the
    /// physical file name. Installing this map in the Bevy asset reader keeps
    /// older typed UI/gameplay call sites on the renamed native file without
    /// turning the old path into a second runtime identity.
    #[must_use]
    pub fn legacy_path_aliases(&self) -> BTreeMap<PathBuf, PathBuf> {
        let mut routes = BTreeMap::new();
        for asset in &self.assets {
            let target = PathBuf::from(&asset.path);
            for identity in std::iter::once(&asset.logical_key).chain(asset.aliases.iter()) {
                let identity = identity
                    .replace('\\', "/")
                    .trim_start_matches("audio/")
                    .trim_end_matches(".ogg")
                    .to_owned();
                if identity.is_empty() {
                    continue;
                }
                let legacy = PathBuf::from(format!("audio/{identity}.ogg"));
                if legacy != target {
                    routes.insert(legacy, target.clone());
                }
            }
        }
        routes
    }

    pub fn character_creation_assets(&self) -> impl Iterator<Item = &NativeAudioAsset> {
        self.assets
            .iter()
            .filter(|asset| asset.classification_rule == CHARACTER_CREATION_AUDIO_RULE)
    }

    /// Voice locales found below `audio/voice`. Mirrored files are discovered
    /// at startup, so adding a locale never requires adding hashes to a catalog.
    pub fn voice_locales(&self) -> impl ExactSizeIterator<Item = &str> {
        self.voice_locales.iter().map(String::as_str)
    }

    #[must_use]
    pub fn absolute_path(&self, asset: &NativeAudioAsset) -> PathBuf {
        self.asset_root.join(native_path(&asset.path))
    }

    #[must_use]
    pub fn locale_variant<'a>(
        &'a self,
        asset: &'a NativeAudioAsset,
        requested_locale: &str,
    ) -> Option<&'a NativeAudioLocaleVariant> {
        if asset.category != NativeAudioCategory::Voice {
            return None;
        }
        let requested = folded_identity(requested_locale);
        if let Some(variant) = asset.locale_variants.get(&requested) {
            return Some(variant);
        }
        if let Some(language) = requested.split('-').next()
            && let Some(variant) = asset.locale_variants.get(language)
        {
            return Some(variant);
        }
        if let Some(fallback) = self.locale_fallbacks.get(&requested)
            && let Some(variant) = asset.locale_variants.get(fallback)
        {
            return Some(variant);
        }
        asset.locale_variants.get(&self.fallback_locale)
    }

    #[must_use]
    pub fn path_for_locale<'a>(
        &'a self,
        asset: &'a NativeAudioAsset,
        requested_locale: &str,
    ) -> Option<&'a str> {
        if asset.category != NativeAudioCategory::Voice {
            return Some(&asset.path);
        }
        self.locale_variant(asset, requested_locale)
            .map(|variant| variant.path.as_str())
    }

    #[must_use]
    pub fn absolute_path_for_locale(
        &self,
        asset: &NativeAudioAsset,
        requested_locale: &str,
    ) -> Option<PathBuf> {
        self.path_for_locale(asset, requested_locale)
            .map(|path| self.asset_root.join(native_path(path)))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EditableCatalogDocument {
    fallback_locale: String,
    #[serde(default)]
    locale_fallbacks: BTreeMap<String, String>,
    counts: EditableCatalogCounts,
    assets: Vec<EditableCatalogAsset>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EditableCatalogCounts {
    assets: u64,
    music: u64,
    ambient: u64,
    voice: u64,
    sfx: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EditableCatalogAsset {
    logical_key: String,
    #[serde(default)]
    aliases: Vec<String>,
    true_name: String,
    category: CatalogCategory,
    owner: String,
    #[serde(default)]
    scope: Option<String>,
    files: Vec<EditableCatalogFile>,
    #[serde(skip)]
    canonical_path: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EditableCatalogFile {
    #[serde(default)]
    locale: Option<String>,
    path: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CatalogCategory {
    Music,
    Ambient,
    Voice,
    Sfx,
}

impl From<CatalogCategory> for NativeAudioCategory {
    fn from(category: CatalogCategory) -> Self {
        match category {
            CatalogCategory::Music => Self::Music,
            CatalogCategory::Ambient => Self::Ambient,
            CatalogCategory::Voice => Self::Voice,
            CatalogCategory::Sfx => Self::Sfx,
        }
    }
}

fn validate_editable_file(
    source: &EditableCatalogAsset,
    category: NativeAudioCategory,
    file: &EditableCatalogFile,
) -> Result<(), String> {
    validate_relative_path(&file.path)?;
    if file.path.contains("/variants/") || !file.path.ends_with(".ogg") {
        return Err(format!(
            "editable runtime audio path is not canonical: {:?}",
            file.path
        ));
    }
    let expected_prefix = if category == NativeAudioCategory::Voice {
        let locale = file
            .locale
            .as_deref()
            .ok_or_else(|| format!("editable voice file has no locale: {:?}", file.path))?;
        validate_locale_id(locale)?;
        format!("audio/voice/{locale}/{}/", source.owner)
    } else {
        if file.locale.is_some() {
            return Err(format!(
                "editable non-voice file unexpectedly has a locale: {:?}",
                file.path
            ));
        }
        format!("audio/{}/", category.directory())
    };
    if !file.path.starts_with(&expected_prefix) {
        return Err(format!(
            "editable audio path {:?} is not below {expected_prefix:?}",
            file.path
        ));
    }
    Ok(())
}

fn validate_editable_voice_take_sets(document: &EditableCatalogDocument) -> Result<(), String> {
    let mut groups = BTreeMap::<String, Vec<(bool, &str)>>::new();
    for asset in &document.assets {
        if !matches!(asset.category, CatalogCategory::Voice) {
            continue;
        }
        groups
            .entry(folded_identity(&asset.true_name))
            .or_default()
            .push((
                asset.scope.as_deref() == Some("alternate_take"),
                asset.logical_key.as_str(),
            ));
    }
    for (true_name, takes) in groups {
        if takes.len() == 1 {
            if takes[0].0 {
                return Err(format!(
                    "editable voice {true_name:?} is marked alternate_take without a primary"
                ));
            }
            continue;
        }
        let primary_count = takes.iter().filter(|(alternate, _)| !alternate).count();
        let keys = takes
            .iter()
            .map(|(_, key)| folded_identity(key))
            .collect::<BTreeSet<_>>();
        if primary_count != 1 || keys.len() != takes.len() {
            return Err(format!(
                "editable voice take set {true_name:?} requires one primary plus distinct alternate_take keys"
            ));
        }
    }
    Ok(())
}

fn validate_locale_id(locale: &str) -> Result<(), String> {
    if !locale.is_empty()
        && !locale.starts_with('-')
        && !locale.ends_with('-')
        && !locale.contains("--")
        && locale
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Ok(())
    } else {
        Err(format!("invalid audio locale {locale:?}"))
    }
}

fn require_audio_file(asset_root: &Path, relative_path: &str) -> Result<u64, String> {
    let absolute = asset_root.join(native_path(relative_path));
    let metadata = fs::metadata(&absolute)
        .map_err(|error| format!("missing editable audio {}: {error}", absolute.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "editable audio path is not a regular file: {}",
            absolute.display()
        ));
    }
    Ok(metadata.len())
}

fn collect_voice_locales(assets: &[NativeAudioAsset]) -> BTreeSet<String> {
    assets
        .iter()
        .filter(|asset| asset.category == NativeAudioCategory::Voice)
        .flat_map(|asset| asset.locale_variants.keys().cloned())
        .collect()
}

fn count_voice_locale(assets: &[NativeAudioAsset], locale: &str) -> usize {
    assets
        .iter()
        .filter(|asset| {
            asset.category == NativeAudioCategory::Voice
                && asset.locale_variants.contains_key(locale)
        })
        .count()
}

fn validate_relative_path(path: &str) -> Result<(), String> {
    if path.is_empty() || path.contains('\\') {
        return Err(format!("invalid semantic audio path {path:?}"));
    }
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe semantic audio path {path:?}"));
    }
    Ok(())
}

fn native_path(path: &str) -> PathBuf {
    path.split('/').collect()
}

fn folded_identity(value: &str) -> String {
    value.to_lowercase()
}

fn index_strict_logical_keys(
    index: &mut BTreeMap<String, usize>,
    logical_key: &str,
    aliases: &[String],
    asset_index: usize,
    aliases_supported: bool,
) -> Result<(), String> {
    if !aliases_supported && !aliases.is_empty() {
        return Err(format!(
            "strict audio aliases require schema v4 for {logical_key:?}"
        ));
    }
    let mut pending = Vec::with_capacity(aliases.len() + 1);
    let mut local = BTreeSet::new();
    for key in std::iter::once(logical_key).chain(aliases.iter().map(String::as_str)) {
        if key.trim().is_empty() {
            return Err("strict audio logical key or alias is empty".to_owned());
        }
        let identity = folded_identity(key);
        if !local.insert(identity.clone()) || index.contains_key(&identity) {
            return Err(format!(
                "case-insensitive strict audio logical key or alias collision at {key:?}"
            ));
        }
        pending.push(identity);
    }
    for identity in pending {
        index.insert(identity, asset_index);
    }
    Ok(())
}

#[cfg(test)]
#[path = "semantic_audio/tests.rs"]
mod tests;
