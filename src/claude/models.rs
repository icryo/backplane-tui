use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

/// Raw JSONL entry from Claude's log files
#[derive(Debug, Clone, Deserialize)]
pub struct RawEntry {
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(rename = "sessionId")]
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub message: Option<MessageData>,
    #[serde(rename = "type")]
    #[serde(default)]
    pub entry_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageData {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub usage: Option<UsageData>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub content: Option<serde_json::Value>,
}

impl MessageData {
    /// Extract text content from message
    pub fn get_text(&self) -> Option<String> {
        let content = self.content.as_ref()?;

        // Content can be a string or an array of content items
        if let Some(text) = content.as_str() {
            return Some(text.to_string());
        }

        if let Some(arr) = content.as_array() {
            // Find first text content item
            for item in arr {
                if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                    if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                        // Truncate long prompts
                        let truncated = if text.len() > 200 {
                            format!("{}...", &text[..200])
                        } else {
                            text.to_string()
                        };
                        return Some(truncated);
                    }
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageData {
    #[serde(default)]
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

/// Processed usage entry
#[derive(Debug, Clone)]
pub struct UsageEntry {
    pub timestamp: DateTime<Utc>,
    pub session_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost_usd: f64,
    pub model: String,
    pub cwd: String,
    pub project_path: String,
    /// User prompt text (only for user entries)
    pub user_prompt: Option<String>,
}

impl UsageEntry {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }
}

/// Per-model statistics
#[derive(Debug, Clone, Default)]
pub struct ModelStats {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub cost_usd: f64,
    pub entries_count: u64,
}

impl ModelStats {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens + self.output_tokens + self.cache_creation_tokens + self.cache_read_tokens
    }

    pub fn add_entry(&mut self, entry: &UsageEntry) {
        self.input_tokens += entry.input_tokens;
        self.output_tokens += entry.output_tokens;
        self.cache_creation_tokens += entry.cache_creation_tokens;
        self.cache_read_tokens += entry.cache_read_tokens;
        self.cost_usd += entry.cost_usd;
        self.entries_count += 1;
    }
}

/// A session as stored by Claude Code (identified by sessionId)
#[derive(Debug, Clone)]
pub struct SessionBlock {
    pub session_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_tokens: u64,
    pub cost_usd: f64,
    pub is_active: bool,
    pub project_path: String,
    pub cwd: String,
    pub per_model_stats: HashMap<String, ModelStats>,
    pub message_count: u64,
    /// Last user prompt in this session
    pub last_prompt: Option<String>,
}

impl Default for SessionBlock {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            start_time: Utc::now(),
            end_time: Utc::now(),
            total_tokens: 0,
            cost_usd: 0.0,
            is_active: false,
            project_path: String::new(),
            cwd: String::new(),
            per_model_stats: HashMap::new(),
            message_count: 0,
            last_prompt: None,
        }
    }
}

impl SessionBlock {
    pub fn duration_minutes(&self) -> f64 {
        let duration = self.end_time.signed_duration_since(self.start_time);
        (duration.num_seconds() as f64 / 60.0).max(1.0)
    }

    pub fn burn_rate(&self) -> f64 {
        if self.duration_minutes() > 0.0 {
            self.total_tokens as f64 / self.duration_minutes()
        } else {
            0.0
        }
    }

    /// Display name - prefer cwd, fall back to project_path
    pub fn display_name(&self) -> &str {
        if !self.cwd.is_empty() {
            &self.cwd
        } else if !self.project_path.is_empty() {
            &self.project_path
        } else {
            "(unknown)"
        }
    }

    /// Get short project name from path
    pub fn short_name(&self, max_len: usize) -> String {
        let path = self.display_name();
        if path.is_empty() || path == "(unknown)" {
            return String::new();
        }

        let name = path.rsplit('/').next().unwrap_or(path);
        if name.len() <= max_len {
            name.to_string()
        } else {
            format!("{}…", &name[..max_len - 1])
        }
    }
}

/// Pricing constants (per million tokens)
pub struct Pricing;

impl Pricing {
    pub fn calculate_cost(model: &str, input: u64, output: u64, cache_create: u64, cache_read: u64) -> f64 {
        let model_lower = model.to_lowercase();

        let (input_price, output_price) = if model_lower.contains("opus") {
            (15.0, 75.0)
        } else if model_lower.contains("haiku") {
            (0.25, 1.25)
        } else {
            // Default to Sonnet pricing
            (3.0, 15.0)
        };

        let cache_create_price = input_price * 1.25;
        let cache_read_price = input_price * 0.1;

        (input as f64 * input_price
            + output as f64 * output_price
            + cache_create as f64 * cache_create_price
            + cache_read as f64 * cache_read_price)
            / 1_000_000.0
    }
}
