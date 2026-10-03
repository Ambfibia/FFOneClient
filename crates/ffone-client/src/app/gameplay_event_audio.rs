/// One opening cue per NPC interaction owner, plus each accepted local chat echo.
/// The caller retains the owner while its offer/reward journal is active.
pub(super) fn gameplay_event_audio(
    outgoing: usize,
    npc: Option<i32>,
    previous_npc: &mut Option<i32>,
) -> impl Iterator<Item = &'static str> {
    let opening = (npc.is_some() && npc != *previous_npc).then_some("Open_Screen");
    *previous_npc = npc;
    opening
        .into_iter()
        .chain(std::iter::repeat_n("Outgoing_chat", outgoing))
}

#[cfg(test)]
mod tests;
