use super::*;

pub(in super::super) fn projectile_action_reverse_mode(action: ProjectileAction) -> bool {
    // BulletGenData defaults m_iSrcStyle=-1 for NPC-to-player. The explicit
    // player-to-position call passes sourceStyle=0, and BulletContainer sets
    // OniMoveScript.bReverse exactly when m_iSrcStyle >= 0.
    matches!(action, ProjectileAction::PlayerToPositionPair { .. })
}

#[derive(SystemParam)]
pub(in super::super) struct TutorialChoreographyModeVisibility<'w> {
    pub(super) barber: Option<Res<'w, ffone_client::barber::BarberModel>>,
    pub(super) race_production: Option<Res<'w, RaceProductionRuntime>>,
    pub(super) email_runtime: Option<Res<'w, EmailProductionRuntime0104>>,
    pub(super) combi_runtime: Option<Res<'w, CombiProductionRuntime0104>>,
    pub(super) enchant_runtime: Option<Res<'w, EnchantProductionRuntime0104>>,
}
