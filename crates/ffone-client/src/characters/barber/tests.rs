use super::*;
fn editing() -> BarberModel {
    use ffone_protocol::WirePayload;
    let mut style = PcStyle0104::decode(&[0; 76]).unwrap();
    style.gender = 1;
    style.hair_style = 2;
    style.face_style = 1;
    let mut prices = PcBarberOpenSuccess0104::decode(&[0; 288]).unwrap();
    prices.change_gender = 1;
    prices.gender_cost = 500;
    prices.hair_style_cost = 100;
    prices.face_style_cost = 100;
    prices.body_cost = 100;
    BarberModel {
        phase: BarberPhase::Editing,
        original: Some(style.clone()),
        draft: Some(style),
        prices: Some(prices),
        taros: 1000,
        hair: [
            vec![(1, "A".into()), (2, "B".into())],
            vec![(25, "C".into()), (26, "D".into())],
        ],
        face: [vec![(1, "A".into())], vec![(6, "B".into())]],
        ..default()
    }
}
#[test]
fn gender_draft_maps_creation_row_and_never_edits_original() {
    let mut model = editing();
    let original = model.original.clone();
    model.step(BarberField::Gender, 1);
    assert_eq!(model.draft.as_ref().unwrap().hair_style, 26);
    assert_eq!(model.draft.as_ref().unwrap().face_style, 6);
    assert_eq!(model.cost(), Some(700));
    assert!(model.can_confirm());
    assert_eq!(model.original, original);
    model.taros = 699;
    assert!(!model.can_confirm());
    model.reset();
    assert_eq!(model.draft, original);
    assert!(!model.can_confirm());
}
#[test]
fn configured_free_change_can_be_confirmed_but_unchanged_draft_cannot() {
    let mut model = editing();
    model.taros = 0;
    model.prices.as_mut().unwrap().body_cost = 0;
    assert!(!model.can_confirm());
    model.step(BarberField::Height, 1);
    assert_eq!(model.cost(), Some(0));
    assert!(model.can_confirm());
}
#[test]
fn pending_confirmation_is_immutable_and_body_price_is_not_doubled() {
    let mut model = editing();
    model.step(BarberField::Height, 1);
    model.step(BarberField::Body, 1);
    assert_eq!(model.cost(), Some(100));
    let draft = model.draft.clone();
    model.phase = BarberPhase::Confirming;
    model.step(BarberField::Hair, 1);
    model.reset();
    model.set_color(BarberField::Eye, 2);
    assert_eq!(model.draft, draft);
    assert!(!model.can_confirm());
}
