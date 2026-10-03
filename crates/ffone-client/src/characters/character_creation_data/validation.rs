use super::*;

#[derive(Debug)]
pub enum CharacterCreationDataError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    Invalid(String),
    MissingRoute(String),
}

impl fmt::Display for CharacterCreationDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "cannot read {}: {source}", path.display())
            }
            Self::Json { path, source } => {
                write!(formatter, "cannot parse {}: {source}", path.display())
            }
            Self::Invalid(message) => formatter.write_str(message),
            Self::MissingRoute(message) => formatter.write_str(message),
        }
    }
}

impl Error for CharacterCreationDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            Self::Invalid(_) | Self::MissingRoute(_) => None,
        }
    }
}

pub(super) fn validate_documents(
    names: &CharacterCreationNameWheel,
    appearance: &CharacterCreationAppearance,
    avatar_items: &CharacterCreationAvatarItems,
    runtime_textures: &CharacterCreationRuntimeTextures,
) -> CharacterCreationDataResult<()> {
    for (label, schema, protocol, expected_schema) in [
        (
            "name wheel",
            names.schema.as_str(),
            names.protocol,
            CHARACTER_CREATION_NAME_WHEEL_SCHEMA,
        ),
        (
            "appearance",
            appearance.schema.as_str(),
            appearance.protocol,
            CHARACTER_CREATION_APPEARANCE_SCHEMA,
        ),
        (
            "avatar item",
            avatar_items.schema.as_str(),
            avatar_items.protocol,
            CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA,
        ),
        (
            "runtime texture",
            runtime_textures.schema.as_str(),
            runtime_textures.protocol,
            CHARACTER_CREATION_RUNTIME_TEXTURES_SCHEMA,
        ),
    ] {
        if schema != expected_schema || protocol != PROTOCOL_0104 {
            return invalid(format!(
                "{label} document has schema/protocol {schema}/{protocol}, expected {expected_schema}/{PROTOCOL_0104}"
            ));
        }
    }
    validate_name_codes("first", &names.first_names)?;
    validate_name_codes("middle", &names.middle_names)?;
    validate_name_codes("last", &names.last_names)?;
    if names.first_names.len() != 601
        || names.middle_names.len() != 601
        || names.last_names.len() != 602
    {
        return invalid(format!(
            "name wheel counts are {}/{}/{}, expected 601/601/602",
            names.first_names.len(),
            names.middle_names.len(),
            names.last_names.len()
        ));
    }
    if appearance.creation_rows.len() != 32 {
        return invalid(format!(
            "appearance has {} creation rows, expected 32",
            appearance.creation_rows.len()
        ));
    }
    if runtime_textures.coverage.creator_choices != appearance.choices.len() as u64
        || runtime_textures.coverage.creator_required_textures != 214
        || runtime_textures.coverage.creator_published_textures != 214
        || runtime_textures.textures.len() < 214
        || runtime_textures.coverage.avatar_published_routes
            + runtime_textures.coverage.avatar_deferred_verified_routes
            != runtime_textures.coverage.avatar_verified_unique_routes
    {
        return invalid(format!(
            "character runtime texture closure is {}/{}/{} for {} choices, expected at least 214 creator contracts and a complete avatar publication partition for {}",
            runtime_textures.coverage.creator_required_textures,
            runtime_textures.coverage.creator_published_textures,
            runtime_textures.textures.len(),
            runtime_textures.coverage.creator_choices,
            appearance.choices.len()
        ));
    }
    Ok(())
}

pub(super) fn validate_name_codes(
    label: &str,
    entries: &[ffone_runtime_contracts::NameWheelEntry],
) -> CharacterCreationDataResult<()> {
    for (index, entry) in entries.iter().enumerate() {
        if usize::from(entry.code) != index {
            return invalid(format!(
                "{label} name entry {index} carries protocol code {}",
                entry.code
            ));
        }
    }
    Ok(())
}
