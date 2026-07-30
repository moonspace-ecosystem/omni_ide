use gpui::{App, Global};
use omni_kit::db::surreal::SurrealClient;
use omni_kit::ConfigLoader;
use std::sync::Arc;

pub struct GlobalOmniClient(pub Arc<SurrealClient>);

impl Global for GlobalOmniClient {}

pub fn init_global(client: Arc<SurrealClient>, cx: &mut App) {
    cx.set_global(GlobalOmniClient(client));
}

pub fn load_and_init(cx: &mut App) {
    // Attempt to load Omni config
    if let Ok(config) = ConfigLoader::load(None) {
        if let Ok(client) = SurrealClient::new(config.memory) {
            init_global(Arc::new(client), cx);
            log::info!("OmniKit SurrealClient initialized successfully.");
        } else {
            log::warn!("Failed to initialize OmniKit SurrealClient.");
        }
    } else {
        log::warn!("Failed to load OmniKit configuration.");
    }
}

pub fn get_client(cx: &App) -> Option<Arc<SurrealClient>> {
    if cx.has_global::<GlobalOmniClient>() {
        Some(cx.global::<GlobalOmniClient>().0.clone())
    } else {
        None
    }
}
