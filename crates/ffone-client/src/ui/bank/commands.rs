use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BankActionBlocked {
    ControlsDisabled,
    Transfer(BankTransferError0104),
}
