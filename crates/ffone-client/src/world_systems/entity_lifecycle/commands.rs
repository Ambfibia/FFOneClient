use super::*;

/// Authoritative ordinary-world NPC attack packets retained until the visual
/// runtime has resolved both the attacking NPC root and every target PC. The
/// lifecycle path still owns animation and entity HP; this queue prevents the
/// clean projectile presentation from being discarded during that update.
#[derive(Debug, Default, Resource)]
pub struct NetworkNpcAttackEventQueue0104 {
    pub(super) events: VecDeque<NpcAttackPcs0104>,
}

impl NetworkNpcAttackEventQueue0104 {
    pub fn take_all(&mut self) -> VecDeque<NpcAttackPcs0104> {
        std::mem::take(&mut self.events)
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[derive(Debug, Default, Resource)]
pub struct NetworkNpcBarkerEventQueue0104 {
    pub(super) events: VecDeque<NetworkNpcBarkerEvent0104>,
}

impl NetworkNpcBarkerEventQueue0104 {
    pub fn take_all(&mut self) -> VecDeque<NetworkNpcBarkerEvent0104> {
        std::mem::take(&mut self.events)
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}
