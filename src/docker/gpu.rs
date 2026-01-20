use std::collections::HashMap;
use std::process::Command;
use std::fs;

/// GPU process info from nvidia-smi
#[derive(Debug)]
struct GpuProcess {
    pid: u32,
    memory_mb: f64,
}

/// Get per-container GPU memory usage by mapping nvidia-smi PIDs to containers
/// Returns a HashMap of container_id -> total VRAM usage in MB
pub fn get_container_gpu_usage() -> HashMap<String, f64> {
    let mut container_vram: HashMap<String, f64> = HashMap::new();

    // Get GPU processes from nvidia-smi
    let gpu_processes = match get_gpu_processes() {
        Some(procs) => procs,
        None => return container_vram,
    };

    if gpu_processes.is_empty() {
        return container_vram;
    }

    // Map each PID to its container
    for proc in gpu_processes {
        if let Some(container_id) = pid_to_container_id(proc.pid) {
            *container_vram.entry(container_id).or_insert(0.0) += proc.memory_mb;
        }
    }

    container_vram
}

/// Query nvidia-smi for GPU compute processes
fn get_gpu_processes() -> Option<Vec<GpuProcess>> {
    // First try DCGM if available (more accurate for containers)
    if let Some(procs) = get_gpu_processes_dcgm() {
        return Some(procs);
    }

    // Fall back to nvidia-smi
    get_gpu_processes_nvidia_smi()
}

/// Try to get GPU processes via DCGM (dcgmi)
fn get_gpu_processes_dcgm() -> Option<Vec<GpuProcess>> {
    // Check if dcgmi is available and try to get process info
    // dcgmi process-stats requires DCGM daemon running
    let output = Command::new("dcgmi")
        .args(["discovery", "-l"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    // DCGM is available, try to get process stats
    // Note: This requires dcgm-exporter or introspection mode
    // For now, we'll use nvidia-smi as DCGM process stats need more setup
    None
}

/// Get GPU processes via nvidia-smi
fn get_gpu_processes_nvidia_smi() -> Option<Vec<GpuProcess>> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,used_memory",
            "--format=csv,noheader,nounits"
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut processes = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            if let (Ok(pid), Ok(memory_mb)) = (parts[0].parse::<u32>(), parts[1].parse::<f64>()) {
                processes.push(GpuProcess { pid, memory_mb });
            }
        }
    }

    Some(processes)
}

/// Map a PID to its container ID by reading cgroup info
fn pid_to_container_id(pid: u32) -> Option<String> {
    // Try cgroup v2 first (unified hierarchy)
    if let Some(id) = pid_to_container_id_cgroupv2(pid) {
        return Some(id);
    }

    // Fall back to cgroup v1
    pid_to_container_id_cgroupv1(pid)
}

/// Get container ID from cgroup v2
fn pid_to_container_id_cgroupv2(pid: u32) -> Option<String> {
    let cgroup_path = format!("/proc/{}/cgroup", pid);
    let content = fs::read_to_string(&cgroup_path).ok()?;

    // cgroup v2 format: "0::/path/to/cgroup"
    // Docker containers: "0::/docker/<container_id>"
    // or "0::/system.slice/docker-<container_id>.scope"
    for line in content.lines() {
        if line.starts_with("0::") {
            let path = &line[3..];

            // Check for docker container patterns
            if let Some(id) = extract_container_id_from_path(path) {
                return Some(id);
            }
        }
    }

    None
}

/// Get container ID from cgroup v1
fn pid_to_container_id_cgroupv1(pid: u32) -> Option<String> {
    let cgroup_path = format!("/proc/{}/cgroup", pid);
    let content = fs::read_to_string(&cgroup_path).ok()?;

    // cgroup v1 format: "N:controller:/path"
    // Look for docker in any controller
    for line in content.lines() {
        let parts: Vec<&str> = line.splitn(3, ':').collect();
        if parts.len() >= 3 {
            if let Some(id) = extract_container_id_from_path(parts[2]) {
                return Some(id);
            }
        }
    }

    None
}

/// Extract container ID from a cgroup path
fn extract_container_id_from_path(path: &str) -> Option<String> {
    // Pattern 1: /docker/<container_id>
    if let Some(pos) = path.find("/docker/") {
        let id_start = pos + 8;
        let id = &path[id_start..];
        // Container IDs are 64 hex chars, but we only need first 12 for matching
        if id.len() >= 12 && id.chars().take(12).all(|c| c.is_ascii_hexdigit()) {
            return Some(id.chars().take(64).collect());
        }
    }

    // Pattern 2: docker-<container_id>.scope
    if let Some(pos) = path.find("docker-") {
        let id_start = pos + 7;
        let remaining = &path[id_start..];
        if let Some(end) = remaining.find('.') {
            let id = &remaining[..end];
            if id.len() >= 12 && id.chars().all(|c| c.is_ascii_hexdigit()) {
                return Some(id.to_string());
            }
        }
    }

    // Pattern 3: /containerd/<container_id>
    if let Some(pos) = path.find("/containerd/") {
        let id_start = pos + 12;
        let id = &path[id_start..];
        if id.len() >= 12 {
            return Some(id.chars().take(64).collect());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_container_id_docker() {
        let path = "/docker/abc123def456789012345678901234567890123456789012345678901234";
        assert!(extract_container_id_from_path(path).is_some());
    }

    #[test]
    fn test_extract_container_id_scope() {
        let path = "/system.slice/docker-abc123def456.scope";
        assert!(extract_container_id_from_path(path).is_some());
    }
}
