mod config;
mod db;
mod transform;

pub use config::{IngestConfig, IngestScope};
pub use transform::{IngestPreview, IngestStats};

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use rememnemosyne_core::*;
use rememnemosyne_engine::RememnosyneEngine;

pub struct OpencodeIngestor {
    engine: Arc<RememnosyneEngine>,
    config: IngestConfig,
}

impl OpencodeIngestor {
    pub fn new(engine: Arc<RememnosyneEngine>, config: IngestConfig) -> Self {
        Self { engine, config }
    }

    /// Ingest from a specific opencode database path.
    pub async fn ingest(&self, db_path: &Path) -> rememnemosyne_core::Result<IngestStats> {
        let start = std::time::Instant::now();
        let conn = db::open_database(db_path)?;

        let session_ids = db::list_sessions(&conn, &self.config)?;
        let total_sessions = session_ids.len();

        let mut all_items = Vec::new();
        let mut messages_scanned = 0usize;
        let mut messages_skipped = 0usize;

        for session_id in &session_ids {
            let messages = db::read_session_messages(&conn, session_id)?;
            for msg in &messages {
                messages_scanned += 1;
                let items = transform::transform_message(msg, &self.config);
                if items.is_empty() {
                    messages_skipped += 1;
                }
                all_items.extend(items);
            }
        }

        let total_items = all_items.len();

        let ids = if self.config.dry_run {
            Vec::new()
        } else {
            self.engine
                .remember_batch(all_items)
                .await?
        };

        let stats = IngestStats {
            sessions_scanned: total_sessions,
            messages_scanned,
            messages_skipped,
            memories_ingested: if self.config.dry_run { 0 } else { ids.len() },
            memories_available: total_items,
            checkpoints_created: 0,
            elapsed: start.elapsed(),
        };

        tracing::info!(
            sessions = total_sessions,
            messages = messages_scanned,
            ingested = stats.memories_ingested,
            elapsed_ms = stats.elapsed.as_millis(),
            "opencode ingest complete"
        );

        Ok(stats)
    }

    /// Ingest from the default opencode data directory.
    /// Searches `$HOME/.opencode/` and `./opencode/` for data.db.
    pub async fn ingest_default(&self) -> rememnemosyne_core::Result<IngestStats> {
        let db_path = db::find_default_database()
            .ok_or_else(|| MemoryError::Storage("opencode database not found".into()))?;
        self.ingest(&db_path).await
    }

    /// Ingest only sessions for a specific project directory.
    pub async fn ingest_project(
        &self,
        project_dir: &Path,
    ) -> rememnemosyne_core::Result<IngestStats> {
        let mut config = self.config.clone();
        config.scope = IngestScope::Project {
            directory: project_dir.to_path_buf(),
        };
        let ingestor = Self::new(self.engine.clone(), config);
        ingestor.ingest_default().await
    }

    /// Dry run: count sessions and messages without ingesting.
    pub async fn dry_run(&self, db_path: &Path) -> rememnemosyne_core::Result<IngestPreview> {
        let conn = db::open_database(db_path)?;
        let session_ids = db::list_sessions(&conn, &self.config)?;

        let mut total_messages = 0usize;
        let mut total_items = 0usize;

        for session_id in &session_ids {
            let messages = db::read_session_messages(&conn, session_id)?;
            total_messages += messages.len();
            for msg in &messages {
                let items = transform::transform_message(msg, &self.config);
                total_items += items.len();
            }
        }

        Ok(IngestPreview {
            sessions: session_ids.len(),
            total_messages,
            estimated_memories: total_items,
        })
    }
}