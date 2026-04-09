# RemeMnemosyne

A high-performance, pure Rust agentic memory engine for LLM agents — with spatial memory organization, verbatim preservation, multi-provider embeddings, and typed intelligence memories.

Born from **RISC.OSINT**, a planetary-scale global risk intelligence system.

---

## ✨ What Makes It Different

| Feature | What It Means |
|---------|---------------|
| **Spatial Memory Palace** | Wings → Halls → Rooms organization (spatial filtering concept from [mempalace](https://github.com/milla-jovovich/mempalace), fully re-engineered in Rust) |
| **Verbatim Preservation** | Raw content never altered, summaries separate (pattern from mempalace, extended with `effective_content()` API) |
| **L0-L4 Layered Context** | Progressive context loading (mempalace's L0-L3 extended to L0-L4 with `DeepSearch` layer) |
| **Multi-Provider Embeddings** | OpenAI, Voyage, Cohere, Ollama, Candle/local, or hash fallback — **our unique development** |
| **Typed Intelligence Memories** | Event, Narrative, RiskNode, Evidence, Simulation — **our unique development** for intelligence work |
| **Temporal Validity** | Entity relationships expire; intelligence data has expiration dates (temporal graph concept from mempalace, re-implemented in Rust) |
| **Pure Rust** | No C/C++ dependencies by default. No Python. No external databases required. |
| **Horizontal Scaling** | Sharding, read replicas, compaction, auto-pruning — **our unique development** |

---

## 📚 Inspiration & Lineage

Every major architectural decision traces to a specific project, with clear attribution of what was adopted, what was modified, and what is our unique development:

| Source Project | What We Adopted | What We Modified | What's Uniquely Ours |
|----------------|-----------------|------------------|---------------------|
| **[mempalace](https://github.com/milla-jovovich/mempalace)** (Python) | Spatial organization concept (wings/halls/rooms), verbatim drawers/closets pattern, L0-L3 context loading concept, tunnels concept, temporal graph concept, agent diaries concept | Re-engineered all concepts in Rust with type safety; extended L0-L3 → L0-L4; added `PalaceRouter` for programmatic routing; added `ValidityWindow` struct with invalidation/reactivation | `MemoryPalace` as a Rust data structure (mempalace uses files/SQLite); `PalaceQuery` API; `ContextStack` with token budgets; `EmbeddingProvider` integration; `ProviderRegistry` |
| **[mem0](https://github.com/mem0ai/mem0)** (Python) | Unified memory interface concept (simple `add`/`search` API), multi-level memory concept (user/session/agent), decoupled memory layer from LLM context | Made it LLM-provider-agnostic (mem0 requires an external LLM); added spatial organization; added verbatim preservation; added typed intelligence memories | `AgentMemory` trait; `EmbeddingProvider` trait; `ReasoningProvider` trait; `AgentProvider` trait; `ProviderRegistry` |
| **[Faiss](https://github.com/facebookresearch/faiss)** (C++, Meta Research) | Product Quantization (PQ) algorithm concept, Optimized PQ (OPQ) concept, 8-bit quantization, sub-vector decomposition | Re-implemented PQ/OPQ in pure Rust; added Polar Quantization; added QJL transforms; integrated with HNSW index | `TurboQuantizer` (pure Rust, no BLAS/LAPACK); seamless integration with `SemanticMemoryStore`; automatic training pipeline |
| **[HNSWLib](https://github.com/nmslib/hnswlib)** (C++) | HNSW approximate nearest neighbor algorithm concept | Re-implemented HNSW in pure Rust; added cosine similarity; added flat index fallback with auto-switching threshold | `HNSWIndex` (pure Rust, no external deps); `FlatIndex`; auto-switching logic based on data size |
| **[sled](https://github.com/sled-db/sled)** (Rust) | Pure Rust embedded database concept, ACID transactions, prefix scanning | Added `StorageBackend` trait for swappable backends; added snapshot management; added archive compression | `SledStorage` wrapper; `SnapshotManager`; `ArchiveCatalog` with zstd compression |
| **[petgraph](https://github.com/petgraph/petgraph)** (Rust) | Graph data structures for entity relationships | Added temporal validity windows; added fuzzy entity resolution; added relationship strength tracking | `GraphMemoryStore` with `ValidityWindow`; `EntityResolver` with Damerau-Levenshtein; `GraphRelationship` with evidence tracking |
| **RISC.OSINT** (private) | Real-world requirements: unlimited events, semantic search at scale, entity graph tracking, pure Rust deployment, multi-provider LLM support | — | **Everything driven by RISC.OSINT needs**: Typed Intelligence Memories, Provider Registry, Horizontal Scaling, Prometheus Metrics, HTTP API, Entity Resolution |

→ See **[ORIGINS_AND_INSPIRATIONS.md](ORIGINS_AND_INSPIRATIONS.md)** for the complete architectural lineage with feature-to-source mapping.

---

## 🏗️ Architecture

```
RemeMnemosyne/
├── crates/
│   ├── core          # Types, traits, errors, MemoryPalace, EmbeddingProvider
│   ├── semantic      # TurboQuant, HNSW index, Flat index, Sharding
│   ├── episodic      # Conversation episodes, sessions, decisions
│   ├── graph         # Entity relationships, temporal validity, entity resolution
│   ├── temporal      # Timeline events, time windows
│   ├── cognitive     # Micro-embeddings, Candle ML embeddings, intent detection
│   ├── storage       # sled (default), RocksDB (optional), backup, read replicas
│   └── engine        # Unified API, context stack, providers, palace router
```

---

## 🧠 Memory Types

| Type | Purpose | Key Features |
|------|---------|--------------|
| **Semantic** | Vector search | TurboQuant (PQ/OPQ/Polar/QJL, from Faiss concept, re-engineered in Rust), HNSW index (from HNSWLib concept, re-engineered in Rust), 8x compression |
| **Episodic** | Chat history | Sessions, episodes, exchanges, decisions, summaries |
| **Graph** | Entity relationships | petgraph-based, temporal validity windows (from mempalace concept, re-engineered), fuzzy entity resolution (our unique development) |
| **Temporal** | Events | Chronological storage, time windows, entity/memory linking |
| **Typed Intelligence** | RISC.OSINT memories | **Our unique development**: EventMemory, NarrativeMemory, RiskNodeMemory, EvidenceMemory, SimulationMemory |

---

## 🏰 Memory Palace (Spatial Organization)

The spatial memory organization concept was **originally demonstrated by [mempalace](https://github.com/milla-jovovich/mempalace)** in Python, which documented a +34% retrieval accuracy improvement from spatial filtering. We **re-engineered this concept in pure Rust** with type-safe structs, programmatic routing APIs, and integration with our embedding provider system.

```
MemoryPalace
├── Wing (Person / Project / Organization)
│   ├── Hall (Facts / Events / Discoveries / Preferences / Advice)
│   │   ├── Room (topic-specific)
│   │   │   ├── Drawers (raw verbatim content, never altered)
│   │   │   └── Closets (summaries/pointers to drawers)
│   │   └── Room
│   └── Hall
└── Tunnel (cross-wing topic links, e.g. "auth" across wings)
```

**What we added beyond the original concept:**
- `PalaceRouter` with programmatic routing and filtering
- `PalaceQuery` builder API for scoped queries
- Integration with `EmbeddingProvider` for spatial + semantic search
- `MemoryArtifact.palace_location` field for spatial metadata on all memories
- `PalaceStats` for monitoring palace health

---

## 🔄 Embedding Pipeline

**Our unique development.** RemeMnemosyne supports pluggable embedding providers — no vendor lock-in:

```
Text → EmbeddingProviderRouter → Arc<dyn EmbeddingProvider> → Vec<f32>
                                    ├── HashEmbedder (default, zero deps) — our implementation
                                    ├── CandleEmbedder (local ML, via feature flag) — our implementation
                                    ├── OpenAI (configurable) — trait interface ready
                                    ├── Voyage (configurable) — trait interface ready
                                    ├── Cohere (configurable) — trait interface ready
                                    ├── Ollama (configurable) — trait interface ready
                                    └── Custom (your API) — implement the trait
```

See **[EMBEDDING_PIPELINE.md](EMBEDDING_PIPELINE.md)** for the complete embedding architecture.

---

## 📊 Context Loading (L0-L4)

The layered context loading concept was **originally demonstrated by [mempalace](https://github.com/milla-jovovich/mempalace)** with L0-L3 layers. We **extended this to L0-L4** with a dedicated Deep Search layer and integrated it with our spatial memory palace for room-aware context retrieval.

| Layer | Tokens | When Loaded | Content |
|-------|--------|-------------|---------|
| **L0** Identity | ~50 | Always | AI role/identity |
| **L1** Critical Facts | ~120 | Always | Core facts, preferences |
| **L2** Room Recall | ~500 | On-demand | Recent session context |
| **L3** Relevant Memories | ~2000 | Query-triggered | Semantic search results |
| **L4** Deep Search | Variable | Explicit request | Full semantic across all data |

**What we added beyond the original concept:**
- L4 Deep Search layer (mempalace only had L0-L3)
- Token budget tracking per layer
- `reset_to_base()` for clearing L2-L4
- Integration with `PalaceRouter` for room-aware L2 loading
- Model-specific presets (`for_small_model()`, `for_medium_model()`, `for_large_model()`)

---

## 🚀 Quick Start

```toml
# Cargo.toml - Pure Rust (default, no external deps)
[dependencies]
rememnemosyne-engine = "0.1"

# With ML embeddings (Candle)
rememnemosyne-engine = { version = "0.1", features = ["candle-embeddings"] }

# Full production suite
rememnemosyne-engine = { version = "0.1", features = [
    "candle-embeddings",    # Real ML embeddings
    "entity-resolution",    # Fuzzy entity matching
    "backup-export",        # JSON import/export
    "metrics",              # Prometheus metrics
    "http-server",          # REST API
    "config-file",          # TOML config parsing
    "structured-logging",   # JSON logging
    "sharding",             # Horizontal scaling
    "read-replicas",        # Read scaling
    "compaction",           # Memory merging
    "auto-pruning",         # Tiered importance deletion
]}
```

```rust
use rememnemosyne_engine::RemeMnemosyneEngine;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = RemeMnemosyneEngine::default()?;

    // Store a memory (verbatim preserved automatically)
    engine.remember(
        "TurboQuant compresses vectors to 4-8 bits",
        "Quantization insight",
        MemoryTrigger::Insight,
    ).await?;

    // Recall with layered context loading
    let context = engine.recall("quantization").await?;
    println!("{}", engine.context_builder.format_context(&context));

    Ok(())
}
```

---

## ⚙️ Feature Flags

| Feature | Purpose | Default |
|---------|---------|---------|
| `sled-storage` | Pure Rust embedded storage | **Yes** |
| `archive` | Zstd-compressed memory archive | **Yes** |
| `candle-embeddings` | Real ML embeddings (Candle framework) | No |
| `entity-resolution` | Fuzzy entity matching (Damerau-Levenshtein) | No |
| `backup-export` | JSON import/export + NDJSON streaming | No |
| `metrics` | Prometheus counters, histograms, gauges | No |
| `http-server` | REST API with `/health` endpoint | No |
| `config-file` | TOML config file parsing | No |
| `structured-logging` | JSON structured log output | No |
| `sharding` | Split memories by entity type | No |
| `read-replicas` | Horizontal read scaling | No |
| `compaction` | Merge old related memories | No |
| `auto-pruning` | Tiered importance-based deletion | No |
| `persistence` | RocksDB backend (requires C++ toolchain) | No |

---

## 📈 Performance

| Operation | Target | Status |
|-----------|--------|--------|
| Micro-embedding | <1ms | ~0.1ms (hash-based) |
| Vector search (1K) | <3ms | ~2ms (HNSW) |
| Memory store | <5ms | ~1ms |
| Context assembly | <10ms | ~5ms |
| Spatial retrieval | +34% over flat | Documented by [mempalace](https://github.com/milla-jovovich/mempalace) on their Python implementation |

---

## 🔧 Key Components

### TurboQuant (Pure Rust)
- Product Quantization (PQ) — algorithm concept from [Faiss](https://github.com/facebookresearch/faiss), re-implemented in pure Rust
- Optimized PQ (OPQ) — algorithm concept from Faiss, re-implemented in pure Rust
- Polar Quantization — **our unique development**
- QJL transforms — **our unique development**
- 8x compression for embeddings
- No external dependencies

### HNSW Index (Pure Rust)
- HNSW algorithm — concept from [HNSWLib](https://github.com/nmslib/hnswlib), re-implemented in pure Rust
- Approximate nearest neighbor search
- Configurable m, ef_construction, ef_search
- Flat index fallback for small datasets
- Cosine similarity metric

### Memory Palace (Pure Rust)
- Spatial organization — concept from [mempalace](https://github.com/milla-jovovich/mempalace), fully re-engineered in Rust
- Wings/Halls/Rooms/Drawers/Closets/Tunnels
- `PalaceRouter` with room-based routing and filtering — **our unique development**
- `PalaceQuery` builder API — **our unique development**
- Tunnel cross-references across wings

### Embedding Providers
- `HashEmbedder` — Default fallback, zero dependencies — **our unique development**
- `CandleEmbedder` — Local ML via Candle framework — **our unique development**
- `EmbeddingProvider` trait — Pluggable interface for OpenAI, Voyage, Cohere, Ollama, custom — **our unique development**
- `EmbeddingProviderRouter` — Manages active provider, handles async Send safety — **our unique development**

### Typed Intelligence Memories
- `EventMemory` — Discrete events with severity, location, correlation — **our unique development for RISC.OSINT**
- `NarrativeMemory` — Evolving storylines with evidence links — **our unique development for RISC.OSINT**
- `RiskNodeMemory` — Risk entities with composite threat scoring — **our unique development for RISC.OSINT**
- `EvidenceMemory` — Attributed evidence with source reliability — **our unique development for RISC.OSINT**
- `SimulationMemory` — Scenario projections with outcome probabilities — **our unique development for RISC.OSINT**

---

## 📖 Documentation

| Document | Content |
|----------|---------|
| [BUILD.md](BUILD.md) | Build instructions, dependencies, reference repos |
| [ORIGINS_AND_INSPIRATIONS.md](ORIGINS_AND_INSPIRATIONS.md) | **Complete architectural lineage** — what came from where, what we modified, what's ours |
| [EMBEDDING_PIPELINE.md](EMBEDDING_PIPELINE.md) | Embedding provider architecture and usage |
| [RISC_OSINT_AUDIT_IMPLEMENTATION.md](RISC_OSINT_AUDIT_IMPLEMENTATION.md) | RISC-OSINT audit recommendations implemented |
| [RISC_OSINT_ARCHITECTURE_REFLECTION.md](RISC_OSINT_ARCHITECTURE_REFLECTION.md) | Architecture analysis and next steps |
| [ARCHITECTURE_SUMMARY.md](ARCHITECTURE_SUMMARY.md) | Mempalace integration summary |
| [CI_FIX.md](CI_FIX.md) | CI workflow troubleshooting |

---

## 🌍 Origin Story

Mnemosyne was born from a real-world need: **RISC.OSINT**, a planetary-scale intelligence system processing global risk data.

The requirements that shaped Mnemosyne:
- **Unlimited event storage** — RISC.OSINT needed to remove hard caps (500→50,000+ events)
- **Semantic search at scale** — Finding relevant past events from millions of records
- **Entity graph tracking** — Mapping relationships between locations, events, and actors
- **Pure Rust by default** — Zero C++ dependencies for simple deployment
- **Multi-provider LLM support** — Classified data cannot leave premises; local embeddings required
- **Intelligence-grade memory** — Not generic notes; typed memories for events, narratives, risks, evidence, simulations

RISC.OSINT was the first system to consume Mnemosyne, stress-testing the API and driving the architecture toward production readiness.

---

## 📄 License

MIT
