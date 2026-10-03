use super::*;

#[test]
fn sell_validation_uses_exact_messages_and_calculator_count_rule() {
    let projected = projection(
        &[],
        &[],
        &[
            (0, item(9, 1, 0, 0)),
            (1, item(8, 1, 0, 0)),
            (2, item(0, 5, 0, 0)),
            (3, item(0, 1, 0, 0)),
            (4, item(7, 4, 10, 0)),
        ],
    );
    assert_eq!(
        projected.request_sell(0, 1),
        VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
            VendorSystemMessageId0104::CannotSellChest
        ))
    );
    assert_eq!(
        projected.request_sell(1, 1),
        VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
            VendorSystemMessageId0104::CannotSellQuestItem
        ))
    );
    assert_eq!(
        projected.request_sell(2, 1),
        VendorActionOutcome0104::SystemMessage(VendorSystemMessage0104::plain(
            VendorSystemMessageId0104::ItemNotSellable
        ))
    );
    assert!(matches!(
        projected.request_sell(3, 99),
        VendorActionOutcome0104::Intent(VendorIntent0104::Sell(VendorSellIntent0104 {
            count: 1,
            ..
        }))
    ));
    assert!(matches!(
        projected.request_sell(4, 6),
        VendorActionOutcome0104::Intent(VendorIntent0104::Sell(VendorSellIntent0104 {
            count: 6,
            ..
        }))
    ));
    assert!(matches!(
        projected.request_sell(4, 0),
        VendorActionOutcome0104::SilentBlocked(VendorSilentBlock0104::InvalidCount {
            requested: 0,
            maximum: Some(10)
        })
    ));
    assert!(matches!(
        projected.request_sell(4, 11),
        VendorActionOutcome0104::SilentBlocked(VendorSilentBlock0104::InvalidCount {
            requested: 11,
            maximum: Some(10)
        })
    ));
}
