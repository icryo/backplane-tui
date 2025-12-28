use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Build configuration for Dockerfile-based projects
#[derive(Debug, Clone, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default = "default_context")]
    pub context: String,
    #[serde(default = "default_dockerfile")]
    pub dockerfile: String,
}

fn default_context() -> String {
    ".".to_string()
}

fn default_dockerfile() -> String {
    "Dockerfile".to_string()
}

/// Route configuration for Traefik
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RouteConfig {
    pub path_prefix: String,
    #[serde(default = "default_true")]
    pub strip_prefix: bool,
    #[serde(default)]
    pub static_paths: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Project manifest from project.yaml
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectManifest {
    pub project: String,
    pub image: Option<String>,
    pub build: Option<BuildConfig>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub port: Option<u16>,
    #[serde(default)]
    pub route: RouteConfig,
    #[serde(default)]
    pub gpu: bool,
    #[serde(default)]
    pub volumes: Vec<String>,
    #[serde(default)]
    pub devices: Vec<String>,
    pub command: Option<String>,
}

impl ProjectManifest {
    /// Load a project manifest from a YAML file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context(format!("Failed to read manifest: {:?}", path.as_ref()))?;
        let manifest: ProjectManifest =
            serde_yaml::from_str(&content).context("Failed to parse manifest YAML")?;
        Ok(manifest)
    }

    /// Check if this is a CLI container (no port, typically "sleep infinity")
    pub fn is_cli(&self) -> bool {
        self.port.is_none()
            || self
                .command
                .as_ref()
                .map(|c| c.contains("sleep"))
                .unwrap_or(false)
    }
}

/// Scan for all project manifests in the base directory
pub fn scan_projects<P: AsRef<Path>>(base_dir: P) -> Result<Vec<ProjectManifest>> {
    let mut manifests = Vec::new();

    let entries = fs::read_dir(base_dir.as_ref())
        .context(format!("Failed to read directory: {:?}", base_dir.as_ref()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_path = path.join("project.yaml");
            if manifest_path.exists() {
                match ProjectManifest::load(&manifest_path) {
                    Ok(manifest) => manifests.push(manifest),
                    Err(e) => {
                        // Log but don't fail on individual manifest errors
                        eprintln!("Warning: Failed to load {:?}: {}", manifest_path, e);
                    }
                }
            }
        }
    }

    // Sort by project name
    manifests.sort_by(|a, b| a.project.cmp(&b.project));

    Ok(manifests)
}
