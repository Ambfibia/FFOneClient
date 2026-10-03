use super::*;

pub(super) fn parse_selector(
    value: Option<std::ffi::OsString>,
    label: &str,
    default: u8,
    max: u8,
) -> u8 {
    let Some(value) = value else {
        return default;
    };
    let value = value
        .into_string()
        .unwrap_or_else(|_| panic!("{label} selector is not UTF-8"));
    let value = value
        .parse::<u8>()
        .unwrap_or_else(|_| panic!("{label} selector {value:?} is not an integer"));
    assert!(
        value <= max,
        "{label} selector {value} is outside 0..={max}"
    );
    value
}
