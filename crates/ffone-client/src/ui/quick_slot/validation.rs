use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuickSlotAssetContractError {
    pub role: QuickSlotAssetRole,
    pub path: String,
}

impl fmt::Display for QuickSlotAssetContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} path must be a relative semantic ui/.../*.png route: {}",
            self.role, self.path
        )
    }
}

impl Error for QuickSlotAssetContractError {}
