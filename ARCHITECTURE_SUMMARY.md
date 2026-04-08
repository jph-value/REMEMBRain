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

### Core Architecture
- `crates/core/src/palace.rs` (811 lines)
  - MemoryPalace, Wing, Hall, Room, Drawer, Closet, Tunnel
  - PalaceQuery, PalaceResult, PalaceStats
  - Complete spatial hierarchy

- `crates/engine/src/palace_router.rs` (430 lines)
  - PalaceRouter with spatial routing
  - Memory indexing by location
  - Tunnel management
  - Room-based filtering

- `crates/engine/src/context_stack.rs` (493 lines)
  - LayeredContextStack (L0-L4)
  - ContextLayer with token budgets
  - Progressive loading
  - Model-specific presets

### Modified Files
- `crates/core/src/types.rs` (+120 lines)
  - Added `raw_content`, `is_summary`, `source_ref`, `palace_location`
  - Added verbatim preservation methods
  
- `crates/graph/src/relationship.rs` (+160 lines)
  - Added `ValidityWindow` struct
  - Added temporal validity methods
  
- `crates/core/src/lib.rs`
  - Export palace module
  
- `crates/engine/src/lib.rs`
  - Export palace_router, context_stack

---

## Test Coverage

**88 tests pass** across all crates:
- `rememnemosyne-core`: 24 tests (+12 new palace tests)
- `rememnemosyne-engine`: 22 tests (+7 new context_stack tests)
- `rememnemosyne-graph`: 6 tests (+1 relationship validity)
- Other crates: 36 tests (unchanged)

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

## Next Steps (Remaining Phases)

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

---

**Date**: April 8, 2026  
**Status**: Core architecture complete, 88 tests pass  
**Next**: Agent diaries, compression, provider implementations
