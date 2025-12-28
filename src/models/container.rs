use super::ContainerStats;

/// Status of a Docker container
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ContainerStatus {
    Running,
    Exited,
    Paused,
    Created,
    Restarting,
    Removing,
    Dead,
    #[default]
    NotDeployed,
}

impl ContainerStatus {
    pub fn from_docker_state(state: &str) -> Self {
        match state.to_lowercase().as_str() {
            "running" => Self::Running,
            "exited" => Self::Exited,
            "paused" => Self::Paused,
            "created" => Self::Created,
            "restarting" => Self::Restarting,
            "removing" => Self::Removing,
            "dead" => Self::Dead,
            _ => Self::NotDeployed,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Exited => "exited",
            Self::Paused => "paused",
            Self::Created => "created",
            Self::Restarting => "restarting",
            Self::Removing => "removing",
            Self::Dead => "dead",
            Self::NotDeployed => "not deployed",
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }
}

/// Port mapping info
#[derive(Debug, Clone)]
pub struct PortMapping {
    pub host_port: Option<u16>,
    pub container_port: u16,
    pub protocol: String,
}

impl PortMapping {
    /// Format as "host:container/proto" or just "container/proto" if no host
    pub fn display(&self) -> String {
        match self.host_port {
            Some(hp) => format!("{}:{}/{}", hp, self.container_port, self.protocol),
            None => format!("{}/{}", self.container_port, self.protocol),
        }
    }
}

/// Information about a container
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: ContainerStatus,
    pub is_cli: bool,
    pub port: Option<u16>,
    pub ports: Vec<PortMapping>,
    pub stats: Option<ContainerStats>,
    pub created: Option<i64>,
}

impl ContainerInfo {
    pub fn new(name: String) -> Self {
        Self {
            id: String::new(),
            name,
            image: String::new(),
            status: ContainerStatus::NotDeployed,
            is_cli: false,
            port: None,
            ports: Vec::new(),
            stats: None,
            created: None,
        }
    }
}
