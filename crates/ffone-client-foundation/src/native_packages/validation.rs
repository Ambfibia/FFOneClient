use super::*;

#[derive(Debug)]
pub enum PackageRegistryError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    UnsupportedSchema {
        path: PathBuf,
        expected: &'static str,
        found: String,
    },
    InvalidIdentifier {
        path: PathBuf,
        field: &'static str,
        value: String,
    },
    InvalidText {
        path: PathBuf,
        field: &'static str,
        value: String,
    },
    NonUtf8Path {
        path: PathBuf,
    },
    SymlinkNotAllowed {
        path: PathBuf,
    },
    MissingDefinitionFile {
        path: PathBuf,
    },
    UnsafeRelativePath {
        path: PathBuf,
        value: String,
    },
    DuplicatePackageId {
        id: String,
        first: PathBuf,
        second: PathBuf,
    },
    DuplicateRequirement {
        package: String,
        dependency: String,
    },
    MissingDependency {
        package: String,
        dependency: String,
    },
    DependencyVersionMismatch {
        package: String,
        dependency: String,
        required: String,
        found: String,
    },
    DependencyCycle {
        packages: Vec<String>,
    },
    DuplicateCanonicalId {
        id: String,
        first: PathBuf,
        second: PathBuf,
    },
    DuplicateNetworkId {
        kind: &'static str,
        key: String,
        first: String,
        second: String,
    },
    MissingReplacementTarget {
        replacement: String,
        target: String,
    },
    ReplacementNotDependency {
        replacement: String,
        target_package: String,
    },
    ReplacementKindMismatch {
        replacement: String,
        target: String,
    },
    ReplacementNetworkMismatch {
        replacement: String,
        target: String,
        expected: String,
        found: String,
    },
    ReplacementTargetInactive {
        replacement: String,
        target: String,
    },
}

impl fmt::Display for PackageRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => {
                write!(formatter, "{}: {source}", path.display())
            }
            Self::Json { path, source } => {
                write!(formatter, "{}: {source}", path.display())
            }
            Self::UnsupportedSchema {
                path,
                expected,
                found,
            } => write!(
                formatter,
                "{} uses schema {found:?}; expected {expected:?}",
                path.display()
            ),
            Self::InvalidIdentifier { path, field, value } => write!(
                formatter,
                "{} has invalid {field} identifier {value:?}",
                path.display()
            ),
            Self::InvalidText { path, field, value } => write!(
                formatter,
                "{} has invalid {field} value {value:?}",
                path.display()
            ),
            Self::NonUtf8Path { path } => write!(formatter, "{} is not UTF-8", path.display()),
            Self::SymlinkNotAllowed { path } => {
                write!(
                    formatter,
                    "symlinked package content is not allowed: {}",
                    path.display()
                )
            }
            Self::MissingDefinitionFile { path } => {
                write!(formatter, "expected definition file {}", path.display())
            }
            Self::UnsafeRelativePath { path, value } => write!(
                formatter,
                "{} contains unsafe relative asset path {value:?}",
                path.display()
            ),
            Self::DuplicatePackageId { id, first, second } => write!(
                formatter,
                "duplicate package {id:?} in {} and {}",
                first.display(),
                second.display()
            ),
            Self::DuplicateRequirement {
                package,
                dependency,
            } => write!(
                formatter,
                "package {package:?} requires {dependency:?} more than once"
            ),
            Self::MissingDependency {
                package,
                dependency,
            } => write!(
                formatter,
                "package {package:?} requires missing package {dependency:?}"
            ),
            Self::DependencyVersionMismatch {
                package,
                dependency,
                required,
                found,
            } => write!(
                formatter,
                "package {package:?} requires {dependency:?} version {required:?}, found {found:?}"
            ),
            Self::DependencyCycle { packages } => {
                write!(
                    formatter,
                    "package dependency cycle includes {}",
                    packages.join(", ")
                )
            }
            Self::DuplicateCanonicalId { id, first, second } => write!(
                formatter,
                "duplicate canonical ID {id:?} in {} and {}",
                first.display(),
                second.display()
            ),
            Self::DuplicateNetworkId {
                kind,
                key,
                first,
                second,
            } => write!(
                formatter,
                "duplicate {kind} network ID {key} for {first:?} and {second:?}"
            ),
            Self::MissingReplacementTarget {
                replacement,
                target,
            } => write!(
                formatter,
                "definition {replacement:?} replaces missing definition {target:?}"
            ),
            Self::ReplacementNotDependency {
                replacement,
                target_package,
            } => write!(
                formatter,
                "definition {replacement:?} replaces content from non-dependency package {target_package:?}"
            ),
            Self::ReplacementKindMismatch {
                replacement,
                target,
            } => write!(
                formatter,
                "definition {replacement:?} has a different kind than target {target:?}"
            ),
            Self::ReplacementNetworkMismatch {
                replacement,
                target,
                expected,
                found,
            } => write!(
                formatter,
                "definition {replacement:?} must preserve network key {expected} from {target:?}, found {found}"
            ),
            Self::ReplacementTargetInactive {
                replacement,
                target,
            } => write!(
                formatter,
                "definition {replacement:?} cannot replace inactive target {target:?}"
            ),
        }
    }
}

impl Error for PackageRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}

pub(super) fn validate_schema(
    path: &Path,
    found: &str,
    expected: &'static str,
) -> Result<(), PackageRegistryError> {
    if found == expected {
        Ok(())
    } else {
        Err(PackageRegistryError::UnsupportedSchema {
            path: path.to_path_buf(),
            expected,
            found: found.to_owned(),
        })
    }
}

pub(super) fn validate_identifier(
    path: &Path,
    field: &'static str,
    value: &str,
) -> Result<(), PackageRegistryError> {
    let mut chars = value.chars();
    let valid_first = chars
        .next()
        .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit());
    let valid_rest = chars.all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '.' | '_' | '-')
    });
    if valid_first && valid_rest && value.len() <= 128 {
        Ok(())
    } else {
        Err(PackageRegistryError::InvalidIdentifier {
            path: path.to_path_buf(),
            field,
            value: value.to_owned(),
        })
    }
}

pub(super) fn validate_text(
    path: &Path,
    field: &'static str,
    value: &str,
) -> Result<(), PackageRegistryError> {
    if !value.is_empty() && value.len() <= 128 && !value.chars().any(char::is_control) {
        Ok(())
    } else {
        Err(PackageRegistryError::InvalidText {
            path: path.to_path_buf(),
            field,
            value: value.to_owned(),
        })
    }
}

pub(super) fn validate_assets(
    source_path: &Path,
    raw_assets: BTreeMap<String, String>,
) -> Result<BTreeMap<String, SafeRelativePath>, PackageRegistryError> {
    raw_assets
        .into_iter()
        .map(|(name, value)| {
            validate_identifier(source_path, "asset name", &name)?;
            let path = validate_relative_path(source_path, value)?;
            Ok((name, path))
        })
        .collect()
}

pub(super) fn validate_relative_path(
    source_path: &Path,
    value: String,
) -> Result<SafeRelativePath, PackageRegistryError> {
    let segments_are_safe = !value.is_empty()
        && !value.starts_with('/')
        && !value.ends_with('/')
        && !value.contains('\\')
        && !value.contains(':')
        && !value.chars().any(char::is_control)
        && value
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..");
    if segments_are_safe && !Path::new(&value).is_absolute() {
        Ok(SafeRelativePath(value))
    } else {
        Err(PackageRegistryError::UnsafeRelativePath {
            path: source_path.to_path_buf(),
            value,
        })
    }
}
