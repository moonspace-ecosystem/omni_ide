use crate::dnd;
use crate::models::{FileStatus, UiTreeChange, UiWorktreeChanges};
use gitbutler_store::{GitButlerStore, GitButlerStoreEvent};
use gpui::{
    Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, Styled, WeakEntity,
    Window, div,
};
use ui::{Tooltip, prelude::*};
use workspace::Workspace;

pub struct UnassignedChanges {
    _workspace: WeakEntity<Workspace>,
    store: Entity<GitButlerStore>,
    changes: Option<UiWorktreeChanges>,
    unassigned_assignments: Vec<but_hunk_assignment::HunkAssignment>,
    selected_all: bool,
    selected_files: std::collections::HashSet<String>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl UnassignedChanges {
    pub fn new(
        workspace: WeakEntity<Workspace>,
        store: Entity<GitButlerStore>,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut subscriptions = Vec::new();

        subscriptions.push(cx.subscribe(&store, |this, store, event, cx| {
            if matches!(event, GitButlerStoreEvent::WorktreeChangesUpdated) {
                this.fetch_changes(&store, cx);
            }
        }));

        let mut this = Self {
            _workspace: workspace,
            store: store.clone(),
            changes: None,
            unassigned_assignments: Vec::new(),
            selected_all: false,
            selected_files: std::collections::HashSet::new(),
            _subscriptions: subscriptions,
        };

        this.fetch_changes(&store, cx);
        this
    }

    fn fetch_changes(&mut self, store: &Entity<GitButlerStore>, cx: &mut Context<Self>) {
        if let Some(changes) = store.read(cx).worktree_changes() {
            self.set_changes(changes.clone(), cx);
        }
    }

    pub fn set_changes(
        &mut self,
        changes: but_hunk_assignment::WorktreeChanges,
        cx: &mut Context<Self>,
    ) {
        let mut unique_paths = std::collections::HashSet::new();
        let mut ui_changes = Vec::new();
        let mut unassigned_assignments = Vec::new();

        for assignment in &changes.assignments {
            if assignment.stack_id.is_none() && assignment.branch_ref_bytes.is_none() {
                unassigned_assignments.push(assignment.clone());
                let path = assignment.path_bytes.to_string();
                if unique_paths.insert(path.clone()) {
                    ui_changes.push(UiTreeChange {
                        path: path.into(),
                        status: FileStatus::Modified,
                    });
                }
            }
        }

        self.changes = Some(UiWorktreeChanges {
            changes: ui_changes,
            assignments: Vec::new(),
        });
        self.unassigned_assignments = unassigned_assignments;
        cx.notify();
    }

    fn toggle_select_all(&mut self, cx: &mut Context<Self>) {
        self.selected_all = !self.selected_all;
        if self.selected_all {
            if let Some(changes) = &self.changes {
                for change in &changes.changes {
                    self.selected_files.insert(change.path.to_string());
                }
            }
        } else {
            self.selected_files.clear();
        }
        cx.notify();
    }

    fn toggle_file_selection(&mut self, path: String, cx: &mut Context<Self>) {
        if self.selected_files.contains(&path) {
            self.selected_files.remove(&path);
            self.selected_all = false;
        } else {
            self.selected_files.insert(path);
            if let Some(changes) = &self.changes {
                if self.selected_files.len() == changes.changes.len() && !changes.changes.is_empty()
                {
                    self.selected_all = true;
                }
            }
        }
        cx.notify();
    }

    fn unassigned_count(&self) -> usize {
        self.changes.as_ref().map(|c| c.changes.len()).unwrap_or(0)
    }
}

impl Render for UnassignedChanges {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut content = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().colors().panel_background);

        content = content.child(
            v_flex()
                .w_full()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(cx.theme().colors().border)
                .gap_1()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_1()
                                .child(
                                    Label::new("Changes")
                                        .weight(gpui::FontWeight::SEMIBOLD)
                                        .size(LabelSize::Small),
                                )
                                .child(
                                    Label::new(format!("{}", self.unassigned_count()))
                                        .size(LabelSize::XSmall)
                                        .color(Color::Muted),
                                ),
                        )
                        .child(
                            h_flex().gap_0p5().child(
                                IconButton::new("stash-button", IconName::Archive)
                                    .size(ButtonSize::Compact)
                                    .tooltip(Tooltip::text("Stash unassigned changes"))
                                    .on_click(cx.listener(|this, _, _window, cx| {
                                        let unassigned = this.unassigned_assignments.clone();
                                        if !unassigned.is_empty() {
                                            let now = std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap_or_default()
                                                .as_secs();
                                            let branch_name = format!("stash-{}", now);

                                            this.store.update(cx, |store, cx| {
                                                store.stash_unassigned_changes(
                                                    branch_name,
                                                    unassigned,
                                                    cx,
                                                );
                                            });
                                        }
                                    })),
                            ),
                        ),
                )
                .child(
                    h_flex()
                        .id("unassigned_changes_header")
                        .gap_2()
                        .items_center()
                        .cursor_pointer()
                        .on_click(cx.listener(|this, _, _window, cx| {
                            this.toggle_select_all(cx);
                        }))
                        // Selection is toggled by the parent row's `on_click` above; adding
                        // another handler here would double-toggle (and cancel out) clicks
                        // that land directly on the checkbox.
                        .child(ui::Checkbox::new(
                            "select-all",
                            if self.selected_all {
                                ui::ToggleState::Selected
                            } else if !self.selected_files.is_empty() {
                                ui::ToggleState::Indeterminate
                            } else {
                                ui::ToggleState::Unselected
                            },
                        ))
                        .child(
                            Label::new("Select All")
                                .color(Color::Muted)
                                .size(LabelSize::XSmall),
                        ),
                ),
        );

        let list_content = if let Some(changes) = &self.changes {
            if changes.changes.is_empty() {
                div()
                    .p_4()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .child(
                        Icon::new(IconName::Check)
                            .size(IconSize::Medium)
                            .color(Color::Success),
                    )
                    .child(
                        Label::new("No unapplied changes")
                            .color(Color::Muted)
                            .size(LabelSize::Small),
                    )
            } else {
                let mut list = v_flex().w_full().px_1().py_0p5().gap_0p5();
                for change in &changes.changes {
                    let drag_path = change.path.clone();
                    let file_path = change.path.to_string();
                    let is_selected = self.selected_files.contains(&file_path);

                    let icon_name = match change.status {
                        FileStatus::Modified => IconName::FileDoc,
                        FileStatus::Added => IconName::File,
                        FileStatus::Deleted => IconName::Trash,
                        _ => IconName::File,
                    };

                    let status_color = match change.status {
                        FileStatus::Modified => Color::Warning,
                        FileStatus::Added => Color::Success,
                        FileStatus::Deleted => Color::Error,
                        _ => Color::Default,
                    };

                    list = list.child(
                        h_flex()
                            .id(format!("unassigned-{}", drag_path))
                            .w_full()
                            .px_1()
                            .py_0p5()
                            .gap_2()
                            .items_center()
                            .rounded_sm()
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().colors().element_hover))
                            .when(is_selected, |this| {
                                this.bg(cx.theme().colors().element_selected)
                            })
                            .on_click(cx.listener({
                                let file_path = file_path.clone();
                                move |this, _, _window, cx| {
                                    this.toggle_file_selection(file_path.clone(), cx);
                                }
                            }))
                            .on_drag(
                                dnd::DragPayload::File(drag_path.to_string()),
                                dnd::create_file_drag_preview,
                            )
                            // Selection is toggled by the row's `on_click` above; adding another
                            // handler here would double-toggle (and cancel out) clicks that land
                            // directly on the checkbox.
                            .child(ui::Checkbox::new(
                                format!("chk-{}", drag_path),
                                if is_selected {
                                    ui::ToggleState::Selected
                                } else {
                                    ui::ToggleState::Unselected
                                },
                            ))
                            .child(
                                Icon::new(icon_name)
                                    .color(status_color)
                                    .size(IconSize::XSmall),
                            )
                            .child(
                                Label::new(change.path.clone())
                                    .size(LabelSize::XSmall)
                                    .truncate(),
                            ),
                    );
                }
                list
            }
        } else {
            div().p_4().flex().items_center().justify_center().child(
                h_flex()
                    .gap_1()
                    .child(
                        Icon::new(IconName::ArrowCircle)
                            .size(IconSize::Small)
                            .color(Color::Muted),
                    )
                    .child(
                        Label::new("Loading...")
                            .color(Color::Muted)
                            .size(LabelSize::Small),
                    ),
            )
        };

        content.child(
            div()
                .id("unassigned-changes-list")
                .flex_1()
                .overflow_y_scroll()
                .on_drop({
                    let store = self.store.clone();
                    move |payload: &dnd::DragPayload, _window, cx| {
                        if let dnd::DragPayload::File(path) = payload {
                            store.update(cx, |store, cx| {
                                store.unassign_file(path.clone(), cx);
                            });
                        }
                    }
                })
                .child(list_content),
        )
    }
}
