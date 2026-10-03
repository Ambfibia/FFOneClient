use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EnchantModelError0104 {
    ModeClosed,
    InputBlocked,
    OverlayOwnsInput,
    UnsupportedTargetType(i16),
    GeneralItemRequired(i16),
    TargetRequired,
    MaterialNotRequired(EnchantAttachmentSlot0104),
    WrongMaterial {
        expected: i16,
        actual: i16,
    },
    IntendedHelperRejected {
        slot: EnchantAttachmentSlot0104,
        item_id: i16,
    },
    MissingRecipe(usize),
    NothingAttached,
    TargetCannotBeEnchanted,
    MaterialQuantityInsufficient(EnchantAttachmentSlot0104),
    NotEnoughTaros,
    WrongSystemMessage,
    NotWaitingForReply,
    NotSuccessOverlay,
    CloseRejectedByInventory,
    RedeemNotOpen,
    Redeem(EnchantRedeemError0104),
}

impl fmt::Display for EnchantModelError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for EnchantModelError0104 {}
