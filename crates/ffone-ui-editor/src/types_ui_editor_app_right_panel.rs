use super::*;

impl UiEditorApp {


    pub(super) fn right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector").resizable(true).default_width(310.0).show(ctx, |ui| {
            ui.heading("Свойства элемента");
            let Some(index) = self.selected else {
                ui.label("Выберите элемент на холсте или в списке слева.");
                return;
            };
            let before = self.document.clone();
            let origin = self.element_origin(index);
            let mut nudge_step = self.nudge_step;
            let mut nudge = Vec2::ZERO;
            let reset_selected;
            let reset_filtered;
            {
                let element = &mut self.document.elements[index];
                ui.strong(&element.label);
                ui.monospace(&element.id);
                ui.label(format!("Группа: {}", element.group));
                ui.label(format!("Форма: {}", element.form));
                ui.label(format!("Система координат: {}", element.space));
                ui.separator();

                let mut form_x = element.rect.x + origin[0];
                let mut form_y = element.rect.y + origin[1];
                egui::Grid::new("rect_grid")
                    .num_columns(2)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("X на форме");
                        if ui.add(egui::DragValue::new(&mut form_x).speed(1.0)).changed() {
                            element.rect.x = form_x - origin[0];
                        }
                        ui.end_row();
                        ui.label("Y на форме");
                        if ui.add(egui::DragValue::new(&mut form_y).speed(1.0)).changed() {
                            element.rect.y = form_y - origin[1];
                        }
                        ui.end_row();
                        ui.label("Ширина");
                        ui.add(
                            egui::DragValue::new(&mut element.rect.width).range(1.0..=4096.0),
                        );
                        ui.end_row();
                        ui.label("Высота");
                        ui.add(
                            egui::DragValue::new(&mut element.rect.height).range(1.0..=4096.0),
                        );
                        ui.end_row();
                    });

                ui.add_space(6.0);
                ui.label("Перемещение кнопками:");
                ui.horizontal(|ui| {
                    ui.label("Шаг");
                    ui.selectable_value(&mut nudge_step, 1.0, "1");
                    ui.selectable_value(&mut nudge_step, 5.0, "5");
                    ui.selectable_value(&mut nudge_step, 10.0, "10");
                });
                ui.horizontal(|ui| {
                    if ui.button("←").clicked() {
                        nudge.x -= nudge_step;
                    }
                    if ui.button("↑").clicked() {
                        nudge.y -= nudge_step;
                    }
                    if ui.button("↓").clicked() {
                        nudge.y += nudge_step;
                    }
                    if ui.button("→").clicked() {
                        nudge.x += nudge_step;
                    }
                });

                ui.checkbox(&mut element.override_enabled, "Применять в игре");
                ui.checkbox(&mut element.locked, "Заблокировать перемещение");
                ui.checkbox(&mut element.editor_hidden, "Скрыть на холсте");
                ui.collapsing("Внешний вид в редакторе", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Порядок (Z)");
                        ui.add(egui::DragValue::new(&mut element.z_index).speed(1));
                    });
                    if let Some(visual) = &element.visual {
                        if let Some(image) = &visual.image {
                            ui.label("Картинка:");
                            ui.monospace(&image.path);
                            ui.label(format!("Режим: {:?}", image.mode));
                        }
                        if let Some(text) = &visual.text {
                            ui.label(format!("Label: {}", text.value));
                            if let Some(font) = &text.font {
                                ui.monospace(font);
                            }
                        }
                        if visual.dynamic_placeholder {
                            ui.colored_label(
                                Color32::LIGHT_BLUE,
                                "Runtime-изображение: в редакторе показан checker",
                            );
                        }
                    } else {
                        ui.colored_label(
                            Color32::YELLOW,
                            "Для объекта не задана визуальная preview-роль",
                        );
                    }
                });
                ui.label("Заметки");
                ui.text_edit_multiline(&mut element.notes);
                ui.separator();
                reset_selected = ui.button("Вернуть исходный Rect").clicked();
                reset_filtered = ui.button("Сбросить элементы текущей группы").clicked();
                ui.collapsing("Технические координаты", |ui| {
                    ui.monospace(format!(
                        "local: x={} y={}\norigin: x={} y={}",
                        element.rect.x, element.rect.y, origin[0], origin[1]
                    ));
                });
            }
            self.nudge_step = nudge_step;
            self.document.elements[index].rect.x += nudge.x;
            self.document.elements[index].rect.y += nudge.y;
            if reset_selected {
                self.document.elements[index].rect = self.document.elements[index].source_rect;
            }
            if reset_filtered {
                for target in &mut self.document.elements {
                    if target.form == self.active_form
                        && (self.group_filter == "All" || target.group == self.group_filter)
                    {
                        target.rect = target.source_rect;
                    }
                }
            }
            if self.document != before {
                self.undo.push(before);
                self.redo.clear();
                self.dirty = true;
            }
            ui.separator();
            ui.heading("Выравнивание");
            let reference_label = self
                .alignment_reference
                .and_then(|reference| self.document.elements.get(reference))
                .map_or_else(|| "Выберите опорный объект".to_owned(), |element| {
                    element.label.clone()
                });
            let candidates: Vec<_> = self
                .document
                .elements
                .iter()
                .enumerate()
                .filter(|(candidate, element)| {
                    *candidate != index && element.form == self.active_form
                })
                .map(|(candidate, element)| {
                    (candidate, element.label.clone(), element.id.clone())
                })
                .collect();
            egui::ComboBox::from_label("Относительно")
                .selected_text(reference_label)
                .width(260.0)
                .show_ui(ui, |ui| {
                    for (candidate, label, id) in candidates {
                        if ui
                            .selectable_label(
                                self.alignment_reference == Some(candidate),
                                format!("{label}  [{id}]"),
                            )
                            .clicked()
                        {
                            self.alignment_reference = Some(candidate);
                        }
                    }
                });
            ui.horizontal_wrapped(|ui| {
                for (label, action, hint) in [
                    ("L", AlignAction::Left, "Левые края"),
                    ("C↔", AlignAction::HorizontalCenter, "Центры по X"),
                    ("R", AlignAction::Right, "Правые края"),
                    ("T", AlignAction::Top, "Верхние края"),
                    ("C↕", AlignAction::VerticalCenter, "Центры по Y"),
                    ("B", AlignAction::Bottom, "Нижние края"),
                    ("W", AlignAction::MatchWidth, "Та же ширина"),
                    ("H", AlignAction::MatchHeight, "Та же высота"),
                ] {
                    if ui
                        .add_enabled(
                            self.alignment_reference.is_some(),
                            egui::Button::new(label),
                        )
                        .on_hover_text(hint)
                        .clicked()
                    {
                        self.align_selected_to_reference(action);
                    }
                }
            });
            ui.label(format!(
                "Вся группа «{}» относительно выбранного:",
                self.document.elements[index].group
            ));
            ui.horizontal_wrapped(|ui| {
                for (label, action, hint) in [
                    ("L", AlignAction::Left, "Левые края"),
                    ("C↔", AlignAction::HorizontalCenter, "Центры по X"),
                    ("R", AlignAction::Right, "Правые края"),
                    ("T", AlignAction::Top, "Верхние края"),
                    ("C↕", AlignAction::VerticalCenter, "Центры по Y"),
                    ("B", AlignAction::Bottom, "Нижние края"),
                    ("W", AlignAction::MatchWidth, "Одинаковая ширина"),
                    ("H", AlignAction::MatchHeight, "Одинаковая высота"),
                ] {
                    if ui.button(label).on_hover_text(hint).clicked() {
                        self.align_group_to_selected(action);
                    }
                }
            });
            ui.horizontal(|ui| {
                if ui.button("Распределить по X").clicked() {
                    self.distribute_selected_group(true);
                }
                if ui.button("Распределить по Y").clicked() {
                    self.distribute_selected_group(false);
                }
            });
            ui.separator();
            ui.small("ЛКМ по элементу — выбрать и перетащить. Белый маркер справа снизу — изменить размер. Стрелки двигают на 1 px, Shift+стрелки — на 10 px.");
        });
    }

    pub(super) fn canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(form) = self.active_form().cloned() else {
                ui.centered_and_justified(|ui| ui.label("В документе нет форм UI."));
                return;
            };
            ui.horizontal_wrapped(|ui| {
                ui.strong("Холст:");
                ui.label("перетаскивайте ЛКМ; размер меняется белым маркером выбранного элемента");
                if let Some(index) = self.selected {
                    ui.separator();
                    ui.label(format!("Выбран: {}", self.document.elements[index].label));
                }
            });
            if self.fit_requested {
                let available = ui.available_size() - Vec2::splat(24.0);
                self.zoom = (available.x / form.canvas_size[0])
                    .min(available.y / form.canvas_size[1])
                    .clamp(0.25, 2.5);
                self.fit_requested = false;
            }
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .drag_to_scroll(false)
                .show(ui, |ui| {
                    let size = Vec2::new(
                        form.canvas_size[0] * self.zoom,
                        form.canvas_size[1] * self.zoom,
                    );
                    let (canvas, _) = ui.allocate_exact_size(size, Sense::hover());
                    let painter = ui.painter_at(canvas);
                    painter.rect_filled(canvas, 0.0, Color32::from_rgb(8, 18, 29));
                    if let Some(background) = self.backgrounds.get(&form.id) {
                        let background_rect = form.background_rect.unwrap_or(UiLayoutRect::new(
                            0.0,
                            0.0,
                            form.canvas_size[0],
                            form.canvas_size[1],
                        ));
                        let target = Rect::from_min_size(
                            canvas.min
                                + Vec2::new(background_rect.x, background_rect.y) * self.zoom,
                            Vec2::new(background_rect.width, background_rect.height) * self.zoom,
                        );
                        let uv = form.background_source_rect.map_or_else(
                            || Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            |source| {
                                let texture_size = background.size_vec2();
                                Rect::from_min_max(
                                    Pos2::new(source.x / texture_size.x, source.y / texture_size.y),
                                    Pos2::new(
                                        (source.x + source.width) / texture_size.x,
                                        (source.y + source.height) / texture_size.y,
                                    ),
                                )
                            },
                        );
                        painter.image(
                            background.id(),
                            target,
                            uv,
                            Color32::from_white_alpha(
                                (self.background_opacity * 255.0).round() as u8
                            ),
                        );
                    }
                    painter.rect_stroke(
                        canvas,
                        0.0,
                        Stroke::new(2.0_f32, Color32::from_rgb(0, 220, 240)),
                        StrokeKind::Inside,
                    );
                    if self.show_grid {
                        draw_grid(&painter, canvas, self.zoom, self.grid);
                    }
                    let active_indices: Vec<_> = (0..self.document.elements.len())
                        .filter(|index| self.element_is_on_active_form(*index))
                        .filter(|index| {
                            !self.document.elements[*index].editor_hidden || self.show_hidden
                        })
                        .collect();

                    if self.show_visuals {
                        let mut paint_indices = active_indices.clone();
                        paint_indices
                            .sort_by_key(|index| (self.document.elements[*index].z_index, *index));
                        for index in paint_indices {
                            let rect = self.screen_rect(index, canvas);
                            if let Some(visual) = &self.document.elements[index].visual {
                                draw_element_visual(
                                    &painter,
                                    rect,
                                    visual,
                                    &self.textures,
                                    &self.font_families,
                                    self.zoom,
                                );
                            }
                        }
                    }

                    for &index in &active_indices {
                        let rect = self.screen_rect(index, canvas);
                        let selected = self.selected == Some(index);
                        let color = group_color(&self.document.elements[index].group);
                        if self.show_bounds || selected {
                            if selected {
                                painter.rect_filled(
                                    rect,
                                    0.0,
                                    Color32::from_rgba_unmultiplied(
                                        color.r(),
                                        color.g(),
                                        color.b(),
                                        28,
                                    ),
                                );
                            }
                            painter.rect_stroke(
                                rect,
                                0.0,
                                Stroke::new(
                                    if selected { 2.0_f32 } else { 1.0 },
                                    if selected {
                                        Color32::WHITE
                                    } else {
                                        color.gamma_multiply(0.65)
                                    },
                                ),
                                StrokeKind::Inside,
                            );
                        }
                        if self.show_object_names && rect.width() > 55.0 && rect.height() > 12.0 {
                            painter.text(
                                rect.min + Vec2::new(3.0, 2.0),
                                egui::Align2::LEFT_TOP,
                                &self.document.elements[index].label,
                                FontId::monospace(10.0),
                                Color32::WHITE,
                            );
                        }
                        let handle = Rect::from_center_size(rect.max, Vec2::splat(HANDLE_SIZE));
                        if selected {
                            painter.rect_filled(handle, 1.0, Color32::WHITE);
                        }
                    }

                    let (pointer, primary_pressed, primary_down, primary_released) =
                        ui.input(|input| {
                            (
                                input.pointer.interact_pos(),
                                input.pointer.button_pressed(egui::PointerButton::Primary),
                                input.pointer.button_down(egui::PointerButton::Primary),
                                input.pointer.button_released(egui::PointerButton::Primary),
                            )
                        });
                    let hovered =
                        self.active_drag
                            .as_ref()
                            .map(|drag| drag.element)
                            .or_else(|| {
                                let position = pointer?;
                                if let Some(selected) = self.selected
                                    && active_indices.contains(&selected)
                                {
                                    let rect = self.screen_rect(selected, canvas);
                                    let handle =
                                        Rect::from_center_size(rect.max, Vec2::splat(HANDLE_SIZE));
                                    if handle.contains(position) || rect.contains(position) {
                                        return Some(selected);
                                    }
                                }
                                active_indices
                                    .iter()
                                    .copied()
                                    .filter(|index| {
                                        self.screen_rect(*index, canvas).contains(position)
                                    })
                                    .min_by(|left, right| {
                                        self.screen_rect(*left, canvas)
                                            .area()
                                            .total_cmp(&self.screen_rect(*right, canvas).area())
                                    })
                            });

                    if let Some(index) = hovered {
                        let rect = self.screen_rect(index, canvas);
                        painter.rect_stroke(
                            rect.expand(2.0),
                            1.0,
                            Stroke::new(3.0_f32, Color32::YELLOW),
                            StrokeKind::Outside,
                        );
                        painter.circle_filled(rect.center(), 4.0, Color32::YELLOW);
                    }

                    if let Some(index) = hovered
                        && self.active_drag.is_none()
                    {
                        let rect = self.screen_rect(index, canvas);
                        let handle = Rect::from_center_size(rect.max, Vec2::splat(HANDLE_SIZE));
                        if self.selected == Some(index)
                            && pointer.is_some_and(|position| handle.contains(position))
                        {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                        } else {
                            ctx.set_cursor_icon(egui::CursorIcon::Grab);
                        }
                    }

                    if primary_pressed {
                        if let (Some(index), Some(position)) = (hovered, pointer) {
                            let was_selected = self.selected == Some(index);
                            if !was_selected {
                                self.alignment_reference = self.selected.filter(|selected| {
                                    self.document.elements[*selected].form == self.active_form
                                });
                            }
                            self.selected = Some(index);
                            if !self.document.elements[index].locked {
                                let rect = self.screen_rect(index, canvas);
                                let handle =
                                    Rect::from_center_size(rect.max, Vec2::splat(HANDLE_SIZE));
                                self.active_drag = Some(ActiveDrag {
                                    element: index,
                                    mode: if was_selected && handle.contains(position) {
                                        DragMode::Resize
                                    } else {
                                        DragMode::Move
                                    },
                                    start: self.document.elements[index].rect,
                                    pointer_start: position,
                                    history_pushed: false,
                                });
                                self.status = format!(
                                    "Перетаскивание: {}",
                                    self.document.elements[index].label
                                );
                            } else {
                                self.status = format!(
                                    "Элемент заблокирован: {}",
                                    self.document.elements[index].label
                                );
                            }
                        } else if pointer.is_some_and(|position| canvas.contains(position)) {
                            self.selected = None;
                            self.alignment_reference = None;
                        }
                    }

                    if let Some(drag) = self.active_drag
                        && (primary_down || primary_released)
                        && let Some(position) = pointer
                    {
                        let delta = (position - drag.pointer_start) / self.zoom;
                        if delta != Vec2::ZERO {
                            if !drag.history_pushed {
                                self.push_undo();
                                if let Some(active) = &mut self.active_drag {
                                    active.history_pushed = true;
                                }
                            }
                            let (x, y, width, height) = match drag.mode {
                                DragMode::Move => (
                                    self.snap_value(drag.start.x + delta.x),
                                    self.snap_value(drag.start.y + delta.y),
                                    drag.start.width,
                                    drag.start.height,
                                ),
                                DragMode::Resize => (
                                    drag.start.x,
                                    drag.start.y,
                                    self.snap_value((drag.start.width + delta.x).max(1.0)),
                                    self.snap_value((drag.start.height + delta.y).max(1.0)),
                                ),
                            };
                            self.document.elements[drag.element].rect =
                                UiLayoutRect::new(x, y, width, height);
                            self.status = format!(
                                "{}: x={x:.0}, y={y:.0}, w={width:.0}, h={height:.0}",
                                self.document.elements[drag.element].label
                            );
                        }
                    }

                    if let Some(drag) = self.active_drag {
                        ctx.set_cursor_icon(match drag.mode {
                            DragMode::Move => egui::CursorIcon::Grabbing,
                            DragMode::Resize => egui::CursorIcon::ResizeNwSe,
                        });
                    }
                    if primary_released {
                        self.active_drag = None;
                    }
                });
        });
    }

}

impl eframe::App for UiEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);
        self.top_bar(ctx);
        self.left_panel(ctx);
        self.right_panel(ctx);
        self.canvas(ctx);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(format!(
            "FFOne UI Editor{}",
            if self.dirty { " *" } else { "" }
        )));
    }
}
