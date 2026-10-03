use super::*;

/// Completes the editor view of the production UserEquip hierarchy. The
/// runtime geometry document originally contained only a small set of hit
/// rectangles; this catalog adds the real static images, text samples and
/// runtime-provided placeholders without treating a GPU screenshot as UI.
pub(in super::super) fn enrich(document: &mut UiLayoutDocument) {
    if document.name != "UserEquip" {
        return;
    }

    for form in &mut document.forms {
        form.label = match form.id.as_str() {
            "inventory" => "Инвентарь — окно".to_owned(),
            "item_popup" => "Карточка — экипировать".to_owned(),
            "item_popup_unequip" => "Карточка — снять".to_owned(),
            "item_popup_use" => "Карточка — использовать".to_owned(),
            "item_popup_chest" => "Карточка — сундук".to_owned(),
            "character_status" => "Статус персонажа".to_owned(),
            "equipment_strip" => "Экипировка".to_owned(),
            _ => form.label.clone(),
        };
    }

    assign_existing_visuals(document);
    inventory_window(document);
    inventory_content(document);
    item_popup(document);
    item_popup_variants(document);
    character_status(document);
    equipment_strip(document);
    character_panel(document);
    nano_gallery(document);
    nano_viewer(document);
    help_panel(document);
    item_mode_overview(document);
}

pub(super) fn assign_existing_visuals(document: &mut UiLayoutDocument) {
    for element in &mut document.elements {
        let visual = match element.id.as_str() {
            "item_tab" => Some(image("item-tab.png")),
            "nano_tab" => Some(image("nano-tab.png")),
            "item_tab_label" => Some(text("EQUIPMENT", 12.0, cyan(), Align::Left)),
            "nano_tab_label" => Some(text("NANOS", 12.0, cyan(), Align::Center)),
            "close" | "item_popup_close" => Some(image("close.png")),
            "trash" | "item_popup_trash" => Some(image("trash.png")),
            "help" => Some(image("help.png")),
            "taros_back" => Some(image("taros.png")),
            "dexlabs" => Some(image("dexlabs.png")),
            "boost_slot" | "potion_slot" => Some(image("slot-empty.png")),
            "boost_icon" => Some(image("boost-icon.png")),
            "potion_icon" => Some(image("potion-icon.png")),
            "boost_label" => Some(text("BOOSTS", 8.0, white(), Align::RightCenter)),
            "boost_value" => Some(text("64", 12.0, white(), Align::Left)),
            "potion_label" => Some(text("POTIONS", 8.0, white(), Align::RightCenter)),
            "potion_value" => Some(text("29", 12.0, white(), Align::Left)),
            "status_name" => Some(text("Dexter", 12.0, cyan(), Align::Left)),
            "status_level" => Some(text("LEVEL : 36", 12.0, yellow(), Align::Left)),
            "status_hp" => Some(text("HEALTH", 8.0, white(), Align::LeftCenter)),
            "status_hp_value" => Some(text("850/1000", 8.0, white(), Align::RightCenter)),
            "status_fusion" => Some(text("FUSION MATTER", 8.0, white(), Align::LeftCenter)),
            "status_fusion_value" => Some(text("742/1000", 8.0, white(), Align::RightCenter)),
            "status_guide_label" => Some(text("GUIDE:", 8.0, cyan(), Align::Left)),
            "status_guide_name" => Some(text("DEXTER", 8.0, cyan(), Align::Left)),
            "status_guide_icon" => Some(image("guide-dexter.png")),
            "item_popup_icon" => Some(dynamic("Предмет")),
            "item_popup_title" => Some(text("EXAMPLE ITEM", 12.0, cyan(), Align::Left)),
            "item_popup_description" => Some(text(
                "Item description preview. Dynamic values are shown as samples.",
                8.0,
                white(),
                Align::Wrap,
            )),
            "item_popup_button_0" => Some(image_text(
                "button-normal.png",
                "EQUIP WEAPON 1",
                12.0,
                white(),
                Align::Center,
                Some([6.0, 6.0, 6.0, 4.0]),
            )),
            "item_popup_button_1" => Some(image_text(
                "button-normal.png",
                "EQUIP WEAPON 2",
                12.0,
                white(),
                Align::Center,
                Some([6.0, 6.0, 6.0, 4.0]),
            )),
            "item_popup_field_0" => Some(text("Level 36", 12.0, yellow(), Align::Left)),
            "item_popup_field_5" => Some(text("INFO", 12.0, white(), Align::Left)),
            "item_popup_field_6" => Some(text("Type", 8.0, cyan(), Align::RightCenter)),
            "item_popup_field_7" => Some(text("Range", 8.0, cyan(), Align::RightCenter)),
            "item_popup_field_8" => Some(text("Rarity", 8.0, cyan(), Align::RightCenter)),
            "item_popup_field_9" => {
                Some(text("Trade Availability", 8.0, cyan(), Align::RightCenter))
            }
            "item_popup_field_10" => Some(text("WEAPON", 8.0, white(), Align::Center)),
            "item_popup_field_11" => Some(text("LONG", 8.0, white(), Align::Center)),
            "item_popup_field_12" => Some(text("RARE", 8.0, white(), Align::Center)),
            "item_popup_field_13" => Some(text("YES", 8.0, green(), Align::Center)),
            _ => None,
        };
        if let Some(visual) = visual {
            element.visual = Some(visual);
            element.z_index = if element.id.contains("label") || element.id.contains("field") {
                30
            } else {
                20
            };
        }
    }
}

pub(super) fn item_popup(document: &mut UiLayoutDocument) {
    ensure(
        document,
        spec(
            "item_popup_backdrop",
            "Фон карточки",
            "item_popup",
            "Фон",
            "item_popup",
            rect(0.0, 14.0, 310.0, 435.0),
        )
        .image("equip-popup.png")
        .z(0),
    );
    ensure(
        document,
        spec(
            "item_popup_equip_info",
            "Панель характеристик",
            "item_popup",
            "Фон",
            "item_popup",
            rect(2.0, 157.0, 305.0, 84.0),
        )
        .nine("equip-item-info.png", [0.0, 0.0, 5.0, 5.0])
        .z(10),
    );
    for (index, x, y, width, label, size, align) in [
        (1, 5.0, 140.0, 200.0, "STATUS", 12.0, Align::Left),
        (2, 28.0, 219.0, 80.0, "120", 8.0, Align::Center),
        (3, 115.0, 219.0, 80.0, "25", 8.0, Align::Center),
        (4, 207.0, 219.0, 80.0, "80", 8.0, Align::Center),
    ] {
        ensure(
            document,
            spec(
                &format!("item_popup_field_{index}"),
                &format!("Поле карточки {index}"),
                "item_popup",
                "Popup table",
                "item_popup",
                rect(x, y, width, 20.0),
            )
            .text(label, size, white(), align)
            .z(30),
        );
    }
    for (index, y, label) in [(2, 337.0, "USE"), (3, 371.0, "DROP")] {
        ensure(
            document,
            spec(
                &format!("item_popup_button_{index}"),
                &format!("Действие {}", index + 1),
                "item_popup",
                "Popup buttons",
                "item_popup",
                rect(122.0, y, 170.0, 28.0),
            )
            .image_text(
                "button-normal.png",
                label,
                12.0,
                white(),
                Align::Center,
                Some([6.0, 6.0, 6.0, 4.0]),
            )
            .preview_only()
            .z(40),
        );
    }
}

pub(super) fn item_popup_variants(document: &mut UiLayoutDocument) {
    ensure_form(
        document,
        "item_popup_unequip",
        "Карточка — снять",
        [310.0, 448.0],
    );
    ensure_form(
        document,
        "item_popup_use",
        "Карточка — использовать",
        [310.0, 235.0],
    );
    ensure_form(
        document,
        "item_popup_chest",
        "Карточка — сундук",
        [310.0, 235.0],
    );

    let base: Vec<_> = document
        .elements
        .iter()
        .filter(|element| element.form == "item_popup")
        .cloned()
        .collect();
    clone_popup_variant(
        document,
        &base,
        "item_popup_unequip",
        "item_unequip_",
        PopupPreviewVariant::Unequip,
    );
    clone_popup_variant(
        document,
        &base,
        "item_popup_use",
        "item_use_",
        PopupPreviewVariant::Use,
    );
    clone_popup_variant(
        document,
        &base,
        "item_popup_chest",
        "item_chest_",
        PopupPreviewVariant::Chest,
    );
}

pub(super) fn clone_popup_variant(
    document: &mut UiLayoutDocument,
    base: &[UiLayoutElement],
    form: &str,
    prefix: &str,
    variant: PopupPreviewVariant,
) {
    for source in base {
        let Some(suffix) = source.id.strip_prefix("item_popup_") else {
            continue;
        };
        let compact = matches!(
            variant,
            PopupPreviewVariant::Use | PopupPreviewVariant::Chest
        );
        if (compact
            && !matches!(
                suffix,
                "backdrop" | "icon" | "title" | "close" | "description" | "button_0" | "trash"
            ))
            || (variant == PopupPreviewVariant::Unequip
                && matches!(suffix, "button_1" | "button_2" | "button_3"))
        {
            continue;
        }

        let id = format!("{prefix}{suffix}");
        if document.elements.iter().any(|element| element.id == id) {
            continue;
        }
        let mut element = source.clone();
        element.id = id;
        element.form = form.to_owned();
        element.space = form.to_owned();
        element.notes = "Отдельная runtime-геометрия варианта карточки.".to_owned();

        match (variant, suffix) {
            (PopupPreviewVariant::Unequip, "backdrop") => {
                set_element_rect(&mut element, rect(0.0, 0.0, 310.0, 448.0));
                element.visual = Some(image("unequip-popup.png"));
            }
            (PopupPreviewVariant::Unequip, "close") => {
                set_element_rect(&mut element, rect(277.0, 0.0, 32.0, 32.0));
            }
            (PopupPreviewVariant::Unequip, "button_0") => {
                set_element_rect(&mut element, rect(160.0, 380.0, 130.0, 28.0));
                set_visual_text(&mut element, "UNEQUIP");
                element.label = "Снять предмет".to_owned();
            }
            (_, "backdrop") if compact => {
                set_element_rect(&mut element, rect(0.0, 0.0, 310.0, 235.0));
                element.visual = Some(image("use-dialog.png"));
            }
            (_, "icon") if compact => {
                set_element_rect(&mut element, rect(16.0, 19.0, 64.0, 64.0));
            }
            (_, "title") if compact => {
                set_element_rect(&mut element, rect(82.0, 19.0, 170.0, 40.0));
                set_visual_text(
                    &mut element,
                    if variant == PopupPreviewVariant::Chest {
                        "10LV CRATE"
                    } else {
                        "POWER ITEM"
                    },
                );
            }
            (_, "close") if compact => {
                set_element_rect(&mut element, rect(277.0, 0.0, 32.0, 32.0));
            }
            (_, "description") if compact => {
                set_element_rect(&mut element, rect(12.0, 99.0, 280.0, 40.0));
                set_visual_text(
                    &mut element,
                    if variant == PopupPreviewVariant::Chest {
                        "SPECIAL"
                    } else {
                        "Use this item to activate its effect."
                    },
                );
            }
            (_, "trash") if compact => {
                set_element_rect(&mut element, rect(10.0, 180.0, 32.0, 32.0));
            }
            (_, "button_0") if compact => {
                set_element_rect(&mut element, rect(157.0, 180.0, 130.0, 28.0));
                let (label, text) = if variant == PopupPreviewVariant::Chest {
                    ("Открыть сундук", "OPEN")
                } else {
                    ("Использовать предмет", "USE")
                };
                element.label = label.to_owned();
                set_visual_text(&mut element, text);
            }
            _ => {}
        }
        document.elements.push(element);
    }
}

pub(super) fn set_element_rect(element: &mut UiLayoutElement, value: UiLayoutRect) {
    element.rect = value;
    element.source_rect = value;
}

pub(super) fn set_visual_text(element: &mut UiLayoutElement, value: &str) {
    if let Some(text) = element
        .visual
        .as_mut()
        .and_then(|visual| visual.text.as_mut())
    {
        text.value = value.to_owned();
    }
}

pub(super) fn equipment_strip(document: &mut UiLayoutDocument) {
    ensure(
        document,
        spec(
            "equipment_title",
            "Заголовок экипировки",
            "equipment_strip",
            "Заголовок",
            "equipment",
            rect(0.0, 3.0, 64.0, 13.0),
        )
        .image_text(
            "equip-title.png",
            "Equipped",
            8.0,
            dark_blue(),
            Align::Center,
            None,
        )
        .z(10),
    );
    let labels = [
        "HEAD", "FACE", "BACK", "CHEST", "LEGS", "FEET", "WEAPON 1", "WEAPON 2", "VEHICLE",
    ];
    for (index, label) in labels.into_iter().enumerate() {
        let y = 14.0 + index as f32 * 62.0;
        ensure(
            document,
            spec(
                &format!("equipment_slot_{index}"),
                &format!("Экипировка: {label}"),
                "equipment_strip",
                "Ячейки экипировки",
                "equipment",
                rect(0.0, y, 64.0, 64.0),
            )
            .image("slot-empty.png")
            .z(0),
        );
        ensure(
            document,
            spec(
                &format!("equipment_slot_label_{index}"),
                &format!("Подпись: {label}"),
                "equipment_strip",
                "Подписи экипировки",
                "equipment",
                rect(0.0, y - 2.0, 60.0, 19.0),
            )
            .text(label, 8.0, white(), Align::BottomCenter)
            .preview_only()
            .z(20),
        );
    }
}

pub(super) fn character_panel(document: &mut UiLayoutDocument) {
    ensure_form(
        document,
        "character_panel",
        "Персонаж и Nano-статус",
        [585.0, 653.0],
    );
    ensure(
        document,
        spec(
            "character_backplate",
            "Панель персонажа",
            "character_panel",
            "Фон",
            "character_panel",
            rect(0.0, 0.0, 585.0, 653.0),
        )
        .image("clothes-panel.png")
        .preview_only()
        .z(0),
    );
    ensure(
        document,
        spec(
            "avatar_preview",
            "3D-персонаж",
            "character_panel",
            "Runtime",
            "character_panel",
            rect(0.0, 0.0, 500.0, 514.0),
        )
        .dynamic("3D AVATAR PREVIEW")
        .z(1),
    );
    for value in [
        spec(
            "turn_left",
            "Повернуть влево",
            "character_panel",
            "Кнопки",
            "character_panel",
            rect(120.0, 440.0, 43.0, 78.0),
        )
        .image("turn-right.png")
        .z(20),
        spec(
            "turn_right",
            "Повернуть вправо",
            "character_panel",
            "Кнопки",
            "character_panel",
            rect(340.0, 440.0, 43.0, 78.0),
        )
        .image("turn-left.png")
        .z(20),
    ] {
        ensure(document, value);
    }

    let panels = [
        (
            rect(24.0, 215.0, 126.0, 69.0),
            rect(45.0, 110.0, 128.0, 128.0),
        ),
        (
            rect(335.0, 175.0, 126.0, 69.0),
            rect(355.0, 69.0, 128.0, 128.0),
        ),
        (
            rect(348.0, 350.0, 126.0, 69.0),
            rect(368.0, 244.0, 128.0, 128.0),
        ),
    ];
    for (slot, (panel, portrait)) in panels.into_iter().enumerate() {
        ensure(
            document,
            spec(
                &format!("nano_status_panel_{slot}"),
                &format!("Nano {} — статус", slot + 1),
                "character_panel",
                "Nano status",
                "character_panel",
                panel,
            )
            .image("nano-dialog.png")
            .z(10),
        );
        ensure(
            document,
            spec(
                &format!("nano_status_portrait_{slot}"),
                &format!("Nano {} — 3D preview", slot + 1),
                "character_panel",
                "Nano status",
                "character_panel",
                portrait,
            )
            .dynamic("NANO 3D")
            .z(5),
        );
        for (suffix, value, local, color) in [
            (
                "label",
                format!("NANO {}", slot + 1),
                rect(5.0, 18.0, 48.0, 9.0),
                white(),
            ),
            (
                "name",
                "EMPTY".to_owned(),
                rect(5.0, 33.0, 118.0, 14.0),
                cyan(),
            ),
            (
                "attribute",
                "ATTRIBUTE".to_owned(),
                rect(5.0, 45.0, 92.0, 14.0),
                white(),
            ),
        ] {
            ensure(
                document,
                spec(
                    &format!("nano_status_{suffix}_{slot}"),
                    &format!("Nano {} — {suffix}", slot + 1),
                    "character_panel",
                    "Nano labels",
                    "character_panel",
                    rect(
                        panel.x + local.x,
                        panel.y + local.y,
                        local.width,
                        local.height,
                    ),
                )
                .text(&value, 8.0, color, Align::Left)
                .z(30),
            );
        }
    }

    for value in [
        spec(
            "character_status_back",
            "Фон статуса",
            "character_panel",
            "Статус",
            "character_panel",
            rect(0.0, 517.0, 498.0, 115.0),
        )
        .nine("user-status-panel.png", [8.0, 8.0, 8.0, 8.0])
        .preview_only()
        .z(10),
        spec(
            "character_status_hp",
            "Полоса здоровья",
            "character_panel",
            "Статус",
            "character_panel",
            rect(14.0, 586.0, 390.0, 15.0),
        )
        .image("hp-bar.png")
        .preview_only()
        .z(11),
        spec(
            "character_status_fm",
            "Полоса Fusion Matter",
            "character_panel",
            "Статус",
            "character_panel",
            rect(14.0, 608.0, 350.0, 15.0),
        )
        .image("fusion-matter-bar.png")
        .preview_only()
        .z(11),
        spec(
            "character_status_guide",
            "Guide",
            "character_panel",
            "Статус",
            "character_panel",
            rect(350.0, 524.0, 139.0, 57.0),
        )
        .image("guide-box.png")
        .preview_only()
        .z(12),
    ] {
        ensure(document, value);
    }
}

pub(super) fn nano_gallery(document: &mut UiLayoutDocument) {
    ensure_form(
        document,
        "nano_gallery",
        "Nano — все 64 ячейки",
        [345.0, 897.0],
    );
    for slot in 0..64 {
        let column = slot % 5;
        let row = slot / 5;
        ensure(
            document,
            spec(
                &format!("nano_slot_{slot:02}"),
                &format!("Nano {}", slot + 1),
                "nano_gallery",
                "Nano gallery",
                "nano_gallery",
                rect(column as f32 * 69.0, row as f32 * 69.0, 67.0, 67.0),
            )
            .image_dynamic("slot-occupied.png", "NANO ICON")
            .z(0),
        );
    }
}
