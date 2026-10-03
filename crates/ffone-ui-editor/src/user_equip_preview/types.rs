use super::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PopupPreviewVariant {
    Unequip,
    Use,
    Chest,
}

#[derive(Clone)]
pub(super) struct ElementSpec {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) form: String,
    pub(super) group: String,
    pub(super) space: String,
    pub(super) rect: UiLayoutRect,
    pub(super) visual: Option<UiLayoutVisual>,
    pub(super) z_index: i32,
    pub(super) override_enabled: bool,
    pub(super) notes: String,
}

impl ElementSpec {
    pub(super) fn image(mut self, name: &str) -> Self {
        self.visual = Some(image(name));
        self
    }

    pub(super) fn nine(mut self, name: &str, border: [f32; 4]) -> Self {
        self.visual = Some(nine(name, border));
        self
    }

    pub(super) fn text(mut self, value: &str, size: f32, color: [u8; 4], align: Align) -> Self {
        self.visual = Some(text(value, size, color, align));
        self
    }

    pub(super) fn image_text(
        mut self,
        name: &str,
        value: &str,
        size: f32,
        color: [u8; 4],
        align: Align,
        border: Option<[f32; 4]>,
    ) -> Self {
        self.visual = Some(image_text(name, value, size, color, align, border));
        self
    }

    pub(super) fn dynamic(mut self, value: &str) -> Self {
        self.visual = Some(dynamic(value));
        self
    }

    pub(super) fn image_dynamic(mut self, name: &str, value: &str) -> Self {
        let mut visual = image(name);
        visual.dynamic_placeholder = true;
        visual.text = Some(text_value(value, 7.0, [255, 255, 255, 190], Align::Center));
        self.visual = Some(visual);
        self
    }

    pub(super) fn fill(mut self, color: [u8; 4]) -> Self {
        self.visual = Some(UiLayoutVisual {
            image: None,
            text: None,
            fill: Some(color),
            dynamic_placeholder: false,
        });
        self
    }

    pub(super) fn fill_text(
        mut self,
        fill: [u8; 4],
        value: &str,
        size: f32,
        color: [u8; 4],
        align: Align,
    ) -> Self {
        let mut visual = text(value, size, color, align);
        visual.fill = Some(fill);
        self.visual = Some(visual);
        self
    }

    pub(super) fn z(mut self, value: i32) -> Self {
        self.z_index = value;
        self
    }

    pub(super) fn preview_only(mut self) -> Self {
        self.override_enabled = false;
        self.notes =
            "Только визуальная копия для редактора; runtime использует исходный узел.".to_owned();
        self
    }
}

#[derive(Clone, Copy)]
pub(super) enum Align {
    Left,
    Center,
    Right,
    LeftCenter,
    RightCenter,
    BottomCenter,
    Wrap,
}
