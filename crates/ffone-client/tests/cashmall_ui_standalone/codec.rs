use super::*;

#[test]
fn cash_is_permanently_nine_zero_digits_and_receive_packet_is_a_no_op() {
    let mut state = CashmallUiState0104::default();
    let before = state;
    assert_eq!(state.receive_packet_ignored(0xDEAD_BEEF), 0xDEAD_BEEF);
    assert_eq!(state, before);
    assert_eq!(state.user_cash(), 0);
    assert_eq!(cashmall_cash_digits_0104(), [0; 9]);
    assert_eq!(cashmall_cash_text_0104(), "000000000");
    assert!(!CASHMALL_RECEIVE_PACKET_MUTATES_STATE);
    assert!(!CASHMALL_USER_CASH_ASSIGNMENT_REACHABLE);
    assert!(!CASHMALL_PURCHASE_NETWORK_CONTRACT_PRESENT);
}
