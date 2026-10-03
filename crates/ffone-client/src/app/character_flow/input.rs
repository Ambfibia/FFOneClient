use super::*;

pub(in super::super) fn resolve_current_creator(
    data: &CharacterCreationData,
    session: &CharacterCreationSession,
    appearance: &ffone_client::character_creation_ui::CharacterAppearance,
) -> Result<ResolvedCreatorSelection, String> {
    let saved = session
        .saved_name
        .as_ref()
        .ok_or_else(|| "character appearance has no reserved OpenFusion identity".to_owned())?;
    data.resolve_creator(
        saved.pc_uid,
        if session.generated_name { 1 } else { 0 },
        &saved.first_name.to_string_lossy(),
        &saved.last_name.to_string_lossy(),
        appearance,
    )
    .map_err(|error| error.to_string())
}
