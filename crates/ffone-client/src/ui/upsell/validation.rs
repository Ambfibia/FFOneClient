
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpsellUiError {
    InvalidLevel(i32),
    NotNewsMode,
    InvalidNewsPagePath { index: usize },
}
