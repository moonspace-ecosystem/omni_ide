# Omni IDE Architecture

## System Overview

Omni IDE extends Zed with GitButler integration. The custom code lives in 4 crates that integrate with Zed's existing workspace/panel/dock system.

```
┌─────────────────────────────────────────────────────────────────┐
│                        Zed Application                          │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ Title Bar│  │   Editor      │  │ Project Panel │              │
│  └──────────┘  └──────────────┘  └──────────────┘              │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    GitButler Panel (Bottom Dock)          │  │
│  │  ┌─────────────┐ ┌──────────┐┌──────────┐ ┌───────────┐ │  │
│  │  │ Unassigned  │ │StackLane ││StackLane │ │  Commit   │ │  │
│  │  │  Changes    │ │ (branch1)││ (branch2)│ │  Detail   │ │  │
│  │  │  (left)     │ │          ││          │ │  (right)  │ │  │
│  │  └─────────────┘ └──────────┘└──────────┘ └───────────┘ │  │
│  │  ┌──────────────────────────────────────────────────────┐│  │
│  │  │                    Status Bar                        ││  │
│  │  └──────────────────────────────────────────────────────┘│  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                     Git Graph (Tab)                       │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Crate Dependency Graph

```
gitbutler_panel
  ├── gitbutler_store (state management)
  │     ├── but-ctx (GitButler context, repo discovery)
  │     ├── but-workspace (virtual branches, ref info, stacks)
  │     ├── but-graph (commit graph traversal)
  │     ├── but-api (high-level API: diff, commit, assign)
  │     ├── but-hunk-assignment (worktree changes, hunk assignments)
  │     ├── but-core (TreeChange, ChangeState, diff types)
  │     ├── but-rebase (commit creation, graph mutations)
  │     └── but-db (forge reviews / PR data)
  ├── gitbutler_bridge (async helpers, ObjectId)
  ├── editor (for CommitModal text editor)
  ├── workspace (Panel trait, Workspace, Dock)
  ├── ui (Button, Icon, Label, Tooltip, etc.)
  └── theme (colors, status colors)

git_graph (separate, independent crate)
  ├── project (GitStore, Repository)
  ├── editor (for inline diff display)
  ├── ui (Table, Chip, DiffStat, etc.)
  └── workspace (Item trait)
```

## Data Flow

### Initialization Flow

```
main.rs
  → gitbutler_panel::panel::init(cx)        // registers ToggleFocus action
  → git_graph::init(cx)                      // registers Git Graph item

zed.rs::initialize_workspace()
  → GitButlerPanel::load(workspace, cx)      // async load
    → GitButlerPanel::new(workspace, cx)
      → GitButlerStore::init()               // creates store entity
      → store.discover(worktree_path)        // discovers Git repo via but-ctx
      → UnassignedChanges::new(store)        // left panel
      → CommitDetailPanel::new(store)        // right panel
      → cx.subscribe(&store, ...)            // listen for store events
  → workspace.add_panel(panel)               // adds to bottom dock

zed.rs::register_actions()
  → CommitModal::register(workspace)         // registers Commit, Push, Fetch actions
```

### State Update Flow

```
File system change / User action
  → GitButlerStore::trigger_update()
    → background: but_api::diff::changes_in_worktree()
    → foreground: store.worktree_changes = Some(changes)
    → cx.emit(WorktreeChangesUpdated)
      → GitButlerPanel listens → cx.notify() → re-render
      → UnassignedChanges listens → fetch_changes() → re-render
```

### Action Dispatch Flow

```
User clicks "Commit..." button (stack_lane.rs)
  → window.dispatch_action(Commit { branch_name })
  → CommitModal::register handler catches it
    → workspace.toggle_modal(CommitModal::new)
  → User types message, clicks "Commit"
    → store.commit(branch_name, message, cx)
      → background: but_api::commit::create::commit_create_only()
      → foreground: emit CommitCompleted / Error
      → trigger_update() to refresh UI
```

## GitButlerStore API

| Method | Description | Async? |
|--------|-------------|--------|
| `init()` | Create empty store | No |
| `discover(path)` | Find Git repo, init but-ctx | No |
| `workspace_ref_info()` | Get branch/stack/commit data | No |
| `worktree_changes()` | Get file changes + hunk assignments | No |
| `changes_count()` | Count of changed files | No |
| `branch_names()` | List all virtual branch names | No |
| `fetch_prs()` | Get PR numbers → URLs from but-db | No |
| `assign_hunk(branch, path)` | Assign file to a branch | Yes (spawned) |
| `unassign_file(path)` | Remove file from branch | Yes (spawned) |
| `commit(branch, message)` | Create commit on branch | Yes (spawned) |
| `push(branch)` | Git push via CLI | Yes (spawned) |
| `stash_unassigned_changes(branch, assignments)` | Move unassigned to new branch | Yes (spawned) |
| `fetch_commit_details(oid)` | Get diff for a commit | Yes (Task) |
| `handle_fs_events(events)` | React to file system changes | No |
| `trigger_update()` | Debounced refresh of worktree changes | Yes (spawned) |

## Event System

`GitButlerStoreEvent` variants:

| Event | Emitted When | Subscribers |
|-------|-------------|-------------|
| `WorkspaceChanged` | Git refs change (.git dir modified) | GitButlerPanel |
| `WorktreeChangesUpdated` | File changes refreshed | GitButlerPanel, UnassignedChanges, GutterBridge |
| `CommitCompleted(id)` | Commit created successfully | (unused currently) |
| `RebaseCompleted` | Rebase operation done | (unused currently) |
| `LoadingStateChanged(state)` | Loading/Idle/Error transition | GitButlerPanel |
| `Error(msg)` | Any operation failed | GitButlerPanel |

## Entry Points in Zed

| File | Line | What it does |
|------|------|-------------|
| `main.rs:771` | `gitbutler_panel::panel::init(cx)` | Registers panel toggle action |
| `main.rs:772` | `git_graph::init(cx)` | Registers git graph |
| `zed.rs:721-746` | `GitButlerPanel::load(...)` | Creates and adds panel to workspace |
| `zed.rs:856` | `CommitModal::register(workspace)` | Registers Commit/Push/Fetch workspace actions |
