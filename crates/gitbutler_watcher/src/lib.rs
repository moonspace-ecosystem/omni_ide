use anyhow::Result;
use gpui::{App, AppContext, Context, Entity, EventEmitter};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher as _};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

pub enum WatcherEvent {
    Changed,
}

pub struct GitButlerWatcher {
    _watcher: RecommendedWatcher,
    _task: gpui::Task<()>,
}

impl EventEmitter<WatcherEvent> for GitButlerWatcher {}

impl GitButlerWatcher {
    pub fn new(path: PathBuf, cx: &mut App) -> Result<Entity<Self>> {
        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Ignore access events
                    if !matches!(event.kind, notify::EventKind::Access(_)) {
                        let _ = tx.send(());
                    }
                }
            },
            Config::default(),
        )?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        Ok(cx.new(|cx| {
            let task = cx.spawn(|this, mut cx: gpui::AsyncApp| async move {
                let mut debounce_timer = None;
                loop {
                    // Check for events every 50ms
                    cx.background_executor().timer(Duration::from_millis(50)).await;
                    
                    let mut has_event = false;
                    while let Ok(()) = rx.try_recv() {
                        has_event = true;
                    }

                    if has_event {
                        debounce_timer = Some(Duration::from_millis(300));
                    }

                    if let Some(timer) = debounce_timer {
                        if timer.is_zero() {
                            let _ = this.update(&mut cx, |_, cx| {
                                cx.emit(WatcherEvent::Changed);
                            });
                            debounce_timer = None;
                        } else {
                            debounce_timer = Some(timer.saturating_sub(Duration::from_millis(50)));
                        }
                    }
                }
            });

            Self {
                _watcher: watcher,
                _task: task,
            }
        }))
    }
}
