use std::path::{Path, PathBuf};
use std::time::Duration;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::Sender;

use crate::simulator::ConfigChange;

/// How long to wait for a burst of events (one editor save emits several) to settle
/// before reloading once.
pub const RELOAD_DEBOUNCE: Duration = Duration::from_millis(250);

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
                    if let Some(change) = classify_event(&event.kind, event.paths.first()) {
                        let _ = tx.blocking_send(change);
                    }
                }
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;
        watcher.watch(&path, RecursiveMode::Recursive)?;
        Ok(Self { _watcher: watcher })
    }
}

/// Maps a filesystem event to a configuration change, or `None` if it is not one.
///
/// Access events are ignored: on Linux, opening and closing a file without writing
/// is reported as `Access`, and reloading reads every YAML file, so treating reads
/// as modifications makes each reload trigger the next one.
fn classify_event(kind: &EventKind, path: Option<&PathBuf>) -> Option<ConfigChange> {
    let path = path?;
    if !is_service_file(path) {
        return None;
    }
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();
    match kind {
        EventKind::Create(_) => Some(ConfigChange::ServiceAdded(name)),
        EventKind::Remove(_) => Some(ConfigChange::ServiceRemoved(name)),
        EventKind::Modify(_) => Some(ConfigChange::ServiceModified(name)),
        EventKind::Access(_) | EventKind::Any | EventKind::Other => None,
    }
}

fn is_service_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{AccessKind, AccessMode, CreateKind, DataChange, ModifyKind, RemoveKind};
    use tokio::sync::mpsc;

    fn yaml() -> PathBuf {
        PathBuf::from("/services/hiring-lab.yaml")
    }

    #[test]
    fn access_events_are_not_changes() {
        for kind in [
            EventKind::Access(AccessKind::Open(AccessMode::Any)),
            EventKind::Access(AccessKind::Close(AccessMode::Read)),
            EventKind::Access(AccessKind::Close(AccessMode::Write)),
            EventKind::Access(AccessKind::Read),
            EventKind::Any,
            EventKind::Other,
        ] {
            assert_eq!(classify_event(&kind, Some(&yaml())), None, "{kind:?}");
        }
    }

    #[test]
    fn create_modify_remove_map_to_changes() {
        let name = || "hiring-lab".to_string();
        assert_eq!(
            classify_event(&EventKind::Create(CreateKind::File), Some(&yaml())),
            Some(ConfigChange::ServiceAdded(name()))
        );
        assert_eq!(
            classify_event(
                &EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                Some(&yaml())
            ),
            Some(ConfigChange::ServiceModified(name()))
        );
        assert_eq!(
            classify_event(&EventKind::Remove(RemoveKind::File), Some(&yaml())),
            Some(ConfigChange::ServiceRemoved(name()))
        );
    }

    #[test]
    fn non_yaml_files_are_ignored() {
        let kind = EventKind::Modify(ModifyKind::Data(DataChange::Content));
        assert_eq!(
            classify_event(&kind, Some(&PathBuf::from("/services/apicentric.db"))),
            None
        );
        assert_eq!(classify_event(&kind, None), None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn reading_a_service_file_does_not_report_a_change() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("hiring-lab.yaml");
        std::fs::write(&file, "name: hiring-lab\n").unwrap();

        let (tx, mut rx) = mpsc::channel(64);
        let _watcher = ConfigWatcher::new(dir.path().to_path_buf(), tx).unwrap();

        // Some backends (FSEvents on macOS) report the setup write after the watcher
        // starts; let those arrive and discard them.
        tokio::time::sleep(Duration::from_secs(1)).await;
        while rx.try_recv().is_ok() {}

        // What a reload does: open and read every service file.
        for _ in 0..20 {
            let _ = std::fs::read_to_string(&file).unwrap();
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        if let Ok(change) = rx.try_recv() {
            panic!("reads were reported as a change: {change:?}");
        }

        // A real write is still reported (as Added or Modified, depending on the backend).
        std::fs::write(&file, "name: hiring-lab\nversion: 2\n").unwrap();
        let change = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("no change reported for a write");
        assert!(
            matches!(
                change,
                Some(ConfigChange::ServiceModified(_) | ConfigChange::ServiceAdded(_))
            ),
            "{change:?}"
        );
    }
}
