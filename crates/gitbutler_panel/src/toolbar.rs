use gpui::*;
use ui::prelude::*;
use ui::Tooltip;

#[derive(IntoElement)]
pub struct Toolbar {
    on_fetch: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_new_branch: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle_left: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_toggle_right: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    left_panel_visible: bool,
    right_panel_visible: bool,
    is_loading: bool,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            on_fetch: None,
            on_new_branch: None,
            on_toggle_left: None,
            on_toggle_right: None,
            left_panel_visible: true,
            right_panel_visible: false,
            is_loading: false,
        }
    }

    pub fn on_fetch(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_fetch = Some(Box::new(handler));
        self
    }

    pub fn on_new_branch(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_new_branch = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_left(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_left = Some(Box::new(handler));
        self
    }

    pub fn on_toggle_right(mut self, handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle_right = Some(Box::new(handler));
        self
    }

    pub fn left_panel_visible(mut self, visible: bool) -> Self {
        self.left_panel_visible = visible;
        self
    }

    pub fn right_panel_visible(mut self, visible: bool) -> Self {
        self.right_panel_visible = visible;
        self
    }

    pub fn loading(mut self, is_loading: bool) -> Self {
        self.is_loading = is_loading;
        self
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .w_full()
            .h(px(36.0))
            .px_2()
            .gap_1()
            .items_center()
            .justify_between()
            .bg(cx.theme().colors().panel_background)
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                    IconButton::new("toggle-left-panel", IconName::ChevronLeft)
                            .tooltip(Tooltip::text("Toggle Changes Panel"))
                            .toggle_state(self.left_panel_visible)
                            .when_some(self.on_toggle_left, |button, handler| {
                                button.on_click(handler)
                            }),
                    )
                    .child(Label::new("GitButler").weight(FontWeight::SEMIBOLD).size(LabelSize::Small))
                    .when(self.is_loading, |this| {
                        this.child(
                            Icon::new(IconName::ArrowCircle)
                                .size(IconSize::Small)
                                .color(Color::Muted),
                        )
                    }),
            )
            .child(
                h_flex()
                    .gap_1()
                    .items_center()
                    .child({
                        let mut button = IconButton::new("fetch-btn", IconName::ArrowDown)
                            .tooltip(Tooltip::text("Fetch from Remote"));
                        if let Some(handler) = self.on_fetch {
                            button = button.on_click(handler);
                        }
                        button
                    })
                    .child({
                        let mut button = IconButton::new("new-branch-btn", IconName::Plus)
                            .tooltip(Tooltip::text("Create New Branch"));
                        if let Some(handler) = self.on_new_branch {
                            button = button.on_click(handler);
                        }
                        button
                    })
                    .child(
                        IconButton::new("toggle-right-panel", IconName::ChevronRight)
                            .tooltip(Tooltip::text("Toggle Commit Details"))
                            .toggle_state(self.right_panel_visible)
                            .when_some(self.on_toggle_right, |button, handler| {
                                button.on_click(handler)
                            }),
                    ),
            )
    }
}
