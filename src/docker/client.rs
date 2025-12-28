use anyhow::{Context, Result};
use bollard::container::{
    Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions,
    RestartContainerOptions, StartContainerOptions, StopContainerOptions,
};
use bollard::image::ListImagesOptions;
use bollard::models::{HostConfig, PortBinding};
use bollard::Docker;
use std::collections::HashMap;

use crate::models::{ContainerInfo, ContainerStatus, PortMapping};

/// Wrapper around the bollard Docker client
pub struct DockerClient {
    client: Docker,
}

impl DockerClient {
    /// Connect to the Docker daemon
    pub fn connect() -> Result<Self> {
        let client = Docker::connect_with_socket_defaults()
            .context("Failed to connect to Docker daemon")?;
        Ok(Self { client })
    }

    /// Get the underlying bollard client (for stats/logs streaming)
    pub fn inner(&self) -> &Docker {
        &self.client
    }

    /// List all containers (running and stopped)
    pub async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        let mut filters = HashMap::new();
        // Include all containers, not just running ones
        filters.insert("status", vec!["running", "exited", "paused", "created", "restarting", "dead"]);

        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = self
            .client
            .list_containers(Some(options))
            .await
            .context("Failed to list containers")?;

        let mut result = Vec::new();
        for container in containers {
            let name = container
                .names
                .and_then(|names| names.first().cloned())
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_default();

            let image = container.image.unwrap_or_default();
            let state = container.state.unwrap_or_default();
            let status = ContainerStatus::from_docker_state(&state);

            // Extract all port mappings first (needed for is_cli check)
            let mut ports: Vec<PortMapping> = Vec::new();
            let mut first_port: Option<u16> = None;

            if let Some(port_list) = container.ports {
                for p in port_list {
                    let container_port = p.private_port;
                    if first_port.is_none() {
                        first_port = Some(container_port);
                    }

                    let host_port = p.public_port;
                    let protocol = p.typ
                        .map(|t| format!("{:?}", t).to_lowercase())
                        .unwrap_or_else(|| "tcp".to_string());

                    ports.push(PortMapping {
                        host_port,
                        container_port,
                        protocol,
                    });
                }
            }

            // Determine container type:
            // - CLI: no host-exposed ports (dev/utility containers)
            // - Web: has at least one host-exposed port (services)
            let has_exposed_ports = ports.iter().any(|p| p.host_port.is_some());
            let is_cli = !has_exposed_ports;

            result.push(ContainerInfo {
                id: container.id.unwrap_or_default(),
                name,
                image,
                status,
                is_cli,
                port: first_port,
                ports,
                stats: None,
                created: container.created,
            });
        }

        // Sort by name
        result.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(result)
    }

    /// Start a container
    pub async fn start_container(&self, name: &str) -> Result<()> {
        self.client
            .start_container(name, None::<StartContainerOptions<String>>)
            .await
            .context(format!("Failed to start container: {}", name))?;
        Ok(())
    }

    /// Stop a container
    pub async fn stop_container(&self, name: &str) -> Result<()> {
        let options = StopContainerOptions { t: 10 };
        self.client
            .stop_container(name, Some(options))
            .await
            .context(format!("Failed to stop container: {}", name))?;
        Ok(())
    }

    /// Restart a container
    pub async fn restart_container(&self, name: &str) -> Result<()> {
        let options = RestartContainerOptions { t: 10 };
        self.client
            .restart_container(name, Some(options))
            .await
            .context(format!("Failed to restart container: {}", name))?;
        Ok(())
    }

    /// Remove a container
    pub async fn remove_container(&self, name: &str) -> Result<()> {
        // First stop if running
        let _ = self.stop_container(name).await;

        let options = RemoveContainerOptions {
            force: true,
            ..Default::default()
        };
        self.client
            .remove_container(name, Some(options))
            .await
            .context(format!("Failed to remove container: {}", name))?;
        Ok(())
    }

    /// List all available images
    pub async fn list_images(&self) -> Result<Vec<String>> {
        let options = ListImagesOptions::<String> {
            all: false,
            ..Default::default()
        };

        let images = self
            .client
            .list_images(Some(options))
            .await
            .context("Failed to list images")?;

        let mut result: Vec<String> = images
            .into_iter()
            .filter_map(|img| {
                img.repo_tags
                    .into_iter()
                    .next()
                    .filter(|tag| tag != "<none>:<none>")
            })
            .collect();

        result.sort();
        Ok(result)
    }

    /// Create and start a new container
    pub async fn create_container(
        &self,
        name: &str,
        image: &str,
        port_host: Option<u16>,
        port_container: Option<u16>,
        env_vars: Vec<String>,
        volumes: Vec<String>,
        command: Option<String>,
    ) -> Result<()> {
        // Build port bindings
        let mut port_bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
        let exposed_ports: HashMap<String, HashMap<(), ()>>;

        if let (Some(host_port), Some(container_port)) = (port_host, port_container) {
            let container_port_key = format!("{}/tcp", container_port);
            port_bindings.insert(
                container_port_key.clone(),
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port.to_string()),
                }]),
            );
            let mut exposed = HashMap::new();
            exposed.insert(container_port_key, HashMap::new());
            exposed_ports = exposed;
        } else {
            exposed_ports = HashMap::new();
        }

        // Build host config
        let host_config = HostConfig {
            port_bindings: Some(port_bindings),
            binds: if volumes.is_empty() { None } else { Some(volumes) },
            restart_policy: Some(bollard::models::RestartPolicy {
                name: Some(bollard::models::RestartPolicyNameEnum::UNLESS_STOPPED),
                maximum_retry_count: None,
            }),
            ..Default::default()
        };

        // Parse command if provided
        let cmd = command.map(|c| {
            c.split_whitespace()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        });

        // Build container config
        let config = Config {
            image: Some(image.to_string()),
            env: if env_vars.is_empty() { None } else { Some(env_vars) },
            exposed_ports: Some(exposed_ports),
            host_config: Some(host_config),
            cmd,
            tty: Some(true),
            open_stdin: Some(true),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name,
            platform: None,
        };

        // Create the container
        self.client
            .create_container(Some(options), config)
            .await
            .context(format!("Failed to create container: {}", name))?;

        // Start the container
        self.client
            .start_container(name, None::<StartContainerOptions<String>>)
            .await
            .context(format!("Failed to start container: {}", name))?;

        Ok(())
    }
}
