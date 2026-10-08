use super::*;

pub const DEFAULT_LANGUAGE: &str = "ru";

#[derive(Clone, Debug, Resource)]
pub struct Localization {
    pub(super) fallback: String,
    pub(super) bundles: BTreeMap<String, BTreeMap<String, String>>,
    /// Compatibility index for older parity slices that still construct a
    /// canonical English label before attaching a semantic key. Ambiguous
    /// source strings (for example the gendered uses of `MEDIUM`) are omitted
    /// and therefore still require an explicit [`LocalizedText`] component.
    pub(super) fallback_keys_by_source: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct Language {
    pub requested: String,
    pub effective: String,
}

/// Independently selected locale for localized voice assets.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct VoiceLanguage {
    pub requested: String,
    pub effective: String,
}

impl VoiceLanguage {
    pub fn select(&mut self, requested: &str) {
        let requested = normalize_language(requested);
        self.requested.clone_from(&requested);
        self.effective = requested;
    }
}

/// Semantic identity for a voice player whose locale must follow
/// [`VoiceLanguage`]. The audio catalog remains the only owner of localized paths
/// and fallback behavior; UI/runtime code must never construct a locale path.
#[derive(Clone, Debug, Component, Eq, PartialEq)]
pub struct LocalizedVoice {
    pub true_name: String,
}

impl LocalizedVoice {
    #[must_use]
    pub fn by_true_name(true_name: impl Into<String>) -> Self {
        Self {
            true_name: true_name.into(),
        }
    }
}

impl From<&Language> for VoiceLanguage {
    fn from(language: &Language) -> Self {
        Self {
            requested: language.requested.clone(),
            effective: language.effective.clone(),
        }
    }
}

#[derive(Clone, Debug, Component, Eq, PartialEq)]
pub struct LocalizedText {
    pub key: String,
    pub fallback: String,
    pub args: BTreeMap<String, String>,
}

/// Presentation-only casing applied after resolving a semantic localization
/// key. This preserves the localized source identity while reproducing legacy
/// fonts whose lowercase glyphs were authored as uppercase display forms.
#[derive(Clone, Copy, Debug, Component, Eq, PartialEq)]
pub enum LocalizedTextCase {
    Uppercase,
}

/// Presentation limit applied after translation, measured in UTF-16 units.
/// Long values reserve up to three units for an ellipsis.
#[derive(Clone, Copy, Debug, Component, Eq, PartialEq)]
pub struct LocalizedTextLimit(pub usize);

impl LocalizedTextLimit {
    #[must_use]
    pub fn apply(self, text: &str) -> String {
        if text.encode_utf16().count() <= self.0 {
            return text.to_owned();
        }
        let mut used = 0;
        let mut shortened: String = text
            .chars()
            .take_while(|character| {
                used += character.len_utf16();
                used <= self.0.saturating_sub(3)
            })
            .collect();
        shortened.push_str(&".".repeat(self.0.min(3)));
        shortened
    }
}

impl LocalizedText {
    /// Creates key-first text. New UI code should use this constructor so
    /// source wording can change without changing the localization identity.
    #[must_use]
    pub fn new(key: impl Into<String>, fallback: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            fallback: fallback.into(),
            args: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn source(fallback: &'static str) -> Self {
        Self::new(
            semantic_key_for_source(fallback).unwrap_or(fallback),
            fallback,
        )
    }

    #[must_use]
    pub fn with_arg(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.insert(name.into(), value.into());
        self
    }
}

impl Localization {
    pub fn open(asset_root: &Path, requested: &str) -> Result<(Self, Language), String> {
        let locator = AssetLocator::open(asset_root)?;
        let catalog_path = locator.path(LOCALIZATION_CATALOG_PATH)?;
        let catalog: LocalizationCatalog = locator.read_json(LOCALIZATION_CATALOG_PATH)?;
        if catalog.schema != "ffone.localization-catalog.v1" {
            return Err(format!(
                "unsupported localization catalog schema {:?} in {}",
                catalog.schema,
                catalog_path.display()
            ));
        }
        validate_locale(&catalog.fallback)?;
        let mut bundles = BTreeMap::new();
        let mut routes = BTreeSet::new();
        for entry in catalog.locales {
            let locale = normalize_language(&entry.id);
            validate_locale(&locale)?;
            validate_localization_route(&entry.text)?;
            if locale != entry.id {
                return Err(format!(
                    "localization locale id must be normalized: {:?}",
                    entry.id
                ));
            }
            if !routes.insert(entry.text.clone()) {
                return Err(format!(
                    "duplicate localization bundle route {:?}",
                    entry.text
                ));
            }
            let path = locator.path(&entry.text)?;
            let bytes = locator.read(&entry.text)?;
            let bundle: TextBundle = serde_json::from_slice(&bytes)
                .map_err(|error| format!("invalid {}: {error}", path.display()))?;
            if bundle.schema != "ffone.text-bundle.v1" || bundle.locale != entry.id {
                return Err(format!(
                    "localization bundle identity mismatch for {}",
                    path.display()
                ));
            }
            if bundles.insert(locale, bundle.entries).is_some() {
                return Err("duplicate localization locale".to_owned());
            }
        }
        if bundles.is_empty() {
            return Err("localization catalog has no locales".to_owned());
        }
        let fallback = normalize_language(&catalog.fallback);
        if !bundles.contains_key(&fallback) {
            return Err(format!(
                "localization fallback {fallback:?} is not present in the catalog"
            ));
        }
        if let Ok(document) = locator.read_json::<serde_json::Value>(crate::assets::TABLE_SET_PATH) {
            for (key, source) in super::mission_links::mission_text_links(&document) {
                for bundle in bundles.values_mut() {
                    if let Some(text) = bundle.get(&source).cloned() {
                        bundle.entry(key.clone()).or_insert(text);
                    }
                }
            }
        }
        let requested = normalize_language(requested);
        let effective = resolve_locale(&bundles, &requested, &fallback);
        let fallback_keys_by_source = unique_fallback_source_keys(
            bundles
                .get(&fallback)
                .expect("fallback bundle checked above"),
        );
        let localization = Self {
            fallback,
            bundles,
            fallback_keys_by_source,
        };
        localization.validate_canonical_keys()?;
        Ok((
            localization,
            Language {
                requested,
                effective,
            },
        ))
    }

    #[must_use]
    pub fn text(&self, language: &Language, localized: &LocalizedText) -> String {
        let template = self
            .bundles
            .get(&language.effective)
            .and_then(|bundle| bundle.get(&localized.key))
            .or_else(|| {
                self.bundles
                    .get(&self.fallback)
                    .and_then(|bundle| bundle.get(&localized.key))
            })
            .map(String::as_str)
            .unwrap_or(&localized.fallback);
        localized
            .args
            .iter()
            .fold(template.to_owned(), |text, (name, value)| {
                text.replace(&format!("{{{name}}}"), value)
            })
    }

    pub(super) fn validate_canonical_keys(&self) -> Result<(), String> {
        let canonical = self
            .bundles
            .get(&self.fallback)
            .expect("fallback checked above");
        for key in REQUIRED_UI_KEYS {
            if !canonical.contains_key(*key) {
                return Err(format!("canonical localization is missing key {key:?}"));
            }
        }
        for (locale, bundle) in &self.bundles {
            for (key, translated) in bundle {
                let Some(source) = canonical.get(key) else {
                    return Err(format!(
                        "localization bundle {locale:?} contains unknown key {key:?}"
                    ));
                };
                let source_args = template_args(source);
                let translated_args = template_args(translated);
                if source_args != translated_args {
                    return Err(format!(
                        "localization placeholder mismatch for {key:?} in {locale:?}: \
                         expected {source_args:?}, found {translated_args:?}"
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn toggle(&self, language: &mut Language) {
        let mut locales = self.bundles.keys();
        let first = locales
            .next()
            .cloned()
            .unwrap_or_else(|| self.fallback.clone());
        let next = self
            .bundles
            .keys()
            .skip_while(|locale| locale.as_str() != language.effective)
            .nth(1)
            .cloned()
            .unwrap_or(first);
        language.requested.clone_from(&next);
        language.effective = next;
    }

    pub fn select(&self, language: &mut Language, requested: &str) {
        let requested = normalize_language(requested);
        language.requested = requested.clone();
        language.effective = resolve_locale(&self.bundles, &requested, &self.fallback);
    }

    /// Locale identifiers declared by `assets/game/localization/catalog.json`.
    /// Adding a bundle to that catalog makes it selectable without recompiling.
    pub fn locales(&self) -> impl ExactSizeIterator<Item = &str> {
        self.bundles.keys().map(String::as_str)
    }

    pub(super) fn localized_text_for_source(&self, source: &str) -> Option<LocalizedText> {
        self.fallback_keys_by_source
            .get(source)
            .map(|key| LocalizedText::new(key, source))
    }
}

#[derive(Default)]
pub struct LocalizationPlugin;

/// Public ordering point for UI binders that update [`LocalizedText`].
///
/// Dynamic UI state must publish its semantic text before this set; the
/// localization system remains the only writer of the rendered [`Text`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum LocalizationSet {
    Apply,
}

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        images::register(app);
        app.add_systems(
            Update,
            (
                toggle_language,
                adopt_known_localized_texts,
                apply_localized_texts.in_set(LocalizationSet::Apply),
                refresh_localized_voice_players,
                admit_bounded_ui_text,
                refresh_ui_text_fit_regions,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            auto_fit_ui_text
                .in_set(UiSystems::PostLayout)
                .after(bevy::ui::widget::text_system),
        );
    }
}

pub(super) fn toggle_language(
    keys: Res<ButtonInput<KeyCode>>,
    localization: Res<Localization>,
    mut language: ResMut<Language>,
) {
    if keys.just_pressed(KeyCode::F9) {
        localization.toggle(&mut language);
    }
}

/// Safety net for established parity slices. New UI must attach
/// [`LocalizedText`] explicitly with a key from the authored EN/RU bundles.
pub(super) fn adopt_known_localized_texts(
    mut commands: Commands,
    localization: Res<Localization>,
    texts: Query<(Entity, &Text), (Or<(Added<Text>, Changed<Text>)>, Without<LocalizedText>)>,
) {
    for (entity, text) in &texts {
        if let Some(localized) = localization.localized_text_for_source(&text.0) {
            commands.entity(entity).insert(localized);
        }
    }
}

pub(super) fn apply_localized_texts(
    localization: Res<Localization>,
    language: Res<Language>,
    mut removed_case: RemovedComponents<LocalizedTextCase>,
    mut removed_limit: RemovedComponents<LocalizedTextLimit>,
    mut texts: Query<(
        Entity,
        Ref<LocalizedText>,
        Option<Ref<LocalizedTextCase>>,
        Option<Ref<LocalizedTextLimit>>,
        &mut Text,
    )>,
) {
    let global_change = localization.is_changed() || language.is_changed();
    let removed_case: BTreeSet<_> = removed_case.read().collect();
    let removed_limit: BTreeSet<_> = removed_limit.read().collect();
    for (entity, localized, case, limit, mut text) in &mut texts {
        // Hidden screens remain resident. Resolving all of their templates
        // every frame performs needless lookups, allocations and substitutions.
        // Text changes are included so external writes still get corrected.
        if !global_change
            && !localized.is_changed()
            && !text.is_changed()
            && !case.as_ref().is_some_and(|case| case.is_changed())
            && !removed_case.contains(&entity)
            && !limit.as_ref().is_some_and(|limit| limit.is_changed())
            && !removed_limit.contains(&entity)
        {
            continue;
        }
        let mut resolved = localization.text(&language, &localized);
        if matches!(case.as_deref(), Some(LocalizedTextCase::Uppercase)) {
            resolved = resolved.to_uppercase();
        }
        if let Some(limit) = limit {
            resolved = limit.apply(&resolved);
        }
        if text.0 != resolved {
            text.0 = resolved;
        }
    }
}

pub(super) fn refresh_localized_voice_players(
    mut commands: Commands,
    voice_language: Option<Res<VoiceLanguage>>,
    catalog: Option<Res<NativeAudioCatalog>>,
    asset_server: Option<Res<AssetServer>>,
    mut players: Query<(Entity, &LocalizedVoice, Option<&mut AudioPlayer>)>,
) {
    let (Some(voice_language), Some(catalog), Some(asset_server)) =
        (voice_language, catalog, asset_server)
    else {
        return;
    };
    if !voice_language.is_changed() {
        return;
    }
    for (entity, voice, player) in &mut players {
        match localized_voice_path(&catalog, &voice_language.effective, voice) {
            Ok(Some(path)) => {
                let handle = asset_server.load(path);
                if let Some(mut player) = player {
                    player.0 = handle;
                } else {
                    commands.entity(entity).insert(AudioPlayer::new(handle));
                }
                commands
                    .entity(entity)
                    .remove::<AudioSink>()
                    .remove::<SpatialAudioSink>();
            }
            Ok(None) => {
                commands
                    .entity(entity)
                    .remove::<AudioPlayer>()
                    .remove::<AudioSink>()
                    .remove::<SpatialAudioSink>();
            }
            Err(error) => warn!("{error}"),
        }
    }
}

pub(super) fn localized_voice_path(
    catalog: &NativeAudioCatalog,
    locale: &str,
    voice: &LocalizedVoice,
) -> Result<Option<String>, String> {
    let matches = catalog
        .by_true_name(&voice.true_name)
        .into_iter()
        .filter(|asset| asset.category == NativeAudioCategory::Voice)
        .collect::<Vec<_>>();
    let [asset] = matches.as_slice() else {
        return Err(format!(
            "localized voice {:?} resolved to {} semantic voice assets; expected exactly one",
            voice.true_name,
            matches.len()
        ));
    };
    Ok(catalog.path_for_locale(asset, locale).map(str::to_owned))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LocalizationCatalog {
    pub(super) schema: String,
    pub(super) fallback: String,
    pub(super) locales: Vec<LocalizationCatalogEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct LocalizationCatalogEntry {
    pub(super) id: String,
    pub(super) text: String,
}

pub(super) fn validate_locale(locale: &str) -> Result<(), String> {
    if !locale.is_empty()
        && !locale.starts_with('-')
        && !locale.ends_with('-')
        && locale
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !locale.contains("--")
    {
        Ok(())
    } else {
        Err(format!("invalid locale {locale:?}"))
    }
}

pub(super) fn normalize_language(language: &str) -> String {
    language.trim().to_ascii_lowercase().replace('_', "-")
}

pub(super) fn resolve_locale(
    bundles: &BTreeMap<String, BTreeMap<String, String>>,
    requested: &str,
    fallback: &str,
) -> String {
    if bundles.contains_key(requested) {
        return requested.to_owned();
    }
    let language = requested.split('-').next().unwrap_or_default();
    if bundles.contains_key(language) {
        return language.to_owned();
    }
    fallback.to_owned()
}

pub(super) fn validate_localization_route(route: &str) -> Result<(), String> {
    let path = native_path(route)?;
    if !route.starts_with("localization/")
        || path.extension().and_then(|extension| extension.to_str()) != Some("json")
        || route == LOCALIZATION_CATALOG_PATH
    {
        return Err(format!("invalid localization bundle route {route:?}"));
    }
    Ok(())
}

/// Localizes login/runtime status strings while preserving opaque server
/// details. Known UI text receives a stable key; arbitrary error payloads are
/// passed through only as a `{error}` argument.
#[must_use]
pub fn localized_status(source: &str) -> LocalizedText {
    if let Some(key) = semantic_key_for_source(source) {
        return LocalizedText::new(key, source);
    }
    if let Some(server) = source
        .strip_prefix("Connecting to ")
        .and_then(|value| value.strip_suffix("..."))
    {
        return LocalizedText::new("status.connecting", "Connecting to {server}...")
            .with_arg("server", server);
    }
    for (prefix, key, fallback) in [
        (
            "Network error: ",
            "status.login.network_error",
            "Network error: {error}",
        ),
        (
            "Login failed: ",
            "status.login.failed",
            "Login failed: {error}",
        ),
        (
            "Disconnected: ",
            "status.login.disconnected_error",
            "Disconnected: {error}",
        ),
        (
            "Offline: ",
            "status.login.offline_error",
            "Offline: {error}",
        ),
    ] {
        if let Some(error) = source.strip_prefix(prefix) {
            return LocalizedText::new(key, fallback).with_arg("error", error);
        }
    }
    LocalizedText::new("status.message", "{message}").with_arg("message", source)
}

/// Resolves the exact recovered tutorial copy through a stable semantic key.
///
/// The passthrough is deliberately only a runtime safety net. The regression
/// test below exhausts every declared tutorial stage and fails when a new
/// instruction has not been assigned a semantic key.
#[must_use]
pub fn localized_tutorial_instruction(source: &str) -> LocalizedText {
    tutorial_instruction_key(source).map_or_else(
        || LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", source),
        |key| LocalizedText::new(key, source),
    )
}

/// Assigns stable identities to literals passed directly by the recovered
/// tutorial script. These strings bypassed the legacy `TextManager`, but the
/// native client still localizes every user-visible tutorial line.
#[must_use]
pub fn localized_tutorial_literal(source: &str) -> LocalizedText {
    match source {
        "I need some help with a sooper important mission! Report to me right away!" => {
            LocalizedText::new("content.tutorial.message.minimap_numbuh_two", source)
        }
        _ => localized_tutorial_instruction(source),
    }
}

/// Resolves the exact tutorial mission TableData field through its stable
/// source string ID. Keeping this mapping beside the localization owner lets
/// both the mission journal and the HUD's current-objective panel react to a
/// live language change without treating English copy as identity.
#[must_use]
pub fn localized_tutorial_mission_text(task_id: i32, field: &str, fallback: &str) -> LocalizedText {
    LocalizedText::new(
        format!("content.mission.task.{task_id}.{field}"),
        fallback.to_owned(),
    )
}

/// Resolves the exact `NpcTableElement.m_iNpcNumber -> m_iNpcName` label used
/// by clean mission-progress rows. The numeric NPC identity remains stable
/// when the English TableData copy changes.
#[must_use]
pub fn localized_tabledata_npc_name(npc_type: i32, fallback: &str) -> LocalizedText {
    LocalizedText::new(format!("content.npc.{npc_type}.name"), fallback.to_owned())
}

/// Exact click greeting selected by `NpcIconMode.NpcGreetingBubble` through
/// `m_iNpcName -> NpcStringTable.m_strComment`.
#[must_use]
pub fn localized_tabledata_npc_greeting(string_id: i32, fallback: &str) -> LocalizedText {
    LocalizedText::new(
        format!("content.tabledata.npc.npc_string.{string_id}.str_comment"),
        fallback.to_owned(),
    )
}

/// Exact `NpcBarkerTableElement` field selected by the clean random switch.
#[must_use]
pub fn localized_tabledata_npc_barker(
    string_id: i32,
    field_index: usize,
    fallback: &str,
) -> LocalizedText {
    let field = match field_index {
        0 => "str_name",
        1 => "str_comment",
        2 => "str_comment1",
        3 => "str_comment2",
        _ => panic!("NpcBarker field index must be in 0..4"),
    };
    LocalizedText::new(
        format!("content.tabledata.npc.npc_barker.{string_id}.{field}"),
        fallback.to_owned(),
    )
}

/// Exact Mission NameString selected by `P_FE2CL_REP_BARKER`.
#[must_use]
pub fn localized_tabledata_mission_barker(string_id: i32, fallback: &str) -> LocalizedText {
    LocalizedText::new(
        format!("content.tabledata.mission.mission_string.{string_id}.str_name_string"),
        fallback.to_owned(),
    )
}

/// Exact SkillString comment selected by NPC skill/corruption READY.
#[must_use]
pub fn localized_tabledata_npc_skill_barker(string_id: i32, fallback: &str) -> LocalizedText {
    LocalizedText::new(
        format!("content.tabledata.skill.skill_string.{string_id}.str_comment1"),
        fallback.to_owned(),
    )
}

/// Resolves the exact `QuestItemTableElement.m_iItemNumber -> m_iItemName`
/// label used by clean mission-progress rows.
#[must_use]
pub fn localized_tabledata_quest_item_name(item_id: i32, fallback: &str) -> LocalizedText {
    LocalizedText::new(
        format!("content.quest_item.{item_id}.name"),
        fallback.to_owned(),
    )
}

/// Resolves the primary TableData `worldname` label through a semantic
/// location key. Repeated world-grid rectangles deliberately share one key.
#[must_use]
pub fn localized_world_location_text(source: &str) -> LocalizedText {
    let slug = source
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split('_')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if slug.is_empty() {
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", source)
    } else {
        LocalizedText::new(format!("content.location.world.{slug}"), source)
    }
}

/// Resolves recovered tutorial cut-scene and gameplay dialogue through the
/// authored locale bundles instead of exposing the English-only XDT value.
#[must_use]
pub fn localized_tutorial_scene_text(event: i32, line: i32, fallback: &str) -> LocalizedText {
    LocalizedText::new(
        format!("content.tutorial.scene.{event}.{line}"),
        fallback.to_owned(),
    )
}
