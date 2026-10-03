use super::*;

#[test]
fn unchanged_npc_mail_refresh_preserves_the_selected_letter_and_page() {
    let (mut harness, _) = open_harness(None, EmailItemFeaturePolicy0104::default(), false);
    let letters = (1..=8)
        .map(|task| EmailGuideMessage {
            mode: 1,
            mission_task_id: task,
            content: format!("Letter {task}"),
            ..EmailGuideMessage::default()
        })
        .collect::<Vec<_>>();
    harness
        .production
        .refresh_guide_projection(letters.clone(), &mut harness.model)
        .unwrap();
    harness.model.guide_page = 1;
    harness.model.selected_row = Some(2);
    for _ in 0..3 {
        harness
            .production
            .refresh_guide_projection(letters.clone(), &mut harness.model)
            .unwrap();
        assert_eq!(harness.model.guide_page, 1);
        assert_eq!(harness.model.selected_guide().unwrap().mission_task_id, 8);
    }
    harness
        .production
        .refresh_guide_projection(Vec::new(), &mut harness.model)
        .unwrap();
    assert_eq!(harness.model.guide_page, 0);
    assert_eq!(harness.model.selected_row, None);
}
