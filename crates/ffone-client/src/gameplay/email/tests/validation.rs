use super::*;

#[test]
fn new_email_is_correlated_only_with_update_check_and_otherwise_is_push() {
    let mut runtime = EmailTransportRuntime0104::default();
    let count = 6_i32.to_le_bytes();
    assert_eq!(
        runtime.accept(EMAIL_REP_NEW_ID, &count).unwrap(),
        Some(EmailRuntimeDelivery0104::UnsolicitedNewEmail(
            EmailReply::NewEmail { count: 6 }
        ))
    );

    runtime.begin(&EmailRequest::UpdateCheck).unwrap();
    assert_eq!(
        runtime.accept(EMAIL_REP_NEW_ID, &count).unwrap(),
        Some(EmailRuntimeDelivery0104::Correlated(EmailReply::NewEmail {
            count: 6
        }))
    );
    assert_eq!(runtime.pending(), None);

    runtime
        .begin(&EmailRequest::ReceiveCash { email_index: 9 })
        .unwrap();
    assert!(matches!(
        runtime.accept(EMAIL_REP_NEW_ID, &count).unwrap(),
        Some(EmailRuntimeDelivery0104::UnsolicitedNewEmail(_))
    ));
    assert_eq!(
        runtime.pending(),
        Some(&EmailPending0104::ReceiveCash { email_index: 9 })
    );
}
