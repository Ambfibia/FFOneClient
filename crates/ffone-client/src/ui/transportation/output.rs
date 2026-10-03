use super::*;

#[derive(Clone, Debug, Resource, PartialEq, Eq)]
pub struct TransportationUiCopy {
    pub title: String,
    pub where_to: String,
    pub unregistered: String,
    pub turbo_travel: String,
    pub go_now: String,
}

impl Default for TransportationUiCopy {
    fn default() -> Self {
        Self {
            title: "TRANSPORTATION".to_owned(),
            where_to: "WHERE TO?".to_owned(),
            unregistered: "Unregistered".to_owned(),
            turbo_travel: "TURBO TRAVEL".to_owned(),
            go_now: "GO NOW!".to_owned(),
        }
    }
}
