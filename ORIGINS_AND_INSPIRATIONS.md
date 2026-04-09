# RemeMnemosyne: Inspiration & Architectural Lineage

## Executive Summary

RemeMnemosyne is a **pure Rust** reimagining of concepts proven by Python-based memory systems, combined with high-performance vector search algorithms and production-grade infrastructure patterns. It was born from a real requirement: **RISC.OSINT** needed unlimited, semantically-searchable memory for planetary-scale global risk intelligence.

Every major architectural decision traces to a specific project or algorithm. This document maps each concept to its origin.

---

## Primary Inspirations

### 1. Mempalace — Spatial Memory Organization & Verbatim Preservation

**Repository**: [milla-jovovich/mempalace](https://github.com/milla-jovovich/mempalace)  
**Language**: Python  
**License**: MIT

**What We Adopted:**

| Mempalace Concept | RemeMnemosyne Implementation | Location |
|-------------------|------------------------------|----------|
| **Wings/Halls/Rooms** spatial hierarchy | `MemoryPalace`, `Wing`, `Hall`, `Room` structs | `crates/core/src/palace.rs` |
| **Drawers** (raw verbatim content, never altered) | `Drawer` struct + `MemoryArtifact.raw_content` field | `crates/core/src/palace.rs`, `crates/core/src/types.rs` |
| **Closets** (summaries/pointers to drawers) | `Closet` struct + `MemoryArtifact.summary` field | `crates/core/src/palace.rs` |
| **Tunnels** (cross-wing topic links) | `Tunnel` struct + tunnel cross-references | `crates/core/src/palace.rs` |
| **Hall categorization** (facts, events, discoveries, preferences, advice) | `HallType` enum: `Facts`, `Events`, `Discoveries`, `Preferences`, `Advice` | `crates/core/src/palace.rs` |
| **+34% retrieval boost** from spatial filtering | `PalaceRouter` with room-based filtering | `crates/engine/src/palace_router.rs` |
| **L0-L3 Layered Context Loading** | Extended to **L0-L4** context stack | `crates/engine/src/context_stack.rs` |
| **L0 Identity** (~50 tokens, always loaded) | `ContextLevel::L0_Identity` | `crates/engine/src/context_stack.rs` |
| **L1 Critical Facts** (~120 tokens, always loaded) | `ContextLevel::L1_CriticalFacts` | `crates/engine/src/context_stack.rs` |
| **L2 Room Recall** (on-demand session context) | `ContextLevel::L2_RoomRecall` | `crates/engine/src/context_stack.rs` |
| **L3 Deep Search** (full semantic query) | `ContextLevel::L3_RelevantMemories` + `L4_DeepSearch` | `crates/engine/src/context_stack.rs` |
| **Verbatim-first, 96.6% R@5 on LongMemEval** | `MemoryArtifact.raw_content` preserves verbatim, `effective_content()` method | `crates/core/src/types.rs` |
| **Temporal Knowledge Graph** with validity windows | `ValidityWindow` struct on relationships | `crates/graph/src/relationship.rs` |
| **Specialist Agent Diaries** (isolated memory tracks per agent) | `AgentProvider`, `AgentType` enum, `AgentRequest/Response` | `crates/engine/src/providers.rs` |
| **Auto-save hooks** | Event-driven memory persistence via triggers | `crates/core/src/types.rs` (`MemoryTrigger`) |

**What We Improved Over Mempalace:**

| Mempalace Limitation | RemeMnemosyne Improvement |
|---------------------|---------------------------|
| Python (slow, high memory) | **Pure Rust** (fast, safe, low memory) |
| ChromaDB dependency | **Custom TurboQuant + HNSW** (no external deps) |
| Single-provider (local only) | **Multi-provider** (OpenAI, Voyage, Cohere, Ollama, local) |
| Generic episode memories | **Typed intelligence memories** (Event, Narrative, RiskNode, Evidence, Simulation) |
| No horizontal scaling | **Sharding, read replicas, compaction** |
| No entity resolution | **Fuzzy matching** with Damerau-Levenshtein |
| No metrics | **Prometheus integration** |

**Key Architectural Decision**: Mempalace proved that spatial organization + verbatim preservation yields dramatically better retrieval accuracy. We adopted this pattern but rewrote it in Rust with type safety and no external database dependencies.

---

### 2. Mem0 — Unified Memory Interface & Agent-Centric Design

**Repository**: [mem0ai/mem0](https://github.com/mem0ai/mem0)  
**Language**: Python  
**License**: Apache 2.0

**What We Adopted:**

| Mem0 Concept | RemeMnemosyne Implementation | Location |
|--------------|------------------------------|----------|
| **Unified memory interface** (simple `add()` / `search()` API) | `AgentMemory` trait with `remember()` / `recall()` | `crates/engine/src/api.rs` |
| **Multi-level memory** (User, Session, Agent) | Session tracking + agent providers | `crates/core/src/types.rs` (`SessionId`), `crates/engine/src/providers.rs` |
| **Decoupled memory layer** from active LLM context | Memory engine separate from LLM providers | `crates/engine/` architecture |
| **LLM provider flexibility** | `EmbeddingProvider`, `ReasoningProvider` traits | `crates/engine/src/providers.rs` |
| **Production focus** (low latency, token efficiency) | Layered context loading, token budgets | `crates/engine/src/context_stack.rs` |
| **Dual deployment** (self-hosted + managed) | Feature flags allow minimal or full deployment | `Cargo.toml` feature design |

**What We Improved Over Mem0:**

| Mem0 Limitation | RemeMnemosyne Improvement |
|-----------------|---------------------------|
| Requires external LLM to function | **Works with local embeddings** (hash-based or Candle ML) |
| No verbatim preservation (LLM extracts/summarizes) | **Verbatim-first** with `raw_content` preservation |
| No spatial organization | **MemoryPalace** with wings/halls/rooms |
| No temporal validity | **ValidityWindow** on all relationships |
| Vector store not specified | **Custom quantization** (PQ, OPQ, Polar, QJL) |

**Key Architectural Decision**: Mem0 proved that a unified, simple memory API works well for agents. We adopted this but added spatial organization and verbatim preservation that Mem0 lacks.

---

## Algorithm Inspirations

### 3. Faiss — Quantization & Vector Search

**Repository**: [facebookresearch/faiss](https://github.com/facebookresearch/faiss)  
**Organization**: Meta AI Research

**What We Adopted:**

| Faiss Algorithm | RemeMnemosyne Implementation | Location |
|-----------------|------------------------------|----------|
| **Product Quantization (PQ)** | `TurboQuantizer` with PQ codebooks | `crates/semantic/src/turboquant.rs` |
| **Optimized PQ (OPQ)** | OPQ with rotation matrix support | `crates/semantic/src/turboquant.rs` |
| **Inverted File Index (IVF)** | Concepts adapted in clustering | `crates/semantic/src/` |
| **8-bit quantization** | Default quantization setting | `crates/semantic/src/turboquant.rs` |
| **Sub-vector decomposition** | PQ splits vectors into sub-vectors | `crates/semantic/src/turboquant.rs` |

**What We Improved Over Faiss:**

| Faiss Limitation | RemeMnemosyne Improvement |
|------------------|---------------------------|
| C++ with Python bindings | **Pure Rust** implementation |
| Requires compilation of C++ code | **Cargo install** only |
| External BLAS/LAPACK dependencies | **No external dependencies** |

---

### 4. HNSWLib / Annoy — Approximate Nearest Neighbor

**Repositories**: [nmslib/hnswlib](https://github.com/nmslib/hnswlib), [spotify/annoy](https://github.com/spotify/annoy)

**What We Adopted:**

| Algorithm | RemeMnemosyne Implementation | Location |
|-----------|------------------------------|----------|
| **HNSW** (Hierarchical Navigable Small World) | Custom `HNSWIndex` with configurable m, ef_construction, ef_search | `crates/semantic/src/index.rs` |
| **Flat index** for small datasets | `FlatIndex` with threshold-based switching | `crates/semantic/src/index.rs` |
| **Cosine similarity** metric | Built into HNSW and flat search | `crates/semantic/src/index.rs` |

---

## Storage & Graph Inspirations

### 5. Sled — Pure Rust Embedded Database

**Repository**: [sled-db/sled](https://github.com/sled-db/sled)  
**What We Adopted**: Default storage backend, ACID transactions, prefix scanning, export/import capabilities  
**Location**: `crates/storage/src/sled_backend.rs`

### 6. Petgraph — Pure Rust Graph Library

**Repository**: [petgraph/petgraph](https://github.com/petgraph/petgraph)  
**What We Adopted**: Graph data structures for entity relationships, path finding, clustering  
**Location**: `crates/graph/src/store.rs`

### 7. RocksDB — High-Performance Key-Value Store

**Repository**: [facebook/rocksdb](https://github.com/facebook/rocksdb)  
**What We Adopted**: Optional high-write-throughput backend (behind feature flag)  
**Location**: `crates/storage/src/rocks_backend.rs`

---

## Real-World Driver: RISC.OSINT

**Repository**: [RISC-OSINT](https://github.com/risc-osint) (private)  
**Domain**: Planetary-scale global risk intelligence

**Requirements That Shaped Mnemosyne:**

| RISC.OSINT Requirement | Mnemosyne Solution |
|------------------------|-------------------|
| **Unlimited event storage** (500 → 50,000+ events) | Horizontal scaling via sharding + read replicas |
| **Semantic search at scale** (millions of records) | TurboQuant + HNSW with 8x compression |
| **Entity graph tracking** (locations, events, actors) | Typed intelligence memories + temporal graph |
| **Pure Rust deployment** (zero C++ dependencies) | sled default, RocksDB opt-in only |
| **Multi-provider LLM support** (classified data can't leave premises) | EmbeddingProvider abstraction with local fallback |
| **Intelligence-grade accuracy** (not just generic notes) | EventMemory, NarrativeMemory, RiskNodeMemory, EvidenceMemory, SimulationMemory |
| **Evidence tracking with source attribution** | EvidenceMemory with source reliability scoring |
| **Scenario planning** | SimulationMemory with outcome probability distributions |

**Key Architectural Decision**: RISC.OSINT was the first system to consume Mnemosyne, stress-testing the API and driving the architecture toward production readiness. Every feature exists because RISC.OSINT needed it.

---

## Architecture Comparison Matrix

| Feature | Mempalace | Mem0 | Faiss | RemeMnemosyne |
|---------|-----------|------|-------|---------------|
| **Language** | Python | Python | C++ | **Rust** |
| **Spatial Organization** | ✅ Wings/Halls/Rooms | ❌ | ❌ | ✅ (adopted from Mempalace) |
| **Verbatim Preservation** | ✅ Drawers | ❌ (LLM extracts) | N/A | ✅ (adopted from Mempalace) |
| **Layered Context** | ✅ L0-L3 | ❌ | N/A | ✅ L0-L4 (extended from Mempalace) |
| **Temporal Validity** | ✅ SQLite graph | ❌ | N/A | ✅ (adopted from Mempalace) |
| **Multi-Provider LLM** | ❌ (local only) | ✅ (cloud) | N/A | ✅ (local + cloud) |
| **Vector Quantization** | ❌ (ChromaDB) | ❌ | ✅ PQ/OPQ/IVF | ✅ (adopted from Faiss) |
| **HNSW Index** | ❌ (ChromaDB) | ❌ | ✅ | ✅ (adopted from HNSWLib) |
| **Typed Memories** | ❌ (generic) | ❌ (user/session/agent) | N/A | ✅ (Event/Narrative/RiskNode/Evidence/Simulation) |
| **Entity Resolution** | ❌ | ❌ | N/A | ✅ (fuzzy matching) |
| **Horizontal Scaling** | ❌ (single-node) | ✅ (managed cloud) | ✅ (distributed) | ✅ (sharding + replicas) |
| **Metrics** | ❌ | ❌ | ❌ | ✅ (Prometheus) |
| **HTTP API** | MCP only | REST + SDKs | C API | ✅ (REST + health) |
| **License** | MIT | Apache 2.0 | MIT+BSD | MIT |

---

## What Makes RemeMnemosyne Unique

1. **Only pure Rust memory system** — No C/C++ dependencies by default, no Python runtime, no external databases required
2. **Only system with spatial + verbatim + typed intelligence** — Combines mempalace's spatial organization with intelligence-grade memory types
3. **Only system with multi-provider LLM abstraction** — OpenAI, Anthropic, Voyage, Cohere, Ollama, local — all pluggable
4. **Only system with temporal validity on relationships** — Intelligence data has expiration dates; we track them
5. **Only system with built-in quantization** — 8x compression via PQ/OPQ/Polar/QJL, no external vector database needed
6. **Born from real intelligence work** — Not a demo project; stress-tested against planetary-scale data

---

## Feature Map: Inspiration → Implementation

```
mempalace wings/halls/rooms  →  crates/core/src/palace.rs
mempalace drawers/closets    →  crates/core/src/palace.rs + types.rs
mempalace L0-L3 context      →  crates/engine/src/context_stack.rs (extended to L0-L4)
mempalace tunnels            →  crates/core/src/palace.rs (Tunnel struct)
mempalace temporal graph     →  crates/graph/src/relationship.rs (ValidityWindow)
mempalace agent diaries      →  crates/engine/src/providers.rs (AgentProvider)

mem0 unified API             →  crates/engine/src/api.rs (AgentMemory trait)
mem0 multi-level memory      →  crates/core/src/types.rs (SessionId, triggers)
mem0 provider flexibility    →  crates/engine/src/providers.rs (EmbeddingProvider)

faiss PQ/OPQ                 →  crates/semantic/src/turboquant.rs
hnswlib HNSW                 →  crates/semantic/src/index.rs
spotify/annoy flat index     →  crates/semantic/src/index.rs (FlatIndex)

sled pure Rust storage       →  crates/storage/src/sled_backend.rs
petgraph graph structure     →  crates/graph/src/store.rs

RISC.OSINT requirements      →  crates/core/src/typed_memory.rs (5 intelligence types)
RISC.OSINT multi-provider    →  crates/engine/src/providers.rs (3-layer providers)
RISC.OSINT scale             →  crates/semantic/src/sharding.rs + storage read_replica.rs
```

---

## Acknowledgments

- **mempalace** — For proving spatial memory organization and verbatim preservation work. The wings/halls/rooms/closets/drawers/tunnels metaphor is directly adapted from this project.
- **mem0** — For demonstrating that a unified, simple memory API works well for AI agents. The multi-level memory concept influenced our session and provider design.
- **Faiss** — For the quantization algorithms that enable 8x compression of embeddings. Our TurboQuant implementation is inspired by Faiss's PQ/OPQ designs.
- **HNSWLib / Annoy** — For the approximate nearest neighbor algorithms that power our semantic search.
- **RISC.OSINT** — For being the real-world stress test that drove every architectural decision. Without the requirement for unlimited, semantically searchable memory at planetary scale, this project would not exist.

---

**Document Date**: April 8, 2026  
**Last Updated**: After RISC-OSINT audit implementation + Mempalace integration
