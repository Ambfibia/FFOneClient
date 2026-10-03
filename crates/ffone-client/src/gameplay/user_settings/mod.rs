//! Durable, versioned storage for user-owned client preferences.
//!
//! The runtime model is intentionally independent from the on-disk envelope.
//! This keeps schema migration at this boundary and lets malformed individual
//! fields fall back to clean defaults without discarding unrelated preferences.

use std::{
    env,
    error::Error,
    fmt,
    fs::{self, File, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use crate::option_ui::{InputSettings, OPTION_CHAT_PALETTE_RGB, OptionSettings};

pub const USER_SETTINGS_SCHEMA: &str = "ffone.user-settings.v1";
pub const USER_SETTINGS_DIRECTORY: &str = "FusionFallOne";
pub const USER_SETTINGS_FILE: &str = "settings.json";

static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, PartialEq)]
pub struct UserSettings {
    pub map: crate::map_preferences::MapPreferences,
    pub options: OptionSettings,
    pub input: InputSettings,
    pub text_locale: String,
    pub voice_locale: String,
    pub character_selection_music: bool,
    pub window_maximized: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            map: crate::map_preferences::MapPreferences::default(),
            options: OptionSettings::default(),
            input: InputSettings::default(),
            text_locale: "ru".to_owned(),
            voice_locale: "en".to_owned(),
            character_selection_music: true,
            window_maximized: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadOrigin {
    Missing,
    Loaded,
    Invalid,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadOutcome {
    pub settings: UserSettings,
    pub origin: LoadOrigin,
    pub warning: Option<String>,
}

impl LoadOutcome {
    #[must_use]
    pub const fn is_writable(&self) -> bool {
        !matches!(self.origin, LoadOrigin::Unsupported)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsPathError {
    LocalAppDataUnavailable,
}

impl fmt::Display for SettingsPathError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalAppDataUnavailable => {
                formatter.write_str("LOCALAPPDATA is unavailable; user settings path is unknown")
            }
        }
    }
}

impl Error for SettingsPathError {}

#[derive(Debug)]
pub enum SettingsSaveError {
    Io { path: PathBuf, source: io::Error },
    Serialization(serde_json::Error),
    UnsupportedSchema { found: String },
}

impl fmt::Display for SettingsSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(
                    formatter,
                    "could not persist user settings at {}: {source}",
                    path.display()
                )
            }
            Self::Serialization(source) => {
                write!(formatter, "could not serialize user settings: {source}")
            }
            Self::UnsupportedSchema { found } => write!(
                formatter,
                "refusing to overwrite unsupported user settings schema {found:?}"
            ),
        }
    }
}

impl Error for SettingsSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Serialization(source) => Some(source),
            Self::UnsupportedSchema { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserSettingsStore {
    path: PathBuf,
}

impl UserSettingsStore {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn from_local_app_data() -> Result<Self, SettingsPathError> {
        default_settings_path().map(Self::new)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn load(&self) -> LoadOutcome {
        load_from_path(&self.path)
    }

    pub fn save(&self, settings: &UserSettings) -> Result<(), SettingsSaveError> {
        guard_supported_schema(&self.path)?;

        let bytes = document_bytes(settings).map_err(SettingsSaveError::Serialization)?;
        let parent = usable_parent(&self.path);
        fs::create_dir_all(parent).map_err(|source| save_io_error(parent, source))?;

        let (temporary_path, mut temporary_file) = create_temporary_file(&self.path)
            .map_err(|source| save_io_error(&self.path, source))?;

        let write_result = (|| {
            temporary_file.write_all(&bytes)?;
            temporary_file.sync_all()?;
            drop(temporary_file);
            replace_settings_file(&temporary_path, &self.path)?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&self.path)?
                .sync_all()?;
            sync_parent_directory(parent)
        })();

        if let Err(source) = write_result {
            let _ = fs::remove_file(&temporary_path);
            return Err(save_io_error(&self.path, source));
        }

        Ok(())
    }
}

pub fn default_settings_path() -> Result<PathBuf, SettingsPathError> {
    let local_app_data = env::var_os("LOCALAPPDATA")
        .filter(|value| !value.is_empty())
        .ok_or(SettingsPathError::LocalAppDataUnavailable)?;
    Ok(settings_path_from_local_app_data(local_app_data))
}

#[must_use]
pub fn settings_path_from_local_app_data(base: impl AsRef<Path>) -> PathBuf {
    base.as_ref()
        .join(USER_SETTINGS_DIRECTORY)
        .join(USER_SETTINGS_FILE)
}

#[derive(Serialize)]
struct SettingsDocument<'a> {
    schema: &'static str,
    map: &'a crate::map_preferences::MapPreferences,
    options: &'a OptionSettings,
    input: &'a InputSettings,
    text_locale: &'a str,
    voice_locale: &'a str,
    character_selection_music: bool,
    window_maximized: bool,
}

fn document_bytes(settings: &UserSettings) -> Result<Vec<u8>, serde_json::Error> {
    let document = SettingsDocument {
        schema: USER_SETTINGS_SCHEMA,
        map: &settings.map,
        options: &settings.options,
        input: &settings.input,
        text_locale: &settings.text_locale,
        voice_locale: &settings.voice_locale,
        character_selection_music: settings.character_selection_music,
        window_maximized: settings.window_maximized,
    };
    let mut bytes = serde_json::to_vec_pretty(&document)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn load_from_path(path: &Path) -> LoadOutcome {
    match fs::read(path) {
        Ok(bytes) => {
            let primary = decode_document(&bytes);
            if primary.origin == LoadOrigin::Invalid {
                if let Some(recovered) = load_backup(path, "the primary settings file was invalid")
                {
                    return recovered;
                }
            }
            primary
        }
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            if let Some(recovered) = load_backup(path, "the primary settings file was missing") {
                return recovered;
            }
            LoadOutcome {
                settings: UserSettings::default(),
                origin: LoadOrigin::Missing,
                warning: None,
            }
        }
        Err(source) => LoadOutcome {
            settings: UserSettings::default(),
            origin: LoadOrigin::Invalid,
            warning: Some(format!(
                "could not read user settings at {}: {source}",
                path.display()
            )),
        },
    }
}

fn load_backup(path: &Path, reason: &str) -> Option<LoadOutcome> {
    let backup = backup_path(path);
    let bytes = fs::read(&backup).ok()?;
    let mut outcome = decode_document(&bytes);
    if matches!(outcome.origin, LoadOrigin::Loaded | LoadOrigin::Unsupported) {
        append_warning(
            &mut outcome.warning,
            format!("{reason}; loaded backup {}", backup.display()),
        );
        Some(outcome)
    } else {
        None
    }
}

fn decode_document(bytes: &[u8]) -> LoadOutcome {
    let value = match serde_json::from_slice::<Value>(bytes) {
        Ok(value) => value,
        Err(source) => {
            return LoadOutcome {
                settings: UserSettings::default(),
                origin: LoadOrigin::Invalid,
                warning: Some(format!("user settings JSON is invalid: {source}")),
            };
        }
    };
    let Some(object) = value.as_object() else {
        return LoadOutcome {
            settings: UserSettings::default(),
            origin: LoadOrigin::Invalid,
            warning: Some("user settings root must be a JSON object".to_owned()),
        };
    };
    let Some(schema) = object.get("schema").and_then(Value::as_str) else {
        return LoadOutcome {
            settings: UserSettings::default(),
            origin: LoadOrigin::Invalid,
            warning: Some("user settings schema is missing or invalid".to_owned()),
        };
    };
    if schema != USER_SETTINGS_SCHEMA {
        return LoadOutcome {
            settings: UserSettings::default(),
            origin: LoadOrigin::Unsupported,
            warning: Some(format!(
                "user settings schema {schema:?} is unsupported; the file is read-only"
            )),
        };
    }

    let defaults = UserSettings::default();
    let mut warnings = Vec::new();

    let (options, recovered_options) =
        recover_section(object.get("options"), &defaults.options, validate_options);
    if recovered_options {
        warnings.push("some option settings were invalid or missing and used defaults".to_owned());
    }

    let (input, recovered_input) =
        recover_section(object.get("input"), &defaults.input, validate_input);
    if recovered_input {
        warnings.push("some input settings were invalid or missing and used defaults".to_owned());
    }

    let text_locale = recover_locale(
        object.get("text_locale"),
        &defaults.text_locale,
        "text locale",
        &mut warnings,
    );
    let voice_locale = recover_locale(
        object.get("voice_locale"),
        &defaults.voice_locale,
        "voice locale",
        &mut warnings,
    );
    let character_selection_music = match object
        .get("character_selection_music")
        .and_then(Value::as_bool)
    {
        Some(value) => value,
        None => {
            warnings.push(
                "character selection music preference was invalid or missing and used its default"
                    .to_owned(),
            );
            defaults.character_selection_music
        }
    };

    LoadOutcome {
        settings: UserSettings {
            map: object
                .get("map")
                .cloned()
                .and_then(|value| {
                    serde_json::from_value::<crate::map_preferences::MapPreferences>(value).ok()
                })
                .filter(|map| map.validate())
                .unwrap_or_default(),
            options,
            input,
            text_locale,
            voice_locale,
            character_selection_music,
            window_maximized: object
                .get("window_maximized")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        },
        origin: LoadOrigin::Loaded,
        warning: (!warnings.is_empty()).then(|| warnings.join("; ")),
    }
}

fn recover_locale(
    candidate: Option<&Value>,
    default: &str,
    label: &str,
    warnings: &mut Vec<String>,
) -> String {
    let Some(candidate) = candidate.and_then(Value::as_str) else {
        warnings.push(format!(
            "{label} was invalid or missing and used its default"
        ));
        return default.to_owned();
    };
    let normalized = candidate.trim().to_ascii_lowercase().replace('_', "-");
    if !is_valid_locale(&normalized) {
        warnings.push(format!(
            "{label} {candidate:?} was invalid and used its default"
        ));
        return default.to_owned();
    }
    if normalized != candidate {
        warnings.push(format!(
            "{label} {candidate:?} was normalized to {normalized:?}"
        ));
    }
    normalized
}

fn is_valid_locale(locale: &str) -> bool {
    !locale.is_empty()
        && locale.len() <= 64
        && !locale.starts_with('-')
        && !locale.ends_with('-')
        && !locale.contains("--")
        && locale
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn recover_section<T>(
    candidate: Option<&Value>,
    default: &T,
    validate: impl Fn(&T) -> bool,
) -> (T, bool)
where
    T: Clone + Serialize + DeserializeOwned,
{
    let Some(candidate) = candidate else {
        return (default.clone(), true);
    };
    let default_value =
        serde_json::to_value(default).expect("runtime setting defaults must serialize");
    let mut leaves = Vec::new();
    let mut recovered = false;
    collect_known_leaves(
        &default_value,
        candidate,
        &mut Vec::new(),
        &mut leaves,
        &mut recovered,
    );

    let mut working = default_value;
    for (path, value) in leaves {
        let mut trial = working.clone();
        let Some(target) = value_at_path_mut(&mut trial, &path) else {
            recovered = true;
            continue;
        };
        *target = value;
        match serde_json::from_value::<T>(trial.clone()) {
            Ok(parsed) if validate(&parsed) => working = trial,
            _ => recovered = true,
        }
    }

    match serde_json::from_value::<T>(working) {
        Ok(parsed) if validate(&parsed) => (parsed, recovered),
        _ => (default.clone(), true),
    }
}

fn collect_known_leaves(
    default: &Value,
    candidate: &Value,
    path: &mut Vec<String>,
    leaves: &mut Vec<(Vec<String>, Value)>,
    recovered: &mut bool,
) {
    let Value::Object(default_fields) = default else {
        leaves.push((path.clone(), candidate.clone()));
        return;
    };
    let Some(candidate_fields) = candidate.as_object() else {
        *recovered = true;
        return;
    };

    if candidate_fields
        .keys()
        .any(|key| !default_fields.contains_key(key))
    {
        *recovered = true;
    }

    for (key, default_value) in default_fields {
        let Some(candidate_value) = candidate_fields.get(key) else {
            *recovered = true;
            continue;
        };
        path.push(key.clone());
        collect_known_leaves(default_value, candidate_value, path, leaves, recovered);
        path.pop();
    }
}

fn value_at_path_mut<'a>(value: &'a mut Value, path: &[String]) -> Option<&'a mut Value> {
    let mut current = value;
    for segment in path {
        current = current.as_object_mut()?.get_mut(segment)?;
    }
    Some(current)
}

fn validate_options(options: &OptionSettings) -> bool {
    let graphics = &options.graphics;
    let dimensions_are_valid =
        (320..=16_384).contains(&graphics.width) && (200..=16_384).contains(&graphics.height);
    let graphics_are_valid = dimensions_are_valid
        && graphics.visibility.is_finite()
        && (0.0..=1.0).contains(&graphics.visibility)
        && graphics.particle_level <= 3;
    let sound_is_valid = [
        options.sound.master,
        options.sound.music,
        options.sound.effects,
        options.sound.ambient,
        options.sound.voice,
    ]
    .into_iter()
    .all(|channel| channel.volume.is_finite() && (0.0..=1.0).contains(&channel.volume));
    let palette_length = OPTION_CHAT_PALETTE_RGB.len();
    let colors_are_valid = usize::from(options.text_colors.general) < palette_length
        && usize::from(options.text_colors.buddy) < palette_length
        && usize::from(options.text_colors.group) < palette_length;
    graphics_are_valid && sound_is_valid && colors_are_valid
}

fn validate_input(input: &InputSettings) -> bool {
    input.has_complete_clean_schema()
        && input.pad_camera_sensitivity.is_finite()
        && (1.0..=10.0).contains(&input.pad_camera_sensitivity)
        && input.camera_sensitivity.is_finite()
        && (1.0..=10.0).contains(&input.camera_sensitivity)
}

fn guard_supported_schema(path: &Path) -> Result<(), SettingsSaveError> {
    if let Some(found) = unsupported_schema_at(path)? {
        return Err(SettingsSaveError::UnsupportedSchema { found });
    }

    if !path.exists() {
        let backup = backup_path(path);
        if let Some(found) = unsupported_schema_at(&backup)? {
            return Err(SettingsSaveError::UnsupportedSchema { found });
        }
    }

    Ok(())
}

fn unsupported_schema_at(path: &Path) -> Result<Option<String>, SettingsSaveError> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(save_io_error(path, source)),
    };
    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return Ok(None);
    };
    let Some(schema) = value.get("schema").and_then(Value::as_str) else {
        return Ok(None);
    };
    Ok((schema != USER_SETTINGS_SCHEMA).then(|| schema.to_owned()))
}

fn create_temporary_file(target: &Path) -> io::Result<(PathBuf, File)> {
    let parent = usable_parent(target);
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(USER_SETTINGS_FILE);

    for _ in 0..32 {
        let id = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let temporary_path = parent.join(format!(".{file_name}.{}.{}.tmp", process::id(), id));
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)
        {
            Ok(file) => return Ok((temporary_path, file)),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => {}
            Err(source) => return Err(source),
        }
    }

    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a unique settings temporary file",
    ))
}

fn usable_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(USER_SETTINGS_FILE);
    path.with_file_name(format!("{file_name}.bak"))
}

#[cfg(windows)]
fn replace_settings_file(temporary: &Path, target: &Path) -> io::Result<()> {
    replace_with_backup_rollback(temporary, target)
}

#[cfg(not(windows))]
fn replace_settings_file(temporary: &Path, target: &Path) -> io::Result<()> {
    fs::rename(temporary, target)
}

#[cfg(any(windows, test))]
fn replace_with_backup_rollback(temporary: &Path, target: &Path) -> io::Result<()> {
    match fs::metadata(target) {
        Err(source) if source.kind() == io::ErrorKind::NotFound => {
            return fs::rename(temporary, target);
        }
        Err(source) => return Err(source),
        Ok(_) => {}
    }

    let backup = backup_path(target);
    match fs::remove_file(&backup) {
        Ok(()) => {}
        Err(source) if source.kind() == io::ErrorKind::NotFound => {}
        Err(source) => return Err(source),
    }

    fs::rename(target, &backup)?;
    if let Err(install_error) = fs::rename(temporary, target) {
        return match fs::rename(&backup, target) {
            Ok(()) => Err(install_error),
            Err(rollback_error) => Err(io::Error::other(format!(
                "settings replacement failed ({install_error}) and rollback failed ({rollback_error})"
            ))),
        };
    }

    let _ = fs::remove_file(backup);
    Ok(())
}

#[cfg(windows)]
fn sync_parent_directory(_parent: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(not(windows))]
fn sync_parent_directory(parent: &Path) -> io::Result<()> {
    File::open(parent)?.sync_all()
}

fn save_io_error(path: &Path, source: io::Error) -> SettingsSaveError {
    SettingsSaveError::Io {
        path: path.to_owned(),
        source,
    }
}

fn append_warning(warning: &mut Option<String>, addition: String) {
    match warning {
        Some(existing) => {
            existing.push_str("; ");
            existing.push_str(&addition);
        }
        None => *warning = Some(addition),
    }
}

#[cfg(test)]
mod tests;
