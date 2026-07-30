# UI Component Catalog

All custom UI components in Omni IDE, organized by crate and purpose.

## gitbutler_panel (15 files, ~2,500 LoC)

### Core Layout

#### GitButlerPanel — [panel.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/panel.rs)
- **Type**: `Entity<GitButlerPanel>` (implements `Render`, `Panel`, `Focusable`)
- **Position**: Bottom Dock
- **Layout**: `flex-col` → Toolbar → Main Viewport (flex-row: Left | Center | Right) → StatusBar
- **Fields**: `store`, `unassigned_changes`, `commit_detail`, `left_panel_visible`, `right_panel_visible`
- **Key methods**: `build_stack_lanes()`, `toggle_left_panel()`, `toggle_right_panel()`

### Toolbar & Status

#### Toolbar — [toolbar.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/toolbar.rs)
- **Type**: `RenderOnce` (stateless)
- **Layout**: `h-flex` with left group (toggle-left, "GitButler" label, spinner) and right group (fetch, new-branch, toggle-right)
- **Buttons**: `toggle-left-panel`, `fetch-btn`, `new-branch-btn`, `toggle-right-panel`
- **Builder pattern**: `.on_fetch()`, `.on_toggle_left()`, `.on_toggle_right()`, `.loading(bool)`

#### StatusBar — [status_bar.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/status_bar.rs)
- **Type**: `RenderOnce` (stateless)
- **Layout**: `h-flex`, height 24px, border-top
- **Shows**: Error (red), Loading spinner, Branch name, Changes count, Commits ahead
- **Builder pattern**: `.branch_name()`, `.changes_count()`, `.error()`, `.loading()`

### Stack View (Center Panel)

#### StackLane — [stack_lane.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/stack_lane.rs#L197-L263)
- **Type**: `RenderOnce` (stateless)
- **Layout**: `v-flex`, width 280px, min-width 240px, full height, right border
- **Children**: StackHeader → WorktreeChangesSection → BranchList → CommitButton
- **DnD**: `.on_drop()` for receiving dragged files → `store.assign_hunk()`

#### StackHeader — [stack_lane.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/stack_lane.rs#L7-L67)
- **Type**: `RenderOnce` (stateless)
- **Layout**: `h-flex`, branch icon + bold name, fold/unfold button + menu button
- **Colors**: panel_background, border-bottom

#### WorktreeChangesSection — [stack_lane.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/stack_lane.rs#L69-L130)
- **Type**: `RenderOnce` (stateless)
- **Shows**: Files assigned to this branch (from `UiHunkAssignment`)
- **DnD**: Each file is draggable with `DragPayload::File`

#### CommitButton — [stack_lane.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/stack_lane.rs#L132-L169)
- **Type**: `RenderOnce` (stateless)
- **Style**: `ButtonStyle::Tinted(TintColor::Accent)`, `ButtonSize::Default`, full-width
- **Action**: Dispatches `Commit { branch_name }` via `window.dispatch_action()`

#### BranchList — [stack_lane.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/stack_lane.rs#L171-L195)
- **Type**: `RenderOnce` (stateless)
- **Layout**: `v-flex`, scrollable, contains `BranchCard` entity

### Branch Cards

#### BranchCard — [branch_card.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/branch_card.rs#L8-L72)
- **Type**: `Entity<BranchCard>` (implements `Render`, `EventEmitter<BranchCardEvent>`)
- **Fields**: `segment` (UiSegment), `branch_name`, `selected_commit_id`, `is_collapsed`
- **Children**: BranchHeader + CommitItem list
- **Events**: Emits `BranchCardEvent::CommitSelected(UiCommit)` on click

#### BranchHeader — [branch_card.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/branch_card.rs#L74-L171)
- **Type**: `RenderOnce` (stateless)
- **Layout**: `h-flex`, branch name + commit count + PR badge + Push button
- **PR badge**: Button with `#{number}`, opens URL on click
- **Push button**: Dispatches `Push { branch_name }`

#### CommitItem — [branch_card.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/branch_card.rs#L173-L261)
- **Type**: `RenderOnce` (stateless)
- **Layout**: `h-flex`, status dot (3px colored bar) + message + metadata (id, author, time)
- **Status colors**: Local=info, Pushed=success, Integrated=hint, Conflict=conflict
- **States**: Selected (element_selected bg), Hover, Active
- **Conflict indicator**: Warning icon when `is_conflicted`

### Left Panel

#### UnassignedChanges — [unassigned_changes.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/unassigned_changes.rs)
- **Type**: `Entity<UnassignedChanges>` (implements `Render`)
- **Fields**: `store`, `changes`, `unassigned_assignments`, `selected_all`, `selected_files`
- **Header**: "Changes" label + count + Stash button
- **Body**: "Select All" checkbox + file list with status icons
- **DnD**: Each file row is draggable with `DragPayload::File`
- **File status icons**: Added=Plus(success), Modified=Pencil(modified), Deleted=Dash(deleted), Renamed=ArrowRight(info)
- **Subscribes to**: `GitButlerStoreEvent::WorktreeChangesUpdated`

### Right Panel

#### CommitDetailPanel — [commit_detail_panel.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/commit_detail_panel.rs)
- **Type**: `Entity<CommitDetailPanel>` (implements `Render`)
- **Fields**: `selected_commit`, `diff_view`
- **Header**: Commit message, author, id, timestamp, close button
- **Body**: `MultiDiffView` showing file diffs
- **Empty state**: "Select a commit to view details"

#### MultiDiffView — [diff_viewer.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/diff_viewer.rs#L134-L212)
- **Type**: `Entity<MultiDiffView>` (implements `Render`)
- **States**: Empty, Loading, Loaded(CommitDetails), Error
- **Loaded view**: List of changed files with status

#### UnifiedDiffView — [diff_viewer.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/diff_viewer.rs#L91-L125)
- **Type**: `RenderOnce` (stateless)
- **Layout**: File path header + hunk sections with line numbers

#### HunkDiff — [diff_viewer.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/diff_viewer.rs#L19-L78)
- **Type**: `RenderOnce` (stateless)
- **Layout**: Line-by-line diff with old/new line numbers + colored content
- **Colors**: Added=created_background, Removed=deleted_background

### Modals

#### CommitModal — [actions.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/actions.rs#L42-L175)
- **Type**: `Entity<CommitModal>` (implements `Render`, `ModalView`, `Focusable`)
- **Layout**: v-flex, width 384px, title + editor + Cancel/Commit buttons
- **Fields**: `text` (Editor entity), `branch_name`, `store`
- **Registered at**: `zed.rs:register_actions` via `CommitModal::register(workspace)`

### Supporting

#### DnD System — [dnd.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/dnd.rs)
- `DragPayload`: Hunk(String), Commit(String), File(String)
- `FileDragPreview`: Rendered during file drag
- `DropZoneIndicator`: Visual feedback for drop targets
- `InsertionIndicator`: Blue line showing insertion point

#### Colors — [gitbutler_colors.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/gitbutler_colors.rs)
- Trait `GitButlerColors` extends `Theme`
- Maps GitButler concepts to theme status colors
- `gitbutler_local_commit` → status.info (blue)
- `gitbutler_pushed_commit` → status.success (green)
- `gitbutler_integrated_commit` → status.hint (gray)
- `gitbutler_conflict` → status.conflict (red)

#### GutterBridge — [gutter_bridge.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/gutter_bridge.rs)
- Bridges GitButler hunk data to Zed editor gutter
- Caches hunks by file path
- Updates on `WorktreeChangesUpdated` events

### Data Models — [models.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/models.rs)

| Model | Fields | Source |
|-------|--------|--------|
| `UiWorkspace` | stacks: Vec<UiStack> | `but_workspace::RefInfo` |
| `UiStack` | segments: Vec<UiSegment> | `but_workspace::branch::Stack` |
| `UiSegment` | commits, branch_name, pr_number, pr_url | `but_workspace::ref_info::Segment` |
| `UiCommit` | id, message, author, email, timestamp, flags, is_conflicted | `but_workspace::ref_info::Commit` |
| `UiWorktreeChanges` | changes: Vec<UiTreeChange>, assignments | `but_hunk_assignment::WorktreeChanges` |
| `UiTreeChange` | path, status (FileStatus) | `but_core::ui::TreeChange` |
| `UiHunkAssignment` | path, branch_ref_bytes | `but_hunk_assignment::HunkAssignment` |
| `FileStatus` | Added, Modified, Deleted, Renamed | `but_core::ui::TreeStatus` |

## git_graph (1 file, ~6,135 LoC)

#### GitGraph — [git_graph.rs](file:///Users/mike/Documents/omni_ide/crates/git_graph/src/git_graph.rs)
- **Type**: Full `Item` implementation (Tab in workspace)
- **Features**: Commit table, graph lines, search, column resize, context menus
- **Integration**: Uses `project::git_store::GitStore` (Zed's native git)
- **Independent**: Does not depend on `gitbutler_store` or `gitbutler_panel`

### Actions (defined in [actions.rs](file:///Users/mike/Documents/omni_ide/crates/gitbutler_panel/src/actions.rs))

| Action | Namespace | Data | Handler Location |
|--------|-----------|------|-----------------|
| `Commit` | gitbutler | `branch_name: String` | `CommitModal::register` (workspace) |
| `Push` | gitbutler | `branch_name: String` | `CommitModal::register` (workspace) + `panel.rs` on_action |
| `Fetch` | gitbutler | (none) | `CommitModal::register` (workspace) |
| `CreateBranch` | gitbutler | (none) | **Not yet implemented** |
| `StashChanges` | gitbutler | (none) | **Not yet implemented** (stash via cx.listener in unassigned_changes) |
| `ToggleLeftPanel` | gitbutler_panel | (none) | `panel.rs` on_action |
| `ToggleRightPanel` | gitbutler_panel | (none) | `panel.rs` on_action |
| `FocusSearch` | gitbutler_panel | (none) | **Not yet implemented** |
| `ToggleFocus` | gitbutler_panel | (none) | `panel.rs::init()` |
