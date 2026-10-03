use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VendorUiAudioCue0104 {
    WeaponEquipped,
    ClothingEquipped,
    ButtonSound,
    TabClick01,
    Purchase { random_pitch: bool },
    Money,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct VendorUiAudioOutbox0104 {
    pub(super) cues: VecDeque<VendorUiAudioCue0104>,
}

impl VendorUiAudioOutbox0104 {
    pub fn push(&mut self, cue: VendorUiAudioCue0104) {
        self.cues.push_back(cue);
    }

    pub fn pop_front(&mut self) -> Option<VendorUiAudioCue0104> {
        self.cues.pop_front()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.cues.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cues.is_empty()
    }

    pub fn clear(&mut self) {
        self.cues.clear();
    }
}
