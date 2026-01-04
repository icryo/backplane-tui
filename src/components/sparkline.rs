use std::collections::{HashMap, VecDeque};

/// Rolling history for sparkline display
#[derive(Debug, Clone, Default)]
pub struct StatsHistory {
    /// CPU history per container (container_name -> values)
    cpu: HashMap<String, VecDeque<f64>>,
    /// Memory history per container (container_name -> values)
    mem: HashMap<String, VecDeque<f64>>,
    /// Maximum samples to keep
    max_samples: usize,
}

impl StatsHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            cpu: HashMap::new(),
            mem: HashMap::new(),
            max_samples,
        }
    }

    /// Record a CPU sample for a container
    pub fn record_cpu(&mut self, container: &str, value: f64) {
        let history = self.cpu.entry(container.to_string()).or_insert_with(VecDeque::new);
        history.push_back(value);
        if history.len() > self.max_samples {
            history.pop_front(); // O(1) instead of O(n)
        }
    }

    /// Record a memory sample for a container
    pub fn record_mem(&mut self, container: &str, value: f64) {
        let history = self.mem.entry(container.to_string()).or_insert_with(VecDeque::new);
        history.push_back(value);
        if history.len() > self.max_samples {
            history.pop_front(); // O(1) instead of O(n)
        }
    }

    /// Get CPU history for a container
    pub fn get_cpu(&self, container: &str) -> Vec<f64> {
        self.cpu.get(container).map(|v| v.iter().copied().collect()).unwrap_or_default()
    }

    /// Get memory history for a container
    pub fn get_mem(&self, container: &str) -> Vec<f64> {
        self.mem.get(container).map(|v| v.iter().copied().collect()).unwrap_or_default()
    }

    /// Remove history for a container (when it's removed)
    pub fn remove(&mut self, container: &str) {
        self.cpu.remove(container);
        self.mem.remove(container);
    }

    /// Convert values to sparkline string
    pub fn to_sparkline(values: &[f64], width: usize) -> String {
        if values.is_empty() {
            return " ".repeat(width);
        }

        const CHARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

        // Take last `width` values
        let start = if values.len() > width {
            values.len() - width
        } else {
            0
        };
        let slice = &values[start..];

        // Find max for scaling (cap at 100 for percentages)
        let max = slice.iter().cloned().fold(0.0_f64, f64::max).max(1.0).min(100.0);

        let mut result = String::new();

        // Pad with spaces if we don't have enough values
        for _ in 0..(width.saturating_sub(slice.len())) {
            result.push(' ');
        }

        for &val in slice {
            let normalized = (val / max).min(1.0);
            let idx = ((normalized * (CHARS.len() - 1) as f64).round() as usize)
                .min(CHARS.len() - 1);
            result.push(CHARS[idx]);
        }

        result
    }
}
