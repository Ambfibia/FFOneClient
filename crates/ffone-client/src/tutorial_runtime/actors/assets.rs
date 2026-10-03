use super::*;

#[derive(Debug, Default, Resource)]
pub struct TutorialActorRegistry {
    pub(super) by_id: BTreeMap<i32, Entity>,
}

impl TutorialActorRegistry {
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    #[must_use]
    pub fn entity(&self, id: i32) -> Option<Entity> {
        self.by_id.get(&id).copied()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (i32, Entity)> + '_ {
        self.by_id.iter().map(|(&id, &entity)| (id, entity))
    }
}
