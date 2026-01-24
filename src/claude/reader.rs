use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use glob::glob;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use super::models::{Pricing, RawEntry, UsageEntry};

/// Find the Claude projects directory
pub fn find_claude_data_path() -> Option<PathBuf> {
    if let Some(home) = dirs::home_dir() {
        let primary = home.join(".claude").join("projects");
        if primary.exists() {
            return Some(primary);
        }

        let fallback = home.join(".config").join("claude").join("projects");
        if fallback.exists() {
            return Some(fallback);
        }
    }
    None
}

/// Extract project path from file path
/// e.g., ".../projects/-home-a-Desktop/session.jsonl" -> "/home/a/Desktop"
fn extract_project_path(file_path: &Path, data_path: &Path) -> String {
    if let Ok(rel) = file_path.strip_prefix(data_path) {
        if let Some(first) = rel.components().next() {
            let dir_name = first.as_os_str().to_string_lossy();
            if dir_name.starts_with('-') {
                return dir_name.replace('-', "/");
            }
            return dir_name.to_string();
        }
    }
    String::new()
}

/// Load all usage entries from JSONL files
pub fn load_usage_entries(data_path: &PathBuf, hours_back: u64) -> Result<Vec<UsageEntry>> {
    let cutoff_time = Utc::now() - chrono::Duration::hours(hours_back as i64);
    let pattern = data_path.join("**/*.jsonl");
    let pattern_str = pattern.to_string_lossy();

    let mut entries = Vec::new();

    for path in glob(&pattern_str).context("Failed to glob pattern")?.flatten() {
        let project_path = extract_project_path(&path, data_path);
        if let Ok(file_entries) = parse_jsonl_file(&path, &cutoff_time, &project_path) {
            entries.extend(file_entries);
        }
    }

    // Sort by timestamp
    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(entries)
}

/// Parse a single JSONL file
fn parse_jsonl_file(path: &Path, cutoff_time: &DateTime<Utc>, project_path: &str) -> Result<Vec<UsageEntry>> {
    let file = File::open(path).context("Failed to open JSONL file")?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        if line.trim().is_empty() {
            continue;
        }

        if let Some(entry) = parse_jsonl_line(&line, cutoff_time, project_path) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// Parse a single JSONL line into a UsageEntry
fn parse_jsonl_line(line: &str, cutoff_time: &DateTime<Utc>, project_path: &str) -> Option<UsageEntry> {
    let raw: RawEntry = serde_json::from_str(line).ok()?;

    let entry_type = raw.entry_type.as_deref()?;

    // Parse timestamp
    let timestamp_str = raw.timestamp.as_ref()?;
    let timestamp = DateTime::parse_from_rfc3339(timestamp_str)
        .ok()?
        .with_timezone(&Utc);

    // Skip old entries
    if timestamp < *cutoff_time {
        return None;
    }

    let cwd = raw.cwd.clone().unwrap_or_default();
    let session_id = raw.session_id.clone().unwrap_or_default();

    // Handle user entries - capture prompt text
    if entry_type == "user" {
        let message = raw.message.as_ref()?;
        let prompt_text = message.get_text();

        // Only create entry if we got a prompt (skip tool results etc)
        if prompt_text.is_some() {
            return Some(UsageEntry {
                timestamp,
                session_id,
                input_tokens: 0,
                output_tokens: 0,
                cache_creation_tokens: 0,
                cache_read_tokens: 0,
                cost_usd: 0.0,
                model: String::new(),
                cwd,
                project_path: project_path.to_string(),
                user_prompt: prompt_text,
            });
        }
        return None;
    }

    // Handle assistant entries - capture token usage
    if entry_type == "assistant" {
        let message = raw.message.as_ref()?;
        let usage = message.usage.as_ref()?;

        // Skip entries with no tokens
        if usage.input_tokens == 0 && usage.output_tokens == 0 {
            return None;
        }

        let model = message.model.clone().unwrap_or_default();

        let cost_usd = Pricing::calculate_cost(
            &model,
            usage.input_tokens,
            usage.output_tokens,
            usage.cache_creation_input_tokens,
            usage.cache_read_input_tokens,
        );

        return Some(UsageEntry {
            timestamp,
            session_id,
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cache_creation_tokens: usage.cache_creation_input_tokens,
            cache_read_tokens: usage.cache_read_input_tokens,
            cost_usd,
            model,
            cwd,
            project_path: project_path.to_string(),
            user_prompt: None,
        });
    }

    None
}
