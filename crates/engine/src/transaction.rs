/// Transaction support for multi-store memory operations.
///
/// Provides atomic write/delete across semantic and episodic stores
/// with rollback on failure.
use rememnemosyne_core::*;
use rememnemosyne_episodic::EpisodicMemoryStore;
use rememnemosyne_graph::GraphMemoryStore;
use rememnemosyne_semantic::SemanticMemoryStore;
use rememnemosyne_temporal::TemporalMemoryStore;

/// Transaction result
#[derive(Debug)]
pub struct TxResult {
    pub committed: bool,
    pub memory_ids: Vec<MemoryId>,
}

#[derive(Debug, Clone)]
enum TxOp {
    Store(MemoryArtifact),
    StoreEpisodic(MemoryArtifact),
    Delete(MemoryId, Option<MemoryArtifact>),
    Update(MemoryArtifact, Option<MemoryArtifact>),
}

struct TxStores<'a> {
    semantic: &'a SemanticMemoryStore,
    episodic: &'a EpisodicMemoryStore,
    graph: &'a GraphMemoryStore,
    temporal: &'a TemporalMemoryStore,
}

/// Transactional memory operations across semantic + episodic stores
pub struct MemoryTransaction<'a> {
    stores: TxStores<'a>,
    ops: Vec<TxOp>,
}

impl<'a> MemoryTransaction<'a> {
    pub fn new(
        semantic: &'a SemanticMemoryStore,
        episodic: &'a EpisodicMemoryStore,
        graph: &'a GraphMemoryStore,
        temporal: &'a TemporalMemoryStore,
    ) -> Self {
        Self {
            stores: TxStores { semantic, episodic, graph, temporal },
            ops: Vec::new(),
        }
    }

    pub fn store(&mut self, artifact: MemoryArtifact) {
        self.ops.push(TxOp::Store(artifact));
    }

    pub fn delete(&mut self, id: MemoryId) {
        self.ops.push(TxOp::Delete(id, None));
    }

    pub fn update(&mut self, artifact: MemoryArtifact) {
        self.ops.push(TxOp::Update(artifact, None));
    }

    /// Execute all operations atomically with rollback on failure
    pub async fn commit(mut self) -> Result<TxResult> {
        if self.ops.is_empty() {
            return Ok(TxResult { committed: true, memory_ids: Vec::new() });
        }

        // Phase 1: Prepare — snapshot data for rollback
        for op in &mut self.ops {
            match op {
                TxOp::Delete(id, prev) => {
                    if let Ok(Some(a)) = self.stores.semantic.get(id).await {
                        *prev = Some(a);
                    }
                }
                TxOp::Update(artifact, prev) => {
                    if let Ok(Some(a)) = self.stores.semantic.get(&artifact.id).await {
                        *prev = Some(a);
                    }
                }
                _ => {}
            }
        }

        // Phase 2: Execute with rollback on failure
        let mut executed_indices = Vec::new();
        let mut memory_ids = Vec::new();

        for (i, op) in self.ops.iter().enumerate() {
            match self.exec(op).await {
                Ok(Some(id)) => {
                    memory_ids.push(id);
                    executed_indices.push(i);
                }
                Ok(None) => {
                    executed_indices.push(i);
                }
                Err(e) => {
                    self.rollback_by_indices(&executed_indices).await;
                    return Err(MemoryError::Storage(format!(
                        "Transaction failed, rolled back: {e}"
                    )));
                }
            }
        }

        Ok(TxResult { committed: true, memory_ids })
    }

    async fn exec(&self, op: &TxOp) -> Result<Option<MemoryId>> {
        match op {
            TxOp::Store(a) => {
                let id = self.stores.semantic.store(a.clone()).await?;
                Ok(Some(id))
            }
            TxOp::StoreEpisodic(a) => {
                let id = self.stores.episodic.store(a.clone()).await?;
                Ok(Some(id))
            }
            TxOp::Delete(id, _) => {
                self.stores.semantic.delete(id).await?;
                Ok(None)
            }
            TxOp::Update(a, _) => {
                self.stores.semantic.update(a.clone()).await?;
                Ok(None)
            }
        }
    }

    async fn rollback_by_indices(&self, indices: &[usize]) {
        for i in indices.iter().rev() {
            let op = &self.ops[*i];
            match op {
                TxOp::Store(a) | TxOp::StoreEpisodic(a) => {
                    let _ = self.stores.semantic.delete(&a.id).await;
                    let _ = self.stores.episodic.delete(&a.id).await;
                }
                TxOp::Delete(id, prev) => {
                    if let Some(a) = prev {
                        let _ = self.stores.semantic.store(a.clone()).await;
                    }
                    let _ = self.stores.graph.delete_entity_by_memory_id(id).await;
                    let _ = self.stores.temporal.delete_events_by_memory_id(id).await;
                }
                TxOp::Update(a, prev) => {
                    if let Some(p) = prev {
                        let _ = self.stores.semantic.update(p.clone()).await;
                    } else {
                        let _ = self.stores.semantic.delete(&a.id).await;
                    }
                }
            }
        }
    }
                Ok(None) => {
                    executed_indices.push(i);
                }
                Err(e) => {
                    self.rollback_by_indices(&executed_indices).await;
                    return Err(MemoryError::Storage(format!(
                        "Transaction failed, rolled back: {e}"
                    )));
                }
            }
        }

        Ok(TxResult { committed: true, memory_ids })
    }

    async fn exec(&self, op: &TxOp) -> Result<Option<MemoryId>> {
        match op {
            TxOp::Store(a) => {
                let id = self.stores.semantic.store(a.clone()).await?;
                Ok(Some(id))
            }
            TxOp::StoreEpisodic(a) => {
                let id = self.stores.episodic.store(a.clone()).await?;
                Ok(Some(id))
            }
            TxOp::Delete(id, _) => {
                self.stores.semantic.delete(id).await?;
                Ok(None)
            }
            TxOp::Update(a, _) => {
                self.stores.semantic.update(a.clone()).await?;
                Ok(None)
            }
        }
    }

    async fn rollback_by_indices(&self, indices: &[usize]) {
        for i in indices.iter().rev() {
            let op = &self.ops[*i];
            match op {
                TxOp::Store(a) | TxOp::StoreEpisodic(a) => {
                    let _ = self.stores.semantic.delete(&a.id).await;
                    let _ = self.stores.episodic.delete(&a.id).await;
                }
                TxOp::Delete(id, prev) => {
                    if let Some(a) = prev {
                        let _ = self.stores.semantic.store(a.clone()).await;
                    }
                    let _ = self.stores.graph.delete_entity_by_memory_id(id).await;
                    let _ = self.stores.temporal.delete_events_by_memory_id(id).await;
                }
                TxOp::Update(a, prev) => {
                    if let Some(p) = prev {
                        let _ = self.stores.semantic.update(p.clone()).await;
                    } else {
                        let _ = self.stores.semantic.delete(&a.id).await;
                    }
                }
            }
        }
    }
}

/// Transactional delete across all stores
pub async fn delete_all_stores(
    semantic: &SemanticMemoryStore,
    episodic: &EpisodicMemoryStore,
    graph: &GraphMemoryStore,
    temporal: &TemporalMemoryStore,
    id: &MemoryId,
) -> Result<bool> {
    let mut tx = MemoryTransaction::new(semantic, episodic, graph, temporal);
    tx.delete(*id);
    let result = tx.commit().await?;
    Ok(result.committed)
}

/// Transactional store to semantic + episodic
pub async fn store_all_stores(
    semantic: &SemanticMemoryStore,
    episodic: &EpisodicMemoryStore,
    graph: &GraphMemoryStore,
    temporal: &TemporalMemoryStore,
    artifact: MemoryArtifact,
) -> Result<MemoryId> {
    let mut tx = MemoryTransaction::new(semantic, episodic, graph, temporal);
    tx.store(artifact.clone());
    let result = tx.commit().await?;
    result.memory_ids.first().copied().ok_or_else(|| {
        MemoryError::Storage("Transaction committed but no ID returned".into())
    })
}
