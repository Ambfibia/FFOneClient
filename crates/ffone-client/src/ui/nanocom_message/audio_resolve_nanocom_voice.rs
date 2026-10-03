use super::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NanocomMessageSound {
    SlideIn,
    SlideOut,
    NanoCreationComplete,
    Yes,
    No,
    Voice(String),
}

#[derive(Component)]
pub(super) struct NanocomMessageAudio;

/// Resolves the requested semantic NanoCom take and, for incomplete native
/// CommOut families, deterministically wraps to another published sibling.
/// The resolved true name is returned with the localized path so
/// `LocalizedVoice` can keep following later VoiceLanguage changes.
pub(super) fn resolve_nanocom_voice(
    catalog: &NativeAudioCatalog,
    locale: &str,
    requested_true_name: &str,
) -> Option<(String, String)> {
    let exact = catalog
        .by_true_name(requested_true_name)
        .into_iter()
        .find(|asset| asset.category == NativeAudioCategory::Voice);
    let asset = if let Some(asset) = exact {
        asset
    } else {
        let folded = requested_true_name.to_ascii_lowercase();
        let marker = "_commout0";
        let marker_index = folded.rfind(marker)?;
        let requested_take = folded[marker_index + marker.len()..]
            .parse::<usize>()
            .ok()?;
        let family = &requested_true_name[..marker_index + marker.len()];
        let mut siblings = (1..=99)
            .flat_map(|take| catalog.by_true_name(&format!("{family}{take}")))
            .filter(|asset| asset.category == NativeAudioCategory::Voice)
            .collect::<Vec<_>>();
        siblings.dedup_by(|left, right| left.true_name.eq_ignore_ascii_case(&right.true_name));
        if siblings.is_empty() {
            return None;
        }
        *siblings.get(requested_take.saturating_sub(1) % siblings.len())?
    };
    Some((
        asset.true_name.clone(),
        catalog.path_for_locale(asset, locale)?.to_owned(),
    ))
}
