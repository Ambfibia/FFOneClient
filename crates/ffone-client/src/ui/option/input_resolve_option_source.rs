use super::*;

pub(super) fn resolve_option_source(
    localization: Option<&Localization>,
    language: Option<&Language>,
    source: &str,
) -> String {
    let Some(localized) = option_localized_text(source) else {
        return source.to_owned();
    };
    match (localization, language) {
        (Some(localization), Some(language)) => localization.text(language, &localized),
        _ => source.to_owned(),
    }
}
