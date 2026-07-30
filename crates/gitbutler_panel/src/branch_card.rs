use crate::gitbutler_colors::GitButlerColors;
use crate::models::{UiCommit, UiSegment};
use but_graph::workspace::StackCommitFlags;
use gpui::*;
use ui::prelude::*;
use ui::{Button, Icon, IconName, Tooltip};

pub struct BranchCard {
    pub segment: UiSegment,
    pub branch_name: SharedString,
    pub selected_commit_id: Option<SharedString>,
    pub is_collapsed: bool,
}

impl BranchCard {
    pub fn new(branch_name: impl Into<SharedString>, segment: UiSegment) -> Self {
        Self {
            segment,
            branch_name: branch_name.into(),
            selected_commit_id: None,
            is_collapsed: false,
        }
    }

    pub fn select_commit(&mut self, commit_id: SharedString, cx: &mut Context<Self>) {
        self.selected_commit_id = Some(commit_id);
        cx.notify();
    }

    pub fn update_segment(&mut self, segment: UiSegment, cx: &mut Context<Self>) {
        self.segment = segment;
        cx.notify();
    }
}

pub enum BranchCardEvent {
    CommitSelected(UiCommit),
}

impl EventEmitter<BranchCardEvent> for BranchCard {}

impl Render for BranchCard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .w_full()
            .gap_1()
            .child(BranchHeader::new(
                self.branch_name.clone(),
                self.segment.pr_number,
                self.segment.pr_url.clone(),
                self.segment.commits.len(),
                self.is_collapsed,
            ))
            .when(!self.is_collapsed, |this| {
                this.child(v_flex().w_full().children(
                    self.segment.commits.clone().into_iter().map(|commit| {
                        let is_selected = self.selected_commit_id.as_ref() == Some(&commit.id);
                        let commit_id = commit.id.clone();
                        let commit_for_event = commit.clone();

                        div()
                            .id(commit_id.clone())
                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                this.selected_commit_id = Some(commit_id.clone());
                                cx.emit(BranchCardEvent::CommitSelected(commit_for_event.clone()));
                                cx.notify();
                            }))
                            .child(CommitItem::new(commit, is_selected))
                    }),
                ))
            })
    }
}

#[derive(IntoElement)]
pub struct BranchHeader {
    branch_name: SharedString,
    pr_number: Option<usize>,
    pr_url: Option<SharedString>,
    commit_count: usize,
    is_collapsed: bool,
}

impl BranchHeader {
    pub fn new(
        branch_name: impl Into<SharedString>,
        pr_number: Option<usize>,
        pr_url: Option<SharedString>,
        commit_count: usize,
        is_collapsed: bool,
    ) -> Self {
        Self {
            branch_name: branch_name.into(),
            pr_number,
            pr_url,
            commit_count,
            is_collapsed,
        }
    }
}

impl RenderOnce for BranchHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .px_2()
            .py_1()
            .bg(cx.theme().colors().elevated_surface_background)
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Icon::new(if self.is_collapsed {
                            IconName::ChevronRight
                        } else {
                            IconName::ChevronDown
                        })
                        .size(IconSize::XSmall)
                        .color(Color::Muted),
                    )
                    .child(
                        Label::new(self.branch_name.clone())
                            .weight(FontWeight::SEMIBOLD)
                            .size(LabelSize::Small),
                    )
                    .child(
                        Label::new(format!("{}", self.commit_count))
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    ),
            )
            .child(
                h_flex()
                    .gap_1()
                    .items_center()
                    .children(self.pr_number.map(|num| {
                        let url = self.pr_url.clone();
                        Button::new(format!("pr_{}", num), format!("#{}", num))
                            .style(ButtonStyle::Subtle)
                            .size(ButtonSize::Compact)
                            .start_icon(Icon::new(IconName::PullRequest))
                            .on_click(move |_event, _window, cx| {
                                if let Some(url) = url.as_ref() {
                                    cx.open_url(url.as_ref());
                                }
                            })
                    }))
                    .child(
                        IconButton::new("push_btn", IconName::ArrowUp)
                            .size(ButtonSize::Compact)
                            .tooltip(Tooltip::text("Push"))
                            .on_click({
                                let branch_name = self.branch_name.clone();
                                move |_, window, cx| {
                                    window.dispatch_action(
                                        crate::actions::Push {
                                            branch_name: branch_name.to_string(),
                                        }
                                        .boxed_clone(),
                                        cx,
                                    );
                                }
                            }),
                    ),
            )
    }
}

#[derive(IntoElement)]
pub struct CommitItem {
    commit: UiCommit,
    is_selected: bool,
}

impl CommitItem {
    pub fn new(commit: UiCommit, is_selected: bool) -> Self {
        Self {
            commit,
            is_selected,
        }
    }
}

impl RenderOnce for CommitItem {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let bg_color = if self.is_selected {
            cx.theme().colors().element_selected
        } else {
            gpui::transparent_black()
        };

        let commit_status_color = if self.commit.flags.contains(StackCommitFlags::HasConflicts) {
            cx.theme().gitbutler_conflict()
        } else if self
            .commit
            .flags
            .contains(StackCommitFlags::ReachableByMatchingRemote)
        {
            cx.theme().gitbutler_pushed_commit()
        } else if self.commit.flags.contains(StackCommitFlags::Integrated) {
            cx.theme().gitbutler_integrated_commit()
        } else {
            cx.theme().gitbutler_local_commit()
        };

        h_flex()
            .id(SharedString::from(format!(
                "commit-item-{}",
                self.commit.id
            )))
            .w_full()
            .gap_2()
            .px_2()
            .py_1()
            .bg(bg_color)
            .hover(|style| style.bg(cx.theme().colors().element_hover))
            .active(|style| style.bg(cx.theme().colors().element_active))
            .cursor_pointer()
            .child(
                div()
                    .w(px(3.0))
                    .h(px(28.0))
                    .rounded_full()
                    .bg(commit_status_color),
            )
            .child(
                v_flex()
                    .flex_1()
                    .overflow_hidden()
                    .gap_0p5()
                    .child(
                        Label::new(self.commit.message.clone())
                            .size(LabelSize::Small)
                            .line_height_style(LineHeightStyle::UiLabel)
                            .truncate(),
                    )
                    .child(
                        h_flex()
                            .w_full()
                            .gap_2()
                            .child(
                                Label::new(self.commit.id.clone())
                                    .color(Color::Muted)
                                    .size(LabelSize::XSmall),
                            )
                            .child(
                                Label::new(self.commit.author.clone())
                                    .color(Color::Muted)
                                    .size(LabelSize::XSmall)
                                    .truncate(),
                            )
                            .when_some(self.commit.timestamp, |this, ts| {
                                this.child(
                                    div().ml_auto().child(
                                        Label::new(format_relative_time(ts))
                                            .color(Color::Muted)
                                            .size(LabelSize::XSmall),
                                    ),
                                )
                            }),
                    ),
            )
            .when(self.commit.is_conflicted, |this| {
                this.child(
                    Icon::new(IconName::Warning)
                        .size(IconSize::XSmall)
                        .color(Color::Warning),
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
