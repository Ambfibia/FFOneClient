
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigAnimationTarget {
    Player,
    Nano,
    Npc(i32),
    NpcRange { first: i32, last: i32 },
}
