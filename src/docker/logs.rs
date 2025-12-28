use anyhow::Result;
use bollard::container::LogsOptions;
use bollard::Docker;
use futures_util::StreamExt;

/// Get logs from a container
pub async fn get_container_logs(
    docker: &Docker,
    container_name: &str,
    tail: usize,
) -> Result<Vec<String>> {
    let options = LogsOptions::<String> {
        stdout: true,
        stderr: true,
        tail: tail.to_string(),
        timestamps: true,
        ..Default::default()
    };

    let mut stream = docker.logs(container_name, Some(options));
    let mut logs = Vec::new();

    while let Some(result) = stream.next().await {
        match result {
            Ok(output) => {
                let line = output.to_string();
                // Clean up the log line (remove any control characters)
                let clean_line = line
                    .chars()
                    .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
                    .collect::<String>()
                    .trim()
                    .to_string();
                if !clean_line.is_empty() {
                    logs.push(clean_line);
                }
            }
            Err(_) => break,
        }
    }

    Ok(logs)
}
