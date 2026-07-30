use fs::FakeFs;
use gpui::{TestAppContext, VisualTestContext};
use project::Project;
use serde_json::json;
use workspace::{AppState, MultiWorkspace};

use crate::GitButlerPanel;

fn init_test(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let app_state = AppState::test(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        editor::init(cx);
        workspace::init(app_state, cx);
    });
}

#[gpui::test]
async fn test_gitbutler_panel_init(cx: &mut TestAppContext) {
    init_test(cx);

    let fs = FakeFs::new(cx.executor());
    fs.insert_tree(
        "/root",
        json!({
            ".git": {
                "config": ""
            },
            "file1.txt": "content"
        }),
    )
    .await;

    let project = Project::test(fs.clone(), ["/root".as_ref()], cx).await;
    let window = cx.add_window(|window, cx| MultiWorkspace::test_new(project.clone(), window, cx));
    
    let workspace = window
        .read_with(cx, |mw, _| mw.workspace().clone())
        .unwrap();

    let cx = &mut VisualTestContext::from_window(window.into(), cx);
    
    // Attach panel
    let _panel = workspace.update_in(cx, GitButlerPanel::new);
    
    cx.run_until_parked();
}

#[gpui::test]
async fn test_gitbutler_panel_unassigned_changes(cx: &mut gpui::TestAppContext) {
    init_test(cx);

    let fs = fs::FakeFs::new(cx.executor());
    fs.insert_tree(
        "/root",
        serde_json::json!({
            ".git": {},
            "file1.txt": "content",
            "file2.txt": "content",
        }),
    )
    .await;

    let project = Project::test(fs, ["/root".as_ref()], cx).await;
    
    let window = cx.add_window(|window, cx| workspace::MultiWorkspace::test_new(project.clone(), window, cx));
    let workspace = window.read_with(cx, |mw, _| mw.workspace().clone()).unwrap();
    let cx = &mut VisualTestContext::from_window(window.into(), cx);
    
    let panel = workspace.update_in(cx, GitButlerPanel::new);
    
    // Construct mock WorktreeChanges
    use but_hunk_assignment::{WorktreeChanges, HunkAssignment};
    use but_core::ui::{WorktreeChanges as CoreWorktreeChanges};
    let assignments = vec![
        HunkAssignment {
            id: None,
            hunk_header: None,
            path: "file1.txt".to_string(),
            path_bytes: "file1.txt".into(),
            branch_ref_bytes: None,
            stack_id: None,
            line_nums_added: None,
            line_nums_removed: None,
            diff: None,
        },
        HunkAssignment {
            id: None,
            hunk_header: None,
            path: "file2.txt".to_string(),
            path_bytes: "file2.txt".into(),
            branch_ref_bytes: None,
            stack_id: None,
            line_nums_added: None,
            line_nums_removed: None,
            diff: None,
        }
    ];
    let changes = WorktreeChanges {
        worktree_changes: CoreWorktreeChanges {
            changes: vec![
                but_core::ui::TreeChange {
                    path: "file1.txt".into(),
                    path_bytes: "file1.txt".into(),
                    status: but_core::ui::TreeStatus::Addition {
                        state: but_core::ui::ChangeState {
                            id: gix::ObjectId::empty_tree(gix::hash::Kind::Sha1),
                            kind: gix::object::tree::EntryKind::Blob,
                        },
                        is_untracked: true,
                    }
                },
                but_core::ui::TreeChange {
                    path: "file2.txt".into(),
                    path_bytes: "file2.txt".into(),
                    status: but_core::ui::TreeStatus::Addition {
                        state: but_core::ui::ChangeState {
                            id: gix::ObjectId::empty_tree(gix::hash::Kind::Sha1),
                            kind: gix::object::tree::EntryKind::Blob,
                        },
                        is_untracked: true,
                    }
                }
            ],
            ignored_changes: vec![],
        },
        assignments,
        assignments_error: None,
        dependencies: None,
        dependencies_error: None,
    };

    // Inject into store
    panel.update(cx, |p, cx| {
        p.store.update(cx, |store, cx| {
            store.set_worktree_changes(changes, cx);
        });
    });

    cx.run_until_parked();

    let store = panel.read_with(cx, |p, _| p.store.clone());
    let (is_some, changes_count) = store.read_with(cx, |s, _| {
        (s.worktree_changes().is_some(), s.changes_count())
    });
    assert!(is_some);
    assert_eq!(changes_count, 2);
}
