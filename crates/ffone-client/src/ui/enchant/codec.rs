use super::*;

pub const ENCHANT_REQUEST_PACKET_ID_0104: u32 = 318_767_268;

pub const ENCHANT_DELETE_REQUEST_PACKET_ID_0104: u32 = 318_767_129;

pub const ENCHANT_DISASSEMBLE_REQUEST_PACKET_ID_0104: u32 = 318_767_266;

pub const ENCHANT_REDEEM_REQUEST_PACKET_ID_0104: u32 = 318_767_111;

pub const ENCHANT_SUCCESS_PACKET_ID_0104: u32 = 822_083_885;

pub const ENCHANT_FAILURE_PACKET_ID_0104: u32 = 822_083_886;

pub const ENCHANT_DELETE_SUCCESS_PACKET_ID_0104: u32 = 822_083_641;

pub const ENCHANT_DISASSEMBLE_SUCCESS_PACKET_ID_0104: u32 = 822_083_882;

pub const ENCHANT_DISASSEMBLE_FAILURE_PACKET_ID_0104: u32 = 822_083_883;

pub const ENCHANT_REQUEST_PACKET_SIZE_0104: usize = 20;

pub const ENCHANT_DELETE_REQUEST_PACKET_SIZE_0104: usize = 8;

pub const ENCHANT_DISASSEMBLE_REQUEST_PACKET_SIZE_0104: usize = 4;

pub const ENCHANT_REDEEM_REQUEST_PACKET_SIZE_0104: usize = 260;

pub const ENCHANT_SUCCESS_PACKET_SIZE_0104: usize = 64;

pub const ENCHANT_FAILURE_PACKET_SIZE_0104: usize = 24;

pub const ENCHANT_RESTRICTED_ITEM_FRAME_PATH: &str = "ui/en/combi/restricted-item-frame.png";

/// Build the exact transport-independent request emitted by
/// `Panel_PCStuffScript.DoRedeemWindow`. The legacy text field has already
/// stripped a newline before this confirmation boundary.
pub fn enchant_redeem_wire_0104(
    code: &str,
) -> Result<EnchantRedeemWire0104, EnchantRedeemError0104> {
    let utf16_len = code.encode_utf16().count();
    if utf16_len < ENCHANT_REDEEM_CODE_MIN_CHARS_0104 {
        return Err(EnchantRedeemError0104::TooShort);
    }
    if utf16_len > ENCHANT_REDEEM_CODE_MAX_CHARS_0104 {
        return Err(EnchantRedeemError0104::TooLong);
    }
    // The clean check is specifically `Contains(" ")`, not a general
    // whitespace predicate.
    if code.contains(' ') {
        return Err(EnchantRedeemError0104::ContainsSpace);
    }
    let request = FreeChatRequest0104 {
        message: FixedUtf16::from_str(&format!("/redeem {code} "))
            .map_err(|_| EnchantRedeemError0104::Utf16Capacity)?,
        emote_code: 0,
    };
    let payload: [u8; ENCHANT_REDEEM_REQUEST_PACKET_SIZE_0104] = request
        .encode()
        .try_into()
        .expect("FreeChatRequest0104::SIZE is pinned to the redeem ABI");
    Ok(EnchantRedeemWire0104 {
        packet_id: ENCHANT_REDEEM_REQUEST_PACKET_ID_0104,
        request,
        payload,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EnchantPacketError0104 {
    pub expected: usize,
    pub actual: usize,
}

impl fmt::Display for EnchantPacketError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "expected {} packet bytes, got {}",
            self.expected, self.actual
        )
    }
}

impl Error for EnchantPacketError0104 {}

pub(super) fn exact_packet_size(bytes: &[u8], expected: usize) -> Result<(), EnchantPacketError0104> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(EnchantPacketError0104 {
            expected,
            actual: bytes.len(),
        })
    }
}

pub const ENCHANT_EQUIPMENT_WIRE_ORDER_0104: [usize; ENCHANT_EQUIPMENT_SLOT_COUNT_0104] =
    [4, 5, 6, 1, 2, 3, 0, 7, 8];
