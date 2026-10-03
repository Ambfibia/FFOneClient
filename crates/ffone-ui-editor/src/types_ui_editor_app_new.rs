use super::*;

#[derive(Clone, Copy)]
pub(super) struct ActiveDrag {
    pub(super) element: usize,
    pub(super) mode: DragMode,
    pub(super) start: UiLayoutRect,
    pub(super) pointer_start: Pos2,
    pub(super) history_pushed: bool,
}

pub(super) struct UiEditorApp {
    pub(super) document: UiLayoutDocument,
    pub(super) layout_path: PathBuf,
    pub(super) background_path: PathBuf,
    pub(super) backgrounds: BTreeMap<String, egui::TextureHandle>,
    pub(super) textures: BTreeMap<String, egui::TextureHandle>,
    pub(super) font_families: BTreeMap<String, FontFamily>,
    pub(super) active_form: String,
    pub(super) selected: Option<usize>,
    pub(super) alignment_reference: Option<usize>,
    pub(super) filter: String,
    pub(super) group_filter: String,
    pub(super) zoom: f32,
    pub(super) grid: f32,
    pub(super) snap: bool,
    pub(super) show_grid: bool,
    pub(super) show_hidden: bool,
    pub(super) show_visuals: bool,
    pub(super) show_bounds: bool,
    pub(super) show_object_names: bool,
    pub(super) background_opacity: f32,
    pub(super) nudge_step: f32,
    pub(super) fit_requested: bool,
    pub(super) active_drag: Option<ActiveDrag>,
    pub(super) undo: Vec<UiLayoutDocument>,
    pub(super) redo: Vec<UiLayoutDocument>,
    pub(super) dirty: bool,
    pub(super) status: String,
}

impl UiEditorApp {
    pub(super) fn new(
        cc: &eframe::CreationContext<'_>,
        layout_path: PathBuf,
        background_path: PathBuf,
    ) -> Self {
        let (mut document, status) = match UiLayoutDocument::load(&layout_path) {
            Ok(document) => (document, format!("Открыто: {}", layout_path.display())),
            Err(error) => panic!("cannot open {}: {error}", layout_path.display()),
        };
        user_equip_preview::enrich(&mut document);
        let backgrounds = load_form_textures(&cc.egui_ctx, &document, Some(&background_path));
        let textures = load_element_textures(&cc.egui_ctx, &document);
        let font_families = install_document_fonts(&cc.egui_ctx, &document);
        let active_form = document
            .forms
            .first()
            .map(|form| form.id.clone())
            .unwrap_or_else(|| "main".to_owned());
        Self {
            document,
            layout_path,
            background_path,
            backgrounds,
            textures,
            font_families,
            active_form,
            selected: None,
            alignment_reference: None,
            filter: String::new(),
            group_filter: "All".to_owned(),
            zoom: 0.9,
            grid: 1.0,
            snap: true,
            show_grid: true,
            show_hidden: false,
            show_visuals: true,
            show_bounds: true,
            show_object_names: false,
            background_opacity: 0.0,
            nudge_step: 1.0,
            fit_requested: true,
            active_drag: None,
            undo: Vec::new(),
            redo: Vec::new(),
            dirty: false,
            status,
        }
    }

    pub(super) fn push_undo(&mut self) {
        self.undo.push(self.document.clone());
        if self.undo.len() > HISTORY_LIMIT {
            self.undo.remove(0);
        }
        self.redo.clear();
        self.dirty = true;
    }

    pub(super) fn undo(&mut self) {
        if let Some(previous) = self.undo.pop() {
            self.redo
                .push(std::mem::replace(&mut self.document, previous));
            self.selected = self
                .selected
                .filter(|index| *index < self.document.elements.len());
            self.dirty = true;
            self.status = "Изменение отменено".to_owned();
        }
    }

    pub(super) fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.document, next));
            self.selected = self
                .selected
                .filter(|index| *index < self.document.elements.len());
            self.dirty = true;
            self.status = "Изменение повторено".to_owned();
        }
    }

    pub(super) fn save(&mut self) {
        match self
            .document
            .to_pretty_json()
            .and_then(|bytes| fs::write(&self.layout_path, bytes).map_err(Into::into))
        {
            Ok(()) => {
                self.dirty = false;
                self.status = format!("Сохранено: {}", self.layout_path.display());
            }
            Err(error) => self.status = format!("Ошибка сохранения: {error}"),
        }
    }

    pub(super) fn reload(&mut self, ctx: &egui::Context) {
        match UiLayoutDocument::load(&self.layout_path) {
            Ok(mut document) => {
                user_equip_preview::enrich(&mut document);
                self.document = document;
                self.backgrounds =
                    load_form_textures(ctx, &self.document, Some(&self.background_path));
                self.textures = load_element_textures(ctx, &self.document);
                self.font_families = install_document_fonts(ctx, &self.document);
                self.undo.clear();
                self.redo.clear();
                self.selected = None;
                self.alignment_reference = None;
                if self.document.form(&self.active_form).is_none() {
                    self.active_form = self
                        .document
                        .forms
                        .first()
                        .map(|form| form.id.clone())
                        .unwrap_or_else(|| "main".to_owned());
                }
                self.dirty = false;
                self.status = "JSON перечитан с диска".to_owned();
            }
            Err(error) => self.status = format!("Ошибка чтения JSON: {error}"),
        }
    }

    pub(super) fn reload_background(&mut self, ctx: &egui::Context) {
        self.backgrounds = load_form_textures(ctx, &self.document, Some(&self.background_path));
        self.textures = load_element_textures(ctx, &self.document);
        self.font_families = install_document_fonts(ctx, &self.document);
        self.status = format!(
            "Обновлено: эталонов {}, UI-текстур {}",
            self.backgrounds.len(),
            self.textures.len()
        );
    }

    pub(super) fn active_form(&self) -> Option<&UiLayoutForm> {
        self.document.form(&self.active_form)
    }

    pub(super) fn element_is_on_active_form(&self, index: usize) -> bool {
        self.document.elements[index].form == self.active_form
    }

    pub(super) fn groups(&self) -> Vec<String> {
        let mut groups: Vec<_> = self
            .document
            .elements
            .iter()
            .filter(|element| element.form == self.active_form)
            .map(|element| element.group.clone())
            .collect();
        groups.sort();
        groups.dedup();
        groups
    }

    pub(super) fn element_visible_in_list(&self, index: usize) -> bool {
        let element = &self.document.elements[index];
        let needle = self.filter.trim().to_ascii_lowercase();
        element.form == self.active_form
            && (self.group_filter == "All" || element.group == self.group_filter)
            && (needle.is_empty()
                || element.id.to_ascii_lowercase().contains(&needle)
                || element.label.to_ascii_lowercase().contains(&needle))
    }

    pub(super) fn screen_rect(&self, index: usize, canvas: Rect) -> Rect {
        let element = &self.document.elements[index];
        let origin = self.element_origin(index);
        Rect::from_min_size(
            canvas.min
                + Vec2::new(
                    (origin[0] + element.rect.x) * self.zoom,
                    (origin[1] + element.rect.y) * self.zoom,
                ),
            Vec2::new(
                element.rect.width * self.zoom,
                element.rect.height * self.zoom,
            ),
        )
    }

    pub(super) fn element_origin(&self, index: usize) -> [f32; 2] {
        let element = &self.document.elements[index];
        self.document
            .form(&element.form)
            .and_then(|form| form.space_origins.get(&element.space).copied())
            .or_else(|| self.document.space_origin(&element.space))
            .unwrap_or([0.0, 0.0])
    }

    pub(super) fn snap_value(&self, value: f32) -> f32 {
        if self.snap && self.grid > 0.0 {
            (value / self.grid).round() * self.grid
        } else {
            value
        }
    }

    pub(super) fn form_rect(&self, index: usize) -> UiLayoutRect {
        let element = &self.document.elements[index];
        let origin = self.element_origin(index);
        UiLayoutRect::new(
            element.rect.x + origin[0],
            element.rect.y + origin[1],
            element.rect.width,
            element.rect.height,
        )
    }

    pub(super) fn align_selected_to_reference(&mut self, action: AlignAction) {
        let (Some(index), Some(reference)) = (self.selected, self.alignment_reference) else {
            return;
        };
        if index == reference
            || !self.element_is_on_active_form(index)
            || !self.element_is_on_active_form(reference)
            || self.document.elements[index].locked
        {
            return;
        }
        let before = self.document.clone();
        let target = self.form_rect(index);
        let anchor = self.form_rect(reference);
        let origin = self.element_origin(index);
        let rect = &mut self.document.elements[index].rect;
        match action {
            AlignAction::Left => rect.x = anchor.x - origin[0],
            AlignAction::HorizontalCenter => {
                rect.x = anchor.x + (anchor.width - target.width) * 0.5 - origin[0];
            }
            AlignAction::Right => {
                rect.x = anchor.x + anchor.width - target.width - origin[0];
            }
            AlignAction::Top => rect.y = anchor.y - origin[1],
            AlignAction::VerticalCenter => {
                rect.y = anchor.y + (anchor.height - target.height) * 0.5 - origin[1];
            }
            AlignAction::Bottom => {
                rect.y = anchor.y + anchor.height - target.height - origin[1];
            }
            AlignAction::MatchWidth => rect.width = anchor.width,
            AlignAction::MatchHeight => rect.height = anchor.height,
        }
        if self.document != before {
            self.undo.push(before);
            self.redo.clear();
            self.dirty = true;
            self.status = "Элемент выровнен относительно опорного объекта".to_owned();
        }
    }

    pub(super) fn align_group_to_selected(&mut self, action: AlignAction) {
        let Some(reference) = self.selected else {
            return;
        };
        let group = self.document.elements[reference].group.clone();
        let anchor = self.form_rect(reference);
        let targets: Vec<_> = self
            .document
            .elements
            .iter()
            .enumerate()
            .filter(|(index, element)| {
                *index != reference
                    && element.form == self.active_form
                    && element.group == group
                    && !element.locked
            })
            .map(|(index, _)| index)
            .collect();
        if targets.is_empty() {
            return;
        }
        let before = self.document.clone();
        for index in targets {
            let target = self.form_rect(index);
            let origin = self.element_origin(index);
            let rect = &mut self.document.elements[index].rect;
            match action {
                AlignAction::Left => rect.x = anchor.x - origin[0],
                AlignAction::HorizontalCenter => {
                    rect.x = anchor.x + (anchor.width - target.width) * 0.5 - origin[0];
                }
                AlignAction::Right => {
                    rect.x = anchor.x + anchor.width - target.width - origin[0];
                }
                AlignAction::Top => rect.y = anchor.y - origin[1],
                AlignAction::VerticalCenter => {
                    rect.y = anchor.y + (anchor.height - target.height) * 0.5 - origin[1];
                }
                AlignAction::Bottom => {
                    rect.y = anchor.y + anchor.height - target.height - origin[1];
                }
                AlignAction::MatchWidth => rect.width = anchor.width,
                AlignAction::MatchHeight => rect.height = anchor.height,
            }
        }
        self.undo.push(before);
        self.redo.clear();
        self.dirty = true;
        self.status = format!("Группа «{group}» выровнена");
    }

    pub(super) fn distribute_selected_group(&mut self, horizontal: bool) {
        let Some(selected) = self.selected else {
            return;
        };
        let group = self.document.elements[selected].group.clone();
        let mut indices: Vec<_> = self
            .document
            .elements
            .iter()
            .enumerate()
            .filter(|(_, element)| {
                element.form == self.active_form && element.group == group && !element.locked
            })
            .map(|(index, _)| index)
            .collect();
        if indices.len() < 3 {
            self.status =
                "Для распределения в группе нужно минимум 3 незаблокированных объекта".to_owned();
            return;
        }
        indices.sort_by(|left, right| {
            let left = self.form_rect(*left);
            let right = self.form_rect(*right);
            let (left, right) = if horizontal {
                (left.x, right.x)
            } else {
                (left.y, right.y)
            };
            left.total_cmp(&right)
        });
        let before = self.document.clone();
        let first = self.form_rect(indices[0]);
        let last = self.form_rect(*indices.last().expect("three or more indices"));
        let total_size: f32 = indices
            .iter()
            .map(|index| {
                let rect = self.form_rect(*index);
                if horizontal { rect.width } else { rect.height }
            })
            .sum();
        let span = if horizontal {
            last.x + last.width - first.x
        } else {
            last.y + last.height - first.y
        };
        let gap = (span - total_size) / (indices.len() - 1) as f32;
        let mut cursor = if horizontal { first.x } else { first.y };
        for index in indices {
            let form_rect = self.form_rect(index);
            let origin = self.element_origin(index);
            let rect = &mut self.document.elements[index].rect;
            if horizontal {
                rect.x = cursor - origin[0];
                cursor += form_rect.width + gap;
            } else {
                rect.y = cursor - origin[1];
                cursor += form_rect.height + gap;
            }
        }
        self.undo.push(before);
        self.redo.clear();
        self.dirty = true;
        self.status = format!(
            "Группа «{group}» равномерно распределена по {}",
            if horizontal { "X" } else { "Y" }
        );
    }

    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let command = ctx.input(|input| input.modifiers.command);
        if command && ctx.input(|input| input.key_pressed(egui::Key::S)) {
            self.save();
        }
        if command && ctx.input(|input| input.key_pressed(egui::Key::Z)) {
            if ctx.input(|input| input.modifiers.shift) {
                self.redo();
            } else {
                self.undo();
            }
        }
        if command && ctx.input(|input| input.key_pressed(egui::Key::Y)) {
            self.redo();
        }
        if ctx.wants_keyboard_input() {
            return;
        }
        let Some(index) = self.selected else { return };
        if self.document.elements[index].locked {
            return;
        }
        let step = if ctx.input(|input| input.modifiers.shift) {
            10.0
        } else {
            1.0
        };
        let delta = Vec2::new(
            f32::from(ctx.input(|input| input.key_pressed(egui::Key::ArrowRight)))
                - f32::from(ctx.input(|input| input.key_pressed(egui::Key::ArrowLeft))),
            f32::from(ctx.input(|input| input.key_pressed(egui::Key::ArrowDown)))
                - f32::from(ctx.input(|input| input.key_pressed(egui::Key::ArrowUp))),
        ) * step;
        if delta != Vec2::ZERO {
            self.push_undo();
            self.document.elements[index].rect.x += delta.x;
            self.document.elements[index].rect.y += delta.y;
        }
    }

    pub(super) fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("FFOne UI Editor");
                ui.separator();
                if ui.button("Сохранить  Ctrl+S").clicked() {
                    self.save();
                }
                if ui
                    .add_enabled(!self.undo.is_empty(), egui::Button::new("Отменить"))
                    .clicked()
                {
                    self.undo();
                }
                if ui
                    .add_enabled(!self.redo.is_empty(), egui::Button::new("Повторить"))
                    .clicked()
                {
                    self.redo();
                }
                if ui.button("Перезагрузить JSON").clicked() {
                    self.reload(ctx);
                }
                if ui.button("Обновить фоны").clicked() {
                    self.reload_background(ctx);
                }
                if self.dirty {
                    ui.colored_label(Color32::YELLOW, "* не сохранено");
                }
            });
            ui.horizontal_wrapped(|ui| {
                ui.checkbox(&mut self.snap, "Привязка");
                ui.add(
                    egui::DragValue::new(&mut self.grid)
                        .range(0.5..=64.0)
                        .prefix("Шаг "),
                );
                ui.checkbox(&mut self.show_grid, "Сетка");
                ui.checkbox(&mut self.show_visuals, "UI");
                ui.checkbox(&mut self.show_bounds, "Границы");
                ui.checkbox(&mut self.show_object_names, "Имена объектов");
                ui.add(egui::Slider::new(&mut self.zoom, 0.25..=2.5).text("Масштаб"));
                ui.add(
                    egui::Slider::new(&mut self.background_opacity, 0.0..=1.0)
                        .text("Эталонный скриншот"),
                );
                if ui.button("Вписать форму").clicked() {
                    self.fit_requested = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label(self.layout_path.display().to_string());
                ui.separator();
                ui.label(&self.status);
            });
            ui.separator();
            ui.horizontal_wrapped(|ui| {
                let selected_label = self
                    .active_form()
                    .map_or_else(|| self.active_form.clone(), |form| form.label.clone());
                let forms: Vec<_> = self
                    .document
                    .forms
                    .iter()
                    .map(|form| (form.id.clone(), form.label.clone()))
                    .collect();
                egui::ComboBox::from_label("Форма UI")
                    .selected_text(selected_label)
                    .width(310.0)
                    .show_ui(ui, |ui| {
                        for (id, label) in forms {
                            if ui.selectable_label(self.active_form == id, label).clicked() {
                                self.active_form = id;
                                self.selected = None;
                                self.alignment_reference = None;
                                self.group_filter = "All".to_owned();
                                self.active_drag = None;
                                self.fit_requested = true;
                            }
                        }
                    });
                let count = self
                    .document
                    .elements
                    .iter()
                    .filter(|element| element.form == self.active_form)
                    .count();
                ui.label(format!("{count} элементов"));
            });
        });
    }

    pub(super) fn left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("elements")
            .resizable(true)
            .default_width(270.0)
            .show(ctx, |ui| {
                let form_label = match self.active_form.as_str() {
                    "inventory" => "Инвентарь".to_owned(),
                    "item_popup" => "Карточка — экипировать".to_owned(),
                    "item_popup_unequip" => "Карточка — снять".to_owned(),
                    "item_popup_use" => "Карточка — использовать".to_owned(),
                    "item_popup_chest" => "Карточка — сундук".to_owned(),
                    "character_status" => "Статус персонажа".to_owned(),
                    "equipment_strip" => "Ячейки экипировки".to_owned(),
                    _ => self
                        .active_form()
                        .map_or_else(|| self.active_form.clone(), |form| form.label.clone()),
                };
                ui.heading(form_label);
                ui.label("Здесь показаны элементы только этой формы.");
                ui.add(
                    egui::TextEdit::singleline(&mut self.filter)
                        .hint_text("Поиск по имени или id..."),
                );
                let groups = self.groups();
                let selected_group = if self.group_filter == "All" {
                    "Все".to_owned()
                } else {
                    self.group_filter.clone()
                };
                egui::ComboBox::from_label("Группа")
                    .selected_text(selected_group)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.group_filter, "All".to_owned(), "Все");
                        for group in groups {
                            ui.selectable_value(&mut self.group_filter, group.clone(), group);
                        }
                    });
                ui.checkbox(&mut self.show_hidden, "Показывать скрытые");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for index in 0..self.document.elements.len() {
                        if !self.element_visible_in_list(index) {
                            continue;
                        }
                        let element = &self.document.elements[index];
                        let prefix = if element.locked {
                            "[закрыт] "
                        } else if element.editor_hidden {
                            "[скрыт] "
                        } else {
                            ""
                        };
                        if ui
                            .selectable_label(
                                self.selected == Some(index),
                                format!("{prefix}{}", element.label),
                            )
                            .on_hover_text(&element.id)
                            .clicked()
                        {
                            if self.selected != Some(index) {
                                self.alignment_reference = self.selected.filter(|selected| {
                                    self.document.elements[*selected].form == self.active_form
                                });
                            }
                            self.selected = Some(index);
                        }
                    }
                });
            });
    }
}
