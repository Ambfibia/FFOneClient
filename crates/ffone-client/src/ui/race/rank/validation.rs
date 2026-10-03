use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaceRankCatalogError {
    Json(String),
    WrongSchema(String),
    WrongSourceBuild(String),
    WrongLocationCount(usize),
    NonPositiveEpId(i32),
    DuplicateEpId(i32),
    WrongOrdering { previous_ep: i32, current_ep: i32 },
    WrongImagePath { ep_id: i32, path: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaceRankParseError {
    MissingSection(&'static str),
    UnterminatedScore,
    MissingAttribute(&'static str),
    WrongAttributeOrder {
        previous: &'static str,
        current: &'static str,
    },
    InvalidInteger {
        field: &'static str,
        value: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaceRankOpenError {
    EmptyCatalog,
    CurrentEpNotFound(i32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaceRankTransitionError {
    NotVisible,
    Loading,
    SlotOutOfRange(usize),
    EmptySlot(usize),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RaceRankCompletionError {
    NoPendingRequest,
    StaleRequest { expected: u64, received: u64 },
    Parse(RaceRankParseError),
}
