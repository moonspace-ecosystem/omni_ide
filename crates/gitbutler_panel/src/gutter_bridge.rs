use anyhow::Result;
use gpui::{App, Context, Entity, Subscription};
use std::collections::HashMap;
use std::path::Path;

use gitbutler_store::{GitButlerStore, GitButlerStoreEvent};

/// A generic struct that the Editor can render for a hunk in the gutter.
#[derive(Clone, Debug)]
pub struct GutterHunk {
    pub start_line: u32,
    pub end_line: u32,
    pub status: String,
}

/// An adapter to bridge GitButler virtual branch data to the Zed editor gutter.
pub struct GitButlerGutterBridge {
    store: Entity<GitButlerStore>,
    hunks_by_file: HashMap<String, Vec<GutterHunk>>,
    _subscription: Subscription,
}

impl GitButlerGutterBridge {
    pub fn new(store: Entity<GitButlerStore>, cx: &mut Context<Self>) -> Self {
        let _subscription = cx.subscribe(&store, |this, _, event: &GitButlerStoreEvent, cx| {
            if let GitButlerStoreEvent::WorktreeChangesUpdated = event {
                this.update_cache(cx);
                cx.notify();
            }
        });

        let mut this = Self {
            store,
            hunks_by_file: HashMap::new(),
            _subscription,
        };
        this.update_cache(cx);
        this
    }

    fn update_cache(&mut self, cx: &mut Context<Self>) {
        self.hunks_by_file.clear();
        let store = self.store.read(cx);
        if let Some(changes) = store.worktree_changes() {
            for assignment in &changes.assignments {
                if let Some(header) = &assignment.hunk_header {
                    let status = match &assignment.branch_ref_bytes {
                        Some(b) => String::from_utf8_lossy(b.as_bstr().as_ref()).to_string(),
                        None => "unassigned".to_string(),
                    };

                    let start_line = header.new_start;
                    let end_line = if header.new_lines > 0 {
                        header.new_start + header.new_lines - 1
                    } else {
                        header.new_start
                    };

                    self.hunks_by_file
                        .entry(assignment.path.clone())
                        .or_default()
                        .push(GutterHunk {
                            start_line,
                            end_line,
                            status,
                        });
                }
            }
        }
    }

    /// Fetches hunks for the currently focused virtual branch for the given file path.
    pub fn hunks_for_focused_branch(
        &self,
        path: &Path,
        _cx: &App,
    ) -> Result<Vec<GutterHunk>> {
        let file_path_str = path.to_string_lossy();
        Ok(self
            .hunks_by_file
            .get(file_path_str.as_ref())
            .cloned()
            .unwrap_or_default())
    }
}
