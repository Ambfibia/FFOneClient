use crate::app::*;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
pub(super) enum ClientState {
    #[default]
    Bootstrap,
    Login,
    CharacterSelect,
    CharacterCreateIntro,
    CharacterCreate,
    TutorialIntro,
    Tutorial,
    World,
}
