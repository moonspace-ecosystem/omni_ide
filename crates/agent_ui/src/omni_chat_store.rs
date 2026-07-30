use std::path::PathBuf;
use std::sync::Arc;
use anyhow::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

/// Metadata for a single context source (URL, PDF, etc.) indexed by the Local RAG pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSource {
    pub id: String,
    pub source_type: ContextSourceType,
    pub uri: String,
    pub title: String,
    pub indexed_at: u128,
    pub chunk_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContextSourceType {
    Web,
    Pdf,
    File,
    Text,
}

/// Persistence layer for Omni Chat sessions built on top of local JSON files.
/// Stored under `<workspace>/.omni/chat_sessions/` to ensure local-first privacy.
pub struct ChatSessionStore {
    workspace_root: PathBuf,
    cache: Mutex<Vec<ChatSessionMetadata>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionMetadata {
    pub id: String,
    pub title: String,
    pub created_at: u128,
    pub updated_at: u128,
    pub context_sources: Vec<ContextSource>,
}

impl ChatSessionStore {
    pub fn new(workspace_root: PathBuf) -> Self {
        let sessions_dir = workspace_root.join(".omni").join("chat_sessions");
        let cache = Mutex::new(Vec::new());
        let store = Self {
            workspace_root: sessions_dir,
            cache,
        };
        let _ = store.discover_sessions();
        store
    }

    fn discover_sessions(&self) -> Result<()> {
        std::fs::create_dir_all(&self.workspace_root)?;
        let mut cache = self.cache.lock();
        cache.clear();

        for entry in std::fs::read_dir(&self.workspace_root)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<ChatSessionMetadata>(&content) {
                        cache.push(session);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn list_sessions(&self) -> Vec<ChatSessionMetadata> {
        self.cache.lock().clone()
    }

    pub fn save_session(&self, session: &ChatSessionMetadata) -> Result<()> {
        std::fs::create_dir_all(&self.workspace_root)?;
        let path = self.workspace_root.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)?;
        std::fs::write(path, content)?;

        let mut cache = self.cache.lock();
        cache.retain(|s| s.id != session.id);
        cache.push(session.clone());
        Ok(())
    }
}

pub type SharedChatSessionStore = Arc<ChatSessionStore>;
