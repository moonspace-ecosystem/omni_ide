// Will use gpui_tokio to bridge between Zed's smol runtime
// and GitButler's tokio runtime.
//
// The general pattern will be:
//   1. Zed spawns work via cx.spawn() / cx.background_spawn() (smol-based)
//   2. When calling GitButler APIs that need tokio, we use gpui_tokio
//      to run those futures on a tokio runtime
//   3. Results flow back to the smol-based GPUI context
//
// Placeholder until we can add gpui_tokio dependency.

use anyhow::Result;
use gpui::{AppContext, Task};
use std::future::Future;

/// Initializes the GPUI Tokio bridge. Must be called once during app startup.
pub fn init_tokio(cx: &mut gpui::App) {
    gpui_tokio::init(cx);
}

/// Run an async operation on the global Tokio runtime and return a GPUI Task.
/// This allows bridging between Zed's smol-based GPUI context and GitButler's 
/// tokio-based async APIs.
pub fn spawn_on_tokio<C, F, R>(cx: &C, future: F) -> Task<Result<R>>
where
    C: AppContext,
    F: Future<Output = Result<R>> + Send + 'static,
    R: Send + 'static,
{
    gpui_tokio::Tokio::spawn_result(cx, future)
}

/// Run a synchronous (blocking) GitButler operation on a background thread.
pub fn run_blocking<F, T>(operation: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    operation()
}
