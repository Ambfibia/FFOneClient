use super::*;

#[derive(Debug)]
pub enum TransportationCatalogError {
    Io { path: String, detail: String },
    Invalid(String),
}

impl fmt::Display for TransportationCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, detail } => write!(formatter, "{path}: {detail}"),
            Self::Invalid(detail) => formatter.write_str(detail),
        }
    }
}

impl std::error::Error for TransportationCatalogError {}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationTurboCheck;
