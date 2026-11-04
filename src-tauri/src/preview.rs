use crate::builder::build_site_pipeline;
use axum::{extract::State, routing::get, Router};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub async fn start_axum_preview_server(
    out_dir: PathBuf,
    revision: Arc<AtomicU64>,
) -> Result<u16, String> {
    let rev_for_route = revision.clone();
    let serve_dir = ServeDir::new(&out_dir).append_index_html_on_directories(true);

    let app = Router::new()
        .route(
            "/__pico_livereload",
            get(move |State(rev): State<Arc<AtomicU64>>| async move {
                rev.load(Ordering::SeqCst).to_string()
            }),
        )
        .with_state(rev_for_route)
        .fallback_service(serve_dir)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .map_err(|e| format!("Could not bind local preview server: {}", e))?;

    let port = listener
        .local_addr()
        .map_err(|e| format!("Could not read local preview port: {}", e))?
        .port();

    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });

    Ok(port)
}

/// Starts a file watcher on `content/`, `templates/`, and `assets/` with a 300ms debounce (§3.5).
/// On change, runs `build_site_pipeline(site_root, true, true)` and increments `revision`.
pub fn start_site_watcher(
    site_root: PathBuf,
    revision: Arc<AtomicU64>,
) -> Result<RecommendedWatcher, String> {
    let (tx, rx) = channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())
        .map_err(|e| format!("Failed to initialize file watcher: {}", e))?;

    for subdir in ["content", "templates", "assets"] {
        let dir = site_root.join(subdir);
        if dir.is_dir() {
            let _ = watcher.watch(&dir, RecursiveMode::Recursive);
        }
    }
    let site_yaml = site_root.join("site.yaml");
    if site_yaml.is_file() {
        let _ = watcher.watch(&site_yaml, RecursiveMode::NonRecursive);
    }

    std::thread::spawn(move || {
        let debounce = Duration::from_millis(300);
        let mut last_event: Option<Instant> = None;

        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    // Ignore backup files or hidden files
                    let ignored = event.paths.iter().all(|p: &PathBuf| {
                        let s = p.to_string_lossy();
                        s.contains("out")
                            || s.contains(".pico_history")
                            || s.ends_with(".bak")
                            || s.ends_with('~')
                    });
                    if !ignored {
                        last_event = Some(Instant::now());
                    }
                }
                Ok(Err(_)) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(ts) = last_event {
                        if ts.elapsed() >= debounce {
                            last_event = None;
                            let res = build_site_pipeline(&site_root, true, true);
                            if res.success {
                                revision.fetch_add(1, Ordering::SeqCst);
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    });

    Ok(watcher)
}
