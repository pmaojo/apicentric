use crate::domain::errors::{PulseError, PulseResult};
use async_trait::async_trait;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver};

use crate::domain::ports::testing::{WatchEvent, WatcherPort};

pub struct FileWatcher {
    debounce_ms: u64,
}

impl FileWatcher {
    pub fn new(debounce_ms: u64) -> Self {
        Self { debounce_ms }
    }
}

#[async_trait]
impl WatcherPort for FileWatcher {
    async fn watch(&self, path: &str) -> PulseResult<Receiver<WatchEvent>> {
        let (tx, rx) = mpsc::channel(100);
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let kind = match event.kind {
                        EventKind::Create(_) => WatchEvent::Created,
                        EventKind::Modify(_) => WatchEvent::Modified,
                        EventKind::Remove(_) => WatchEvent::Removed,
                        _ => WatchEvent::Other,
                    };
                    let _ = tx.blocking_send(kind);
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(self.debounce_ms)),
        )
        .map_err(|e| {
            PulseError::Fs(format!("Failed to create file watcher: {}", e))
        })?;

        watcher
            .watch(Path::new(path), RecursiveMode::Recursive)
            .map_err(|e| {
                PulseError::Fs(format!("Failed to watch directory {}: {}", path, e))
            })?;

        // Mantener vivo el watcher
        Box::leak(Box::new(watcher));

        Ok(rx)
    }
}
