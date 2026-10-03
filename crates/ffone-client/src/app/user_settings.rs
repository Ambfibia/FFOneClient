//! Bevy-facing adapter for durable user-owned settings.
//!
//! The library-level [`ffone_client::user_settings`] module owns the stable
//! JSON contract and atomic file I/O. This module keeps those concerns out of
//! UI models: it resolves startup precedence, translates the loaded document
//! into live application resources, and flushes one complete committed
//! snapshot when any of those owners changes.

use std::path::{Path, PathBuf};

#[cfg(windows)]
use bevy::ecs::system::NonSendMarker;
use bevy::prelude::*;
use ffone_client::{
    character_selection_ui::CharacterSelectionUiModel,
    localization::{Language, VoiceLanguage},
    user_settings::{
        LoadOrigin, SettingsPathError, SettingsSaveError, UserSettings, UserSettingsStore,
    },
};

use super::OptionProductionRuntime;

/// Runtime persistence state inserted after startup settings have been read.
///
/// `last_observed` is deliberately the complete document, not a collection of
/// per-page dirty flags. OptionMode sound changes and localization changes have
/// immediate commit paths outside `PersistOptions`, so observing all live
/// owners is the only way to cover every durable setting without changing the
/// clean Apply/Save/Cancel transaction.
#[derive(Debug, Resource)]
pub(super) struct UserSettingsPersistence {
    path: PathBuf,
    last_observed: UserSettings,
    writes_enabled: bool,
    startup_text_override: Option<String>,
}

impl UserSettingsPersistence {
    fn new(path: PathBuf, last_observed: UserSettings, writes_enabled: bool) -> Self {
        Self {
            path,
            last_observed,
            writes_enabled,
            startup_text_override: None,
        }
    }

    #[must_use]
    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub(super) fn last_observed(&self) -> &UserSettings {
        &self.last_observed
    }

    #[must_use]
    pub(super) const fn writes_enabled(&self) -> bool {
        self.writes_enabled
    }

    pub(super) fn observe_without_saving(&mut self, snapshot: UserSettings) {
        self.last_observed = snapshot;
    }
}

/// Startup result kept separate from Bevy so bootstrap can resolve language
/// and construct the initial production option owner before `App` exists.
#[derive(Debug)]
pub(super) struct PreparedUserSettings {
    pub(super) settings: UserSettings,
    pub(super) origin: LoadOrigin,
    pub(super) warning: Option<String>,
    pub(super) persistence: UserSettingsPersistence,
}

impl PreparedUserSettings {
    pub(super) fn retain_saved_text_during_override(&mut self, requested: &str) {
        if requested != self.settings.text_locale {
            self.persistence.startup_text_override = Some(requested.to_owned());
        }
    }

    /// Builds the live option owner from a supported document. Missing,
    /// invalid and unsupported files retain the existing first-run behavior:
    /// the real Window/orbit-camera owners seed the defaults once they exist.
    #[must_use]
    pub(super) fn option_runtime(&self) -> OptionProductionRuntime {
        OptionProductionRuntime {
            options: self.settings.options.clone(),
            input: self.settings.input.clone(),
            initialized_from_live_runtime: self.origin == LoadOrigin::Loaded,
        }
    }

    #[must_use]
    pub(super) fn character_selection_music(&self) -> bool {
        self.settings.character_selection_music
    }
}

/// Resolves and loads the platform-local settings document.
pub(super) fn prepare_user_settings() -> Result<PreparedUserSettings, SettingsPathError> {
    UserSettingsStore::from_local_app_data().map(prepare_user_settings_from_store)
}

/// Injectable-path variant used by tests.
#[cfg(test)]
#[must_use]
pub(super) fn prepare_user_settings_at(path: impl Into<PathBuf>) -> PreparedUserSettings {
    prepare_user_settings_from_store(UserSettingsStore::new(path.into()))
}

/// Keeps startup non-fatal on hosts where the OS-local settings directory is
/// unavailable. No relative or executable-adjacent fallback is invented.
#[must_use]
pub(super) fn user_settings_without_persistence(warning: String) -> PreparedUserSettings {
    let settings = UserSettings::default();
    PreparedUserSettings {
        settings: settings.clone(),
        origin: LoadOrigin::Missing,
        warning: Some(warning),
        persistence: UserSettingsPersistence::new(PathBuf::new(), settings, false),
    }
}

fn prepare_user_settings_from_store(store: UserSettingsStore) -> PreparedUserSettings {
    let path = store.path().to_path_buf();
    let outcome = store.load();
    let writes_enabled = outcome.is_writable();
    let settings = outcome.settings;
    let persistence = UserSettingsPersistence::new(path, settings.clone(), writes_enabled);
    PreparedUserSettings {
        settings,
        origin: outcome.origin,
        warning: outcome.warning,
        persistence,
    }
}

/// Resolves the requested text-locale precedence. Explicit overrides must map
/// to an available exact or base language; a syntactically valid saved locale
/// remains requested even when its pack is temporarily unavailable, while
/// [`Localization`] independently owns the effective fallback.
///
/// Precedence is command line, environment, saved preference, explicit
/// fallback, then the first available locale.
#[must_use]
pub(super) fn resolve_startup_text_locale<'a>(
    command_line: Option<&str>,
    environment: Option<&str>,
    saved: Option<&str>,
    fallback: &str,
    available: impl IntoIterator<Item = &'a str>,
) -> String {
    let available = normalized_available_locales(available);
    let explicit = [command_line, environment]
        .into_iter()
        .flatten()
        .filter_map(normalize_locale)
        .find(|candidate| resolve_available_locale(candidate, &available).is_some());
    explicit
        .or_else(|| saved.and_then(normalize_locale))
        .or_else(|| {
            normalize_locale(fallback)
                .filter(|candidate| resolve_available_locale(candidate, &available).is_some())
        })
        .or_else(|| available.first().cloned())
        .or_else(|| normalize_locale(fallback))
        .unwrap_or_else(|| ffone_client::localization::DEFAULT_LANGUAGE.to_owned())
}

/// Resolves the independently persisted voice locale against the voice
/// catalog. Exact and base-language matches win; otherwise the effective text
/// language and finally the first available voice locale provide fallbacks.
#[must_use]
pub(super) fn resolve_startup_voice_language<'a>(
    saved: Option<&str>,
    text: &Language,
    available: impl IntoIterator<Item = &'a str>,
) -> VoiceLanguage {
    let available = normalized_available_locales(available);
    let saved_requested = saved.and_then(normalize_locale);
    let text_requested =
        normalize_locale(&text.requested).or_else(|| normalize_locale(&text.effective));
    let text_effective = normalize_locale(&text.effective);

    let effective = saved_requested
        .as_deref()
        .and_then(|locale| resolve_available_locale(locale, &available))
        .or_else(|| {
            text_effective
                .as_deref()
                .and_then(|locale| resolve_available_locale(locale, &available))
        })
        .or_else(|| available.first().cloned())
        .or_else(|| text_effective.clone())
        .unwrap_or_else(|| "en".to_owned());
    let requested = saved_requested
        .or(text_requested)
        .unwrap_or_else(|| effective.clone());

    VoiceLanguage {
        requested,
        effective,
    }
}

fn normalized_available_locales<'a>(available: impl IntoIterator<Item = &'a str>) -> Vec<String> {
    let mut normalized = Vec::new();
    for locale in available.into_iter().filter_map(normalize_locale) {
        if !normalized.contains(&locale) {
            normalized.push(locale);
        }
    }
    normalized
}

fn normalize_locale(locale: &str) -> Option<String> {
    let normalized = locale.trim().to_ascii_lowercase().replace('_', "-");
    let valid = !normalized.is_empty()
        && normalized.len() <= 35
        && !normalized.starts_with('-')
        && !normalized.ends_with('-')
        && !normalized.contains("--")
        && normalized
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    valid.then_some(normalized)
}

fn resolve_available_locale(candidate: &str, available: &[String]) -> Option<String> {
    if available.iter().any(|locale| locale == candidate) {
        return Some(candidate.to_owned());
    }
    let base = candidate.split('-').next().unwrap_or_default();
    available
        .iter()
        .find(|locale| locale.as_str() == base)
        .cloned()
}

/// Applies the non-option startup preference to the plugin-owned selection
/// model after that plugin has inserted its resource.
pub(super) fn apply_startup_character_selection_settings(
    settings: &UserSettings,
    model: &mut CharacterSelectionUiModel,
) {
    model.music_enabled = settings.character_selection_music;
}

/// Collects the one durable document from all committed live owners.
#[must_use]
pub(super) fn collect_user_settings_snapshot(
    options: &OptionProductionRuntime,
    text: &Language,
    voice: &VoiceLanguage,
    character_selection: &CharacterSelectionUiModel,
) -> UserSettings {
    UserSettings {
        map: Default::default(),
        options: options.options.clone(),
        input: options.input.clone(),
        text_locale: text.requested.clone(),
        voice_locale: voice.requested.clone(),
        character_selection_music: character_selection.music_enabled,
        window_maximized: false,
    }
}

/// Public ordering point for the synchronous durable-settings commit.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(super) enum UserSettingsSet {
    Flush,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UserSettingsFlushOutcome {
    Unchanged,
    WritesDisabled,
    Saved,
}

/// Compare-and-save core shared by the Bevy system and focused unit tests.
///
/// `last_observed` advances even after a write error so a read-only directory
/// does not cause an I/O attempt and warning on every rendered frame. Any later
/// settings change produces a new snapshot and therefore another retry.
pub(super) fn flush_user_settings_snapshot(
    persistence: &mut UserSettingsPersistence,
    mut snapshot: UserSettings,
) -> Result<UserSettingsFlushOutcome, SettingsSaveError> {
    if let Some(override_locale) = persistence.startup_text_override.as_deref() {
        if snapshot.text_locale == override_locale {
            snapshot.text_locale = persistence.last_observed.text_locale.clone();
        } else {
            persistence.startup_text_override = None;
        }
    }
    if &snapshot == persistence.last_observed() {
        return Ok(UserSettingsFlushOutcome::Unchanged);
    }
    persistence.last_observed = snapshot.clone();
    if !persistence.writes_enabled() {
        return Ok(UserSettingsFlushOutcome::WritesDisabled);
    }
    UserSettingsStore::new(persistence.path.clone()).save(&snapshot)?;
    Ok(UserSettingsFlushOutcome::Saved)
}

/// Synchronous end-of-frame flush. The JSON document is small, and the store
/// performs an atomic sibling-file replacement, so keeping commit ordering on
/// the main schedule is preferable to racing multiple background saves.
pub(super) fn flush_user_settings(
    world_map: Res<ffone_client::world_map::WorldMapPresentation>,
    options: Res<OptionProductionRuntime>,
    text: Res<Language>,
    voice: Res<VoiceLanguage>,
    character_selection: Res<CharacterSelectionUiModel>,
    mut persistence: ResMut<UserSettingsPersistence>,
    #[cfg(windows)] windows: Query<(Entity, &Window), With<bevy::window::PrimaryWindow>>,
    #[cfg(windows)] _non_send_marker: NonSendMarker,
    #[cfg(windows)] mut placement_initialized: Local<bool>,
) {
    let mut snapshot =
        collect_user_settings_snapshot(&options, &text, &voice, &character_selection);
    // Preserve placement while fullscreen or minimized: neither state tells
    // us how the decorated window should be restored on the next launch.
    snapshot.map = world_map.model.preferences.clone();
    snapshot.window_maximized = persistence.last_observed.window_maximized;
    #[cfg(windows)]
    if *placement_initialized
        && let Ok((entity, window)) = windows.single()
        && matches!(window.mode, bevy::window::WindowMode::Windowed)
    {
        bevy_winit::WINIT_WINDOWS.with_borrow(|windows| {
            if let Some(window) = windows.get_window(entity)
                && window.is_minimized() != Some(true)
            {
                snapshot.window_maximized = window.is_maximized();
            }
        });
    }
    // Startup's maximize request reaches winit in Last, after this first flush.
    #[cfg(windows)]
    {
        *placement_initialized = true;
    }
    if let Err(error) = flush_user_settings_snapshot(&mut persistence, snapshot) {
        warn!(
            "could not persist user settings to {}: {error}",
            persistence.path.display()
        );
    }
}

#[cfg(test)]
mod tests;
