use super::*;

#[test]
fn validates_unique_spaces_elements_and_positive_rects() {
    let document = UiLayoutDocument {
        schema: UI_LAYOUT_SCHEMA_V1.to_owned(),
        name: "test".to_owned(),
        reference_resolution: [1280.0, 720.0],
        background: None,
        forms: vec![UiLayoutForm {
            id: "main".to_owned(),
            label: "Main".to_owned(),
            canvas_size: [1280.0, 720.0],
            background: None,
            background_rect: None,
            background_source_rect: None,
            space_origins: BTreeMap::new(),
        }],
        spaces: vec![UiLayoutSpace {
            id: "screen".to_owned(),
            label: "Screen".to_owned(),
            origin: [0.0, 0.0],
        }],
        elements: vec![UiLayoutElement {
            id: "close".to_owned(),
            label: "Close".to_owned(),
            form: "main".to_owned(),
            group: "window".to_owned(),
            space: "screen".to_owned(),
            rect: UiLayoutRect::new(10.0, 10.0, 32.0, 32.0),
            source_rect: UiLayoutRect::new(10.0, 10.0, 32.0, 32.0),
            override_enabled: true,
            editor_hidden: false,
            locked: false,
            visual: None,
            z_index: 0,
            notes: String::new(),
        }],
    };
    assert!(document.validate().is_ok());
    assert_eq!(
        document.rect("close"),
        Some(UiLayoutRect::new(10.0, 10.0, 32.0, 32.0))
    );
}

#[test]
fn production_user_equip_layout_is_valid() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game/ui/en/user-equip/user-equip.ffui.json");
    let document = UiLayoutDocument::load(&path).expect("production UserEquip layout");
    assert_eq!(document.reference_resolution, [1264.0, 681.0]);
    assert_eq!(document.forms.len(), 4);
    assert_eq!(
        document.form("item_popup").map(|form| form.canvas_size),
        Some([310.0, 449.0])
    );
    assert!(document.elements.len() >= 35);
    assert_eq!(
        document.rect("item_popup_close"),
        Some(UiLayoutRect::new(277.0, 15.0, 32.0, 32.0))
    );
}

#[test]
fn validates_editor_image_and_text_visuals() {
    let mut document = UiLayoutDocument {
        schema: UI_LAYOUT_SCHEMA_V1.to_owned(),
        name: "visual".to_owned(),
        reference_resolution: [320.0, 200.0],
        background: None,
        forms: vec![UiLayoutForm {
            id: "main".to_owned(),
            label: "Main".to_owned(),
            canvas_size: [320.0, 200.0],
            background: None,
            background_rect: None,
            background_source_rect: None,
            space_origins: BTreeMap::new(),
        }],
        spaces: vec![UiLayoutSpace {
            id: "screen".to_owned(),
            label: "Screen".to_owned(),
            origin: [0.0, 0.0],
        }],
        elements: vec![UiLayoutElement {
            id: "button".to_owned(),
            label: "Button".to_owned(),
            form: "main".to_owned(),
            group: "controls".to_owned(),
            space: "screen".to_owned(),
            rect: UiLayoutRect::new(10.0, 10.0, 100.0, 24.0),
            source_rect: UiLayoutRect::new(10.0, 10.0, 100.0, 24.0),
            override_enabled: true,
            editor_hidden: false,
            locked: false,
            visual: Some(UiLayoutVisual {
                image: Some(UiLayoutImage {
                    path: "ui/button.png".to_owned(),
                    mode: UiLayoutImageMode::NineSlice,
                    border: [4.0; 4],
                    ..UiLayoutImage::default()
                }),
                text: Some(UiLayoutText {
                    value: "PLAY".to_owned(),
                    horizontal: UiLayoutHorizontalAlign::Center,
                    vertical: UiLayoutVerticalAlign::Center,
                    ..UiLayoutText::default()
                }),
                fill: None,
                dynamic_placeholder: false,
            }),
            z_index: 10,
            notes: String::new(),
        }],
    };
    assert!(document.validate().is_ok());
    document.elements[0]
        .visual
        .as_mut()
        .expect("visual")
        .image
        .as_mut()
        .expect("image")
        .path
        .clear();
    assert!(matches!(
        document.validate(),
        Err(UiLayoutError::InvalidVisual(id)) if id == "button"
    ));
}
