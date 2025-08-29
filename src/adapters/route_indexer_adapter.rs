use crate::domain::{errors::*, ports::testing::RouteIndexerPort};
use std::sync::Arc;

pub struct RouteIndexerAdapter {
    inner: Arc<crate::adapters::route_indexer::RouteIndexer>,
}

impl RouteIndexerAdapter {
    pub fn new(inner: Arc<crate::adapters::route_indexer::RouteIndexer>) -> Self {
        Self { inner }
    }
}

impl RouteIndexerPort for RouteIndexerAdapter {
    fn map_changes_to_specs(&self, changed_files: &[String]) -> PulseResult<Vec<String>> {
        // Leverage existing RouteIndex by building it and mapping changes
        let index = self
            .inner
            .build_index()
            .map_err(|e| PulseError::Runtime(e.to_string()))?;
        index
            .map_changes_to_specs(changed_files)
            .map_err(|e| PulseError::Runtime(e.to_string()))
    }
}
