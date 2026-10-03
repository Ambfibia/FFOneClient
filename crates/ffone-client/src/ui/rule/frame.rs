use super::*;

pub const RULE_UI_FRAME_RECT: RuleUiRect = RuleUiRect::new(10.0, 50.0, 980.0, 540.0);

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub struct RuleUiFrame;
