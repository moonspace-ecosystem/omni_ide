# GPUI Patterns Used in Omni IDE

Reference for the specific GPUI patterns and conventions used across the custom crates.

## Entity Types vs RenderOnce

Omni IDE uses two patterns for UI components:

### Entity (Stateful, Long-lived)
Used when a component needs to maintain state across renders or subscribe to events.

```rust
pub struct UnassignedChanges {
    store: Entity<GitButlerStore>,
    changes: Option<UiWorktreeChanges>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl Render for UnassignedChanges {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Can access &mut self, can use cx.listener(), cx.subscribe()
    }
}
```

**Used for**: `GitButlerPanel`, `UnassignedChanges`, `CommitDetailPanel`, `MultiDiffView`, `BranchCard`, `CommitModal`

### RenderOnce (Stateless, Transient)
Used for presentational components that are rebuilt every render cycle.

```rust
#[derive(IntoElement)]
pub struct StatusBar {
    branch_name: Option<SharedString>,
    changes_count: usize,
}

impl RenderOnce for StatusBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Takes ownership of self, receives &mut App (not Context)
    }
}
```

**Used for**: `Toolbar`, `StatusBar`, `StackLane`, `StackHeader`, `CommitButton`, `BranchHeader`, `CommitItem`, `WorktreeChangesSection`, `HunkDiff`, `UnifiedDiffView`

## Builder Pattern for RenderOnce

All stateless components use the builder pattern:

```rust
StatusBar::new()
    .branch_name("main")
    .changes_count(5)
    .loading("Syncing...")
```

## Action System

### Defining Actions

```rust
// Simple action (no data)
actions!(gitbutler_panel, [ToggleLeftPanel, ToggleRightPanel]);

// Action with data
#[derive(Clone, PartialEq, serde::Deserialize, schemars::JsonSchema, gpui::Action)]
#[action(namespace = gitbutler)]
pub struct Commit {
    pub branch_name: String,
}
```

### Dispatching Actions

From UI event handlers (when you don't have a Context<T>):
```rust
// In RenderOnce components
.on_click({
    let branch_name = self.branch_name.clone();
    move |_, window, cx| {
        window.dispatch_action(
            Commit { branch_name: branch_name.to_string() }.boxed_clone(),
            cx,
        );
    }
})
```

### Handling Actions

On panel's root element:
```rust
div()
    .id("gitbutler-panel")
    .track_focus(&self.focus_handle)  // CRITICAL: needed for action routing
    .on_action(cx.listener(|this, action: &Push, _window, cx| {
        this.store.update(cx, |store, cx| {
            store.push(&action.branch_name, cx);
        });
    }))
```

At workspace level (for modals):
```rust
workspace.register_action(
    |workspace, action: &Commit, window, cx| {
        let panel = workspace.panel::<GitButlerPanel>(cx);
        if let Some(panel) = panel {
            let store = panel.read(cx).store.clone();
            workspace.toggle_modal(window, cx, |window, cx| {
                CommitModal::new(branch_name, store, window, cx)
            });
        }
    },
);
```

## Event System

### Defining Events
```rust
pub enum GitButlerStoreEvent {
    WorkspaceChanged,
    WorktreeChangesUpdated,
    Error(String),
}

impl EventEmitter<GitButlerStoreEvent> for GitButlerStore {}
```

### Subscribing to Events
```rust
// In Entity constructor
let subscription = cx.subscribe(&store, |this: &mut Self, _store, event, cx| {
    match event {
        GitButlerStoreEvent::WorktreeChangesUpdated => {
            this.fetch_changes(&store, cx);
        }
        _ => {}
    }
});
// Store subscription to prevent it from being dropped
self._subscriptions.push(subscription);
```

## Async Operations Pattern

All Git operations in `GitButlerStore` follow this pattern:

```rust
pub fn some_operation(&mut self, cx: &mut gpui::Context<Self>) {
    // 1. Get sync context (needed to cross thread boundary)
    let Some(sync_ctx) = self.ctx.as_ref().map(|c| c.to_sync()) else {
        return;
    };
    
    // 2. Set loading state
    self.set_loading(LoadingState::Loading, cx);
    
    // 3. Spawn foreground task
    cx.spawn(async move |this, cx| {
        // 4. Do heavy work on background thread
        let result = cx.background_executor().spawn(async move {
            let mut ctx = Context::from(sync_ctx);
            // ... Git operation using but_api ...
        }).await;
        
        // 5. Update state on foreground thread
        this.update(cx, |this, cx| {
            match result {
                Ok(_) => this.set_loading(LoadingState::Idle, cx),
                Err(e) => {
                    this.set_loading(LoadingState::Error(e.to_string()), cx);
                    cx.emit(GitButlerStoreEvent::Error(e.to_string()));
                }
            }
            this.trigger_update(cx);
        }).ok();
    }).detach();
}
```

## Conditional Rendering

```rust
// Boolean condition
.when(self.left_panel_visible, |this| {
    this.child(left_panel_content)
})

// Option condition
.when_some(self.error_message, |this, error| {
    this.child(Label::new(error).color(Color::Error))
})
```

## Drag and Drop

### Making elements draggable
```rust
div()
    .id(unique_id)
    .on_drag(
        DragPayload::File(path.clone()),
        create_file_drag_preview,  // function that creates preview Entity
    )
```

### Making elements drop targets
```rust
div()
    .on_drop(|payload: &DragPayload, _window, cx| {
        if let DragPayload::File(path) = payload {
            // Handle drop
        }
    })
```

## Testing Patterns

### Test Setup
```rust
fn init_test(cx: &mut TestAppContext) {
    cx.update(|cx| {
        let app_state = AppState::test(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        editor::init(cx);
        workspace::init(app_state, cx);
    });
}
```

### Creating Test Workspace
```rust
let fs = FakeFs::new(cx.executor());  // NOT RealFs
fs.insert_tree("/root", json!({
    ".git": {},
    "file1.txt": "content",
})).await;

let project = Project::test(fs, ["/root".as_ref()], cx).await;
let window = cx.add_window(|window, cx| {
    MultiWorkspace::test_new(project, window, cx)
});
let workspace = window.read_with(cx, |mw, _| mw.workspace().clone()).unwrap();
let cx = &mut VisualTestContext::from_window(window.into(), cx);
```

### Injecting Mock Data
```rust
panel.update(cx, |p, cx| {
    p.store.update(cx, |store, cx| {
        store.set_worktree_changes(mock_changes, cx);  // requires test-support feature
    });
});
cx.run_until_parked();
```

## Theme Colors Used

| Purpose | Token |
|---------|-------|
| Panel background | `cx.theme().colors().panel_background` |
| Editor background | `cx.theme().colors().editor_background` |
| Elevated surface | `cx.theme().colors().elevated_surface_background` |
| Surface | `cx.theme().colors().surface_background` |
| Border | `cx.theme().colors().border` |
| Border variant | `cx.theme().colors().border_variant` |
| Border focused | `cx.theme().colors().border_focused` |
| Element hover | `cx.theme().colors().element_hover` |
| Element active | `cx.theme().colors().element_active` |
| Element selected | `cx.theme().colors().element_selected` |
| Drop target bg | `cx.theme().colors().drop_target_background` |
| Diff added bg | `cx.theme().status().created_background` |
| Diff removed bg | `cx.theme().status().deleted_background` |
| Diff added text | `cx.theme().status().created` |
| Diff removed text | `cx.theme().status().deleted` |
