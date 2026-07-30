use but_core::ui::TreeStatus;
use but_graph::workspace::StackCommitFlags;
use but_hunk_assignment::WorktreeChanges;
use but_workspace::branch::Stack as ButStack;
use but_workspace::ref_info::{Commit, Segment};
use but_workspace::RefInfo as ButRefInfo;
use gpui::SharedString;

#[derive(Clone, Debug)]
pub struct UiWorkspace {
    pub stacks: Vec<UiStack>,
}

impl UiWorkspace {
    pub fn from_ref_info(info: &ButRefInfo, prs: &std::collections::HashMap<usize, String>) -> Self {
        Self {
            stacks: info.stacks.iter().map(|s| UiStack::from_stack(s, prs)).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiStack {
    pub segments: Vec<UiSegment>,
}

impl UiStack {
    pub fn from_stack(stack: &ButStack, prs: &std::collections::HashMap<usize, String>) -> Self {
        Self {
            segments: stack.segments.iter().map(|s| UiSegment::from_segment(s, prs)).collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiSegment {
    pub commits: Vec<UiCommit>,
    pub branch_name: Option<SharedString>,
    pub pr_number: Option<usize>,
    pub pr_url: Option<SharedString>,
}

impl UiSegment {
    pub fn from_segment(seg: &Segment, prs: &std::collections::HashMap<usize, String>) -> Self {
        let pr_number = seg.metadata.as_ref().and_then(|m| m.review.pull_request);
        let pr_url = pr_number.and_then(|num| prs.get(&num).map(|u| gpui::SharedString::from(u.clone())));
        let branch_name = seg
            .ref_info
            .as_ref()
            .map(|r| SharedString::from(r.ref_name.as_bstr().to_string()));
        Self {
            commits: seg
                .commits
                .iter()
                .map(|c| UiCommit::from(&c.inner))
                .collect(),
            branch_name,
            pr_number,
            pr_url,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiCommit {
    pub id: SharedString,
    pub message: SharedString,
    pub author: SharedString,
    pub author_email: SharedString,
    pub timestamp: Option<i64>,
    pub flags: StackCommitFlags,
    pub is_conflicted: bool,
}

impl From<&Commit> for UiCommit {
    fn from(commit: &Commit) -> Self {
        Self {
            id: commit.id.to_hex_with_len(7).to_string().into(),
            message: commit.message.to_string().into(),
            author: commit.author.name.to_string().into(),
            author_email: commit.author.email.to_string().into(),
            timestamp: Some(commit.author.time.seconds),
            flags: commit.flags,
            is_conflicted: commit.flags.contains(StackCommitFlags::HasConflicts),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiWorktreeChanges {
    pub changes: Vec<UiTreeChange>,
    pub assignments: Vec<UiHunkAssignment>,
}

#[derive(Clone, Debug)]
pub struct UiHunkAssignment {
    pub path: SharedString,
    pub branch_ref_bytes: Option<gpui::SharedString>,
}

impl From<&but_hunk_assignment::HunkAssignment> for UiHunkAssignment {
    fn from(assignment: &but_hunk_assignment::HunkAssignment) -> Self {
        Self {
            path: assignment.path_bytes.to_string().into(),
            branch_ref_bytes: assignment.branch_ref_bytes.as_ref().map(|b| b.as_bstr().to_string().into()),
        }
    }
}

impl From<&WorktreeChanges> for UiWorktreeChanges {
    fn from(wc: &WorktreeChanges) -> Self {
        Self {
            changes: wc
                .worktree_changes
                .changes
                .iter()
                .map(UiTreeChange::from)
                .collect(),
            assignments: wc
                .assignments
                .iter()
                .map(UiHunkAssignment::from)
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UiTreeChange {
    pub path: SharedString,
    pub status: FileStatus,
}

impl From<&but_core::ui::TreeChange> for UiTreeChange {
    fn from(change: &but_core::ui::TreeChange) -> Self {
        Self {
            path: change.path_bytes.to_string().into(),
            status: FileStatus::from(&change.status),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl From<&TreeStatus> for FileStatus {
    fn from(status: &TreeStatus) -> Self {
        match status {
            TreeStatus::Addition { .. } => FileStatus::Added,
            TreeStatus::Modification { .. } => FileStatus::Modified,
            TreeStatus::Deletion { .. } => FileStatus::Deleted,
            TreeStatus::Rename { .. } => FileStatus::Renamed,
        }
    }
}

pub use gitbutler_store::LoadingState;
