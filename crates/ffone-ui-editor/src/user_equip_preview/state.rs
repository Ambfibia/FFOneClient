use super::*;

pub(super) fn inventory_window(document: &mut UiLayoutDocument) {
    ensure(
        document,
        spec(
            "inventory_panel",
            "Фон инвентаря",
            "inventory",
            "Фон",
            "pc_stuff",
            rect(0.0, 0.0, 380.0, 632.0),
        )
        .nine("inventory-panel.png", [136.0, 10.0, 33.0, 77.0])
        .z(0),
    );
    for (id, x, y, width, height) in [
        ("inventory_shadow_a", 3.0, 33.0, 350.0, 503.0),
        ("inventory_shadow_b", 6.0, 33.0, 348.0, 504.0),
    ] {
        ensure(
            document,
            spec(
                id,
                "Тень области",
                "inventory",
                "Фон",
                "pc_stuff",
                rect(x, y, width, height),
            )
            .nine("inventory-shadow.png", [0.0, 0.0, 32.0, 32.0])
            .z(2),
        );
    }
    for value in [
        spec(
            "scroll_track",
            "Полоса прокрутки",
            "inventory",
            "Прокрутка",
            "pc_stuff",
            rect(355.0, 48.0, 17.0, 476.0),
        )
        .nine("scroll-track.png", [2.0, 2.0, 4.0, 4.0])
        .z(40),
        spec(
            "scroll_up",
            "Прокрутка вверх",
            "inventory",
            "Прокрутка",
            "pc_stuff",
            rect(355.0, 36.0, 17.0, 12.0),
        )
        .image("scroll-up.png")
        .z(41),
        spec(
            "scroll_down",
            "Прокрутка вниз",
            "inventory",
            "Прокрутка",
            "pc_stuff",
            rect(355.0, 524.0, 17.0, 12.0),
        )
        .image("scroll-down.png")
        .z(41),
        spec(
            "scroll_thumb",
            "Бегунок",
            "inventory",
            "Прокрутка",
            "pc_stuff",
            rect(357.0, 48.0, 13.0, 15.0),
        )
        .nine("scroll-thumb.png", [2.0, 2.0, 4.0, 4.0])
        .z(42),
    ] {
        ensure(document, value);
    }

    // Forty frames are visible in the unscrolled viewport. The complete
    // 50-slot content is exposed in a separate form below.
    for slot in 0..40 {
        let column = slot % 5;
        let row = slot / 5;
        ensure(
            document,
            spec(
                &format!("inventory_preview_slot_{slot:02}"),
                &format!("Ячейка {}", slot + 1),
                "inventory",
                "Ячейки (видимая область)",
                "pc_stuff",
                rect(
                    6.0 + column as f32 * 69.0,
                    36.0 + row as f32 * 69.0,
                    67.0,
                    67.0,
                ),
            )
            .image("slot-empty.png")
            .preview_only()
            .z(10),
        );
    }
}

pub(super) fn inventory_content(document: &mut UiLayoutDocument) {
    ensure_form(
        document,
        "inventory_content",
        "Инвентарь — все 50 ячеек",
        [345.0, 690.0],
    );
    for slot in 0..50 {
        let column = slot % 5;
        let row = slot / 5;
        ensure(
            document,
            spec(
                &format!("inventory_slot_{slot:02}"),
                &format!("Ячейка инвентаря {}", slot + 1),
                "inventory_content",
                "Все ячейки",
                "inventory_content",
                rect(column as f32 * 69.0, row as f32 * 69.0, 67.0, 67.0),
            )
            .image("slot-empty.png")
            .z(0),
        );
    }
}

pub(super) fn character_status(document: &mut UiLayoutDocument) {
    for value in [
        spec(
            "status_panel_back",
            "Фон статуса",
            "character_status",
            "Фон",
            "user_clothes",
            rect(0.0, 517.0, 498.0, 115.0),
        )
        .nine("user-status-panel.png", [8.0, 8.0, 8.0, 8.0])
        .z(0),
        spec(
            "status_hp_back",
            "Фон здоровья",
            "character_status",
            "Полосы",
            "user_clothes",
            rect(13.0, 585.0, 472.0, 17.0),
        )
        .image("hp-back.png")
        .z(10),
        spec(
            "status_hp_bar",
            "Здоровье",
            "character_status",
            "Полосы",
            "user_clothes",
            rect(14.0, 586.0, 390.0, 15.0),
        )
        .image("hp-bar.png")
        .z(11),
        spec(
            "status_fusion_back",
            "Фон Fusion Matter",
            "character_status",
            "Полосы",
            "user_clothes",
            rect(13.0, 607.0, 472.0, 17.0),
        )
        .image("hp-back.png")
        .z(10),
        spec(
            "status_fusion_bar",
            "Fusion Matter",
            "character_status",
            "Полосы",
            "user_clothes",
            rect(14.0, 608.0, 350.0, 15.0),
        )
        .image("fusion-matter-bar.png")
        .z(11),
        spec(
            "status_guide_box",
            "Панель Guide",
            "character_status",
            "Guide",
            "user_clothes",
            rect(350.0, 524.0, 139.0, 57.0),
        )
        .image("guide-box.png")
        .z(12),
    ] {
        ensure(document, value);
    }
}

pub(super) fn item_mode_overview(document: &mut UiLayoutDocument) {
    ensure_form(
        document,
        "item_mode_overview",
        "Item Mode — взаимное расположение",
        [1036.0, 653.0],
    );
    for value in [
        spec(
            "overview_clothes",
            "Панель персонажа",
            "item_mode_overview",
            "Панели",
            "item_mode_overview",
            rect(0.0, 0.0, 585.0, 653.0),
        )
        .image("clothes-panel.png")
        .preview_only()
        .z(0),
        spec(
            "overview_right",
            "Панель инвентаря",
            "item_mode_overview",
            "Панели",
            "item_mode_overview",
            rect(585.0, 0.0, 451.0, 653.0),
        )
        .nine("right-panel.png", [5.0, 5.0, 0.0, 0.0])
        .preview_only()
        .z(0),
        spec(
            "overview_inventory",
            "Окно инвентаря",
            "item_mode_overview",
            "Панели",
            "item_mode_overview",
            rect(585.0, 0.0, 380.0, 632.0),
        )
        .nine("inventory-panel.png", [136.0, 10.0, 33.0, 77.0])
        .preview_only()
        .z(5),
        spec(
            "overview_equipment",
            "Полоса экипировки",
            "item_mode_overview",
            "Панели",
            "item_mode_overview",
            rect(504.0, 0.0, 66.0, 639.0),
        )
        .fill([2, 12, 22, 180])
        .preview_only()
        .z(5),
    ] {
        ensure(document, value);
    }
    for index in 0..9 {
        ensure(
            document,
            spec(
                &format!("overview_equipment_slot_{index}"),
                "Ячейка экипировки",
                "item_mode_overview",
                "Экипировка",
                "item_mode_overview",
                rect(504.0, 14.0 + index as f32 * 62.0, 64.0, 64.0),
            )
            .image("slot-empty.png")
            .preview_only()
            .z(10),
        );
    }
}
