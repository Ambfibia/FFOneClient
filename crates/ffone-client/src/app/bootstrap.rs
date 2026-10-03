use crate::app::*;

pub(crate) fn entry() -> ExitCode {
    let mut config = match ClientConfig::from_args(env::args().skip(1)) {
        Ok(Some(config)) => config,
        Ok(None) => {
            println!("{HELP}");
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            eprintln!("error: {error}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    if let Err(error) = config.make_asset_roots_absolute() {
        eprintln!("error: {error}");
        return ExitCode::from(2);
    }

    let mut prepared_user_settings = match if performance_probe::requested() {
        Ok(user_settings_without_persistence(
            "Offline performance capture; settings persistence disabled".into(),
        ))
    } else {
        prepare_user_settings()
    } {
        Ok(settings) => settings,
        Err(error) => {
            let warning =
                format!("could not resolve the platform-local user-settings path: {error}");
            user_settings_without_persistence(warning)
        }
    };
    if let Some(warning) = prepared_user_settings.warning.as_deref() {
        eprintln!("warning: user settings: {warning}");
    }

    let asset_locator = match AssetLocator::open(&config.asset_root) {
        Ok(assets) => assets,
        Err(error) => {
            eprintln!(
                "native project-assets validation failed for {}: {error}\n\n\
                 Import them with:\n  cargo run --manifest-path ../FusionForge/Cargo.toml -p ffone-asset-pipeline --bin ffone-asset-pipeline -- import \
                 --pack <NATIVE_PACK> --output {}",
                config.asset_root.display(),
                config.asset_root.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let (localization, mut language) =
        match Localization::open(&config.asset_root, DEFAULT_LANGUAGE) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("native localization validation failed: {error}");
                return ExitCode::FAILURE;
            }
        };
    let requested_text_locale = resolve_startup_text_locale(
        config.language_command_line.as_deref(),
        config.language_environment.as_deref(),
        Some(prepared_user_settings.settings.text_locale.as_str()),
        DEFAULT_LANGUAGE,
        localization.locales(),
    );
    if config.language_command_line.is_some() || config.language_environment.is_some() {
        prepared_user_settings.retain_saved_text_during_override(&requested_text_locale);
    }
    localization.select(&mut language, &requested_text_locale);
    config.language = requested_text_locale;
    let diagnostics = asset_locator.diagnostics();
    if config.validate_assets {
        if let Err(error) = (|| -> Result<(), String> {
            NativeAudioCatalog::open(&config.asset_root, false)?;
            let models: serde_json::Value = asset_locator.read_character_models()?;
            for model in models["models"].as_array().ok_or("missing model routes")? {
                asset_locator
                    .require_file(model["glb"].as_str().ok_or("model has no GLB path")?)?;
            }
            Ok(())
        })() {
            eprintln!("native TableData asset validation failed: {error}");
            return ExitCode::FAILURE;
        }
        return match diagnostics.to_pretty_json() {
            Ok(summary) => {
                print!("{summary}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("failed to serialize asset summary: {error}");
                ExitCode::FAILURE
            }
        };
    }
    let pending_login = if performance_probe::requested() {
        PendingLogin {
            credentials: None,
            note: String::new(),
        }
    } else {
        PendingLogin::from_environment(&config)
    };
    if config.network_smoke {
        return run_network_smoke(&config, pending_login);
    }
    if let Err(error) = config.validate_character_asset() {
        eprintln!("native character asset validation failed: {error}");
        return ExitCode::FAILURE;
    }

    // The unified map catalog currently contains the complete authored
    // resource provenance and 170 scene manifests. Parse/hash it alongside
    // the independent audio/player catalogs instead of serializing all of
    // that startup I/O before the application can begin initialization.
    let world_asset_root = config.asset_root.clone();
    let native_world_catalog_task = match thread::Builder::new()
        .name("native-world-catalog".to_owned())
        .spawn(move || {
            load_native_world_scenes(world_asset_root).map_err(|error| error.to_string())
        }) {
        Ok(task) => task,
        Err(error) => {
            eprintln!("failed to start native world-catalog loader: {error}");
            return ExitCode::FAILURE;
        }
    };
    let native_audio_catalog = match NativeAudioCatalog::open(&config.asset_root, false) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("native semantic-audio catalog validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let voice_language = resolve_startup_voice_language(
        Some(prepared_user_settings.settings.voice_locale.as_str()),
        &language,
        native_audio_catalog.voice_locales(),
    );
    let retrobution_world_audio_catalog =
        match RetrobutionWorldAudioCatalog::open(&asset_locator, &native_audio_catalog) {
            Ok(catalog) => catalog,
            Err(error) => {
                eprintln!("native Retrobution world-audio validation failed: {error}");
                return ExitCode::FAILURE;
            }
        };
    let character_creation_data = match CharacterCreationData::open(&config.asset_root) {
        Ok(data) => data,
        Err(error) => {
            eprintln!("native character-creation data validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    let native_player_rig_catalog = match NativePlayerRigCatalog::open(&config.asset_root) {
        Ok(catalog) => catalog,
        Err(error) => {
            eprintln!("native shared player-rig validation failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    run(
        config,
        asset_locator,
        localization,
        language,
        voice_language,
        prepared_user_settings,
        native_world_catalog_task,
        native_audio_catalog,
        retrobution_world_audio_catalog,
        character_creation_data,
        native_player_rig_catalog,
        pending_login,
    )
}

impl ClientConfig {
    pub(super) fn from_args(
        args: impl IntoIterator<Item = String>,
    ) -> Result<Option<Self>, String> {
        let mut asset_root = PathBuf::from("assets/game");
        let mut asset_root_explicit = false;
        let mut login_address = "127.0.0.1:23000".to_owned();
        let mut login_user = None;
        let mut password_env = "FFONE_PASSWORD".to_owned();
        let mut character_uid = None;
        let mut character_asset_root = PathBuf::from(DEFAULT_CHARACTER_ASSET_ROOT);
        let mut character_asset_root_explicit = false;
        let mut character_model = DEFAULT_CHARACTER_MODEL.to_owned();
        let mut character_root = DEFAULT_CHARACTER_ROOT.to_owned();
        let mut character_animation = DEFAULT_CHARACTER_ANIMATION.to_owned();
        let mut validate_assets = false;
        let mut network_smoke = false;
        let language_environment = env::var("FFONE_LANGUAGE").ok();
        let mut language_command_line = None;
        let mut language = language_environment
            .clone()
            .unwrap_or_else(|| DEFAULT_LANGUAGE.to_owned());
        let mut args = args.into_iter();

        while let Some(flag) = args.next() {
            if matches!(flag.as_str(), "-h" | "--help") {
                return Ok(None);
            }
            if flag == "--validate-assets" {
                validate_assets = true;
                continue;
            }
            if flag == "--network-smoke" {
                network_smoke = true;
                continue;
            }
            let value = args
                .next()
                .ok_or_else(|| format!("{flag} requires a value"))?;
            match flag.as_str() {
                "--asset-root" => {
                    asset_root = PathBuf::from(value);
                    asset_root_explicit = true;
                }
                "--login-address" => login_address = value,
                "--login-user" => login_user = Some(value),
                "--password-env" => password_env = value,
                "--character" => {
                    character_uid = Some(
                        value
                            .parse::<i64>()
                            .map_err(|_| format!("invalid character UID {value:?}"))?,
                    );
                }
                "--character-asset-root" => {
                    character_asset_root = PathBuf::from(value);
                    character_asset_root_explicit = true;
                }
                "--character-model" => character_model = value,
                "--character-root" => character_root = value,
                "--character-animation" => character_animation = value,
                "--language" => {
                    language.clone_from(&value);
                    language_command_line = Some(value);
                }
                _ => return Err(format!("unknown option: {flag}")),
            }
        }

        if password_env.is_empty() {
            return Err("--password-env may not be empty".to_owned());
        }
        validate_native_relative_path("--character-model", &character_model)?;
        if !character_model.to_ascii_lowercase().ends_with(".glb") {
            return Err("--character-model must name a GLB file".to_owned());
        }
        if character_root.is_empty() {
            return Err("--character-root may not be empty".to_owned());
        }
        if character_animation.is_empty() {
            return Err("--character-animation may not be empty".to_owned());
        }
        Ok(Some(Self {
            asset_root,
            asset_root_explicit,
            login_address,
            login_user,
            password_env,
            character_uid,
            character_asset_root,
            character_asset_root_explicit,
            character_model,
            character_root,
            character_animation,
            validate_assets,
            network_smoke,
            language,
            language_command_line,
            language_environment,
        }))
    }

    pub(super) fn validate_character_asset(&self) -> Result<(), String> {
        let registry =
            ffone_client_foundation::asset_tables::character_models(&self.character_asset_root)?;
        let model_entry = registry
            .get("models")
            .and_then(serde_json::Value::as_array)
            .and_then(|models| {
                models.iter().find(|model| {
                    model.get("glb").and_then(serde_json::Value::as_str)
                        == Some(self.character_model.as_str())
                        && model.get("logicalName").and_then(serde_json::Value::as_str)
                            == Some(self.character_root.as_str())
                })
            })
            .ok_or_else(|| {
                format!(
                    "generated character registry does not map {:?} to exact root {:?}",
                    self.character_model, self.character_root
                )
            })?;
        let entry_id = model_entry
            .get("id")
            .and_then(serde_json::Value::as_str)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| {
                format!(
                    "generated character registry has no stable ID for {:?}",
                    self.character_model
                )
            })?;
        let has_animation = model_entry
            .get("animations")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|animations| {
                animations
                    .iter()
                    .any(|animation| animation.as_str() == Some(self.character_animation.as_str()))
            });
        if !has_animation {
            return Err(format!(
                "character registry entry {entry_id:?} does not contain animation {:?}",
                self.character_animation
            ));
        }
        let model = self.character_asset_root.join(
            self.character_model
                .split('/')
                .fold(PathBuf::new(), |path, component| path.join(component)),
        );
        let metadata = fs::metadata(&model)
            .map_err(|error| format!("missing native GLB {}: {error}", model.display()))?;
        if !metadata.is_file() {
            return Err(format!(
                "native character GLB is not a file: {}",
                model.display()
            ));
        }
        Ok(())
    }

    fn make_asset_roots_absolute(&mut self) -> Result<(), String> {
        let current_dir = env::current_dir()
            .map_err(|error| format!("failed to resolve current working directory: {error}"))?;
        let executable_dir = env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf));
        self.make_asset_roots_absolute_from(&current_dir, executable_dir.as_deref());
        Ok(())
    }

    pub(super) fn make_asset_roots_absolute_from(
        &mut self,
        current_dir: &Path,
        executable_dir: Option<&Path>,
    ) {
        self.asset_root = resolve_startup_asset_root(
            &self.asset_root,
            self.asset_root_explicit,
            current_dir,
            executable_dir,
        );
        self.character_asset_root = resolve_startup_asset_root(
            &self.character_asset_root,
            self.character_asset_root_explicit,
            current_dir,
            executable_dir,
        );
    }
}

fn resolve_startup_asset_root(
    configured: &Path,
    explicit: bool,
    current_dir: &Path,
    executable_dir: Option<&Path>,
) -> PathBuf {
    if configured.is_absolute() {
        return configured.to_path_buf();
    }
    let current_dir_candidate = current_dir.join(configured);
    if explicit || current_dir_candidate.is_dir() {
        return current_dir_candidate;
    }
    executable_dir
        .map(|directory| directory.join(configured))
        .filter(|candidate| candidate.is_dir())
        .unwrap_or(current_dir_candidate)
}

fn validate_native_relative_path(flag: &str, value: &str) -> Result<(), String> {
    if value.is_empty() || value.contains('\\') {
        return Err(format!("{flag} must be a non-empty forward-slash path"));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("{flag} must stay below its native asset root"));
    }
    Ok(())
}
