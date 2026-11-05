use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

pub struct PreviewServerHandle {
    pub port: u16,
    pub out_dir: PathBuf,
    pub revision: Arc<AtomicU64>,
}

pub struct AppState {
    pub current_site: Mutex<Option<PathBuf>>,
    pub template_backups: Mutex<HashMap<String, Vec<String>>>,
    pub preview_server: Mutex<Option<PreviewServerHandle>>,
    pub watcher_handle: Mutex<Option<notify::RecommendedWatcher>>,
    pub preview_revision: Arc<AtomicU64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_site: Mutex::new(None),
            template_backups: Mutex::new(HashMap::new()),
            preview_server: Mutex::new(None),
            watcher_handle: Mutex::new(None),
            preview_revision: Arc::new(AtomicU64::new(1)),
        }
    }
}
