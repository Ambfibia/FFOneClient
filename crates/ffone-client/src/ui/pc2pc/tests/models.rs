use super::*;

pub(super) fn model_with(wallet: i32, inventory: &[(usize, ItemBase0104)]) -> Pc2pcUiModel0104 {
    let snapshot = Pc2pcAuthoritativeSnapshot0104::from_accepted_trade(
        identity(),
        Pc2pcParticipantNames0104::new("Dexter", "Mandark").unwrap(),
        wallet,
        &runtime(inventory),
    )
    .unwrap();
    let mut model = Pc2pcUiModel0104::default();
    model.begin_session(snapshot, &Catalog, &AllowEquip);
    model.state.tick(PC2PC_OPEN_SECONDS);
    model
}
