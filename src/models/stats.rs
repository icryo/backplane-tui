use std::process::Command;

/// Statistics for a single container
#[derive(Debug, Clone, Default)]
pub struct ContainerStats {
    pub cpu_percent: f64,
    pub memory_usage_mb: f64,
    pub memory_limit_mb: f64,
    pub memory_percent: f64,
    // Network I/O (cumulative bytes)
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    // Network I/O rates (bytes per second, calculated from delta)
    pub net_rx_rate: f64,
    pub net_tx_rate: f64,
}

/// System-wide statistics
#[derive(Debug, Clone, Default)]
pub struct SystemStats {
    pub cpu_percent: f32,
    pub memory_percent: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub disk_percent: f32,
    pub disk_used_gb: f32,
    pub disk_total_gb: f32,
    pub vram_percent: Option<f32>,
}

impl SystemStats {
    /// Get VRAM usage from nvidia-smi if available
    pub fn get_vram_percent() -> Option<f32> {
        let output = Command::new("nvidia-smi")
            .args(["--query-gpu=memory.used,memory.total", "--format=csv,noheader,nounits"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next()?;
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

        if parts.len() >= 2 {
            let used: f32 = parts[0].parse().ok()?;
            let total: f32 = parts[1].parse().ok()?;
            if total > 0.0 {
                return Some((used / total) * 100.0);
            }
        }

        None
    }
}
