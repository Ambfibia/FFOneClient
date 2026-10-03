use super::*;

#[test]
fn request_identity_preserves_clean_signed_byte_list_alias() {
    let session = VendorSession0104 {
        requested_npc_id: 9_001,
        table_vendor_id: 650,
        accepted_npc_id: 9_001,
    };
    assert_eq!(
        session.request_identity(),
        VendorRequestIdentity0104 {
            npc_id: 9_001,
            vendor_id: 9_001,
            list_id: 41,
        }
    );

    assert_eq!(
        VendorSession0104 {
            accepted_npc_id: 200,
            ..session
        }
        .request_identity()
        .list_id,
        -56,
        "clean `(sbyte)` conversion retains the signed low byte"
    );
}
