use super::*;

pub(super) fn meaningful_source_texture(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value.eq_ignore_ascii_case("white")
        && !value.eq_ignore_ascii_case("builtin:white")
}

pub(super) fn source_texture_matches(source: &str, expected_true_name: &str) -> bool {
    let source = source
        .trim()
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(source)
        .strip_suffix(".dds")
        .or_else(|| source.strip_suffix(".DDS"))
        .unwrap_or(source);
    source.eq_ignore_ascii_case(expected_true_name)
}
