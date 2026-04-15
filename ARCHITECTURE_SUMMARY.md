# RemeMnemosyne Architecture Summary

## Mempalace-Inspired Pure Rust Implementation

This document summarizes our adaptation of mempalace's brilliant concepts into a pure Rust systems-level architecture, while retaining our competitive advantages.

---

## What We Adopted from Mempalace

### ✅ 1. Spatial Memory Organization (+34% retrieval accuracy)
**Original**: Python-based Wings/Halls/Rooms/Drawers/Closets structure  
**Our Implementation**: Pure Rust in `crates/core/src/palace.rs`

```rust
MemoryPalace
├── Wing (Person/Project/Organization)
│   ├── Hall (Facts/Events/Discoveries/Preferences/Advice)
│   │   ├── Room (topic-specific)
│   │   │   ├── Drawers (raw verbatim content)
│   │   │   └── Closets (summaries/pointers)
│   │   └── Room
│   └── Hall
└── Tunnel (cross-wing topic links)
```

**Key files:**
- `crates/core/src/palace.rs` (811 lines) - Complete palace structure
- `crates/engine/src/palace_router.rs` (430 lines) - Spatial routing
- `crates/core/src/types.rs` - Added `PalaceLocation` to `MemoryArtifact`

### ✅ 2. Verbatim Preservation (96.6% R@5 on LongMemEval)
**Original**: Raw exchanges stored unaltered, summaries separate  
**Our Implementation**: Added to `MemoryArtifact`

```rust
pub struct MemoryArtifact {
    pub summary: String,          // Brief summary (closet)
    pub content: String,           // Verbatim content (drawer)
    pub raw_content: Option<String>, // Preserved raw source
    pub is_summary: bool,          // Whether this is summary-only
    pub source_ref: Option<String>, // Source document/URL
    pub palace_location: Option<PalaceLocation>, // Spatial position
}
```

**Methods added:**
- `with_raw_content()` - Preserve verbatim source
- `as_summary()` - Mark as summary
- `with_source_ref()` - Link to original
- `in_palace_room()` - Set spatial location
- `effective_content()` - Get raw or processed content
- `has_raw_content()` - Check if verbatim preserved
- `is_in_palace_room()` - Check spatial location

### ✅ 3. Layered Context Loading (L0-L4) (10x token efficiency)
**Original**: 4-tier stack (L0-L3)  
**Our Implementation**: Extended to 5 tiers in `crates/engine/src/context_stack.rs`

```
L0 - Identity (~50 tokens, always loaded)
     "You are a risk analysis assistant."

L1 - Critical Facts (~120 tokens, always loaded)
     "- User prefers Rust over Python"
     "- Project deadline: 2026-05-01"

L2 - Room Recall (on-demand, ~500 tokens)
     Recent session context from current room

L3 - Relevant Memories (triggered, ~2000 tokens)
     Semantic search results

L4 - Deep Search (explicit request, variable)
     Full semantic query across all data
```

**Key methods:**
- `load_identity()` - Set L0
- `load_critical_facts()` - Set L1
- `load_room_recall()` - Set L2
- `load_relevant_memories()` - Set L3
- `load_deep_search()` - Set L4
- `get_always_loaded()` - Get L0+L1 only
- `reset_to_base()` - Clear L2-L4

### ✅ 4. Temporal Validity for Relationships
**Original**: SQLite temporal graph with validity windows  
**Our Implementation**: Pure Rust in `crates/graph/src/relationship.rs`

```rust
pub struct ValidityWindow {
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub invalidated: bool,
    pub invalidation_reason: Option<String>,
    pub invalidated_by: Option<String>,
}

pub struct GraphRelationship {
    // ... existing fields ...
    pub validity: ValidityWindow,
}
```

**Methods added:**
- `with_expiration(duration)` - Set relationship expiry
- `with_expiration_at(datetime)` - Set specific expiration
- `is_valid()` - Check if currently valid
- `is_expired()` - Check if expired
- `invalidate(reason, by)` - Mark as invalid
- `reactivate()` - Restore validity
- `days_until_expiration()` - Get remaining time

**Why this matters for RISC-OSINT:**
- Intelligence relationships expire ("Person X was associated with Company Y until 2024")
- Threat assessments have validity periods
- Evidence links become stale over time

---

## What We Retain (Our Competitive Advantages)

### ✅ Multi-Provider Architecture (Mempalace has none)
```rust
Embedding Providers: Local | OpenAI | Voyage | Cohere | Ollama | Custom
Reasoning Providers: OpenAI | Anthropic | OpenRouter | Ollama | Custom
Agent Providers: Verification | Analysis | Report | SimulationPlanner | EvidenceExtractor
```

### ✅ Typed Intelligence Memories (Mempalace has generic episodes)
```rust
EventMemory - Discrete events with severity/location
NarrativeMemory - Evolving storylines with evidence links
RiskNodeMemory - Risk entities with composite threat scoring
EvidenceMemory - Attributed evidence with reliability scoring
SimulationMemory - Scenario projections with outcomes
```

### ✅ Vector Search with Quantization (Mempalace uses ChromaDB)
- Custom TurboQuant (PQ, OPQ, Polar, QJL)
- HNSW index for approximate nearest neighbor
- 8x compression for embeddings
- No external dependencies

### ✅ Horizontal Scaling (Mempalace is single-node)
- Sharding by entity type
- Read replicas
- Memory compaction
- Auto-pruning

### ✅ Provider Registry
Central hub for managing all provider instances:
```rust
pub struct ProviderRegistry {
    embedding_provider: RwLock<Option<Arc<dyn EmbeddingProvider>>>,
    reasoning_provider: RwLock<Option<Arc<dyn ReasoningProvider>>>,
    agent_providers: RwLock<HashMap<AgentType, Arc<dyn AgentProvider>>>,
}
```

---

## Architecture Comparison

| Feature | Mempalace | RemeMnemosyne (Before) | RemeMnemosyne (Now) |
|---------|-----------|------------------------|---------------------|
| **Language** | Python | Rust | Rust |
| **Spatial Organization** | ✅ Wings/Halls/Rooms | ❌ Flat | ✅ Palace structure |
| **Verbatim Preservation** | ✅ Drawers/Closets | ❌ Summarized | ✅ Raw + summary |
| **Layered Context** | ✅ L0-L3 | ❌ All-at-once | ✅ L0-L4 stack |
| **Temporal Validity** | ✅ SQLite graph | ❌ No expiration | ✅ ValidityWindow |
| **Memory Caching** | ❌ None | ❌ None | ✅ CheckpointStore + SSC router + GRM |
| **Multi-Provider** | ❌ None | ❌ None | ✅ 3-layer providers |
| **Typed Memories** | ❌ Generic | ✅ Typed | ✅ Typed + spatial |
| **Vector Search** | ChromaDB | ✅ Custom | ✅ Custom + quantization |
| **Horizontal Scale** | ❌ Single-node | ✅ Sharding/replicas | ✅ + spatial routing |
| **Entity Resolution** | ❌ None | ❌ None | ✅ Fuzzy matching |
| **Backup/Export** | ❌ None | ❌ None | ✅ JSON/NDJSON |
| **Metrics** | ❌ None | ❌ None | ✅ Prometheus |
| **HTTP API** | MCP only | ❌ None | ✅ REST + health |

---

## New Files Created (This Session)

## New Files Created

### Core Architecture
- `crates/core/src/palace.rs` (811 lines)
  - MemoryPalace, Wing, Hall, Room, Drawer, Closet, Tunnel
  - PalaceQuery, PalaceResult, PalaceStats
  - Complete spatial hierarchy

- `crates/core/src/math.rs`
  - Shared vector math utilities: `cosine_similarity`, `softmax`, `mean_pool`, `weighted_mean_pool`, `max_pool`
  - Used by CheckpointStore, SSC router, and GRM gate computation

- `crates/engine/src/palace_router.rs` (430 lines)
  - PalaceRouter with spatial routing
  - Memory indexing by location
  - Tunnel management
  - Room-based filtering

- `crates/engine/src/context_stack.rs` (493 lines)
  - LayeredContextStack (L0-L4)
  - ContextLayer with token budgets and layer embeddings
  - Progressive loading, auto-escalation via `should_escalate()`
  - `load_checkpoint_context()` for MC segment summaries
  - Model-specific presets

### Memory Caching (arXiv:2602.24281)
- `crates/episodic/src/checkpoint.rs`
  - `CheckpointStore` — dual-trigger (count + time) checkpoint creation
  - `CheckpointConfig` — configurable thresholds, embedding methods, expansion
  - `search_checkpoints()` — cosine similarity over checkpoint embeddings
  - `evict_and_return_ids()` — FIFO eviction with SSC deregistration support

- `crates/cognitive/src/ssc_router.rs`
  - `SSCRouter` — Top-k checkpoint routing with `score_segments_with_transitions()`
  - `SegmentProfile` — dual embeddings (mean + importance-weighted) per segment
  - 70/30 cosine+transition blend via `route_with_transitions()`

- `crates/cognitive/src/engine.rs`
  - `CognitiveEngineImpl` — first concrete `CognitiveEngine` implementation

- `crates/engine/tests/mc_integration.rs`
  - 9-test integration suite covering all three MC phases

### Modified Files
- `crates/core/src/types.rs`
  - Added `MemoryType::Checkpoint`, `EventType::Checkpoint`
  - Added `MemoryCheckpoint` with `mean_embedding` field (Bug 3 fix)
  - Added `CheckpointEmbeddingMethod` enum
  - Added `contribution_weights` on `ContextBundle`
  - Added `add_memory_weighted()`, fixed `merge()` to preserve contribution_weights

- `crates/core/src/lib.rs`
  - Export `palace` module, `math` module

- `crates/engine/src/router.rs`
  - `MemoryRouterConfig` with checkpoint and SSC config
  - `MemoryRouter.checkpoint_aware_search()` with 1.3× boost
  - `MemoryRouter.store()` triggers dual-checkpoint creation + SSC registration + eviction deregistration
  - `MemoryResponse.query_embedding` to avoid re-embedding
  - `collect_recent_memories_for_checkpoint()` using timestamp-ordered retrieval (Bug 6 fix)

- `crates/engine/src/context.rs`
  - `build_context_weighted()` with GRM gate computation
  - Top-k softmax normalization, 60/40 blended contribution weights
  - Weight-tiered formatting in all 4 strategies

- `crates/engine/src/context_stack.rs`
  - `layer_embedding: Option<Vec<f32>>` on `ContextLayer`
  - `should_escalate()` for query-layer similarity
  - `load_checkpoint_context()` for MC segment summaries

- `crates/cognitive/src/predictor.rs`
  - Activated `transition_matrix` with `record_transition()`
  - `get_transition_prob()` — public for SSC router blending
  - `last_intent_state` — public for SSC router blending
  - `transition_capacity()` — public for cold-start guard
  - 70/30 embedding+transition blend in `predict()` after >10 observations

- `crates/cognitive/src/prefetcher.rs`
  - Implemented `intent_based_prefetch()`
  - `last_query_embedding` for similarity computation

- `crates/cognitive/src/micro_embed.rs`
  - `extract_entities_ner()` stub

- `crates/storage/src/archive.rs`
  - v2 archive format (u64 length prefix)
  - Backward-compatible v1→v2 migration
  - Format versioning in `ArchiveCatalog`

- `crates/semantic/src/sharding.rs`
  - `MemoryType::Checkpoint` in match arms

- `crates/graph/src/entity_resolution.rs`
  - `EntityType` import fix

- `crates/temporal/src/timeline.rs`
  - NaN-safe `partial_cmp` (Bug C5 fix)

- `crates/engine/src/palace_router.rs`
  - Non-exhaustive `MemoryType` match (Bug C1 fix)

- `crates/engine/src/metrics.rs`
  - Prometheus 0.23 API rewrite
  - `MemoryTypeLabel` with all 11 variants

- `crates/engine/src/http_server.rs`
  - Axum 0.7 API rewrite

---

## Test Coverage

**132 tests pass** across all crates:
- `rememnemosyne-core`: 16 tests
- `rememnemosyne-episodic`: 9 tests (7 checkpoint tests)
- `rememnemosyne-cognitive`: 2 tests (SSC router + predictor)
- `rememnemosyne-engine`: 29 tests (9 MC integration tests)
- `rememnemosyne-semantic`: 6 tests
- `rememnemosyne-graph`: 6 tests
- `rememnemosyne-storage`: 5 tests
- Other crates: 59 tests

---

## Usage Examples

### Creating a Memory Palace
```rust
use rememnemosyne_core::*;

// Create palace
let mut palace = MemoryPalace::new("RISC-OSINT Palace");

// Add wings
let person_wing = Wing::new("Alice", WingType::Person);
let project_wing = Wing::new("ProjectX", WingType::Project);
palace.add_wing(person_wing);
palace.add_wing(project_wing);

// Create tunnel between wings
palace.add_tunnel("rust", "Alice");
palace.add_tunnel("rust", "ProjectX");
```

### Storing Memory with Verbatim Preservation
```rust
let memory = MemoryArtifact::new(
    MemoryType::Semantic,
    "Rust lifetime discussion", // Summary (closet)
    "User asked about Rust lifetimes...", // Verbatim content (drawer)
    embedding,
    MemoryTrigger::UserInput,
)
.with_raw_content(raw_conversation) // Preserve full source
.with_source_ref("session_123.log") // Link to original
.in_palace_room("Alice", "hall_facts", "rust"); // Spatial location
```

### Layered Context Loading
```rust
use rememnemosyne_engine::context_stack::*;

let mut stack = LayeredContextStack::for_large_model();

// L0: Always loaded
stack.load_identity("You are a risk analysis assistant.");

// L1: Always loaded
stack.load_critical_facts(vec![&critical_memory]);

// L2: On-demand
stack.load_room_recall(vec![&recent_memories], Some(&location));

// L3: Triggered by query
stack.load_relevant_memories(vec![&relevant], "query");

// L4: Explicit request
if stack.can_deep_search() {
    stack.load_deep_search(full_context, all_ids);
}

// Get appropriate context
let context = stack.get_full_context();
```

### Temporal Relationship Validity
```rust
use rememnemosyne_graph::GraphRelationship;
use chrono::Duration;

let rel = GraphRelationship::new(
    source_id,
    target_id,
    RelationshipType::Related,
    0.8,
)
.with_expiration(Duration::days(90)) // Expires in 3 months
.with_evidence(evidence);

// Check validity
if rel.is_valid() {
    // Use relationship
}

if rel.is_expired() {
    // Relationship expired, needs review
}

// Invalidate manually
rel.invalidate("Source discredited", "analyst_42");
```

---

## Next Steps

### Phase B.3: Agent Diary System
- Per-agent memory tracks (separate from main memory)
- Compressed logs persisting expertise across sessions
- Prevents agent memory from bloating main context

### Phase B.4: AAAK-Style Optional Compression
- Lossy, LLM-readable abbreviation dialect
- Token efficiency at scale (3-5x compression)
- Kept separate from default raw storage

### Phase C.1-C.4: Provider Integration
- Concrete OpenAI/Anthropic provider implementations
- Wire providers into engine router
- Typed memory store integration
- End-to-end RISC-OSINT integration tests

### MC Hardening (Recommended)
- CheckpointStore persistence (currently in-memory only — all MC state lost on restart)
- Feature flag gating (`mc-checkpoints`, `mc-gated-context`, `mc-ssc` exist in Cargo.toml but gate zero code)
- Integration benchmark at 10k+ memories to validate F1 improvement claim

---

**Date**: April 15, 2026
**Status**: Core architecture + MC integration complete, 132 tests pass, all critical/high/medium bugs fixed
**Next**: Agent diaries, compression, provider implementations, MC persistence
