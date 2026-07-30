use gpui::*;
use ui::prelude::*;
use crate::diff_viewer::MultiDiffView;
use crate::models::UiCommit;

pub struct CommitDetailPanel {
    selected_commit: Option<UiCommit>,
    diff_view: Entity<MultiDiffView>,
}

impl CommitDetailPanel {
    pub fn new(store: Entity<gitbutler_store::GitButlerStore>, cx: &mut Context<Self>) -> Self {
        let diff_view = cx.new(|_| MultiDiffView::new(store));
        Self {
            selected_commit: None,
            diff_view,
        }
    }

    pub fn select_commit(&mut self, commit: UiCommit, cx: &mut Context<Self>) {
        let commit_id = commit.id.to_string();
        self.selected_commit = Some(commit);
        self.diff_view.update(cx, |view, cx| {
            view.load_commit(commit_id, cx);
        });
        cx.notify();
    }

    pub fn clear_selection(&mut self, cx: &mut Context<Self>) {
        self.selected_commit = None;
        cx.notify();
    }

    pub fn has_selection(&self) -> bool {
        self.selected_commit.is_some()
    }

    fn render_header(&self, cx: &Context<Self>) -> impl IntoElement {
        if let Some(commit) = &self.selected_commit {
            v_flex()
                .w_full()
                .p_3()
                .gap_2()
                .border_b_1()
                .border_color(cx.theme().colors().border)
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(Label::new("Commit Details").weight(FontWeight::BOLD).size(LabelSize::Small))
                        .child(
                            IconButton::new("close-detail", IconName::Close)
                                .on_click(cx.listener(|this, _, _window, cx| {
                                    this.clear_selection(cx);
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1()
                        .child(Label::new(commit.message.clone()).weight(FontWeight::SEMIBOLD))
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(Icon::new(IconName::Person).size(IconSize::XSmall).color(Color::Muted))
                                        .child(Label::new(commit.author.clone()).size(LabelSize::Small).color(Color::Muted)),
                                )
                                .child(
                                    Label::new(commit.id.clone())
                                        .size(LabelSize::Small)
                                        .color(Color::Muted),
                                )
                                .when_some(commit.timestamp, |this, ts| {
                                    this.child(
                                        Label::new(format_relative_time(ts))
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                }),
                        ),
                )
                .into_any_element()
        } else {
            div()
                .w_full()
                .p_4()
                .flex()
                .items_center()
                .justify_center()
                .child(Label::new("Select a commit to view details").color(Color::Muted))
                .into_any_element()
        }
    }
}

impl Render for CommitDetailPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().colors().panel_background)
            .child(self.render_header(cx))
            .when(self.selected_commit.is_some(), |this| {
                this.child(
                    div()
                        .id("commit-diff-scroll")
                        .flex_1()
                        .overflow_y_scroll()
                        .child(self.diff_view.clone()),
                )
            })
    }
}

fn format_relative_time(timestamp: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else {
        format!("{}w ago", diff / 604800)
    }
}
