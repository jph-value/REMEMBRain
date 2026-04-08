/// Layered Context Loading Stack (L0-L4)
/// 
/// Implements mempalace's layered context loading strategy to minimize
/// context window usage. Instead of loading all memories at once, loads
/// progressively deeper layers only when needed.
/// 
/// Layers:
/// - L0 (Identity): ~50 tokens, AI's role/identity. Always loaded.
/// - L1 (Critical Facts): ~120 tokens, key facts/preferences. Always loaded.
/// - L2 (Room Recall): Recent sessions/current project data. On-demand.
/// - L3 (Relevant Memories): Semantic search results. Triggered by query.
/// - L4 (Deep Search): Full semantic queries across all data. Explicit request.
/// 
/// This yields ~10x token efficiency compared to loading everything.

use rememnemosyne_core::{MemoryArtifact, MemoryId, EntityId, Result, PalaceLocation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Context Layers
// ============================================================================

/// A single context layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextLayer {
    /// Layer level (0-4)
    pub level: ContextLevel,
    /// Layer name
    pub name: String,
    /// Token budget for this layer
    pub token_budget: usize,
    /// Whether this layer is always loaded
    pub always_loaded: bool,
    /// Content for this layer
    pub content: String,
    /// Source memories (if applicable)
    pub source_memories: Vec<MemoryId>,
    /// Source entities (if applicable)
    pub source_entities: Vec<EntityId>,
}

impl ContextLayer {
    /// Create a new context layer
    pub fn new(level: ContextLevel, name: impl Into<String>, token_budget: usize) -> Self {
        Self {
            level,
            name: name.into(),
            token_budget,
            always_loaded: matches!(level, ContextLevel::L0_Identity | ContextLevel::L1_CriticalFacts),
            content: String::new(),
            source_memories: Vec::new(),
            source_entities: Vec::new(),
        }
    }
    
    /// Add content to this layer
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }
    
    /// Add source memories
    pub fn with_memories(mut self, memories: Vec<MemoryId>) -> Self {
        self.source_memories = memories;
        self
    }
    
    /// Add source entities
    pub fn with_entities(mut self, entities: Vec<EntityId>) -> Self {
        self.source_entities = entities;
        self
    }
    
    /// Check if layer has content
    pub fn has_content(&self) -> bool {
        !self.content.is_empty()
    }
    
    /// Estimate token count
    pub fn estimate_tokens(&self) -> usize {
        // Rough estimate: 4 chars per token
        self.content.len() / 4
    }
    
    /// Check if within token budget
    pub fn within_budget(&self) -> bool {
        self.estimate_tokens() <= self.token_budget
    }
}

/// Context levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextLevel {
    /// L0: Identity (~50 tokens, always loaded)
    L0_Identity,
    /// L1: Critical Facts (~120 tokens, always loaded)
    L1_CriticalFacts,
    /// L2: Room Recall (recent sessions, on-demand)
    L2_RoomRecall,
    /// L3: Relevant Memories (semantic search, triggered)
    L3_RelevantMemories,
    /// L4: Deep Search (full semantic query, explicit)
    L4_DeepSearch,
}

impl ContextLevel {
    /// Get numeric level (0-4)
    pub fn level_num(&self) -> usize {
        match self {
            ContextLevel::L0_Identity => 0,
            ContextLevel::L1_CriticalFacts => 1,
            ContextLevel::L2_RoomRecall => 2,
            ContextLevel::L3_RelevantMemories => 3,
            ContextLevel::L4_DeepSearch => 4,
        }
    }
    
    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            ContextLevel::L0_Identity => "Identity and role definition",
            ContextLevel::L1_CriticalFacts => "Critical facts and preferences",
            ContextLevel::L2_RoomRecall => "Recent session context",
            ContextLevel::L3_RelevantMemories => "Relevant memories from semantic search",
            ContextLevel::L4_DeepSearch => "Deep search across all data",
        }
    }
}

// ============================================================================
/// Layered Context Stack
/// 
/// Manages the progressive loading of context layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayeredContextStack {
    /// All layers
    pub layers: HashMap<ContextLevel, ContextLayer>,
    /// Current loaded level (highest level loaded so far)
    pub current_level: ContextLevel,
    /// Total token budget
    pub total_token_budget: usize,
    /// Whether deep search has been triggered
    pub deep_search_triggered: bool,
}

impl LayeredContextStack {
    /// Create a new layered context stack
    pub fn new(total_token_budget: usize) -> Self {
        let mut layers = HashMap::new();
        
        // L0: Identity (50 tokens, always loaded)
        layers.insert(
            ContextLevel::L0_Identity,
            ContextLayer::new(ContextLevel::L0_Identity, "Identity", 50),
        );
        
        // L1: Critical Facts (120 tokens, always loaded)
        layers.insert(
            ContextLevel::L1_CriticalFacts,
            ContextLayer::new(ContextLevel::L1_CriticalFacts, "Critical Facts", 120),
        );
        
        // L2: Room Recall (variable, on-demand)
        layers.insert(
            ContextLevel::L2_RoomRecall,
            ContextLayer::new(ContextLevel::L2_RoomRecall, "Room Recall", 500),
        );
        
        // L3: Relevant Memories (variable, triggered)
        layers.insert(
            ContextLevel::L3_RelevantMemories,
            ContextLayer::new(ContextLevel::L3_RelevantMemories, "Relevant Memories", 2000),
        );
        
        // L4: Deep Search (variable, explicit)
        layers.insert(
            ContextLevel::L4_DeepSearch,
            ContextLayer::new(ContextLevel::L4_DeepSearch, "Deep Search", total_token_budget),
        );
        
        Self {
            layers,
            current_level: ContextLevel::L1_CriticalFacts, // Start with always-loaded layers
            total_token_budget,
            deep_search_triggered: false,
        }
    }
    
    /// Create with default budget (4000 tokens for large models)
    pub fn for_large_model() -> Self {
        Self::new(4000)
    }
    
    /// Create with medium budget (1500 tokens for 7B-13B models)
    pub fn for_medium_model() -> Self {
        Self::new(1500)
    }
    
    /// Create with small budget (500 tokens for 2B-4B models)
    pub fn for_small_model() -> Self {
        Self::new(500)
    }
    
    // ========================================================================
    // Layer Loading
    // ========================================================================
    
    /// Load L0: Identity layer
    pub fn load_identity(&mut self, identity: impl Into<String>) {
        if let Some(layer) = self.layers.get_mut(&ContextLevel::L0_Identity) {
            layer.content = identity.into();
        }
    }
    
    /// Load L1: Critical facts layer
    pub fn load_critical_facts(&mut self, facts: Vec<&MemoryArtifact>) {
        if let Some(layer) = self.layers.get_mut(&ContextLevel::L1_CriticalFacts) {
            let mut content = String::from("## Critical Facts\n");
            
            for memory in &facts {
                content.push_str(&format!("- {}\n", memory.summary));
                layer.source_memories.push(memory.id);
            }
            
            layer.content = content;
            self.current_level = ContextLevel::L1_CriticalFacts;
        }
    }
    
    /// Load L2: Room recall layer (recent session context)
    pub fn load_room_recall(&mut self, memories: Vec<&MemoryArtifact>, location: Option<&PalaceLocation>) {
        if let Some(layer) = self.layers.get_mut(&ContextLevel::L2_RoomRecall) {
            let mut content = if let Some(loc) = location {
                format!("## Recent Context: {}/{}\n", loc.hall, loc.room)
            } else {
                String::from("## Recent Context\n")
            };
            
            for memory in &memories {
                content.push_str(&format!("- {}\n", memory.summary));
                layer.source_memories.push(memory.id);
            }
            
            layer.content = content;
            self.current_level = ContextLevel::L2_RoomRecall;
        }
    }
    
    /// Load L3: Relevant memories from semantic search
    pub fn load_relevant_memories(&mut self, memories: Vec<&MemoryArtifact>, query: &str) {
        if let Some(layer) = self.layers.get_mut(&ContextLevel::L3_RelevantMemories) {
            let mut content = format!("## Relevant Memories: {}\n", query);
            
            for memory in &memories {
                content.push_str(&format!("- {}\n", memory.summary));
                layer.source_memories.push(memory.id);
            }
            
            layer.content = content;
            self.current_level = ContextLevel::L3_RelevantMemories;
        }
    }
    
    /// Load L4: Deep search (full context)
    pub fn load_deep_search(&mut self, content: impl Into<String>, memories: Vec<MemoryId>) {
        if let Some(layer) = self.layers.get_mut(&ContextLevel::L4_DeepSearch) {
            layer.content = content.into();
            layer.source_memories = memories;
            self.current_level = ContextLevel::L4_DeepSearch;
            self.deep_search_triggered = true;
        }
    }
    
    // ========================================================================
    // Context Assembly
    // ========================================================================
    
    /// Get context up to a specific level
    pub fn get_context_up_to(&self, level: ContextLevel) -> String {
        let mut context = String::new();
        
        for layer_level in &[
            ContextLevel::L0_Identity,
            ContextLevel::L1_CriticalFacts,
            ContextLevel::L2_RoomRecall,
            ContextLevel::L3_RelevantMemories,
            ContextLevel::L4_DeepSearch,
        ] {
            if layer_level.level_num() > level.level_num() {
                break;
            }
            
            if let Some(layer) = self.layers.get(layer_level) {
                if layer.has_content() {
                    context.push_str(&layer.content);
                    context.push('\n');
                }
            }
        }
        
        context
    }
    
    /// Get full context (all loaded layers)
    pub fn get_full_context(&self) -> String {
        self.get_context_up_to(self.current_level)
    }
    
    /// Get always-loaded context (L0 + L1 only)
    pub fn get_always_loaded(&self) -> String {
        self.get_context_up_to(ContextLevel::L1_CriticalFacts)
    }
    
    /// Check if can load more layers within budget
    pub fn can_load_layer(&self, level: ContextLevel) -> bool {
        let current_tokens = self.estimate_total_tokens();
        if let Some(layer) = self.layers.get(&level) {
            current_tokens + layer.estimate_tokens() <= self.total_token_budget
        } else {
            false
        }
    }
    
    /// Estimate total token count
    pub fn estimate_total_tokens(&self) -> usize {
        self.layers.values().map(|l| l.estimate_tokens()).sum()
    }
    
    /// Get current layer description
    pub fn current_layer_info(&self) -> String {
        format!(
            "Layer {}: {} ({} tokens estimated)",
            self.current_level.level_num(),
            self.current_level.description(),
            self.estimate_total_tokens()
        )
    }
    
    /// Reset to always-loaded layers (L0 + L1)
    pub fn reset_to_base(&mut self) {
        self.current_level = ContextLevel::L1_CriticalFacts;
        self.deep_search_triggered = false;
        
        // Clear L2-L4 content
        if let Some(layer) = self.layers.get_mut(&ContextLevel::L2_RoomRecall) {
            layer.content.clear();
            layer.source_memories.clear();
        }
        if let Some(layer) = self.layers.get_mut(&ContextLevel::L3_RelevantMemories) {
            layer.content.clear();
            layer.source_memories.clear();
        }
        if let Some(layer) = self.layers.get_mut(&ContextLevel::L4_DeepSearch) {
            layer.content.clear();
            layer.source_memories.clear();
        }
    }
    
    /// Get layer by level
    pub fn get_layer(&self, level: ContextLevel) -> Option<&ContextLayer> {
        self.layers.get(&level)
    }
    
    /// Check if deep search is available
    pub fn can_deep_search(&self) -> bool {
        self.can_load_layer(ContextLevel::L4_DeepSearch)
    }
}

impl Default for LayeredContextStack {
    fn default() -> Self {
        Self::for_large_model()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rememnemosyne_core::{MemoryType, MemoryTrigger, Importance};

    #[test]
    fn test_stack_creation() {
        let stack = LayeredContextStack::new(4000);
        assert_eq!(stack.layers.len(), 5);
        assert_eq!(stack.current_level, ContextLevel::L1_CriticalFacts);
    }

    #[test]
    fn test_load_identity() {
        let mut stack = LayeredContextStack::for_small_model();
        stack.load_identity("You are a helpful assistant.");
        
        let l0 = stack.get_layer(ContextLevel::L0_Identity).unwrap();
        assert!(l0.has_content());
        assert!(l0.within_budget());
    }

    #[test]
    fn test_load_critical_facts() {
        let mut stack = LayeredContextStack::default();
        
        let memory = MemoryArtifact::new(
            MemoryType::Semantic,
            "Rust is a systems language",
            "Rust content",
            vec![0.1; 128],
            MemoryTrigger::UserInput,
        )
        .with_importance(Importance::Critical);
        
        stack.load_critical_facts(vec![&memory]);
        
        let l1 = stack.get_layer(ContextLevel::L1_CriticalFacts).unwrap();
        assert!(l1.has_content());
        assert_eq!(stack.current_level, ContextLevel::L1_CriticalFacts);
    }

    #[test]
    fn test_context_assembly() {
        let mut stack = LayeredContextStack::for_medium_model();
        
        stack.load_identity("You are a risk analysis assistant.");
        
        let memory = MemoryArtifact::new(
            MemoryType::Semantic,
            "Critical risk fact",
            "Fact content",
            vec![0.1; 128],
            MemoryTrigger::UserInput,
        )
        .with_importance(Importance::Critical);
        
        stack.load_critical_facts(vec![&memory]);
        
        let base_context = stack.get_always_loaded();
        assert!(base_context.contains("risk analysis"));
        assert!(base_context.contains("Critical risk fact"));
    }

    #[test]
    fn test_layer_progression() {
        let mut stack = LayeredContextStack::for_large_model();
        
        // Start at L1
        assert_eq!(stack.current_level, ContextLevel::L1_CriticalFacts);
        
        // Load L2
        stack.load_room_recall(vec![], None);
        assert_eq!(stack.current_level, ContextLevel::L2_RoomRecall);
        
        // Load L3
        stack.load_relevant_memories(vec![], "test query");
        assert_eq!(stack.current_level, ContextLevel::L3_RelevantMemories);
        
        // Load L4
        stack.load_deep_search("Full context", vec![]);
        assert_eq!(stack.current_level, ContextLevel::L4_DeepSearch);
        assert!(stack.deep_search_triggered);
    }

    #[test]
    fn test_reset_to_base() {
        let mut stack = LayeredContextStack::default();
        
        stack.load_identity("Identity");
        stack.load_critical_facts(vec![]);
        stack.load_room_recall(vec![], None);
        stack.load_relevant_memories(vec![], "query");
        stack.load_deep_search("deep", vec![]);
        
        assert_eq!(stack.current_level, ContextLevel::L4_DeepSearch);
        
        stack.reset_to_base();
        
        assert_eq!(stack.current_level, ContextLevel::L1_CriticalFacts);
        assert!(!stack.deep_search_triggered);
    }

    #[test]
    fn test_token_budget_small_model() {
        let stack = LayeredContextStack::for_small_model();
        assert_eq!(stack.total_token_budget, 500);
        
        // L0 and L1 should fit
        let l0 = stack.get_layer(ContextLevel::L0_Identity).unwrap();
        assert_eq!(l0.token_budget, 50);
        
        let l1 = stack.get_layer(ContextLevel::L1_CriticalFacts).unwrap();
        assert_eq!(l1.token_budget, 120);
    }
}
