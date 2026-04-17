use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum IngestScope {
    Session { id: String },
    Project { directory: PathBuf },
    Global,
}

impl Default for IngestScope {
    fn default() -> Self {
        Self::Global
    }
}

#[derive(Debug, Clone)]
pub struct IngestConfig {
    pub scope: IngestScope,
    pub batch_size: usize,
    pub include_compacted: bool,
    pub include_errors: bool,
    pub include_tool_outputs: bool,
    pub include_reasoning: bool,
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
    pub dry_run: bool,
    pub max_content_length: usize,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            scope: IngestScope::Global,
            batch_size: 50,
            include_compacted: true,
            include_errors: true,
            include_tool_outputs: true,
            include_reasoning: true,
            since: None,
            until: None,
            max_content_length: 10_000,
            dry_run: false,
        }
    }
}
