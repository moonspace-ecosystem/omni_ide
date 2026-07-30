use gpui::{div, px, Div, Entity, IntoElement, ParentElement, RenderOnce, Styled, Window};
use ui::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DragPayload {
    Hunk(String),
    Commit(String),
    File(String),
}

pub struct DragState {
    pub payload: Option<DragPayload>,
}

impl Default for DragState {
    fn default() -> Self {
        Self::new()
    }
}

impl DragState {
    pub fn new() -> Self {
        Self { payload: None }
    }
}

pub trait Draggable {
    fn draggable(self, payload: DragPayload) -> Self;
}

impl Draggable for Div {
    fn draggable(self, _payload: DragPayload) -> Self {
        self
    }
}

#[derive(IntoElement)]
pub struct InsertionIndicator;

impl RenderOnce for InsertionIndicator {
    fn render(self, _window: &mut Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .w_full()
            .h(px(2.0))
            .bg(gpui::blue())
            .rounded_full()
    }
}

pub struct FileDragPreview {
    path: String,
}

impl FileDragPreview {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Render for FileDragPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        h_flex()
            .p_2()
            .gap_2()
            .rounded_md()
            .bg(cx.theme().colors().element_active)
            .border_1()
            .border_color(cx.theme().colors().border_focused)
            .shadow_md()
            .child(Icon::new(IconName::File).color(Color::Muted))
            .child(Label::new(self.path.clone()).size(LabelSize::Small))
    }
}

pub fn create_file_drag_preview(
    payload: &DragPayload,
    _offset: gpui::Point<gpui::Pixels>,
    _window: &mut Window,
    cx: &mut gpui::App,
) -> Entity<FileDragPreview> {
    let path = match payload {
        DragPayload::File(path) => path.clone(),
        _ => String::new(),
    };
    cx.new(|_| FileDragPreview::new(path))
}

#[derive(IntoElement)]
pub struct DropZoneIndicator {
    is_active: bool,
}

impl DropZoneIndicator {
    pub fn new(is_active: bool) -> Self {
        Self { is_active }
    }
}

impl RenderOnce for DropZoneIndicator {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .rounded_md()
            .when(self.is_active, |this| {
                this.border_2()
                    .border_color(cx.theme().colors().border_focused)
                    .bg(cx.theme().colors().drop_target_background)
            })
    }
}
