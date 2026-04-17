use rememnemosyne_core::*;

use crate::config::IngestConfig;
use crate::db::{MessageInfo, PartInfo};

#[allow(dead_code)]
fn classify_importance_reexport() -> Importance {
    Importance::Medium
}

pub fn transform_message(msg: &MessageInfo, config: &IngestConfig) -> Vec<MemoryInput> {
    let mut items = Vec::new();

    if msg.parts.is_empty() {
        if let Some(_text) = extract_fallback_text(msg) {
            return items;
        }
        return items;
    }

    for part in &msg.parts {
        if let Some(input) = transform_part(&msg.id, &msg.role, part, config) {
            items.push(input);
        }
    }

    items
}

fn transform_part(
    message_id: &str,
    role: &str,
    part: &PartInfo,
    config: &IngestConfig,
) -> Option<MemoryInput> {
    let (content, part_type) = match part.part_type.as_str() {
        "text" | "reasoning" => {
            if part.part_type == "reasoning" && !config.include_reasoning {
                return None;
            }
            let text = part.text.as_deref().unwrap_or("").trim();
            if text.is_empty() {
                return None;
            }
            (text.to_string(), part.part_type.clone())
        }
        "tool" => {
            if !config.include_tool_outputs && !config.include_errors {
                return None;
            }

            if part.is_compacted && !config.include_compacted {
                return None;
            }

            let mut parts = Vec::new();

            if let Some(ref name) = part.tool_name {
                parts.push(format!("[{name}]"));
            }

            if let Some(ref input) = part.tool_input {
                let truncated = truncate_content(input, config.max_content_length);
                parts.push(format!("Input: {truncated}"));
            }

            if let Some(ref output) = part.tool_output {
                if config.include_tool_outputs {
                    let truncated = truncate_content(output, config.max_content_length);
                    parts.push(format!("Output: {truncated}"));
                }
            }

            if let Some(ref error) = part.tool_error {
                if config.include_errors {
                    parts.push(format!("Error: {error}"));
                }
            }

            if parts.is_empty() {
                return None;
            }

            (parts.join("\n"), "tool".to_string())
        }
        _ => return None,
    };

    Some(build_input(
        message_id,
        role,
        &part_type,
        &content,
        part.tool_name.as_deref(),
        config,
    ))
}

fn build_input(
    message_id: &str,
    role: &str,
    part_type: &str,
    content: &str,
    tool_name: Option<&str>,
    config: &IngestConfig,
) -> MemoryInput {
    let importance = classify_importance(role, part_type, tool_name, content);
    let trigger = classify_trigger(role, part_type);
    let memory_type = MemoryType::Semantic;
    let summary = generate_summary(role, part_type, tool_name, content);

    let mut tags = Vec::new();
    tags.push(format!("opencode"));
    tags.push(format!("role:{role}"));
    if let Some(name) = tool_name {
        tags.push(format!("tool:{name}"));
    }

    MemoryInput::new(content, trigger)
        .with_summary(summary)
        .with_type(memory_type)
        .with_importance(importance)
        .with_tags(tags)
        .with_namespace("opencode")
        .with_source_id(format!("opencode:{message_id}"))
}

fn classify_importance(
    role: &str,
    part_type: &str,
    tool_name: Option<&str>,
    content: &str,
) -> Importance {
    if part_type == "tool" {
        if tool_name.is_some() && content.contains("Error:") {
            return Importance::High;
        }
        if tool_name.is_some() {
            return Importance::High;
        }
    }

    if role == "user" {
        if content.len() > 200 {
            return Importance::High;
        }
        Importance::Medium
    } else if part_type == "reasoning" {
        Importance::Medium
    } else {
        Importance::Low
    }
}

fn classify_trigger(role: &str, part_type: &str) -> MemoryTrigger {
    match (role, part_type) {
        ("user", "text") => MemoryTrigger::UserInput,
        ("assistant", "tool") => MemoryTrigger::Decision,
        ("assistant", "reasoning") => MemoryTrigger::Insight,
        ("assistant", "text") => MemoryTrigger::SystemOutput,
        _ => MemoryTrigger::Custom("opencode_session".into()),
    }
}

fn generate_summary(role: &str, part_type: &str, tool_name: Option<&str>, content: &str) -> String {
    let prefix = match (role, part_type) {
        ("user", "text") => "User request",
        ("assistant", "tool") => {
            if let Some(name) = tool_name {
                return format!("Tool: {name}");
            }
            "Tool call"
        }
        ("assistant", "reasoning") => "Reasoning",
        ("assistant", "text") => "Response",
        _ => "Message",
    };

    let truncated: String = content.chars().take(80).collect();
    if content.len() > 80 {
        format!("{prefix}: {truncated}...")
    } else {
        format!("{prefix}: {truncated}")
    }
}

fn extract_fallback_text(_msg: &MessageInfo) -> Option<String> {
    None
}

fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        let truncated: String = content.chars().take(max_len).collect();
        format!("{truncated}...[truncated]")
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IngestStats {
    pub sessions_scanned: usize,
    pub messages_scanned: usize,
    pub messages_skipped: usize,
    pub memories_ingested: usize,
    pub memories_available: usize,
    pub checkpoints_created: usize,
    pub elapsed: std::time::Duration,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IngestPreview {
    pub sessions: usize,
    pub total_messages: usize,
    pub estimated_memories: usize,
}
