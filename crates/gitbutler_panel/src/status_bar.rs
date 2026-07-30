use gpui::*;
use ui::prelude::*;

#[derive(IntoElement)]
pub struct StatusBar {
    branch_name: Option<SharedString>,
    changes_count: usize,
    commits_ahead: usize,
    loading_message: Option<SharedString>,
    error_message: Option<SharedString>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            branch_name: None,
            changes_count: 0,
            commits_ahead: 0,
            loading_message: None,
            error_message: None,
        }
    }

    pub fn branch_name(mut self, name: impl Into<SharedString>) -> Self {
        self.branch_name = Some(name.into());
        self
    }

    pub fn changes_count(mut self, count: usize) -> Self {
        self.changes_count = count;
        self
    }

    pub fn commits_ahead(mut self, count: usize) -> Self {
        self.commits_ahead = count;
        self
    }

    pub fn loading(mut self, message: impl Into<SharedString>) -> Self {
        self.loading_message = Some(message.into());
        self
    }

    pub fn error(mut self, message: impl Into<SharedString>) -> Self {
        self.error_message = Some(message.into());
        self
    }
}

impl RenderOnce for StatusBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .w_full()
            .h(px(24.0))
            .px_3()
            .gap_3()
            .items_center()
            .bg(cx.theme().colors().panel_background)
            .border_t_1()
            .border_color(cx.theme().colors().border)
            .when_some(self.error_message, |this, error| {
                this.child(
                    h_flex()
                        .gap_1()
                        .child(Icon::new(IconName::XCircle).size(IconSize::XSmall).color(Color::Error))
                        .child(Label::new(error).size(LabelSize::XSmall).color(Color::Error)),
                )
            })
            .when_some(self.loading_message, |this, message| {
                this.child(
                    h_flex()
                        .gap_1()
                        .child(Icon::new(IconName::ArrowCircle).size(IconSize::XSmall).color(Color::Muted))
                        .child(Label::new(message).size(LabelSize::XSmall).color(Color::Muted)),
                )
            })
            .when_some(self.branch_name, |this, name| {
                this.child(
                    h_flex()
                        .gap_1()
                        .child(Icon::new(IconName::GitBranch).size(IconSize::XSmall).color(Color::Muted))
                        .child(Label::new(name).size(LabelSize::XSmall).color(Color::Muted)),
                )
            })
            .when(self.changes_count > 0, |this| {
                this.child(
                    h_flex()
                        .gap_1()
                        .child(Icon::new(IconName::FileDoc).size(IconSize::XSmall).color(Color::Warning))
                        .child(
                            Label::new(format!("{} changes", self.changes_count))
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                )
            })
            .when(self.commits_ahead > 0, |this| {
                this.child(
                    h_flex()
                        .gap_1()
                        .child(Icon::new(IconName::ArrowUp).size(IconSize::XSmall).color(Color::Success))
                        .child(
                            Label::new(format!("{} ahead", self.commits_ahead))
                                .size(LabelSize::XSmall)
                                .color(Color::Muted),
                        ),
                )
            })
    }
}
