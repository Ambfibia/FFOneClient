
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectLocaleGate {
    Any,
    LegacyLocalEquals(i32),
}
