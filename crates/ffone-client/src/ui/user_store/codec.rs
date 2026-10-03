use super::*;

pub const USER_STORE_RESTRICTED_FRAME_PATH: &str = "ui/en/vendor/restricted-item-frame.png";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserStoreCodecError0104 {
    UnexpectedPacketId(u32),
    ExactSize {
        packet_id: u32,
        expected: usize,
        actual: usize,
    },
    NegativeListCount(i32),
    ListCountOverMaximum(i32),
    ListPayloadSizeOverflow,
    InvalidListSlot(i32),
    DuplicateListSlot(i32),
    NonFiniteTax,
}

impl fmt::Display for UserStoreCodecError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedPacketId(id) => {
                write!(formatter, "unexpected street-stall packet {id:#010x}")
            }
            Self::ExactSize {
                packet_id,
                expected,
                actual,
            } => write!(
                formatter,
                "street-stall packet {packet_id:#010x} requires exactly {expected} bytes, got {actual}"
            ),
            Self::NegativeListCount(count) => {
                write!(formatter, "negative street-stall list count {count}")
            }
            Self::ListCountOverMaximum(count) => {
                write!(formatter, "street-stall list count {count} exceeds five")
            }
            Self::ListPayloadSizeOverflow => {
                formatter.write_str("street-stall list byte count overflow")
            }
            Self::InvalidListSlot(slot) => {
                write!(formatter, "invalid street-stall list slot {slot}")
            }
            Self::DuplicateListSlot(slot) => {
                write!(formatter, "duplicate street-stall list slot {slot}")
            }
            Self::NonFiniteTax => formatter.write_str("non-finite street-stall tax"),
        }
    }
}

impl Error for UserStoreCodecError0104 {}

pub fn decode_user_store_reply_0104(
    packet_id: u32,
    payload: &[u8],
) -> Result<UserStoreReply0104, UserStoreCodecError0104> {
    match packet_id {
        STREETSTALL_REP_READY_SUCCESS => {
            require_exact(packet_id, payload, STREETSTALL_READY_SUCCESS_SIZE)?;
            let tax_percentage = read_f32(payload, 8);
            if !tax_percentage.is_finite() {
                return Err(UserStoreCodecError0104::NonFiniteTax);
            }
            Ok(UserStoreReply0104::ReadySuccess(
                UserStoreReadySuccess0104 {
                    open_item_inventory_slot: read_i32(payload, 0),
                    item_list_count_max: read_i32(payload, 4),
                    tax_percentage,
                    pc_char_state: payload[12] as i8,
                },
            ))
        }
        STREETSTALL_REP_READY_FAIL => {
            decode_failure(packet_id, payload, UserStoreReply0104::ReadyFail)
        }
        STREETSTALL_REP_CANCEL_SUCCESS => {
            require_exact(packet_id, payload, STREETSTALL_CANCEL_SUCCESS_SIZE)?;
            Ok(UserStoreReply0104::CancelSuccess {
                pc_char_state: payload[0] as i8,
            })
        }
        STREETSTALL_REP_CANCEL_FAIL => {
            decode_failure(packet_id, payload, UserStoreReply0104::CancelFail)
        }
        STREETSTALL_REP_REGISTER_ITEM_SUCCESS => {
            require_exact(packet_id, payload, STREETSTALL_REGISTER_SUCCESS_SIZE)?;
            Ok(UserStoreReply0104::RegisterSuccess(
                UserStoreRegisterSuccess0104 {
                    list_slot: read_i32(payload, 0),
                    inventory_slot: read_i32(payload, 4),
                    item: decode_item(&payload[8..20]),
                    price: read_i32(payload, 20),
                },
            ))
        }
        STREETSTALL_REP_REGISTER_ITEM_FAIL => {
            decode_failure(packet_id, payload, UserStoreReply0104::RegisterFail)
        }
        STREETSTALL_REP_UNREGISTER_ITEM_SUCCESS => {
            require_exact(packet_id, payload, STREETSTALL_UNREGISTER_SUCCESS_SIZE)?;
            Ok(UserStoreReply0104::UnregisterSuccess {
                list_slot: read_i32(payload, 0),
            })
        }
        STREETSTALL_REP_UNREGISTER_ITEM_FAIL => {
            decode_failure(packet_id, payload, UserStoreReply0104::UnregisterFail)
        }
        STREETSTALL_REP_SALE_START_SUCCESS => {
            require_exact(packet_id, payload, STREETSTALL_SALE_START_SUCCESS_SIZE)?;
            Ok(UserStoreReply0104::SaleStartSuccess(
                UserStoreSaleStartSuccess0104 {
                    open_item_inventory_slot: read_i32(payload, 0),
                    open_item: decode_item(&payload[4..16]),
                    pc_char_state: read_i32(payload, 16),
                },
            ))
        }
        STREETSTALL_REP_SALE_START_FAIL => {
            decode_failure(packet_id, payload, UserStoreReply0104::SaleStartFail)
        }
        STREETSTALL_REP_ITEM_LIST => decode_item_list(payload),
        STREETSTALL_REP_ITEM_LIST_FAIL => {
            decode_failure(packet_id, payload, UserStoreReply0104::ItemListFail)
        }
        STREETSTALL_REP_ITEM_BUY_SUCCESS_BUYER => {
            require_exact(packet_id, payload, STREETSTALL_ITEM_BUY_SUCCESS_SIZE)?;
            Ok(UserStoreReply0104::BuySuccessBuyer(
                UserStoreBuyBuyerSuccess0104 {
                    target_pc_id: read_i32(payload, 0),
                    buyer_taros: read_i32(payload, 4),
                    buyer_inventory_slot: read_i32(payload, 8),
                    item: decode_item(&payload[12..24]),
                    list_slot: read_i32(payload, 24),
                },
            ))
        }
        STREETSTALL_REP_ITEM_BUY_SUCCESS_SELLER => {
            require_exact(packet_id, payload, STREETSTALL_ITEM_BUY_SUCCESS_SIZE)?;
            Ok(UserStoreReply0104::BuySuccessSeller(
                UserStoreBuySellerSuccess0104 {
                    buyer_pc_id: read_i32(payload, 0),
                    seller_taros: read_i32(payload, 4),
                    seller_inventory_slot: read_i32(payload, 8),
                    item: decode_item(&payload[12..24]),
                    list_slot: read_i32(payload, 24),
                },
            ))
        }
        STREETSTALL_REP_ITEM_BUY_FAIL => {
            decode_failure(packet_id, payload, UserStoreReply0104::BuyFail)
        }
        _ => Err(UserStoreCodecError0104::UnexpectedPacketId(packet_id)),
    }
}

pub(super) fn decode_failure(
    packet_id: u32,
    payload: &[u8],
    make: impl FnOnce(i32) -> UserStoreReply0104,
) -> Result<UserStoreReply0104, UserStoreCodecError0104> {
    require_exact(packet_id, payload, STREETSTALL_FAILURE_SIZE)?;
    Ok(make(read_i32(payload, 0)))
}

pub(super) fn decode_item_list(payload: &[u8]) -> Result<UserStoreReply0104, UserStoreCodecError0104> {
    if payload.len() < STREETSTALL_ITEM_LIST_HEADER_SIZE {
        return Err(UserStoreCodecError0104::ExactSize {
            packet_id: STREETSTALL_REP_ITEM_LIST,
            expected: STREETSTALL_ITEM_LIST_HEADER_SIZE,
            actual: payload.len(),
        });
    }
    let target_pc_id = read_i32(payload, 0);
    let count = read_i32(payload, 4);
    if count < 0 {
        return Err(UserStoreCodecError0104::NegativeListCount(count));
    }
    if count as usize > USER_STORE_LIST_CAPACITY {
        return Err(UserStoreCodecError0104::ListCountOverMaximum(count));
    }
    let expected = STREETSTALL_ITEM_LIST_HEADER_SIZE
        .checked_add(
            (count as usize)
                .checked_mul(STREETSTALL_ITEM_LIST_RECORD_SIZE)
                .ok_or(UserStoreCodecError0104::ListPayloadSizeOverflow)?,
        )
        .ok_or(UserStoreCodecError0104::ListPayloadSizeOverflow)?;
    require_exact(STREETSTALL_REP_ITEM_LIST, payload, expected)?;
    let mut seen = [false; USER_STORE_LIST_CAPACITY];
    let mut records = Vec::with_capacity(count as usize);
    for index in 0..count as usize {
        let offset = STREETSTALL_ITEM_LIST_HEADER_SIZE + index * STREETSTALL_ITEM_LIST_RECORD_SIZE;
        let list_slot = read_i32(payload, offset);
        let slot = list_slot_index(list_slot)?;
        if seen[slot] {
            return Err(UserStoreCodecError0104::DuplicateListSlot(list_slot));
        }
        seen[slot] = true;
        records.push(UserStoreListingRecord0104 {
            list_slot,
            item: decode_item(&payload[offset + 4..offset + 16]),
            price: read_i32(payload, offset + 16),
        });
    }
    Ok(UserStoreReply0104::ItemListSuccess(
        UserStoreItemListSuccess0104 {
            target_pc_id,
            records,
        },
    ))
}

pub(super) fn encode_item(item: ItemBase0104, payload: &mut [u8]) {
    debug_assert_eq!(payload.len(), 12);
    payload[0..2].copy_from_slice(&item.item_type.to_le_bytes());
    payload[2..4].copy_from_slice(&item.item_id.to_le_bytes());
    write_i32(payload, 4, item.option);
    write_i32(payload, 8, item.time_limit);
}

pub(super) fn decode_item(payload: &[u8]) -> ItemBase0104 {
    debug_assert_eq!(payload.len(), 12);
    ItemBase0104 {
        item_type: i16::from_le_bytes([payload[0], payload[1]]),
        item_id: i16::from_le_bytes([payload[2], payload[3]]),
        option: read_i32(payload, 4),
        time_limit: read_i32(payload, 8),
    }
}
