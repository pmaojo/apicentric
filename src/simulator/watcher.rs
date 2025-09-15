use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::errors::PulseResult;
use crate::simulator::ConfigChange;

/// Trait for components that react to configuration changes.
#[async_trait]
pub trait ConfigChangeHandler: Send + Sync {
    /// Handle a configuration change event.
    async fn on_config_change(&self, change: ConfigChange) -> PulseResult<()>;
}

/// Watches the services directory for configuration changes
pub struct ConfigWatcher {
    _watcher: RecommendedWatcher,
}

impl ConfigWatcher {
    /// Create a new configuration watcher for the given path
    pub fn new(path: PathBuf, handler: Arc<dyn ConfigChangeHandler>) -> notify::Result<Self> {
        let handle = tokio::runtime::Handle::current();
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
                                let handler = handler.clone();
                                let handle = handle.clone();
                                handle.spawn(async move {
                                    if let Err(e) = handler.on_config_change(change).await {
                                        eprintln!("Error handling config change: {}", e);
                                    }
                                });
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
