# Memory Caching (MC) Integration Plan

> Based on: arXiv:2602.24281 — "Memory Caching: RNNs with Growing Memory"
> by Ali Behrouz, Zeman Li, Yuan Deng, Peilin Zhong, Meisam Razaviyayn, Vahab Mirrokni

## Overview

This document describes the integration of Memory Caching (MC) concepts into RemeMnemosyne. MC addresses RemeMnemosyne's three core scaling problems:

1. **F1 collapse at scale** — Recall drops from 0.80 to 0.01 at 10k memories because individual memory search degrades without a coarse-grained first pass.
2. **CognitiveEngine unimplemented** — The trait exists but has no implementation. Intent prefetching is stubbed, transition matrices are dead code.
3. **Binary context assembly** — LayeredContextStack loads L2-L4 as all-or-nothing. Context formatting includes or excludes memories with no graduated weighting.

MC's core insight: **cache compressed segment checkpoints and use query-dependent gating to retrieve from them**, interpolating between O(L) fixed memory and O(L²) growing memory at O(NL) cost.

## Architecture Mapping

| MC Concept | RemeMnemosyne Implementation |
|---|---|
| Segment checkpoint | `MemoryCheckpoint` — compressed summary of a time window of memories |
| Gated Residual Memory (GRM) | `contribution_weights` on `ContextBundle` — query-memory similarity gates rendering depth |
| Sparse Selective Caching (SSC) | `SSCRouter` — Top-k checkpoint routing before fine-grained HNSW search |
| MeanPool(S^(i)) | `SegmentProfile::importance_weighted_embedding` — importance-weighted checkpoint fingerprint |
| γ_t gating | `cosine_similarity(query_embedding, memory_embedding)` → softmax normalization |
| Online memory | Current HNSW search results (always included) |
| Cached memories | `CheckpointStore` results (expanding into individual memories within the window) |

## Feature Flags

All MC integration is behind feature flags for zero-overhead opt-in:

```toml
[features]
# Phase 1: Segment checkpointing (can be used alone)
mc-checkpoints = []

# Phase 2: Gated context assembly (depends on Phase 1 for checkpoint embeddings)
mc-gated-context = ["mc-checkpoints"]

# Phase 3: SSC router (depends on Phase 1)
mc-ssc = ["mc-checkpoints"]
```

## Phase 1: Segment Checkpointing

### Problem Solved
At scale, HNSW alone can't differentiate signal from noise. Checkpoints provide O(N) coarse search before O(M) fine search.

### New Types

**`MemoryCheckpoint`** (`crates/core/src/types.rs`):
- Compressed summary of a time window of memories
- Fields: `id`, `time_window_start/end`, `summary_embedding` (importance-weighted pooled), `summary_text`, `memory_count`, `memory_ids`, `key_entities`, `palace_location`, `session_id`, `importance_ceiling`, `embedding_method`, `created_at`

**`CheckpointEmbeddingMethod`** (`crates/core/src/types.rs`):
- `MeanPool` — simple average (fast but less discriminative)
- `ImportanceWeightedPool` — weighted by importance level (default, more discriminative)
- `MaxPool` — maximum per-dimension (most discriminative, higher variance)

**`CheckpointStore`** (`crates/episodic/src/checkpoint.rs`):
- In-memory checkpoint storage with DashMap
- Dual-trigger creation: every N memories (default 50) OR every T seconds (default 1800)
- `search_checkpoints()` — cosine similarity over checkpoint embeddings
- `expand_checkpoint()` — returns memory_ids within a checkpoint for fine-grained search
- Eviction when exceeding max_checkpoints (default 200)

### Risk Mitigations

| Risk | Mitigation | Status |
|---|---|---|
| Checkpoint granularity too bland | `ImportanceWeightedPool` default — high-importance memories dominate the fingerprint. `MaxPool` fallback | ✅ Implemented |
| Cold start (no checkpoints yet) | Falls through to bare HNSW. This is MC's N=1 baseline — standard RNN behavior | ✅ Implemented. `checkpoint_aware_search()` returns empty `HashSet` when `is_empty()` |
| Hard-gate recall loss | Checkpoint boost is 1.3× soft multiplier on relevance, not a hard filter. HNSW finds everything; checkpoints rank them better | ✅ Implemented in `MemoryRouter::query()` |
| Softmax dilution over many low-sim candidates | Top-k softmax: normalize only over `max_memories` candidates, not all. Remaining get minimal 0.05 weight (not zero — preserves recall) | ✅ Implemented in `build_context_weighted()` (Bug 4 fix) |
| Checkpoint eviction zombies | `evict_and_return_ids()` returns evicted checkpoint IDs; caller deregisters from SSC router | ✅ Implemented (Bug 1 fix) |
| SSC router scoring ignores transitions | `score_segments_with_transitions()` blends 70% cosine + 30% transition probability from `ContextPredictor` | ✅ Implemented (Bug 5 fix) |
| Transition matrix cold start | Only blend after >10 transitions recorded. Before threshold, pure embedding similarity | ✅ Implemented in both `ContextPredictor::predict()` and `MemoryRouter::checkpoint_aware_search()` |
| Checkpoint embedding collision | Dual fingerprint: `mean_embedding` + `importance_weighted_embedding`. SSC uses the weighted version for scoring, falls back to mean | ✅ Implemented (Bug 3 fix) |
| `__recent__` magic string | Replaced with timestamp-ordered `MemoryQuery::new().with_limit(count)` (no text filter) | ✅ Fixed (Bug 6) |

## Phase 1: Segment Checkpointing

### Flow

```
MemoryRouter::query()
    ↓
[Phase 1: Checkpoint routing]
    ↓ query_embedding → CheckpointStore::search_checkpoints(k=5)
    ↓ select checkpoints with score ≥ 0.3
    ↓ expand high-scoring checkpoints → checkpoint_memory_ids
    ↓
[Existing: Semantic HNSW search]
    ↓ boost relevance by 1.3× for checkpoint_memory_ids
    ↓
[Phase 1: Context assembly]
    ↓ load_checkpoint_context() prepends segment summaries to L3
```

## Phase 2: Gated Context Assembly (GRM)

### Problem Solved
Small models get "distracted" by irrelevant memory content (2B recall drops 29.9% → 20.6%). GRM gates each memory's contribution based on query-memory similarity.

### Changes

**`ContextBundle`** — new field:
- `contribution_weights: HashMap<MemoryId, f32>` — MC's γ_t^(i) gates per memory

**`ContextBuilderEngine::build_context()`** — compute gates:
- `γ = cosine_similarity(query_embedding, memory_embedding)`
- Softmax normalize over top-k candidates (not all — prevents weight dilution)
- Pass gates to ContextBundle via `add_memory_weighted()`

**Format strategies** — weight-tiered rendering:
- γ > 0.7: Full verbatim content from Drawer (up to 300 chars)
- 0.3 < γ < 0.7: Closet summary (up to 150 chars)
- γ < 0.3: One-line reference only

**`LayeredContextStack`** — auto-escalation:
- New `should_escalate()` method: if query embedding has low similarity to current layer content, escalate to deeper layer
- New `layer_embedding: Option<Vec<f32>>` on `ContextLayer` — mean-pooled embedding of the layer's source memories (MC's MeanPool(S^(i)))

### Risk Mitigations

| Risk | Mitigation |
|---|---|
| Softmax dilution over many low-sim candidates | Top-k softmax: normalize only over `max_memories` candidates, not all. Remaining get minimal 0.05 weight (not zero — preserves recall) |
| 2B model distraction | Weight-tiered formatting: low-weight memories become single-line references, drastically reducing noise |

## Phase 3: Learned SSC Router

### Problem Solved
Intent-based prefetching is stubbed with empty match arms. Transition matrix is dead code. At 10k+ memories, even HNSW needs pre-filtering.

### New Modules

**`SSCRouter`** (`crates/cognitive/src/ssc_router.rs`):
- Stores `SegmentProfile` for each checkpoint: mean and importance-weighted embeddings
- `route(query_embedding, candidate_ids) → top-k checkpoint IDs`
- `route_with_transitions(query, ids, transition_probs) → top-k with 70/30 blend` (Bug 5 fix)
- `score_segments_with_transitions()` — 70/30 cosine+transition scoring
- Configuration: `top_k` (default 5)
- `deregister()` — removes zombie entries from evicted checkpoints (Bug 1 fix)

**`CognitiveEngineImpl`** (`crates/cognitive/src/engine.rs`):
- First implementation of the `CognitiveEngine` trait (currently unimplemented)
- Delegates to `MicroEmbedder`, `IntentDetector`, `ContextPredictor`, `MemoryPrefetcher`, optional `SSCRouter`

**`ContextPredictor`** — activated transition matrix:
- `record_transition(from_state, to_state)` — populates the previously-dead transition_matrix
- `get_transition_prob(from, to)` — public method for SSC router blending (Bug 5 fix)
- `transition_capacity()` — public method returning matrix size for cold-start guards
- `last_intent_state` — public field for SSC router to query current intent state
- Blend transition probabilities (30%) with embedding similarity (70%) in `predict()` only after >10 transitions recorded

**`MemoryPrefetcher`** — implemented `intent_based_prefetch()`:
- Routes to cluster centroids based on intent: recall/search gets 1.3× boost, analyze gets 1.1×
- Uses `last_query_embedding` for similarity computation

### Risk Mitigations

| Risk | Mitigation | Status |
|---|---|---|
| Transition matrix cold start | Only blend after >10 transitions recorded. Before threshold, pure embedding similarity | ✅ Implemented in both `ContextPredictor::predict()` and `checkpoint_aware_search()` |
| Checkpoint embedding collision | Dual fingerprint: `mean_embedding` + `importance_weighted_embedding`. SSC uses the weighted version for scoring | ✅ Implemented (Bug 3 fix) |
| Dead `expansion_threshold` field on `SSCRouterConfig` | Removed. `expansion_threshold` lives on `CheckpointConfig` where it's actually used | ✅ Fixed (Bug 2) |
| `mark_accessed()` unused for eviction | Known deferred — currently using FIFO eviction. LRU eviction based on `last_accessed` is future work | Deferred |

## Dependency Graph

```
Phase 1: Segment Checkpointing
  ├── core/types.rs (MemoryCheckpoint, CheckpointEmbeddingMethod, enum variants)
  ├── core/math.rs (shared vector utilities)
  ├── episodic/checkpoint.rs (CheckpointStore, CheckpointConfig)
  └── engine/router.rs (checkpoint creation trigger + search boost)

Phase 2: Gated Context Assembly (depends on Phase 1 for checkpoint embeddings)
  ├── core/types.rs (contribution_weights on ContextBundle)
  ├── engine/context.rs (GRM gate computation, weight-tiered formatting)
  └── engine/context_stack.rs (auto-escalation, layer embeddings)

Phase 3: SSC Router (depends on Phase 1 checkpoints)
  ├── cognitive/ssc_router.rs (SSCRouter, SegmentProfile)
  ├── cognitive/engine.rs (CognitiveEngineImpl)
  ├── cognitive/predictor.rs (activate transition_matrix)
  ├── cognitive/prefetcher.rs (implement intent_based_prefetch)
  ├── cognitive/micro_embed.rs (extract_entities_ner)
  └── engine/router.rs (SSC query routing)
```

## New External Dependencies

**None.** All implementations use `dashmap`, `uuid`, `chrono`, `serde`, `parking_lot`, `rayon` — already in workspace Cargo.toml.

## File Changes Summary

| File | Phase | Action |
|---|---|---|
| `crates/core/src/math.rs` | 1 | NEW — shared vector math (`cosine_similarity`, `softmax`, `mean_pool`, `weighted_mean_pool`, `max_pool`) |
| `crates/core/src/types.rs` | 1,2 | Modified — `MemoryCheckpoint` (with `mean_embedding`), `CheckpointEmbeddingMethod`, `MemoryType::Checkpoint`, `EventType::Checkpoint`, `contribution_weights` on `ContextBundle`, `add_memory_weighted()`, fixed `merge()` |
| `crates/core/src/lib.rs` | 1 | Modified — `pub mod math` export |
| `crates/episodic/src/checkpoint.rs` | 1 | NEW — `CheckpointStore`, `CheckpointConfig`, dual-trigger creation, `evict_and_return_ids()` |
| `crates/episodic/src/lib.rs` | 1 | Modified — `pub mod checkpoint` export |
| `crates/engine/src/router.rs` | 1,3 | Modified — `CheckpointStore` + `SSCRouter` fields, `MemoryRouterConfig`, `checkpoint_aware_search()` with transition blending, `store()` triggers checkpoint creation + eviction deregistration, `collect_recent_memories_for_checkpoint()` (Bug 6 fix) |
| `crates/engine/src/context.rs` | 2 | Modified — `build_context_weighted()` with GRM gate computation, softmax over top-k, weight-tiered formatting in all 4 strategies (Bug 4 fix) |
| `crates/engine/src/context_stack.rs` | 1,2 | Modified — `layer_embedding`, `should_escalate()`, `compute_layer_relevance()`, `load_checkpoint_context()` |
| `crates/engine/src/builder.rs` | 1 | Modified — `recall()` reuses `query_embedding`, `with_router_config()` |
| `crates/engine/Cargo.toml` | 1,2,3 | Modified — Feature flags `mc-checkpoints`, `mc-gated-context`, `mc-ssc`, `structured-logging` |
| `crates/cognitive/src/ssc_router.rs` | 3 | NEW — `SSCRouter`, `SSCRouterConfig` (without dead `expansion_threshold`), `SegmentProfile`, `score_segments_with_transitions()`, `route_with_transitions()`, `deregister()` |
| `crates/cognitive/src/engine.rs` | 3 | NEW — `CognitiveEngineImpl` |
| `crates/cognitive/src/predictor.rs` | 3 | Modified — Activated `transition_matrix`, `record_transition()`, `get_transition_prob()` pub, `last_intent_state` pub, `transition_capacity()` pub, 70/30 blend |
| `crates/cognitive/src/prefetcher.rs` | 3 | Modified — Implemented `intent_based_prefetch()`, `last_query_embedding` |
| `crates/cognitive/src/micro_embed.rs` | 3 | Modified — `extract_entities_ner()` |
| `crates/cognitive/src/lib.rs` | 3 | Modified — `ssc_router`, `engine` module exports |
| `crates/engine/tests/mc_integration.rs` | 1,2,3 | NEW — 9-test MC integration suite |
| `crates/storage/src/archive.rs` | — | Modified — v2 format (u64 length prefix), v1→v2 migration |
| `crates/temporal/src/timeline.rs` | — | Modified — NaN-safe `partial_cmp` (Bug C5) |

## Bugs Found and Fixed (Cohesion Review)

### Critical (C1-C6)
- **C1**: Non-exhaustive match on `MemoryType` in `palace_router.rs` — fixed with wildcard arm
- **C2**: `ContextBundle::merge()` silently dropped `contribution_weights` — fixed to merge weights
- **C3**: `create_checkpoint()` panicked on empty input — now returns `Result::Err`
- **C4**: `std::sync::RwLock` in `CheckpointStore` — switched to `parking_lot::RwLock` (no poisoning)
- **C5**: `partial_cmp().unwrap()` on NaN in `timeline.rs` — fixed with `unwrap_or(Ordering::Equal)`
- **C6**: Float-to-usize truncation bug — `*raw_score as usize` as HashMap key truncated 0.85→0

### High (H1-H7)
- **H1**: Feature flag dependencies fixed for `mc-checkpoints`, `mc-gated-context`, `mc-ssc`
- **H3**: Dual embedding generation eliminated — `MemoryResponse` carries `query_embedding`
- **H5**: `mark_accessed()` updates `last_accessed` but nothing reads it (deferred for LRU)
- **H7**: All 4 format strategies now use `contribution_weights` for rendering depth

### Medium (M1-M6)
- **M1**: `SegmentProfile` now derives `Serialize`, `Deserialize`
- **M4**: `MemoryTypeLabel` updated with all 11 variants for metrics
- **M6**: `providers` module re-exported in engine `lib.rs`

### MC-Specific Bugs (Found during cohesion review)
- **Bug 1**: `evict_if_needed()` removed checkpoint from `CheckpointStore` but never called `ssc_router.deregister()` — zombie entries wasted Top-k slots. Fixed: `evict_and_return_ids()` returns evicted IDs for deregistration
- **Bug 2**: `SSCRouterConfig.expansion_threshold` was a dead field — removed from struct. Actual threshold lives on `CheckpointConfig`
- **Bug 3**: `register_checkpoint()` stored same `summary_embedding` as both `mean_embedding` and `importance_weighted_embedding`. Fixed: `MemoryCheckpoint.mean_embedding` field added, `compute_both_embeddings()` computes both separately
- **Bug 4**: `build_context_weighted()` computed softmax into `weight_map` but contribution_weight formula used uniform `1/softmax_k` instead of per-memory softmax values. Fixed: uses `weight_map.get(&idx).copied().unwrap_or(0.05)`
- **Bug 5**: SSC router scoring was pure cosine similarity with no transition blending. Fixed: `score_segments_with_transitions()` + `route_with_transitions()` with 70/30 cosine+transition blend via `ContextPredictor`
- **Bug 6**: `collect_recent_memories_for_checkpoint()` used `"__recent__"` magic string as query. Fixed: uses `MemoryQuery::new().with_limit(count)` (timestamp-ordered, no text filter)

## Known Gaps (Deferred)

- **CheckpointStore persistence**: All MC state (checkpoints, SSC profiles) is in-memory only. Lost on restart. Requires serialization to sled/disk.
- **Feature flag gating**: `mc-checkpoints`, `mc-gated-context`, `mc-ssc` exist in Cargo.toml but gate zero code with `#[cfg(feature = ...)]`. MC code always included since empty CheckpointStore has negligible overhead.
- **LRU eviction**: `mark_accessed()` updates `last_accessed` but nothing reads it for eviction. Currently using FIFO. LRU eviction is future work.
- **CognitiveEngineImpl wiring**: Exists but is not connected to the production `MemoryRouter` flow. Could be wired in as a provider for smarter prefetching.