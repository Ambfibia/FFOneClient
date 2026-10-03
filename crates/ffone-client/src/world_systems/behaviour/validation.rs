use super::*;

#[derive(Debug)]
pub enum WorldBehaviourError {
    Io {
        path: String,
        source: std::io::Error,
    },
    Json {
        path: String,
        source: serde_json::Error,
    },
    Contract(String),
}

impl std::fmt::Display for WorldBehaviourError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "could not read {path}: {source}"),
            Self::Json { path, source } => write!(
                formatter,
                "invalid native world behaviour document {path}: {source}"
            ),
            Self::Contract(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for WorldBehaviourError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            Self::Contract(_) => None,
        }
    }
}
