use crate::branch_card::{BranchCard, BranchCardEvent};
use crate::commit_detail_panel::CommitDetailPanel;
use crate::stack_lane::StackLane;
use crate::status_bar::StatusBar;
use crate::toolbar::Toolbar;
use crate::unassigned_changes;
use gitbutler_store::LoadingState;
use gpui::{
    Action, App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, ParentElement,
    Render, Styled, WeakEntity, Window, actions, div, px,
};
use ui::prelude::*;
use workspace::{
    Workspace,
    dock::{DockPosition, Panel, PanelEvent},
};

actions!(gitbutler_panel, [ToggleGitButlerPanel]);

pub const GITBUTLER_PANEL_KEY: &str = "GitButlerPanel";

pub struct GitButlerPanel {
    _workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    unassigned_changes: Entity<unassigned_changes::UnassignedChanges>,
    commit_detail: Entity<CommitDetailPanel>,
    pub store: Entity<gitbutler_store::GitButlerStore>,
    left_panel_visible: bool,
    right_panel_visible: bool,
    // Branch cards are cached across renders (keyed by branch name) instead of being
    // rebuilt from scratch every render, otherwise per-card state such as
    // `is_collapsed`/`selected_commit_id` would be lost on every `cx.notify()`.
    branch_cards: std::collections::HashMap<String, Entity<BranchCard>>,
    branch_card_subscriptions: std::collections::HashMap<String, gpui::Subscription>,
    _subscriptions: Vec<gpui::Subscription>,
}

pub fn init(cx: &mut App) {
    cx.observe_new(|workspace: &mut Workspace, _, _| {
        workspace.register_action(|workspace, _: &ToggleGitButlerPanel, window, cx| {
            workspace.toggle_panel_focus::<GitButlerPanel>(window, cx);
        });
    })
    .detach();
}

impl GitButlerPanel {
    pub async fn load(
        workspace_handle: WeakEntity<Workspace>,
        mut cx: gpui::AsyncWindowContext,
    ) -> anyhow::Result<Entity<Self>> {
        workspace_handle
            .update(&mut cx, |workspace, window, cx| Self::new(workspace, window, cx))
    }

    pub fn new(
        workspace: &mut Workspace,
        _window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> Entity<Self> {
        let store = cx.new(|_| gitbutler_store::GitButlerStore::init());
        let worktree_path = workspace
            .worktrees(cx)
            .next()
            .map(|w| w.read(cx).abs_path());
        if let Some(path) = worktree_path {
            store.update(cx, |store, model_cx| {
                if let Err(error) = store.discover(path, model_cx) {
                    model_cx.emit(gitbutler_store::GitButlerStoreEvent::Error(
                        error.to_string(),
                    ));
                    model_cx.notify();
                }
            });
        }

        let workspace_handle = cx.entity().downgrade();
        let unassigned_changes = cx.new(|cx| {
            unassigned_changes::UnassignedChanges::new(workspace_handle.clone(), store.clone(), cx)
        });

        let commit_detail = cx.new(|cx| CommitDetailPanel::new(store.clone(), cx));

        cx.new(|cx| {
            let mut subscriptions = Vec::new();

            subscriptions.push(cx.subscribe(&store, |_this: &mut Self, _store, event, cx| {
                match event {
                    gitbutler_store::GitButlerStoreEvent::WorkspaceChanged => cx.notify(),
                    gitbutler_store::GitButlerStoreEvent::WorktreeChangesUpdated => cx.notify(),
                    gitbutler_store::GitButlerStoreEvent::LoadingStateChanged(_) => cx.notify(),
                    gitbutler_store::GitButlerStoreEvent::Error(_) => cx.notify(),
                    _ => {}
                }
            }));

            Self {
                _workspace: workspace_handle.clone(),
                focus_handle: cx.focus_handle(),
                unassigned_changes,
                commit_detail,
                store,
                left_panel_visible: true,
                right_panel_visible: false,
                branch_cards: std::collections::HashMap::new(),
                branch_card_subscriptions: std::collections::HashMap::new(),
                _subscriptions: subscriptions,
            }
        })
    }

    pub async fn load(
        workspace: WeakEntity<Workspace>,
        mut cx: gpui::AsyncWindowContext,
    ) -> anyhow::Result<Entity<Self>> {
        workspace.update_in(&mut cx, |workspace, window, cx| {
            Self::new(workspace, window, cx)
        })
    }

    fn toggle_left_panel(&mut self, cx: &mut Context<Self>) {
        self.left_panel_visible = !self.left_panel_visible;
        cx.notify();
    }

    fn toggle_right_panel(&mut self, cx: &mut Context<Self>) {
        self.right_panel_visible = !self.right_panel_visible;
        cx.notify();
    }

    fn build_stack_lanes(&mut self, cx: &mut Context<Self>) -> Vec<gpui::AnyElement> {
        let mut children = Vec::new();
        let mut active_branch_names = std::collections::HashSet::new();

        if let Some(ref_info) = self.store.read(cx).workspace_ref_info() {
            let prs = self.store.read(cx).fetch_prs();
            let ui_workspace = crate::models::UiWorkspace::from_ref_info(&ref_info, &prs);

            for (stack_index, stack) in ui_workspace.stacks.into_iter().enumerate() {
                for (segment_index, segment) in stack.segments.into_iter().enumerate() {
                    let branch_name = segment.branch_name.clone().unwrap_or_else(|| {
                        format!("branch-{}-{}", stack_index, segment_index).into()
                    });
                    let branch_key = branch_name.to_string();
                    active_branch_names.insert(branch_key.clone());

                    let branch_card = if let Some(existing) = self.branch_cards.get(&branch_key) {
                        existing.update(cx, |card, cx| card.update_segment(segment, cx));
                        existing.clone()
                    } else {
                        let new_card = cx.new(|_cx| BranchCard::new(branch_name.clone(), segment));
                        let subscription = cx.subscribe(&new_card, {
                            move |this: &mut Self, _card, event: &BranchCardEvent, cx| match event {
                                BranchCardEvent::CommitSelected(commit) => {
                                    this.right_panel_visible = true;
                                    this.commit_detail.update(cx, |detail, cx| {
                                        detail.select_commit(commit.clone(), cx);
                                    });
                                    cx.notify();
                                }
                            }
                        });
                        self.branch_card_subscriptions
                            .insert(branch_key.clone(), subscription);
                        self.branch_cards
                            .insert(branch_key.clone(), new_card.clone());
                        new_card
                    };

                    children.push(
                        StackLane::new(branch_name, branch_card, self.store.clone())
                            .into_any_element(),
                    );
                }
            }
        }

        self.branch_cards
            .retain(|key, _| active_branch_names.contains(key));
        self.branch_card_subscriptions
            .retain(|key, _| active_branch_names.contains(key));

        if children.is_empty() {
            vec![
                div()
                    .p_4()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .child(
                        Icon::new(IconName::GitBranch)
                            .size(IconSize::Medium)
                            .color(Color::Muted),
                    )
                    .child(Label::new("No virtual branches found.").color(Color::Muted))
                    .child(
                        Label::new("Open a GitButler-initialized project to get started.")
                            .color(Color::Muted)
                            .size(LabelSize::Small),
                    )
                    .child(Button::new("create-branch", "New Virtual Branch").on_click(
                        |_, window, cx| {
                            window.dispatch_action(crate::actions::CreateBranch.boxed_clone(), cx);
                        },
                    ))
                    .into_any_element(),
            ]
        } else {
            children
        }
    }
}

impl Focusable for GitButlerPanel {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<PanelEvent> for GitButlerPanel {}

impl Panel for GitButlerPanel {
    fn persistent_name() -> &'static str {
        "GitButler Panel"
    }

    fn panel_key() -> &'static str {
        GITBUTLER_PANEL_KEY
    }

    fn position(&self, _window: &Window, _cx: &App) -> DockPosition {
        DockPosition::Bottom
    }

    fn position_is_valid(&self, position: DockPosition) -> bool {
        matches!(
            position,
            DockPosition::Bottom | DockPosition::Left | DockPosition::Right
        )
    }

    fn set_position(
        &mut self,
        _position: DockPosition,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn default_size(&self, _window: &Window, _cx: &App) -> Pixels {
        px(400.0)
    }

    fn icon(&self, _window: &Window, _cx: &App) -> Option<ui::IconName> {
        Some(ui::IconName::GitBranch)
    }

    fn icon_tooltip(&self, _window: &Window, _cx: &App) -> Option<&'static str> {
        Some("GitButler Panel")
    }

    fn toggle_action(&self) -> Box<dyn gpui::Action> {
        Box::new(ToggleGitButlerPanel)
    }

    fn activation_priority(&self) -> u32 {
        1
    }
}

impl Render for GitButlerPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_loading = self.store.read(cx).is_loading();
        let changes_count = self.store.read(cx).changes_count();
        let loading_state = self.store.read(cx).loading_state().clone();
        let left_visible = self.left_panel_visible;
        let right_visible = self.right_panel_visible;
        div()
            .id("gitbutler-panel")
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .on_action(
                cx.listener(|this, action: &crate::actions::Push, _window, cx| {
                    let branch_name = action.branch_name.clone();
                    this.store.update(cx, |store, cx| {
                        store.push(&branch_name, cx);
                    });
                }),
            )
            .on_action(
                cx.listener(|this, _: &crate::actions::ToggleLeftPanel, _window, cx| {
                    this.toggle_left_panel(cx);
                }),
            )
            .on_action(
                cx.listener(|this, _: &crate::actions::ToggleRightPanel, _window, cx| {
                    this.toggle_right_panel(cx);
                }),
            )
            .on_action(
                cx.listener(|_this, _: &crate::actions::CreateBranch, _window, _cx| {
                    // Placeholder for actual GitButler branch creation
                }),
            )
            // Toolbar
            .child(
                Toolbar::new()
                    .left_panel_visible(left_visible)
                    .right_panel_visible(right_visible)
                    .loading(is_loading)
                    .on_fetch({
                        let store = self.store.clone();
                        move |_, _, cx| {
                            store.update(cx, |store, cx| {
                                store.fetch(cx);
                            });
                        }
                    })
                    .on_toggle_left(cx.listener(|this, _, _window, cx| {
                        this.toggle_left_panel(cx);
                    }))
                    .on_toggle_right(cx.listener(|this, _, _window, cx| {
                        this.toggle_right_panel(cx);
                    })),
            )
            // Main Viewport
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .bg(cx.theme().colors().editor_background)
                    // Left: Unassigned Changes (collapsible)
                    .when(left_visible, |this| {
                        this.child(
                            div()
                                .flex()
                                .w(px(240.0))
                                .min_w(px(200.0))
                                .h_full()
                                .border_r_1()
                                .border_color(cx.theme().colors().border)
                                .child(self.unassigned_changes.clone()),
                        )
                    })
                    // Middle: Multi Stack View
                    .child(
                        div()
                            .id("multi-stack-view")
                            .flex()
                            .flex_row()
                            .flex_1()
                            .h_full()
                            .overflow_x_scroll()
                            .children(self.build_stack_lanes(cx)),
                    )
                    // Right: Commit Detail (collapsible)
                    .when(right_visible, |this| {
                        this.child(
                            div()
                                .flex()
                                .w(px(320.0))
                                .min_w(px(260.0))
                                .h_full()
                                .border_l_1()
                                .border_color(cx.theme().colors().border)
                                .child(self.commit_detail.clone()),
                        )
                    }),
            )
            // Status Bar
            .child({
                let mut status_bar = StatusBar::new().changes_count(changes_count);

                if let LoadingState::Error(ref error) = loading_state {
                    status_bar = status_bar.error(error.clone());
                } else if is_loading {
                    status_bar = status_bar.loading("Syncing...");
                }

                let branch_names = self.store.read(cx).branch_names();
                if let Some(first_branch) = branch_names.first() {
                    status_bar = status_bar.branch_name(first_branch.clone());
                }

                status_bar
            })
    }
}
