use axum::{response::Html, routing::get, Router};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebounceEventResult};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::data;
use crate::html;
use crate::types::HomeworkEntry;

/// Application state shared across requests
pub struct AppState {
    pub entries: RwLock<Vec<HomeworkEntry>>,
    pub output_dir: PathBuf,
}

/// Start the web server with file watching
pub async fn serve(port: u16, output_dir: PathBuf) -> anyhow::Result<()> {
    // Process data on startup
    println!("Scanning data directory...");
    let entries = data::process_all_exports(&output_dir)?;

    let state = Arc::new(AppState {
        entries: RwLock::new(entries),
        output_dir: output_dir.clone(),
    });

    // Start file watcher
    let watcher_state = state.clone();
    start_file_watcher(watcher_state)?;

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/entries", get(entries_handler))
        .route("/api/refresh", get(refresh_handler))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("\nServer running at http://{}", addr);
    println!("Watching data/ for changes...");
    println!("Press Ctrl+C to stop\n");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Start watching the data directory for changes
fn start_file_watcher(state: Arc<AppState>) -> anyhow::Result<()> {
    let data_dir = PathBuf::from("data");

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
        println!("Created data/ directory");
    }

    // Create a channel to receive events
    let (tx, mut rx) = tokio::sync::mpsc::channel(10);

    // Spawn a blocking task for the file watcher
    let watch_dir = data_dir.clone();
    std::thread::spawn(move || {
        let tx_clone = tx.clone();
        let mut debouncer = new_debouncer(
            Duration::from_secs(2),
            move |result: DebounceEventResult| {
                if let Ok(events) = result {
                    // Check if any event is for an export file
                    let has_export = events.iter().any(|e| {
                        e.path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.starts_with("export_") && n.contains(".xls"))
                            .unwrap_or(false)
                    });

                    if has_export {
                        let _ = tx_clone.blocking_send(());
                    }
                }
            },
        )
        .expect("Failed to create debouncer");

        debouncer
            .watcher()
            .watch(&watch_dir, RecursiveMode::NonRecursive)
            .expect("Failed to watch directory");

        // Keep the watcher alive
        loop {
            std::thread::sleep(Duration::from_secs(60));
        }
    });

    // Spawn a task to handle file change notifications
    tokio::spawn(async move {
        while rx.recv().await.is_some() {
            println!("\nDetected changes in data/...");
            match data::process_all_exports(&state.output_dir) {
                Ok(new_entries) => {
                    let mut entries = state.entries.write().await;
                    let old_count = entries.len();
                    *entries = new_entries;
                    let new_count = entries.len();
                    if new_count != old_count {
                        println!(
                            "Updated: {} entries ({:+})",
                            new_count,
                            new_count as i64 - old_count as i64
                        );
                    } else {
                        println!("No new entries found");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to refresh: {}", e);
                }
            }
        }
    });

    Ok(())
}

/// Serve the main HTML page
async fn index_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> Html<String> {
    let entries = state.entries.read().await;
    let markup = html::render_page(&entries);
    Html(markup.into_string())
}

/// Return entries as JSON
async fn entries_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::Json<Vec<HomeworkEntry>> {
    let entries = state.entries.read().await;
    axum::Json(entries.clone())
}

/// Refresh data from disk (manual trigger)
async fn refresh_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> &'static str {
    println!("\nManual refresh triggered...");

    match data::process_all_exports(&state.output_dir) {
        Ok(new_entries) => {
            let mut entries = state.entries.write().await;
            *entries = new_entries;
            "OK"
        }
        Err(e) => {
            eprintln!("Refresh failed: {}", e);
            "ERROR"
        }
    }
}
