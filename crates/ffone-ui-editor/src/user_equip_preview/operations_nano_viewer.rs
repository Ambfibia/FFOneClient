use super::*;

pub(super) fn nano_viewer(document: &mut UiLayoutDocument) {
    ensure_form(document, "nano_viewer", "Карточка Nano", [360.0, 620.0]);
    for value in [
        spec(
            "nano_viewer_back",
            "Фон карточки Nano",
            "nano_viewer",
            "Фон",
            "nano_viewer",
            rect(0.0, 0.0, 360.0, 620.0),
        )
        .image("nano-popup.png")
        .z(0),
        spec(
            "nano_viewer_close",
            "Закрыть",
            "nano_viewer",
            "Кнопки",
            "nano_viewer",
            rect(327.0, 0.0, 32.0, 32.0),
        )
        .image("close.png")
        .z(50),
        spec(
            "nano_viewer_icon",
            "Иконка Nano",
            "nano_viewer",
            "Runtime",
            "nano_viewer",
            rect(17.0, 15.0, 62.0, 62.0),
        )
        .dynamic("NANO")
        .z(10),
    ] {
        ensure(document, value);
    }
    for (id, label, value, area, size, color, align) in [
        (
            "nano_viewer_title",
            "Заголовок",
            "EQUIPPED",
            rect(13.5, 3.0, 100.0, 30.0),
            16.0,
            dark_blue(),
            Align::Left,
        ),
        (
            "nano_viewer_name",
            "Имя Nano",
            "BLOSSOM",
            rect(88.0, 15.0, 200.0, 15.0),
            14.0,
            cyan(),
            Align::Left,
        ),
        (
            "nano_viewer_attribute",
            "Атрибут",
            "COSMIX",
            rect(88.0, 47.0, 100.0, 15.0),
            14.0,
            yellow(),
            Align::Left,
        ),
        (
            "nano_viewer_description",
            "Описание",
            "LEVEL 36 NANO — sample description",
            rect(17.0, 85.0, 335.0, 27.0),
            8.0,
            aqua(),
            Align::Wrap,
        ),
        (
            "nano_viewer_power",
            "Текущая сила",
            "CURRENT POWER",
            rect(20.0, 120.0, 100.0, 10.0),
            8.0,
            dark_blue(),
            Align::Center,
        ),
        (
            "nano_viewer_notice",
            "Подсказка",
            "Visit a Nano Station to unlock or change a power.",
            rect(37.0, 565.0, 280.0, 33.0),
            8.0,
            cyan(),
            Align::Wrap,
        ),
    ] {
        ensure(
            document,
            spec(id, label, "nano_viewer", "Текст", "nano_viewer", area)
                .text(value, size, color, align)
                .z(20),
        );
    }

    let rows = [
        (rect(18.0, 133.0, 35.0, 35.0), 135.0, 151.0, 173.0, 27.0),
        (rect(16.0, 222.0, 35.0, 35.0), 227.0, 243.0, 268.0, 40.0),
        (rect(15.0, 383.0, 35.0, 35.0), 387.0, 403.0, 430.0, 40.0),
    ];
    for (index, (icon, name_y, type_y, description_y, description_height)) in
        rows.into_iter().enumerate()
    {
        ensure(
            document,
            spec(
                &format!("nano_skill_icon_{index}"),
                &format!("Сила {} — иконка", index + 1),
                "nano_viewer",
                "Силы",
                "nano_viewer",
                icon,
            )
            .dynamic("SKILL")
            .z(20),
        );
        for (suffix, value, area, size, color) in [
            (
                "name",
                format!("POWER {}", index + 1),
                rect(53.0, name_y, 200.0, 15.0),
                14.0,
                cyan(),
            ),
            (
                "type",
                "STUN — CONE".to_owned(),
                rect(53.0, type_y, 150.0, 12.0),
                8.0,
                yellow(),
            ),
            (
                "description",
                "Power description sample".to_owned(),
                rect(20.0, description_y, 310.0, description_height),
                8.0,
                aqua(),
            ),
        ] {
            ensure(
                document,
                spec(
                    &format!("nano_skill_{suffix}_{index}"),
                    &format!("Сила {} — {suffix}", index + 1),
                    "nano_viewer",
                    "Силы",
                    "nano_viewer",
                    area,
                )
                .text(&value, size, color, Align::Left)
                .z(20),
            );
        }
    }

    for (skill, prompt_y, label_y, bar_y, value_y) in [
        (1, 310.0, [328.0, 345.0], [331.0, 348.0], [329.0, 346.0]),
        (2, 474.0, [493.0, 509.0], [492.0, 509.0], [490.0, 508.0]),
    ] {
        ensure(
            document,
            spec(
                &format!("nano_requirement_prompt_{skill}"),
                "Требования",
                "nano_viewer",
                "Требования",
                "nano_viewer",
                rect(70.0, prompt_y, 250.0, 13.0),
            )
            .text("REQUIREMENTS", 8.0, aqua(), Align::Left)
            .z(20),
        );
        for requirement in 0..2 {
            let (label, asset) = if requirement == 0 {
                ("FUSION MATTER", "nano-popup-fm-bar.png")
            } else {
                ("ITEMS", "nano-popup-item-bar.png")
            };
            ensure(
                document,
                spec(
                    &format!("nano_requirement_label_{skill}_{requirement}"),
                    label,
                    "nano_viewer",
                    "Требования",
                    "nano_viewer",
                    rect(20.0, label_y[requirement], 140.0, 13.0),
                )
                .text(label, 8.0, aqua(), Align::Right)
                .z(20),
            );
            ensure(
                document,
                spec(
                    &format!("nano_requirement_bar_{skill}_{requirement}"),
                    "Полоса требования",
                    "nano_viewer",
                    "Требования",
                    "nano_viewer",
                    rect(176.0, bar_y[requirement], 126.0, 9.0),
                )
                .image(asset)
                .z(20),
            );
            ensure(
                document,
                spec(
                    &format!("nano_requirement_value_{skill}_{requirement}"),
                    "Значение требования",
                    "nano_viewer",
                    "Требования",
                    "nano_viewer",
                    rect(177.0, value_y[requirement], 129.0, 13.0),
                )
                .text("500 / 1000", 8.0, white(), Align::Center)
                .z(30),
            );
        }
    }
}

pub(super) fn help_panel(document: &mut UiLayoutDocument) {
    ensure_form(document, "help", "Помощь инвентаря", [310.0, 310.0]);
    for value in [
        spec(
            "help_background",
            "Фон помощи",
            "help",
            "Фон",
            "help",
            rect(0.0, 0.0, 310.0, 310.0),
        )
        .fill([5, 20, 36, 247])
        .z(0),
        spec(
            "help_title",
            "Заголовок помощи",
            "help",
            "Текст",
            "help",
            rect(16.0, 16.0, 278.0, 24.0),
        )
        .text("INVENTORY HELP", 15.0, cyan(), Align::Center)
        .z(10),
        spec(
            "help_body",
            "Текст помощи",
            "help",
            "Текст",
            "help",
            rect(16.0, 55.0, 278.0, 174.0),
        )
        .text(
            "Left-click an item to choose an action. Right-click to equip or use it immediately. Drag items between inventory and equipment slots to move or swap them.",
            12.0,
            white(),
            Align::Wrap,
        )
        .z(10),
        spec(
            "help_close",
            "Закрыть помощь",
            "help",
            "Кнопки",
            "help",
            rect(16.0, 256.0, 278.0, 38.0),
        )
        .fill_text(
            [15, 64, 92, 255],
            "CLOSE",
            12.0,
            white(),
            Align::Center,
        )
        .z(10),
    ] {
        ensure(document, value);
    }
}

pub(super) fn spec(
    id: &str,
    label: &str,
    form: &str,
    group: &str,
    space: &str,
    rect: UiLayoutRect,
) -> ElementSpec {
    ElementSpec {
        id: id.to_owned(),
        label: label.to_owned(),
        form: form.to_owned(),
        group: group.to_owned(),
        space: space.to_owned(),
        rect,
        visual: None,
        z_index: 0,
        override_enabled: true,
        notes: String::new(),
    }
}

pub(super) fn ensure(document: &mut UiLayoutDocument, value: ElementSpec) {
    if let Some(existing) = document
        .elements
        .iter_mut()
        .find(|element| element.id == value.id)
    {
        if existing.visual.is_none() {
            existing.visual = value.visual;
        }
        existing.z_index = value.z_index;
        return;
    }
    document.elements.push(UiLayoutElement {
        id: value.id,
        label: value.label,
        form: value.form,
        group: value.group,
        space: value.space,
        rect: value.rect,
        source_rect: value.rect,
        override_enabled: value.override_enabled,
        editor_hidden: false,
        locked: false,
        visual: value.visual,
        z_index: value.z_index,
        notes: value.notes,
    });
}

pub(super) fn ensure_form(document: &mut UiLayoutDocument, id: &str, label: &str, size: [f32; 2]) {
    if document.forms.iter().any(|form| form.id == id) {
        return;
    }
    document.spaces.push(UiLayoutSpace {
        id: id.to_owned(),
        label: label.to_owned(),
        origin: [0.0, 0.0],
    });
    document.forms.push(UiLayoutForm {
        id: id.to_owned(),
        label: label.to_owned(),
        canvas_size: size,
        background: None,
        background_rect: None,
        background_source_rect: None,
        space_origins: BTreeMap::from([(id.to_owned(), [0.0, 0.0])]),
    });
}

pub(super) fn image(name: &str) -> UiLayoutVisual {
    UiLayoutVisual::image(format!("{UI}{name}"))
}

pub(super) fn nine(name: &str, border: [f32; 4]) -> UiLayoutVisual {
    let mut value = image(name);
    let image = value.image.as_mut().expect("image helper creates image");
    image.mode = UiLayoutImageMode::NineSlice;
    image.border = border;
    value
}

pub(super) fn image_text(
    name: &str,
    value: &str,
    size: f32,
    color: [u8; 4],
    align: Align,
    border: Option<[f32; 4]>,
) -> UiLayoutVisual {
    let mut visual = border.map_or_else(|| image(name), |border| nine(name, border));
    visual.text = Some(text_value(value, size, color, align));
    visual
}

pub(super) fn text(value: &str, size: f32, color: [u8; 4], align: Align) -> UiLayoutVisual {
    UiLayoutVisual {
        image: None,
        text: Some(text_value(value, size, color, align)),
        fill: None,
        dynamic_placeholder: false,
    }
}

pub(super) fn dynamic(value: &str) -> UiLayoutVisual {
    UiLayoutVisual {
        image: None,
        text: Some(text_value(value, 8.0, [255, 255, 255, 210], Align::Center)),
        fill: None,
        dynamic_placeholder: true,
    }
}

pub(super) fn text_value(value: &str, size: f32, color: [u8; 4], align: Align) -> UiLayoutText {
    let (horizontal, vertical, wrap) = match align {
        Align::Left => (
            UiLayoutHorizontalAlign::Left,
            UiLayoutVerticalAlign::Top,
            false,
        ),
        Align::Center => (
            UiLayoutHorizontalAlign::Center,
            UiLayoutVerticalAlign::Center,
            false,
        ),
        Align::Right => (
            UiLayoutHorizontalAlign::Right,
            UiLayoutVerticalAlign::Top,
            false,
        ),
        Align::LeftCenter => (
            UiLayoutHorizontalAlign::Left,
            UiLayoutVerticalAlign::Center,
            false,
        ),
        Align::RightCenter => (
            UiLayoutHorizontalAlign::Right,
            UiLayoutVerticalAlign::Center,
            false,
        ),
        Align::BottomCenter => (
            UiLayoutHorizontalAlign::Center,
            UiLayoutVerticalAlign::Bottom,
            false,
        ),
        Align::Wrap => (
            UiLayoutHorizontalAlign::Left,
            UiLayoutVerticalAlign::Top,
            true,
        ),
    };
    UiLayoutText {
        value: value.to_owned(),
        font: Some(FONT.to_owned()),
        font_size: size,
        color,
        horizontal,
        vertical,
        wrap,
    }
}

pub(super) fn rect(x: f32, y: f32, width: f32, height: f32) -> UiLayoutRect {
    UiLayoutRect::new(x, y, width, height)
}

pub(super) const fn white() -> [u8; 4] {
    [255, 255, 255, 255]
}

pub(super) const fn cyan() -> [u8; 4] {
    [203, 255, 255, 255]
}

pub(super) const fn aqua() -> [u8; 4] {
    [0, 255, 255, 255]
}

pub(super) const fn yellow() -> [u8; 4] {
    [255, 255, 0, 255]
}

pub(super) const fn green() -> [u8; 4] {
    [0, 255, 0, 255]
}

pub(super) const fn dark_blue() -> [u8; 4] {
    [0, 51, 102, 255]
}
