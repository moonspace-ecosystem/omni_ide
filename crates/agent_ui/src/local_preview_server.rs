use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// In-memory file system for storing generated UI code before serving it.
/// This avoids writing temporary files to disk during prototyping.
#[derive(Default, Clone)]
pub struct VirtualFileSystem {
    files: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        let vfs = Self {
            files: Arc::new(Mutex::new(HashMap::new())),
        };
        vfs.seed_default_files();
        vfs
    }

    fn seed_default_files(&self) {
        self.write_file(
            "index.html".to_string(),
            include_str!("preview_template.html").as_bytes().to_vec(),
        );
    }

    pub fn write_file(&self, path: String, content: Vec<u8>) {
        if let Ok(mut files) = self.files.lock() {
            files.insert(path, content);
        }
    }

    pub fn read_file(&self, path: &str) -> Option<Vec<u8>> {
        self.files.lock().ok()?.get(path).cloned()
    }

    pub fn update_component(&self, component_code: &str) {
        let html = Self::wrap_in_html(component_code);
        self.write_file("index.html".to_string(), html.into_bytes());
    }

    fn wrap_in_html(component_code: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Omni Design Preview</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        body {{ margin: 0; font-family: 'Inter', system-ui, sans-serif; background: #1a1a2e; color: #eee; }}
    </style>
</head>
<body>
    <div id="root">{}</div>
    <script>
        const ws = new WebSocket(`ws://${{window.location.host}}/ws`);
        ws.onmessage = (event) => {{
            const data = JSON.parse(event.data);
            if (data.type === 'hmr') {{
                document.getElementById('root').innerHTML = data.html;
            }}
        }};
    </script>
</body>
</html>"#,
            component_code
        )
    }
}

/// Configuration for the Local Preview Server.
pub struct PreviewServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for PreviewServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 0,
        }
    }
}

/// Manages the lifecycle of the local HTTP + WebSocket server
/// that serves generated UI previews to the WebView.
pub struct LocalPreviewServer {
    pub config: PreviewServerConfig,
    pub vfs: VirtualFileSystem,
    pub bound_addr: Option<SocketAddr>,
}

impl LocalPreviewServer {
    pub fn new(config: PreviewServerConfig) -> Self {
        Self {
            config,
            vfs: VirtualFileSystem::new(),
            bound_addr: None,
        }
    }

    pub fn url(&self) -> Option<String> {
        self.bound_addr
            .map(|addr| format!("http://{}", addr))
    }

    pub fn push_update(&self, component_html: &str) {
        self.vfs.update_component(component_html);
    }

    pub fn content_type_for(path: &str) -> &'static str {
        if path.ends_with(".html") {
            "text/html; charset=utf-8"
        } else if path.ends_with(".js") {
            "application/javascript; charset=utf-8"
        } else if path.ends_with(".css") {
            "text/css; charset=utf-8"
        } else if path.ends_with(".json") {
            "application/json; charset=utf-8"
        } else {
            "application/octet-stream"
        }
    }
}
