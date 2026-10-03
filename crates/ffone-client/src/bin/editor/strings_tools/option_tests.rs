use super::*;
#[test]
fn three_filter_modes_distinguish_cell_word_and_substring() {
    let mut t = Tools::default();
    t.filter = "бег".into();
    assert!(t.filter_matches("побег и беготня"));
    t.filter_mode = 1;
    assert!(!t.filter_matches("побег и беготня"));
    assert!(t.filter_matches("БЕГ, а потом отдых"));
    t.filter_mode = 2;
    assert!(t.filter_matches("БЕГ"));
    assert!(!t.filter_matches("Бег и отдых"));
    t.filter.clear();
    assert!(t.filter_matches("любая строка"));
}
#[test]
fn replacement_case_and_whole_word_are_independent_and_utf8_safe() {
    let text = "БЕГ бег беготня побег (бег) бег_1";
    let mut t = Tools::default();
    assert_eq!(replacement_matches(text, "бег", &t).len(), 6);
    t.case_sensitive = true;
    assert_eq!(replacement_matches(text, "БЕГ", &t), vec![(0, 6)]);
    t.whole_word = true;
    assert_eq!(replacement_matches(text, "бег", &t).len(), 2);
    t.case_sensitive = false;
    let ranges = replacement_matches(text, "бег", &t);
    assert_eq!(ranges.len(), 3);
    for (a, b) in ranges {
        assert_eq!(text[a..b].to_lowercase(), "бег");
    }
    assert!(replacement_matches(text, "", &t).is_empty());
}
#[test]
fn stationary_pointer_does_not_reposition_caret_after_redraw() {
    let mut previous = None;
    let point = Some(Vec2::new(120., 70.));
    assert!(pointer_moved(&mut previous, point));
    assert!(!pointer_moved(&mut previous, point));
    assert!(pointer_moved(&mut previous, Some(Vec2::new(121., 70.))));
}
