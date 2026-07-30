use anyhow::Result;
use but_ctx::Context;
use but_graph::{RefInfo, Workspace};
use gpui::{EventEmitter, Task};
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LoadingState {
    Idle,
    Loading,
    Error(String),
}

impl Default for LoadingState {
    fn default() -> Self {
        Self::Idle
    }
}

pub enum GitButlerStoreEvent {
    WorkspaceChanged,
    WorktreeChangesUpdated,
    CommitCompleted(String),
    RebaseCompleted,
    LoadingStateChanged(LoadingState),
    Error(String),
}

pub struct GitButlerStore {
    ctx: Option<Context>,
    worktree_changes: Option<but_hunk_assignment::WorktreeChanges>,
    loading_state: LoadingState,
    _update_task: Option<Task<()>>,
    _watcher_model: Option<gpui::Entity<gitbutler_watcher::GitButlerWatcher>>,
    _watcher_subscription: Option<gpui::Subscription>,
}

impl GitButlerStore {
    pub fn init() -> Self {
        Self {
            ctx: None,
            worktree_changes: None,
            loading_state: LoadingState::Idle,
            _update_task: None,
            _watcher_model: None,
            _watcher_subscription: None,
        }
    }

    pub fn loading_state(&self) -> &LoadingState {
        &self.loading_state
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.loading_state, LoadingState::Loading)
    }

    fn set_loading(&mut self, state: LoadingState, cx: &mut gpui::Context<Self>) {
        self.loading_state = state.clone();
        cx.emit(GitButlerStoreEvent::LoadingStateChanged(state));
        cx.notify();
    }

    pub fn discover(&mut self, path: impl AsRef<Path>, cx: &mut gpui::Context<Self>) -> Result<()> {
        let ctx = Context::discover(path.as_ref())?;
        self.ctx = Some(ctx);

        // Initialize watcher
        if let Ok(watcher_model) = gitbutler_watcher::GitButlerWatcher::new(path.as_ref().to_path_buf(), cx) {
            let subscription = cx.subscribe(&watcher_model, |this, _model, event, cx| {
                match event {
                    gitbutler_watcher::WatcherEvent::Changed => {
                        this.refresh(cx);
                    }
                }
            });
            self._watcher_model = Some(watcher_model);
            self._watcher_subscription = Some(subscription);
        }

        self.trigger_update(cx);
        Ok(())
    }

    pub fn workspace(&self) -> Option<Workspace> {
        let ctx = self.ctx.as_ref()?;
        let (_guard, _repo, ws, _db) = ctx.workspace_and_db().ok()?;
        Some(ws.clone())
    }

    pub fn fetch_prs(&self) -> std::collections::HashMap<usize, String> {
        let mut prs = std::collections::HashMap::new();
        let Some(ctx) = self.ctx.as_ref() else {
            return prs;
        };
        let Ok((_guard, _repo, _ws, db)) = ctx.workspace_and_db() else {
            return prs;
        };

        if let Ok(reviews) = db.forge_reviews().list_all() {
            for review in reviews {
                if review.number > 0 {
                    prs.insert(review.number as usize, review.html_url);
                }
            }
        }

        prs
    }

    pub fn fetch(&mut self, cx: &mut gpui::Context<Self>) {
        let Some(ctx) = self.ctx.as_ref() else {
            return;
        };
        let sync_ctx = ctx.to_sync();
        self.set_loading(LoadingState::Loading, cx);

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    use gitbutler_git::GitContextExt as _;
                    let ctx = Context::from(sync_ctx);
                    ctx.fetch("origin", None)
                })
                .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(_) => {
                        this.set_loading(LoadingState::Idle, cx);
                    }
                    Err(e) => {
                        this.set_loading(LoadingState::Error(e.to_string()), cx);
                    }
                }
                this.trigger_update(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn ref_info(&self) -> Option<RefInfo> {
        let ctx = self.ctx.as_ref()?;
        let (_guard, _repo, ws, _db) = ctx.workspace_and_db().ok()?;
        ws.ref_info().cloned()
    }

    pub fn workspace_ref_info(&self) -> Option<but_workspace::RefInfo> {
        let ctx = self.ctx.as_ref()?;
        let (_guard, repo, ws, _db) = ctx.workspace_and_db().ok()?;
        but_workspace::graph_to_ref_info(
            &ws,
            &repo,
            but_workspace::ref_info::Options {
                traversal: but_graph::init::Options::limited(),
                expensive_commit_info: true,
            },
        )
        .ok()
    }

    pub fn worktree_changes(&self) -> Option<&but_hunk_assignment::WorktreeChanges> {
        self.worktree_changes.as_ref()
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn set_worktree_changes(
        &mut self,
        changes: but_hunk_assignment::WorktreeChanges,
        cx: &mut gpui::Context<Self>,
    ) {
        self.worktree_changes = Some(changes);
        cx.emit(GitButlerStoreEvent::WorktreeChangesUpdated);
        cx.notify();
    }

    pub fn take_worktree_changes(&mut self) -> Option<but_hunk_assignment::WorktreeChanges> {
        self.worktree_changes.take()
    }

    pub fn changes_count(&self) -> usize {
        self.worktree_changes
            .as_ref()
            .map(|c| c.worktree_changes.changes.len())
            .unwrap_or(0)
    }

    pub fn context_mut(&mut self) -> Option<&mut Context> {
        self.ctx.as_mut()
    }

    pub fn branch_names(&self) -> Vec<String> {
        let Some(ref_info) = self.workspace_ref_info() else {
            return Vec::new();
        };
        ref_info
            .stacks
            .iter()
            .flat_map(|stack| {
                stack.segments.iter().filter_map(|seg| {
                    seg.ref_info
                        .as_ref()
                        .map(|r| r.ref_name.as_bstr().to_string())
                })
            })
            .collect()
    }

    pub fn assign_hunk(&mut self, branch_name: &str, path: String, cx: &mut gpui::Context<Self>) {
        let Some(sync_ctx) = self.ctx.as_ref().map(|c| c.to_sync()) else {
            return;
        };
        self.set_loading(LoadingState::Loading, cx);
        let branch_name = branch_name.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    use but_hunk_assignment::{HunkAssignmentRequest, HunkAssignmentTarget};
                    use gix::bstr::BString;

                    let mut ctx = Context::from(sync_ctx);
                    let target = HunkAssignmentTarget::Branch {
                        branch_ref_bytes: BString::from(
                            format!("refs/heads/{}", branch_name).as_bytes(),
                        ),
                    };

                    let req = HunkAssignmentRequest {
                        hunk_header: None,
                        path_bytes: BString::from(path),
                        target: Some(target),
                    };

                    but_api::diff::assign_hunk(&mut ctx, vec![req])
                })
                .await;

            this.update(cx, |this, cx| {
                if let Err(error) = result {
                    this.set_loading(LoadingState::Error(error.to_string()), cx);
                    cx.emit(GitButlerStoreEvent::Error(error.to_string()));
                } else {
                    this.set_loading(LoadingState::Idle, cx);
                }
                this.trigger_update(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn unassign_file(&mut self, path: String, cx: &mut gpui::Context<Self>) {
        let Some(sync_ctx) = self.ctx.as_ref().map(|c| c.to_sync()) else {
            return;
        };
        self.set_loading(LoadingState::Loading, cx);
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    use but_hunk_assignment::HunkAssignmentRequest;
                    use gix::bstr::BString;

                    let mut ctx = Context::from(sync_ctx);
                    let req = HunkAssignmentRequest {
                        hunk_header: None,
                        path_bytes: BString::from(path),
                        target: None,
                    };

                    but_api::diff::assign_hunk(&mut ctx, vec![req])
                })
                .await;

            this.update(cx, |this, cx| {
                if let Err(error) = result {
                    this.set_loading(LoadingState::Error(error.to_string()), cx);
                    cx.emit(GitButlerStoreEvent::Error(error.to_string()));
                } else {
                    this.set_loading(LoadingState::Idle, cx);
                }
                this.trigger_update(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn commit(&mut self, branch_name: &str, message: &str, cx: &mut gpui::Context<Self>) {
        let Some(sync_ctx) = self.ctx.as_ref().map(|c| c.to_sync()) else {
            return;
        };
        // `commit_create_only` only commits the `DiffSpec`s it is given, so we must
        // gather the hunks the user assigned to this branch and convert them,
        // otherwise the commit would always be created empty.
        let full_ref = format!("refs/heads/{}", branch_name);
        let assignments: Vec<but_hunk_assignment::HunkAssignment> = self
            .worktree_changes
            .as_ref()
            .map(|changes| {
                changes
                    .assignments
                    .iter()
                    .filter(|assignment| {
                        assignment
                            .branch_ref_bytes
                            .as_ref()
                            .map(|branch_ref| branch_ref.as_bstr().to_string() == full_ref)
                            .unwrap_or(false)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        self.set_loading(LoadingState::Loading, cx);
        let branch_name = branch_name.to_string();
        let message = message.to_string();

        cx.spawn(async move |this, cx| {
            let res = cx
                .background_executor()
                .spawn(async move {
                    use but_core::DryRun;
                    use but_rebase::graph_rebase::mutate::{InsertSide, RelativeTo};
                    let mut ctx = Context::from(sync_ctx);

                    let ref_name = gix::refs::FullName::try_from(
                        format!("refs/heads/{}", branch_name).as_str(),
                    )?;
                    let relative_to = RelativeTo::Reference(ref_name);
                    let changes =
                        but_hunk_assignment::convert_assignments_to_diff_specs(&assignments)?;

                    but_api::commit::create::commit_create_only(
                        &mut ctx,
                        relative_to,
                        InsertSide::Above,
                        changes,
                        message,
                        DryRun::No,
                    )
                })
                .await;

            this.update(cx, |this, cx| {
                match res {
                    Ok(res) => {
                        if let Some(commit_id) = res.new_commit {
                            cx.emit(GitButlerStoreEvent::CommitCompleted(commit_id.to_string()));
                        }
                        this.set_loading(LoadingState::Idle, cx);
                    }
                    Err(error) => {
                        this.set_loading(LoadingState::Error(error.to_string()), cx);
                        cx.emit(GitButlerStoreEvent::Error(error.to_string()));
                    }
                }
                this.trigger_update(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn push(&mut self, branch_name: &str, cx: &mut gpui::Context<Self>) {
        let Some(sync_ctx) = self.ctx.as_ref().map(|c| c.to_sync()) else {
            return;
        };
        self.set_loading(LoadingState::Loading, cx);
        let branch_name = branch_name.to_string();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    use gitbutler_git::GitContextExt as _;
                    let ctx = Context::from(sync_ctx);
                    ctx.git_test_push("origin", &branch_name, Some(None))

                })
                .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(()) => {
                        this.set_loading(LoadingState::Idle, cx);
                    }
                    Err(error) => {
                        this.set_loading(LoadingState::Error(error.to_string()), cx);
                        cx.emit(GitButlerStoreEvent::Error(error.to_string()));
                    }
                }
                this.trigger_update(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn stash_unassigned_changes(
        &mut self,
        branch_name: String,
        unassigned: Vec<but_hunk_assignment::HunkAssignment>,
        cx: &mut gpui::Context<Self>,
    ) {
        let Some(sync_ctx) = self.ctx.as_ref().map(|c| c.to_sync()) else {
            return;
        };
        self.set_loading(LoadingState::Loading, cx);

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move {
                    use but_hunk_assignment::{HunkAssignmentRequest, HunkAssignmentTarget};
                    use gix::bstr::BString;

                    let mut ctx = Context::from(sync_ctx);
                    let target = HunkAssignmentTarget::Branch {
                        branch_ref_bytes: BString::from(
                            format!("refs/heads/{}", branch_name).as_bytes(),
                        ),
                    };

                    let requests: Vec<HunkAssignmentRequest> = unassigned
                        .iter()
                        .map(|assignment| HunkAssignmentRequest {
                            hunk_header: assignment.hunk_header.clone(),
                            path_bytes: assignment.path_bytes.clone(),
                            target: Some(target.clone()),
                        })
                        .collect();

                    but_api::diff::assign_hunk(&mut ctx, requests)
                })
                .await;

            this.update(cx, |this, cx| {
                match result {
                    Ok(_) => this.set_loading(LoadingState::Idle, cx),
                    Err(error) => {
                        this.set_loading(LoadingState::Error(error.to_string()), cx);
                        cx.emit(GitButlerStoreEvent::Error(error.to_string()));
                    }
                }
                this.trigger_update(cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn handle_fs_events(
        &mut self,
        events: impl IntoIterator<Item = impl AsRef<Path>>,
        cx: &mut gpui::Context<Self>,
    ) {
        let mut git_changed = false;
        let mut worktree_changed = false;

        for event in events {
            let path = event.as_ref();
            if path.components().any(|c| c.as_os_str() == ".git") {
                git_changed = true;
            } else {
                worktree_changed = true;
            }
        }

        if git_changed {
            if let Some(ctx) = &mut self.ctx {
                if let Err(error) = ctx.invalidate_workspace_cache() {
                    cx.emit(GitButlerStoreEvent::Error(error.to_string()));
                }
            }
            cx.emit(GitButlerStoreEvent::WorkspaceChanged);
        }

        if git_changed || worktree_changed {
            self.trigger_update(cx);
        }
    }

    fn trigger_update(&mut self, cx: &mut gpui::Context<Self>) {
        if let Some(ctx) = &self.ctx {
            let sync_ctx = ctx.to_sync();
            self._update_task = Some(cx.spawn(async move |this, cx| {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;

                let result = cx
                    .background_executor()
                    .spawn(async move {
                        let ctx = Context::from(sync_ctx);
                        but_api::diff::changes_in_worktree(&ctx)
                    })
                    .await;

                this.update(cx, |this, cx| match result {
                    Ok(changes) => {
                        this.worktree_changes = Some(changes);
                        cx.emit(GitButlerStoreEvent::WorktreeChangesUpdated);
                        cx.notify();
                    }
                    Err(error) => {
                        cx.emit(GitButlerStoreEvent::Error(error.to_string()));
                    }
                })
                .ok();
            }));
        }
    }

    pub fn fetch_commit_details(
        &mut self,
        commit_oid: String,
        cx: &mut gpui::Context<Self>,
    ) -> gpui::Task<anyhow::Result<but_core::diff::CommitDetails>> {
        let Some(ctx) = self.ctx.as_ref() else {
            return gpui::Task::ready(Err(anyhow::anyhow!("GitButler not initialized")));
        };
        let sync_ctx = ctx.to_sync();
        cx.spawn(async move |_this, cx| {
            cx.background_executor()
                .spawn(async move {
                    let ctx = Context::from(sync_ctx);
                    let oid = gix::ObjectId::from_hex(commit_oid.as_bytes())?;
                    but_api::diff::commit_details(&ctx, oid, but_api::diff::ComputeLineStats::Yes)
                })
                .await
        })
    }
}

impl EventEmitter<GitButlerStoreEvent> for GitButlerStore {}
