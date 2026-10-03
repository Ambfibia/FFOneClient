use super::*;

pub(super) fn enqueue_level_up(
    content: &TutorialMissionContent,
    guide: &GuideRuntime,
    previous_level: u16,
    player_level: u16,
    messages: &mut NanocomMessageUiModel,
) -> bool {
    if player_level <= previous_level {
        return false;
    }
    let Some(authority) = guide.authoritative() else {
        return false;
    };
    let raw_mentor = authority.raw_mentor();
    let Some(definition) = content.gameplay_guide_nanocom(raw_mentor) else {
        return false;
    };
    enqueue(
        content,
        raw_mentor,
        definition.level_up_string_id,
        &definition.level_up_text,
        messages,
    )
}

pub(super) fn enqueue(
    content: &TutorialMissionContent,
    raw_mentor: i16,
    string_id: i32,
    fallback: &str,
    messages: &mut NanocomMessageUiModel,
) -> bool {
    let Some(definition) = content.gameplay_guide_nanocom(raw_mentor) else {
        return false;
    };
    let Some(npc) = content.gameplay_npc(definition.npc_type) else {
        return false;
    };
    let icon_path = if raw_mentor == 5 {
        Some(GUIDE_COMPUTRESS_ICON_PATH)
    } else {
        content.gameplay_npc_portrait_icon_path(definition.npc_type)
    };
    let Some(icon_path) = icon_path else {
        return false;
    };
    messages.enqueue_type_9_localized(
        LocalizedText::new(
            format!("content.npc.{}.name", definition.npc_type),
            npc.name.clone(),
        ),
        LocalizedText::new(
            format!("content.tabledata.guide.guide_string.{string_id}.sz_string"),
            fallback,
        ),
        icon_path,
        Some(npc.move_voice_owner.as_str()),
    );
    true
}

#[cfg(test)]
mod tests;
