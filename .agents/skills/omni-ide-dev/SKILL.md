---
name: omni-ide-dev
description: Development guide for Omni IDE (forked from Zed). Use this when working on ANY feature in the project. Covers the full feature landscape (editor, panels, git, AI agent, terminal, extensions, etc.), custom crate architecture (gitbutler_panel, gitbutler_store, gitbutler_bridge, git_graph), fork-specific modifications, data flow, UI component catalog, GPUI patterns, and feature development checklists.
---

# Omni IDE Development Skill

Omni IDE is a **fork of Zed** (high-performance code editor). It inherits ALL of Zed's features and adds custom Git integration via GitButler.

## Project Identity

- **App Name**: Still "Zed" internally (see `crates/paths/src/paths.rs:18`)
- **Binary**: `target/debug/zed`
- **Total crates**: ~238 in `crates/` directory

## Full Feature Landscape

Read `references/feature-map.md` for the complete mapping of all features to their crates.

### Inherited from Zed (core features)

| Category | Features | Key Crate(s) |
|----------|----------|--------------|
| **Editor** | Multi-cursor, Tree-sitter syntax, LSP, diagnostics, code actions | `editor`, `language`, `lsp`, `diagnostics` |
| **AI Agent** | AI assistant panel, inline assist, thread management | `agent`, `agent_ui`, `agent_settings`, `agent_servers` |
| **Project** | File tree, outline, symbol search, worktree management | `project_panel`, `outline_panel`, `project`, `worktree` |
| **Terminal** | Integrated terminal emulator | `terminal`, `terminal_view` |
| **Search** | Buffer search, project-wide search | `search` |
| **Collaboration** | Real-time collab, channels, screen sharing | `collab`, `collab_ui`, `call`, `channel` |
| **Extensions** | Extension system, marketplace UI | `extension`, `extension_host`, `extensions_ui` |
| **Vim** | Full vim mode | `vim` |
| **Themes** | Theme system, theme selector, custom themes | `theme`, `theme_selector`, `theme_settings` |
| **Languages** | Language support, toolchain selection | `language`, `languages`, `language_selector` |
| **Edit Prediction** | AI-powered code completion | `edit_prediction`, `edit_prediction_ui` |
| **Settings** | Settings UI, keymap editor, profile selector | `settings_ui`, `keymap_editor`, `settings` |
| **Debugger** | DAP protocol, debugger UI | `dap`, `debugger_ui` |
| **Markdown** | Preview, editing | `markdown`, `markdown_preview` |
| **REPL** | Jupyter-style notebooks | `repl` |
| **File Formats** | CSV preview, SVG preview, image viewer | `csv_preview`, `svg_preview`, `image_viewer` |

### Custom / Modified by Omni IDE

| Category | What changed | Key Crate(s) |
|----------|-------------|--------------|
| **GitButler Panel** | NEW — Virtual branch management panel | `gitbutler_panel`, `gitbutler_store`, `gitbutler_bridge` |
| **Git Graph** | MODIFIED — Removed `git_ui` dependency, custom commit avatar | `git_graph` |
| **Title Bar** | MODIFIED — Disabled WorktreePicker and BranchPicker popover menus | `title_bar` |
| **Project Panel** | MODIFIED — Removed FileDiffView references | `project_panel` |
| **Open Listener** | MODIFIED — Removed MultiDiffView/FileDiffView handling | `zed/open_listener.rs` |
| **Vim** | MODIFIED — Removed `git_ui::init()` from test context | `vim` |

### What was REMOVED from Zed

The `git_ui` crate (Zed's native git panel) was **completely removed** from Omni IDE. The `clean_*.py` scripts document each removal:

| Script | What it removes |
|--------|----------------|
| `clean_main.py` | `git_ui::init(cx)` from main.rs |
| `clean_git_panel.py` | `git_ui` references from git_graph.rs |
| `clean_title_bar.py` | WorktreePicker and BranchPicker from title bar |
| `clean_commit_avatar.py` | CommitAvatar component, replaced with plain div |
| `clean_commit_view.py` | CommitView open calls and OpenCommitView action |
| `clean_open_listener.py` | FileDiffView/MultiDiffView from CLI handler |
| `clean_project_panel.py` | FileDiffView references from project panel |
| `clean_vim.py` | `git_ui::init()` from vim test context |

## Quick Reference — Custom Crates

| Crate | Purpose | Key File |
|-------|---------|----------|
| `gitbutler_panel` | UI panel for GitButler virtual branches | `crates/gitbutler_panel/src/panel.rs` |
| `gitbutler_store` | State management, Git operations via `but-*` APIs | `crates/gitbutler_store/src/gitbutler_store.rs` |
| `gitbutler_bridge` | Async bridge + ObjectId conversion | `crates/gitbutler_bridge/src/` |
| `git_graph` | Git commit graph visualization (table + graph lines) | `crates/git_graph/src/git_graph.rs` |

## Before Any Change

1. Read `references/feature-map.md` for the full feature → crate mapping
2. Read `references/architecture.md` for custom crate data flow
3. Read `references/component-catalog.md` to find which file to edit
4. Read `references/ui-patterns.md` for GPUI conventions

## Feature Development Checklist

### For GitButler features (custom crates):
1. **Store layer** (`gitbutler_store`): Add method for the Git operation
2. **Event** (`gitbutler_store`): Add variant to `GitButlerStoreEvent` if UI needs to react
3. **Action** (`gitbutler_panel/actions.rs`): Define action struct
4. **UI Component** (`gitbutler_panel/`): Create or modify component
5. **Wire up** (`panel.rs`): Connect action handlers via `.on_action(cx.listener(...))`
6. **Register** (`zed/src/zed.rs:register_actions`): Register workspace-level actions if needed
7. **Test** (`gitbutler_panel_tests.rs`): Add test with `FakeFs` + mock store data

### For Zed-inherited features:
1. **Find the crate**: Use `references/feature-map.md`
2. **Read the crate's `.rules`**: Many crates have their own `.rules` file
3. **Follow Zed patterns**: Entity, Action, Render, Panel trait
4. **Test**: Each crate typically has its own test infrastructure

## Workspace Panel Architecture

All panels in Omni IDE implement the `Panel` trait from the `workspace` crate:

| Panel | Position | Crate | Icon |
|-------|----------|-------|------|
| Project Panel | Left | `project_panel` | FileTree |
| Outline Panel | Left | `outline_panel` | ListTree |
| Collab Panel | Left | `collab_ui` | People |
| Agent Panel | Right | `agent_ui` | ZedAssistant |
| Terminal Panel | Bottom | `terminal_view` | Terminal |
| GitButler Panel | Bottom | `gitbutler_panel` | GitBranch |
| Debug Panel | Bottom | `debugger_ui` | Bug |

## Build & Test Commands

```bash
# Check specific crate
cargo check -p gitbutler_panel
cargo check -p git_graph
cargo check -p editor
cargo check -p zed

# Run specific tests
cargo test -p gitbutler_panel
cargo test -p git_graph
cargo test -p editor

# Build full app
cargo build -p zed

# Run app
./target/debug/zed

# Clippy (project-specific script)
./script/clippy
```

## Critical Patterns

- The root `div()` in any custom Panel's `render` MUST have `.id(...)` and `.track_focus(&self.focus_handle)` — without this, action dispatch silently fails
- Use `FakeFs` (not `RealFs`) in tests to avoid `Parking forbidden` panics
- `set_worktree_changes` is gated behind `#[cfg(any(test, feature = "test-support"))]`
- Store methods use `ctx.to_sync()` + `cx.spawn` + `cx.background_executor().spawn` pattern for async Git operations
- The `git_ui` crate does NOT exist in this fork — any Zed upstream references to it must be removed/replaced
- `clean_*.py` scripts at project root document all fork modifications
