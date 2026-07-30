# Omni IDE — Complete Feature Map

Complete mapping of every feature to its crate(s), initialization, and UI location.

## Application Panels (7 panels registered at startup)

| Panel | Position | Crate | Init | Load Location |
|-------|----------|-------|------|---------------|
| Project Panel | Left sidebar | `project_panel` | `project_panel::init(cx)` | `zed.rs:initialize_panels()` |
| Outline Panel | Left sidebar | `outline_panel` | `outline_panel::init(cx)` | `zed.rs:initialize_panels()` |
| Collab Panel | Left sidebar | `collab_ui` | `collab_ui::init(&app_state, cx)` | `zed.rs:initialize_panels()` |
| Agent Panel | Right sidebar | `agent_ui` | via `initialize_agent_panel()` | `zed.rs:initialize_panels()` |
| Terminal Panel | Bottom dock | `terminal_view` | `terminal_view::init(cx)` | `zed.rs:initialize_panels()` |
| GitButler Panel | Bottom dock | `gitbutler_panel` | `gitbutler_panel::panel::init(cx)` | `zed.rs:initialize_panels()` |
| Debug Panel | Bottom dock | `debugger_ui` | via DebugPanel::load() | `zed.rs:initialize_panels()` |

## Editor System

| Feature | Crate | Description |
|---------|-------|-------------|
| Core Editor | `editor` | Text editing, multi-cursor, selections, folds, inlay hints |
| Multi-buffer | `multi_buffer` | Multiple buffers in one editor (e.g., search results) |
| Buffer Diff | `buffer_diff` | Inline diff display within editor |
| Diagnostics | `diagnostics` | Error/warning display, diagnostic panel |
| Edit Prediction | `edit_prediction`, `edit_prediction_ui` | AI code completion suggestions |
| Snippet | `snippet`, `snippet_provider`, `snippets_ui` | Code snippets |
| Breadcrumbs | `breadcrumbs` | File path breadcrumbs above editor |

## Git Integration (Omni IDE Custom)

| Feature | Crate | Description |
|---------|-------|-------------|
| GitButler Panel | `gitbutler_panel` | Virtual branch management UI (stacks, lanes, commits, DnD) |
| GitButler Store | `gitbutler_store` | State management, Git ops via but-* API (commit, push, assign) |
| GitButler Bridge | `gitbutler_bridge` | Async helpers, ObjectId conversion between gix/gitoxide |
| Git Graph | `git_graph` | Commit history graph (table view, graph lines, search, cherry-pick) |
| Git Core | `git` | Git primitives (Oid, status, remotes, hosting providers) |
| Git Hosting | `git_hosting_providers` | GitHub, GitLab, Bitbucket integration |

### GitButler Panel Components (15 files)

| File | Component | Type | Purpose |
|------|-----------|------|---------|
| `panel.rs` | GitButlerPanel | Entity | Main layout (toolbar + 3-column viewport + status) |
| `toolbar.rs` | Toolbar | RenderOnce | Top bar (toggle, fetch, new branch) |
| `status_bar.rs` | StatusBar | RenderOnce | Bottom bar (branch, changes, errors) |
| `stack_lane.rs` | StackLane | RenderOnce | Column per virtual stack |
| `branch_card.rs` | BranchCard | Entity | Branch header + commit list |
| `unassigned_changes.rs` | UnassignedChanges | Entity | Left panel (changed files) |
| `commit_detail_panel.rs` | CommitDetailPanel | Entity | Right panel (commit details + diff) |
| `diff_viewer.rs` | MultiDiffView | Entity | Diff display for selected commit |
| `actions.rs` | CommitModal | Entity | Commit message dialog |
| `dnd.rs` | DragState | — | Drag-and-drop infrastructure |
| `gutter_bridge.rs` | GitButlerGutterBridge | Entity | Hunk data → editor gutter |
| `gitbutler_colors.rs` | GitButlerColors | Trait | Color tokens for statuses |
| `models.rs` | UiWorkspace/UiStack/etc | Structs | UI data models |

## AI / Agent System

| Feature | Crate | Description |
|---------|-------|-------------|
| Agent Panel | `agent_ui` | Main AI assistant panel (chat, threads) |
| Agent Core | `agent` | Thread management, tool execution |
| Agent Settings | `agent_settings` | AI provider configuration |
| Agent Servers | `agent_servers` | MCP and language model servers |
| Agent Skills | `agent_skills` | Built-in skills (create-skill, etc.) |
| ACP Thread | `acp_thread` | Agent Communication Protocol threading |
| ACP Tools | `acp_tools` | Agent tools (file read/write, terminal, etc.) |
| Inline Assist | `agent_ui` | Inline code generation/editing |
| Language Models | `language_model`, `language_models` | LLM provider abstraction |
| LLM Providers | `anthropic`, `google_ai`, `open_ai`, `ollama`, `bedrock`, `deepseek`, `mistral`, `lmstudio`, `codestral`, `open_router`, `x_ai` | Individual provider implementations |

## Project & File Management

| Feature | Crate | Description |
|---------|-------|-------------|
| Project Panel | `project_panel` | File tree sidebar |
| Outline Panel | `outline_panel` | Code symbol outline |
| File Finder | `file_finder` | Cmd+P file search |
| Project Symbols | `project_symbols` | Workspace-wide symbol search |
| Recent Projects | `recent_projects` | Recent project history |
| Open Path Prompt | `open_path_prompt` | Custom path open dialog |
| Worktree | `worktree` | File system tree management |
| FS | `fs` | Filesystem abstraction (Real/Fake) |

## Terminal

| Feature | Crate | Description |
|---------|-------|-------------|
| Terminal Core | `terminal` | Terminal emulation engine |
| Terminal View | `terminal_view` | Terminal panel UI |

## Search

| Feature | Crate | Description |
|---------|-------|-------------|
| Buffer Search | `search` (buffer_search) | In-file search bar |
| Project Search | `search` (project_search) | Cross-file search panel |
| Fuzzy Matching | `fuzzy`, `fuzzy_nucleo` | Fuzzy match algorithms |

## Collaboration

| Feature | Crate | Description |
|---------|-------|-------------|
| Collab Panel | `collab_ui` | Collaboration panel (contacts, channels) |
| Collab Server | `collab` | Server-side collaboration |
| Channels | `channel` | Channel management |
| Call | `call` | Audio/video calling |
| LiveKit | `livekit_client`, `livekit_api` | Real-time communication |

## Themes & UI

| Feature | Crate | Description |
|---------|-------|-------------|
| Theme System | `theme` | Theme loading, active theme |
| Theme Settings | `theme_settings` | User theme preferences |
| Theme Selector | `theme_selector` | Theme picker UI |
| UI Components | `ui` | Shared UI kit (Button, Icon, Label, etc.) |
| Title Bar | `title_bar` | Custom title bar with branch info |
| Sidebar | `sidebar` | Sidebar container management |
| Component Preview | `component_preview` | UI component storybook |

## Extensions

| Feature | Crate | Description |
|---------|-------|-------------|
| Extension Host | `extension_host` | Extension runtime |
| Extensions UI | `extensions_ui` | Extension marketplace panel |
| Extension API | `extension_api` | Public API for extensions |
| Language Extension | `language_extension` | Language-specific extensions |
| Theme Extension | `theme_extension` | Theme extensions |

## Settings

| Feature | Crate | Description |
|---------|-------|-------------|
| Settings Core | `settings` | Settings infrastructure |
| Settings UI | `settings_ui` | GUI settings editor |
| Settings JSON | `settings_json` | JSON settings file handling |
| Keymap Editor | `keymap_editor` | Keyboard shortcut editor |
| Profile Selector | `settings_profile_selector` | Settings profile switching |

## Other Features

| Feature | Crate | Description |
|---------|-------|-------------|
| Vim Mode | `vim` | Full vim emulation |
| Go to Line | `go_to_line` | Ctrl+G line jump |
| Command Palette | `command_palette` | Cmd+Shift+P command palette |
| Tab Switcher | `tab_switcher` | Tab cycling UI |
| Journal | `journal` | Daily journal/notes |
| Feedback | `feedback` | User feedback submission |
| Onboarding | `onboarding` | First-run experience |
| Auto Update | `auto_update`, `auto_update_ui` | Application update system |
| Markdown Preview | `markdown_preview` | Live markdown rendering |
| CSV Preview | `csv_preview` | CSV table display |
| SVG Preview | `svg_preview` | SVG rendering |
| Image Viewer | `image_viewer` | Image display |
| Inspector | `inspector_ui` | UI inspector/debugger |
| Which Key | `which_key` | Keyboard shortcut guide popup |
| Miniprofiler | `miniprofiler_ui` | Performance profiling overlay |
| Debugger | `dap`, `debugger_ui`, `dap_adapters` | Debug Adapter Protocol |
| REPL | `repl` | Jupyter notebook support |
| Dev Container | `dev_container` | Container-based development |
| Web Search | `web_search`, `web_search_providers` | Web search integration |

## Initialization Order (main.rs)

All features are initialized in this order in `main.rs`:

```
1.  repl::init()
2.  recent_projects::init()
3.  dev_container::init()
4.  editor::init()
5.  image_viewer::init()
6.  repl::notebook::init()
7.  diagnostics::init()
8.  audio::init()
9.  workspace::init()
10. go_to_line::init()
11. file_finder::init()
12. tab_switcher::init()
13. outline::init()
14. project_symbols::init()
15. project_panel::init()
16. outline_panel::init()
17. tasks_ui::init()
18. snippets_ui::init()
19. channel::init()
20. search::init()
21. vim::init()
22. terminal_view::init()
23. journal::init()
24. encoding_selector::init()
25. language_selector::init()
26. line_ending_selector::init()
27. toolchain_selector::init()
28. theme_selector::init()
29. settings_profile_selector::init()
30. language_tools::init()
31. call::init()
32. notifications::init()
33. collab_ui::init()
34. gitbutler_panel::panel::init()  ← CUSTOM
35. git_graph::init()               ← CUSTOM/MODIFIED
36. feedback::init()
37. markdown_preview::init()
38. csv_preview::init()
39. svg_preview::init()
40. onboarding::init()
41. settings_ui::init()
42. keymap_editor::init()
43. extensions_ui::init()
44. edit_prediction::init()
45. inspector_ui::init()
46. json_schema_store::init()
47. miniprofiler_ui::init()
48. which_key::init()
```

## Fork Modifications from Zed

### Removed: `git_ui` crate
Zed's native git panel (`git_ui`) has been completely removed and replaced by `gitbutler_panel`. The following `clean_*.py` scripts document each removal point:

| Script | Target File | What was removed |
|--------|------------|-----------------|
| `clean_main.py` | `main.rs` | `git_ui::init(cx)` |
| `clean_git_panel.py` | `git_graph.rs` | `git_ui` references, git panel test |
| `clean_title_bar.py` | `title_bar.rs` | WorktreePicker and BranchPicker menus |
| `clean_commit_avatar.py` | `git_graph.rs` | CommitAvatar → plain colored div |
| `clean_commit_view.py` | `git_graph.rs` | CommitView open, OpenCommitView action |
| `clean_open_listener.py` | `open_listener.rs` | FileDiffView, MultiDiffView imports |
| `clean_project_panel.py` | `project_panel.rs` | FileDiffView references |
| `clean_vim.py` | `vim_test_context.rs` | `git_ui::init()` |

### Added: Custom Crates
- `gitbutler_panel` — Virtual branch management UI
- `gitbutler_store` — State management via but-* API
- `gitbutler_bridge` — Async/ObjectId helpers

### Modified: Existing Crates
- `git_graph` — Removed git_ui dependency, replaced CommitAvatar
- `title_bar` — Disabled native branch/worktree pickers (replaced by GitButler)
- `project_panel` — Removed FileDiffView integration
