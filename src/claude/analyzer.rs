use chrono::{Duration, Utc};
use std::collections::HashMap;

use super::models::{ModelStats, SessionBlock, UsageEntry};

const ACTIVE_THRESHOLD_MINUTES: i64 = 3;

/// Normalize model names for consistent tracking
pub fn normalize_model_name(model: &str) -> String {
    let lower = model.to_lowercase();

    if lower.contains("opus") {
        if lower.contains("4.5") || lower.contains("4-5") || lower.contains("4_5") {
            return "opus-4.5".to_string();
        }
        if lower.contains("4-") || lower.contains("4.") || lower.contains("4_") {
            return "opus-4".to_string();
        }
        return "opus-3".to_string();
    }

    if lower.contains("sonnet") {
        if lower.contains("4-") || lower.contains("4.") || lower.contains("4_") {
            return "sonnet-4".to_string();
        }
        if lower.contains("3.5") || lower.contains("3-5") || lower.contains("3_5") {
            return "sonnet-3.5".to_string();
        }
        return "sonnet-3".to_string();
    }

    if lower.contains("haiku") {
        if lower.contains("3.5") || lower.contains("3-5") || lower.contains("3_5") {
            return "haiku-3.5".to_string();
        }
        return "haiku-3".to_string();
    }

    model.to_string()
}

/// Analyze entries and group by sessionId
pub fn analyze_sessions(entries: Vec<UsageEntry>) -> Vec<SessionBlock> {
    if entries.is_empty() {
        return Vec::new();
    }

    // Group entries by session_id
    let mut sessions_map: HashMap<String, Vec<UsageEntry>> = HashMap::new();

    for entry in entries {
        let key = if entry.session_id.is_empty() {
            format!("{}:{}", entry.project_path, entry.timestamp.timestamp())
        } else {
            entry.session_id.clone()
        };

        sessions_map.entry(key).or_default().push(entry);
    }

    // Convert to SessionBlocks
    let now = Utc::now();
    let active_threshold = Duration::minutes(ACTIVE_THRESHOLD_MINUTES);

    let mut blocks: Vec<SessionBlock> = sessions_map
        .into_iter()
        .filter_map(|(session_id, mut entries)| {
            if entries.is_empty() {
                return None;
            }

            entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

            let start_time = entries.first()?.timestamp;
            let end_time = entries.last()?.timestamp;

            let is_active = now.signed_duration_since(end_time) < active_threshold;

            let project_path = entries.first().map(|e| e.project_path.clone()).unwrap_or_default();
            let cwd = entries.iter()
                .find(|e| !e.cwd.is_empty())
                .map(|e| e.cwd.clone())
                .unwrap_or_default();

            let mut total_tokens = 0u64;
            let mut cost_usd = 0.0f64;
            let mut per_model_stats = HashMap::new();
            let mut last_prompt: Option<String> = None;
            let mut assistant_count = 0u64;

            for entry in &entries {
                // Capture last user prompt
                if let Some(ref prompt) = entry.user_prompt {
                    last_prompt = Some(prompt.clone());
                }

                // Only count assistant entries for stats
                if entry.input_tokens > 0 || entry.output_tokens > 0 {
                    total_tokens += entry.total_tokens();
                    cost_usd += entry.cost_usd;
                    assistant_count += 1;

                    let model = normalize_model_name(&entry.model);
                    let stats: &mut ModelStats = per_model_stats.entry(model).or_default();
                    stats.add_entry(entry);
                }
            }

            let message_count = assistant_count;

            Some(SessionBlock {
                session_id,
                start_time,
                end_time,
                total_tokens,
                cost_usd,
                is_active,
                project_path,
                cwd,
                per_model_stats,
                message_count,
                last_prompt,
            })
        })
        .collect();

    // Sort by start_time descending (most recent first)
    blocks.sort_by(|a, b| b.start_time.cmp(&a.start_time));

    // Deduplicate by directory - keep only the most recent session per path
    let mut seen_paths: HashMap<String, usize> = HashMap::new();
    let mut deduplicated: Vec<SessionBlock> = Vec::new();

    for block in blocks {
        // Use cwd if available, otherwise project_path
        let path_key = if !block.cwd.is_empty() {
            block.cwd.clone()
        } else {
            block.project_path.clone()
        };

        if path_key.is_empty() {
            // Keep sessions with no path (shouldn't happen, but just in case)
            deduplicated.push(block);
        } else if !seen_paths.contains_key(&path_key) {
            seen_paths.insert(path_key, deduplicated.len());
            deduplicated.push(block);
        }
        // else: skip this session, we already have a more recent one for this path
    }

    deduplicated
}
