use std::path::PathBuf;
use std::time::Duration;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::Sender;

use crate::simulator::ConfigChange;

/// Watches the services directory for configuration changes
pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    /// Create a new configuration watcher for the given path
    pub fn new(path: PathBuf, tx: Sender<ConfigChange>) -> notify::Result<Self> {
        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    if let Some(p) = event.paths.first() {
                        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                            if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
                                let name = p
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                let change = match event.kind {
                                    EventKind::Create(_) => ConfigChange::ServiceAdded(name),
                                    EventKind::Remove(_) => ConfigChange::ServiceRemoved(name),
                                    _ => ConfigChange::ServiceModified(name),
                                };
                                let _ = tx.blocking_send(change);
                            }
                        }
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;
        watcher.watch(&path, RecursiveMode::Recursive)?;
        Ok(Self { _watcher: watcher })
    }
}
