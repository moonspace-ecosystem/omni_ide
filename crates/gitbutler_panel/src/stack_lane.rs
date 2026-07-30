use crate::branch_card::BranchCard;
use crate::dnd;
use gpui::*;
use ui::Tooltip;
use ui::prelude::*;

#[derive(IntoElement)]
pub struct StackHeader {
    stack_name: SharedString,
    is_collapsed: bool,
    on_toggle_collapse: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl StackHeader {
    pub fn new(stack_name: impl Into<SharedString>) -> Self {
        Self {
            stack_name: stack_name.into(),
            is_collapsed: false,
            on_toggle_collapse: None,
        }
    }

    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.is_collapsed = collapsed;
        self
    }

    pub fn on_toggle_collapse(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle_collapse = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for StackHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        h_flex()
            .w_full()
            .justify_between()
            .items_center()
            .px_2()
            .py_1()
            .border_b_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().panel_background)
            .child(
                h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        Icon::new(IconName::GitBranch)
                            .size(IconSize::Small)
                            .color(Color::Accent),
                    )
                    .child(
                        Label::new(self.stack_name)
                            .weight(FontWeight::BOLD)
                            .size(LabelSize::Small),
                    ),
            )
            .child(
                h_flex()
                    .gap_0p5()
                    .child({
                        let mut button = IconButton::new(
                            "fold-unfold",
                            if self.is_collapsed {
                                IconName::ChevronRight
                            } else {
                                IconName::ChevronDown
                            },
                        )
                        .size(ButtonSize::Compact);
                        if let Some(handler) = self.on_toggle_collapse {
                            button = button.on_click(handler);
                        }
                        button
                    })
                    .child(
                        IconButton::new("stack-menu", IconName::Ellipsis)
                            .size(ButtonSize::Compact)
                            .tooltip(Tooltip::text("Stack Options")),
                    ),
            )
    }
}

#[derive(IntoElement)]
pub struct WorktreeChangesSection {
    assigned_files: Vec<crate::models::UiHunkAssignment>,
}

impl WorktreeChangesSection {
    pub fn new(assigned_files: Vec<crate::models::UiHunkAssignment>) -> Self {
        Self { assigned_files }
    }
}

impl RenderOnce for WorktreeChangesSection {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .px_2()
            .py_1()
            .gap_0p5()
            .when(!self.assigned_files.is_empty(), |this| {
                this.child(
                    h_flex()
                        .gap_1()
                        .child(
                            Label::new("Assigned Files")
                                .color(Color::Muted)
                                .size(LabelSize::XSmall),
                        )
                        .child(
                            Label::new(format!("{}", self.assigned_files.len()))
                                .color(Color::Muted)
                                .size(LabelSize::XSmall),
                        ),
                )
            })
            .children(self.assigned_files.into_iter().map(|file| {
                let path = file.path.clone();
                let drag_path = path.to_string();
                div()
                    .id(drag_path.clone())
                    .w_full()
                    .px_1()
                    .py_0p5()
                    .rounded_sm()
                    .hover(|s| s.bg(cx.theme().colors().element_hover))
                    .on_drag(
                        dnd::DragPayload::File(drag_path.clone()),
                        dnd::create_file_drag_preview,
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Icon::new(IconName::File)
                                    .color(Color::Muted)
                                    .size(IconSize::XSmall),
                            )
                            .child(Label::new(path).size(LabelSize::XSmall).truncate()),
                    )
            }))
    }
}

#[derive(IntoElement)]
pub struct CommitButton {
    branch_name: SharedString,
}

impl CommitButton {
    pub fn new(branch_name: SharedString) -> Self {
        Self { branch_name }
    }
}

impl RenderOnce for CommitButton {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().w_full().px_2().py_2().child(
            Button::new("commit-button", "Commit…")
                .full_width()
                .size(ButtonSize::Default)
                .style(ButtonStyle::Tinted(ui::TintColor::Accent))
                .start_icon(Icon::new(IconName::Check))
                .on_click({
                    let branch_name = self.branch_name.clone();
                    move |_, window, cx| {
                        window.dispatch_action(
                            crate::actions::Commit {
                                branch_name: branch_name.to_string(),
                            }
                            .boxed_clone(),
                            cx,
                        );
                    }
                }),
        )
    }
}

#[derive(IntoElement)]
pub struct BranchList {
    branch_name: SharedString,
    branch_card: Entity<BranchCard>,
}

impl BranchList {
    pub fn new(branch_name: impl Into<SharedString>, branch_card: Entity<BranchCard>) -> Self {
        Self {
            branch_name: branch_name.into(),
            branch_card,
        }
    }
}

impl RenderOnce for BranchList {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        v_flex()
            .id(SharedString::from(format!(
                "branch-list-{}",
                self.branch_name
            )))
            .w_full()
            .flex_1()
            .overflow_y_scroll()
            .child(self.branch_card.clone())
    }
}

#[derive(IntoElement)]
pub struct StackLane {
    stack_name: SharedString,
    branch_card: Entity<BranchCard>,
    store: Entity<::gitbutler_store::GitButlerStore>,
}

impl StackLane {
    pub fn new(
        stack_name: impl Into<SharedString>,
        branch_card: Entity<BranchCard>,
        store: Entity<::gitbutler_store::GitButlerStore>,
    ) -> Self {
        Self {
            stack_name: stack_name.into(),
            branch_card,
            store,
        }
    }
}

impl RenderOnce for StackLane {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w(px(280.0))
            .min_w(px(240.0))
            .h_full()
            .border_r_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().background)
            .on_drop({
                let store = self.store.clone();
                let branch_name = self.stack_name.to_string();
                move |payload: &dnd::DragPayload, _window, cx| {
                    if let dnd::DragPayload::File(path) = payload {
                        store.update(cx, |store, cx| {
                            store.assign_hunk(&branch_name, path.clone(), cx);
                        });
                    }
                }
            })
            .child(
                StackHeader::new(self.stack_name.clone())
                    .collapsed(self.branch_card.read(cx).is_collapsed)
                    .on_toggle_collapse({
                        let branch_card = self.branch_card.clone();
                        move |_, _window, cx| {
                            branch_card.update(cx, |card, cx| {
                                card.is_collapsed = !card.is_collapsed;
                                cx.notify();
                            });
                        }
                    }),
            )
            .child(WorktreeChangesSection::new({
                let stack_name = self.stack_name.to_string();
                self.store
                    .read(cx)
                    .worktree_changes()
                    .map(|changes| {
                        changes
                            .assignments
                            .iter()
                            .filter(|a| {
                                a.branch_ref_bytes.as_ref().map(|b| b.as_bstr().to_string())
                                    == Some(format!("refs/heads/{}", stack_name))
                            })
                            .map(crate::models::UiHunkAssignment::from)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            }))
            .child(CommitButton::new(self.stack_name.clone()))
            .child(BranchList::new(
                self.stack_name.clone(),
                self.branch_card.clone(),
            ))
    }
}
