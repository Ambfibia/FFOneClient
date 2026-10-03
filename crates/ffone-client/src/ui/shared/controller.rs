//! Screen-owned navigation hints, independent of the input device adapter.
use bevy::prelude::*;

/// Confirm edge for controls whose pointer path normally starts a drag.
#[derive(Resource, Default)]
pub struct ControllerUiInput {
    pub confirmed: Option<Entity>,
}

#[derive(Component, Default)]
pub struct ControllerUiClose;

#[derive(Component, Default)]
pub struct ControllerUiDefault;

/// Repeat a discrete action while confirm is held (continuous controls need no hint).
#[derive(Component, Default)]
pub struct ControllerUiRepeat;

#[derive(Component, Default)]
pub struct ControllerUiIgnore;

/// A child popup captures focus independently of its parent window.
#[derive(Component, Default)]
pub struct ControllerUiBoundary;

#[derive(Component, Clone, Copy)]
pub struct ControllerUiTab(pub u8);

/// Separate rendering roots that belong to one navigable screen.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum ControllerUiScope {
    CharacterSelection,
}
