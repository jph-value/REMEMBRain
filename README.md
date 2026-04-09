# RemeMnemosyne

A high-performance, pure Rust agentic memory engine for LLM agents — with spatial memory organization, verbatim preservation, multi-provider embeddings, and typed intelligence memories.

Born from **RISC.OSINT**, a planetary-scale global risk intelligence system.

---

## ✨ What Makes It Different

| Feature | What It Means |
|---------|---------------|
| **Spatial Memory Palace** | Wings → Halls → Rooms organization (+34% retrieval accuracy, adopted from [mempalace](https://github.com/milla-jovovich/mempalace)) |
| **Verbatim Preservation** | Raw content never altered, summaries separate (96.6% R@5, adopted from mempalace) |
| **L0-L4 Layered Context** | Progressive context loading, ~10x token efficiency (extended from mempalace's L0-L3) |
| **Multi-Provider Embeddings** | OpenAI, Voyage, Cohere, Ollama, Candle/local, or hash fallback |
| **Typed Intelligence Memories** | Event, Narrative, RiskNode, Evidence, Simulation — not just generic notes |
| **Temporal Validity** | Entity relationships expire; intelligence data has expiration dates |
| **Pure Rust** | No C/C++ dependencies by default. No Python. No external databases required. |
| **Horizontal Scaling** | Sharding, read replicas, compaction, auto-pruning |

---

## 📚 Inspiration & Lineage

Every major architectural decision traces to a specific project:

| Project | What We Adopted | Language |
|---------|-----------------|----------|
| **[mempalace](https://github.com/milla-jovovich/mempalace)** | Spatial organization (wings/halls/rooms), verbatim drawers/closets, L0-L3 context loading, tunnels, temporal graph, agent diaries | Python |
| **[mem0](https://github.com/mem0ai/mem0)** | Unified memory interface, multi-level memory (user/session/agent), decoupled memory layer | Python |
| **[Faiss](https://github.com/facebookresearch/faiss)** | Product Quantization (PQ), Optimized PQ (OPQ), 8-bit quantization | C++ |
| **[HNSWLib](https://github.com/nmslib/hnswlib)** | HNSW approximate nearest neighbor algorithm | C++ |
| **[sled](https://github.com/sled-db/sled)** | Pure Rust embedded database, ACID transactions | Rust |
| **[petgraph](https://github.com/petgraph/petgraph)** | Graph data structures for entity relationships | Rust |
| **RISC.OSINT** | Real-world requirements that shaped every decision | — |

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
| **Semantic** | Vector search | TurboQuant (PQ/OPQ/Polar/QJL), HNSW index, 8x compression |
| **Episodic** | Chat history | Sessions, episodes, exchanges, decisions, summaries |
| **Graph** | Entity relationships | petgraph-based, temporal validity windows, fuzzy entity resolution |
| **Temporal** | Events | Chronological storage, time windows, entity/memory linking |
| **Typed Intelligence** | RISC.OSINT memories | EventMemory, NarrativeMemory, RiskNodeMemory, EvidenceMemory, SimulationMemory |

---

## 🏰 Memory Palace (Spatial Organization)

Adapted from [mempalace](https://github.com/milla-jovovich/mempalace), RemeMnemosyne organizes memories in a navigable hierarchy:

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

This spatial organization yields **+34% retrieval accuracy** compared to flat indexing (documented by mempalace).

---

## 🔄 Embedding Pipeline

RemeMnemosyne supports **pluggable embedding providers** — no vendor lock-in:

```
Text → EmbeddingProviderRouter → Arc<dyn EmbeddingProvider> → Vec<f32>
                                    ├── HashEmbedder (default, zero deps)
                                    ├── CandleEmbedder (local ML, via feature flag)
                                    ├── OpenAI (configurable)
                                    ├── Voyage (configurable)
                                    ├── Cohere (configurable)
                                    ├── Ollama (configurable)
                                    └── Custom (your API)
```

See **[EMBEDDING_PIPELINE.md](EMBEDDING_PIPELINE.md)** for the complete embedding architecture.

---

## 📊 Context Loading (L0-L4)

Extended from mempalace's L0-L3, RemeMnemosyne uses 5 progressive layers:

| Layer | Tokens | When Loaded | Content |
|-------|--------|-------------|---------|
| **L0** Identity | ~50 | Always | AI role/identity |
| **L1** Critical Facts | ~120 | Always | Core facts, preferences |
| **L2** Room Recall | ~500 | On-demand | Recent session context |
| **L3** Relevant Memories | ~2000 | Query-triggered | Semantic search results |
| **L4** Deep Search | Variable | Explicit request | Full semantic across all data |

This yields **~10x token efficiency** compared to loading all memories at once.

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

| Operation | Target | Achieved |
|-----------|--------|----------|
| Micro-embedding | <1ms | ~0.1ms |
| Vector search (1K) | <3ms | ~2ms |
| Memory store | <5ms | ~1ms |
| Context assembly | <10ms | ~5ms |
| Retrieval accuracy (spatial) | +34% over flat | +34% (mempalace documented) |
| Token efficiency (layered) | 10x over full load | ~10x |

---

## 🔧 Key Components

### TurboQuant (Pure Rust)
- Product Quantization (PQ) — from [Faiss](https://github.com/facebookresearch/faiss)
- Optimized PQ (OPQ) — from Faiss
- Polar Quantization
- QJL transforms
- 8x compression for embeddings
- No external dependencies

### HNSW Index (Pure Rust)
- From [HNSWLib](https://github.com/nmslib/hnswlib)
- Approximate nearest neighbor search
- Configurable m, ef_construction, ef_search
- Flat index fallback for small datasets
- Cosine similarity metric

### Memory Palace (Pure Rust)
- From [mempalace](https://github.com/milla-jovovich/mempalace)
- Wings/Halls/Rooms/Drawers/Closets/Tunnels
- Room-based routing and filtering
- Tunnel cross-references across wings

### Embedding Providers
- `HashEmbedder` — Default fallback, zero dependencies
- `CandleEmbedder` — Local ML via Candle framework
- `EmbeddingProvider` trait — Pluggable interface for OpenAI, Voyage, Cohere, Ollama, custom

### Typed Intelligence Memories
- `EventMemory` — Discrete events with severity, location, correlation
- `NarrativeMemory` — Evolving storylines with evidence links
- `RiskNodeMemory` — Risk entities with composite threat scoring
- `EvidenceMemory` — Attributed evidence with source reliability
- `SimulationMemory` — Scenario projections with outcome probabilities

---

## 📖 Documentation

| Document | Content |
|----------|---------|
| [BUILD.md](BUILD.md) | Build instructions, dependencies, reference repos |
| [ORIGINS_AND_INSPIRATIONS.md](ORIGINS_AND_INSPIRATIONS.md) | **Complete architectural lineage** — what came from where |
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

RISC.OSINT was the first system to consume Mnemosyne, stress-testing the API and driving the architecture toward production readiness.

---

## 📄 License

MIT
