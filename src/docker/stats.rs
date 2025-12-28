use anyhow::Result;
use bollard::container::StatsOptions;
use bollard::Docker;
use futures_util::StreamExt;

use crate::models::ContainerStats;

/// Get stats for a container (single snapshot)
pub async fn get_container_stats(docker: &Docker, container_name: &str) -> Result<ContainerStats> {
    let options = StatsOptions {
        stream: false,
        one_shot: true,
    };

    let mut stream = docker.stats(container_name, Some(options));

    if let Some(result) = stream.next().await {
        let stats = result?;

        // Calculate CPU percentage
        let cpu_percent = calculate_cpu_percent(&stats);

        // Calculate memory usage
        let memory_usage = stats
            .memory_stats
            .usage
            .unwrap_or(0) as f64;
        let memory_limit = stats
            .memory_stats
            .limit
            .unwrap_or(1) as f64;

        let memory_usage_mb = memory_usage / 1024.0 / 1024.0;
        let memory_limit_mb = memory_limit / 1024.0 / 1024.0;
        let memory_percent = if memory_limit > 0.0 {
            (memory_usage / memory_limit) * 100.0
        } else {
            0.0
        };

        // Calculate network I/O (sum across all interfaces)
        let (net_rx_bytes, net_tx_bytes) = if let Some(networks) = &stats.networks {
            let mut rx_total: u64 = 0;
            let mut tx_total: u64 = 0;
            for (_iface, net_stats) in networks {
                rx_total += net_stats.rx_bytes;
                tx_total += net_stats.tx_bytes;
            }
            (rx_total, tx_total)
        } else {
            (0, 0)
        };

        Ok(ContainerStats {
            cpu_percent,
            memory_usage_mb,
            memory_limit_mb,
            memory_percent,
            net_rx_bytes,
            net_tx_bytes,
            net_rx_rate: 0.0, // Rate calculated separately
            net_tx_rate: 0.0,
        })
    } else {
        Ok(ContainerStats::default())
    }
}

/// Calculate CPU percentage from Docker stats
fn calculate_cpu_percent(stats: &bollard::container::Stats) -> f64 {
    let cpu_stats = &stats.cpu_stats;
    let precpu_stats = &stats.precpu_stats;

    let cpu_delta = cpu_stats.cpu_usage.total_usage as f64
        - precpu_stats.cpu_usage.total_usage as f64;

    let system_delta = cpu_stats.system_cpu_usage.unwrap_or(0) as f64
        - precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

    let num_cpus = cpu_stats
        .online_cpus
        .or(cpu_stats.cpu_usage.percpu_usage.as_ref().map(|v| v.len() as u64))
        .unwrap_or(1) as f64;

    if system_delta > 0.0 && cpu_delta > 0.0 {
        (cpu_delta / system_delta) * num_cpus * 100.0
    } else {
        0.0
    }
}
