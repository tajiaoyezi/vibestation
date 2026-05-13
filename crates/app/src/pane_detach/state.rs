use std::collections::HashMap;
use std::sync::Mutex;
use std::time::SystemTime;

use thiserror::Error;
use vibestation_core::{DetachedWindowBounds, PaneDetachListEntry};

pub type PaneId = String;

#[derive(Debug, Clone)]
pub struct DetachedWindowInfo {
    pub window_label: String,
    pub workspace_id: String,
    pub bounds: DetachedWindowBounds,
    pub created_at: SystemTime,
}

impl DetachedWindowInfo {
    pub fn new(
        window_label: impl Into<String>,
        workspace_id: impl Into<String>,
        bounds: DetachedWindowBounds,
    ) -> Self {
        Self {
            window_label: window_label.into(),
            workspace_id: workspace_id.into(),
            bounds,
            created_at: SystemTime::now(),
        }
    }
}

#[derive(Debug, Error)]
pub enum DetachError {
    #[error("Pane {0} already detached")]
    AlreadyDetached(PaneId),
    #[error("Lock poisoned")]
    LockPoisoned,
}

pub struct DetachedPaneMap {
    inner: Mutex<HashMap<PaneId, DetachedWindowInfo>>,
}

impl DetachedPaneMap {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }

    pub fn insert(&self, pane_id: PaneId, info: DetachedWindowInfo) -> Result<(), DetachError> {
        let mut map = self.inner.lock().map_err(|_| DetachError::LockPoisoned)?;
        if map.contains_key(&pane_id) {
            return Err(DetachError::AlreadyDetached(pane_id));
        }
        map.insert(pane_id, info);
        Ok(())
    }

    pub fn remove(&self, pane_id: &PaneId) -> Option<DetachedWindowInfo> {
        self.lock_recovering().remove(pane_id)
    }

    pub fn remove_by_label(&self, window_label: &str) -> Option<(PaneId, DetachedWindowInfo)> {
        let mut map = self.lock_recovering();
        let pane_id = map
            .iter()
            .find(|(_, info)| info.window_label == window_label)
            .map(|(pane_id, _)| pane_id.clone())?;
        let info = map.remove(&pane_id)?;
        Some((pane_id, info))
    }

    pub fn get(&self, pane_id: &PaneId) -> Option<DetachedWindowInfo> {
        self.lock_recovering().get(pane_id).cloned()
    }

    pub fn list(&self) -> Vec<(PaneId, DetachedWindowInfo)> {
        self.lock_recovering()
            .iter()
            .map(|(pane_id, info)| (pane_id.clone(), info.clone()))
            .collect()
    }

    pub fn clear(&self) -> Result<(), DetachError> {
        self.inner
            .lock()
            .map_err(|_| DetachError::LockPoisoned)?
            .clear();
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.lock_recovering().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn list_entries(&self) -> Vec<PaneDetachListEntry> {
        let mut entries: Vec<PaneDetachListEntry> = self
            .list()
            .into_iter()
            .map(|(pane_id, info)| PaneDetachListEntry {
                pane_id,
                window_label: info.window_label,
                bounds: info.bounds,
            })
            .collect();
        entries.sort_by(|a, b| a.pane_id.cmp(&b.pane_id));
        entries
    }

    fn lock_recovering(&self) -> std::sync::MutexGuard<'_, HashMap<PaneId, DetachedWindowInfo>> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Default for DetachedPaneMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_info(label: &str) -> DetachedWindowInfo {
        DetachedWindowInfo::new(label, "workspace-1", DetachedWindowBounds::default())
    }

    #[test]
    fn insert_records_detached_pane() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1".to_string(), sample_info("pane-detach-1"))
            .expect("insert should succeed");

        let info = map.get(&"pane-1".to_string()).expect("entry present");
        assert_eq!(info.window_label, "pane-detach-1");
        assert_eq!(info.workspace_id, "workspace-1");
    }

    #[test]
    fn insert_duplicate_returns_already_detached() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1".to_string(), sample_info("pane-detach-1"))
            .expect("first insert");

        let err = map
            .insert("pane-1".to_string(), sample_info("pane-detach-2"))
            .expect_err("duplicate should fail");

        assert!(matches!(err, DetachError::AlreadyDetached(_)));
    }

    #[test]
    fn remove_existing_returns_info_and_clears_entry() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1".to_string(), sample_info("pane-detach-1"))
            .expect("insert");

        let removed = map.remove(&"pane-1".to_string()).expect("removed");

        assert_eq!(removed.window_label, "pane-detach-1");
        assert!(map.get(&"pane-1".to_string()).is_none());
    }

    #[test]
    fn remove_missing_is_idempotent() {
        let map = DetachedPaneMap::new();

        assert!(map.remove(&"missing".to_string()).is_none());
    }

    #[test]
    fn list_returns_cloned_entries_without_holding_lock() {
        let map = DetachedPaneMap::new();
        map.insert("pane-b".to_string(), sample_info("pane-detach-b"))
            .expect("insert b");
        map.insert("pane-a".to_string(), sample_info("pane-detach-a"))
            .expect("insert a");

        let mut pane_ids: Vec<String> =
            map.list().into_iter().map(|(pane_id, _)| pane_id).collect();
        pane_ids.sort();

        assert_eq!(pane_ids, vec!["pane-a".to_string(), "pane-b".to_string()]);
    }

    #[test]
    fn clear_removes_all_entries() {
        let map = DetachedPaneMap::new();
        map.insert("pane-1".to_string(), sample_info("pane-detach-1"))
            .expect("insert 1");
        map.insert("pane-2".to_string(), sample_info("pane-detach-2"))
            .expect("insert 2");

        map.clear().expect("clear");

        assert!(map.list().is_empty());
    }
}
