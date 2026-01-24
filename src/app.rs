use anyhow::Result;
use sysinfo::{Disks, System};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::action::Action;
use crate::claude::{analyze_sessions, find_claude_data_path, load_usage_entries};
use crate::components::{
    ClaudeDashboard, ConfirmModal, ContainerList, CopyFilesModal, CreateContainerForm, CreateModal,
    CreateMode, ExecModal, FilterBar, Header, HelpModal, InfoModal, LogsView,
    ProcessesModal, RenameModal, StatsHistory, StatusBar,
};
use crate::components::confirm_modal::ConfirmAction;
use crate::components::claude_dashboard::resume_session;
use crate::docker::client::DockerClient;
use crate::docker::gpu::get_container_gpu_usage;
use crate::docker::logs::get_container_logs;
use crate::docker::stats::get_container_stats;
use crate::effects::EffectManager;
use crate::models::{ContainerInfo, SystemStats};

/// Current view mode
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    List,
    Logs,
    Create,
    Filter,
    Exec,
    Info,
    Rename,
    Processes,
    CopyFiles,
    ClaudeDashboard,
}

/// Container list view modes (horizontal scroll)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ListViewMode {
    #[default]
    Stats,    // Name, Type, Port, CPU bar, MEM bar
    Network,  // Name, ↓RX rate, ↑TX rate, Total RX, Total TX
    Details,  // Name, Image, Container ID, Uptime
}

/// Quick status filter for container list
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StatusFilter {
    #[default]
    All,      // Show all containers
    Groups,   // Show all, grouped by compose project with headers
    Running,  // Only running containers
    Stopped,  // Exited, dead, created (not running)
}

impl StatusFilter {
    /// Cycle to the next filter state
    pub fn cycle(&self) -> Self {
        match self {
            StatusFilter::All => StatusFilter::Groups,
            StatusFilter::Groups => StatusFilter::Running,
            StatusFilter::Running => StatusFilter::Stopped,
            StatusFilter::Stopped => StatusFilter::All,
        }
    }

    /// Get display name for the filter
    pub fn as_str(&self) -> &'static str {
        match self {
            StatusFilter::All => "All",
            StatusFilter::Groups => "Groups",
            StatusFilter::Running => "Running",
            StatusFilter::Stopped => "Stopped",
        }
    }
}

/// Active modal state
#[derive(Debug, Clone)]
pub enum ModalState {
    None,
    Help,
    Confirm(ConfirmAction),
}

/// Main application state
pub struct App {
    // Docker client
    docker: DockerClient,

    // View state
    pub view_mode: ViewMode,
    pub list_view_mode: ListViewMode,
    pub modal: ModalState,
    pub should_quit: bool,
    pub loading: bool,

    // Status filter (quick toggle with 'f')
    pub status_filter: StatusFilter,

    // Container data (auto-discovered)
    pub containers: Vec<ContainerInfo>,
    pub filtered_indices: Vec<usize>,

    // Logs data
    pub logs: Vec<String>,
    pub logs_container: String,

    // Create container form
    pub create_form: CreateContainerForm,

    // Filter
    pub filter: FilterBar,

    // Exec modal
    pub exec_modal: Option<ExecModal>,

    // Rename modal
    pub rename_modal: Option<RenameModal>,

    // Processes modal
    pub processes_modal: Option<ProcessesModal>,

    // Copy files modal
    pub copy_modal: Option<CopyFilesModal>,

    // Stats history for sparklines
    pub stats_history: StatsHistory,

    // System stats
    pub system_stats: SystemStats,

    // Components
    pub container_list: ContainerList,
    pub logs_view: LogsView,

    // System info
    sys: System,
    disks: Disks,

    // Refresh timing
    last_container_refresh: Instant,
    last_stats_refresh: Instant,
    last_vram_refresh: Instant,
    last_logs_refresh: Instant,
    container_refresh_interval: Duration,
    stats_refresh_interval: Duration,
    vram_refresh_interval: Duration,
    logs_refresh_interval: Duration,
    cached_vram: Option<f32>,
    /// Cached per-container GPU usage (container_id -> VRAM MB)
    cached_container_gpu: std::collections::HashMap<String, f64>,

    // Visual effects
    pub effects: EffectManager,

    // Claude dashboard
    pub claude_dashboard: ClaudeDashboard,
    claude_data_path: Option<std::path::PathBuf>,
    last_claude_refresh: Instant,
    claude_refresh_interval: Duration,
}

impl App {
    pub async fn new() -> Result<Self> {
        let docker = DockerClient::connect()?;
        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();

        let mut app = Self {
            docker,
            view_mode: ViewMode::List,
            list_view_mode: ListViewMode::Stats,
            modal: ModalState::None,
            should_quit: false,
            loading: false,
            status_filter: StatusFilter::All,
            containers: Vec::new(),
            filtered_indices: Vec::new(),
            logs: Vec::new(),
            logs_container: String::new(),
            create_form: CreateContainerForm::new(),
            filter: FilterBar::new(),
            exec_modal: None,
            rename_modal: None,
            processes_modal: None,
            copy_modal: None,
            stats_history: StatsHistory::new(30), // Keep 30 samples
            system_stats: SystemStats::default(),
            container_list: ContainerList::new(),
            logs_view: LogsView::new(),
            sys,
            disks,
            last_container_refresh: Instant::now() - Duration::from_secs(10),
            last_stats_refresh: Instant::now() - Duration::from_secs(10),
            last_vram_refresh: Instant::now() - Duration::from_secs(10),
            last_logs_refresh: Instant::now() - Duration::from_secs(10),
            container_refresh_interval: Duration::from_secs(3),
            stats_refresh_interval: Duration::from_secs(2),
            vram_refresh_interval: Duration::from_secs(5),
            logs_refresh_interval: Duration::from_secs(2),
            cached_vram: None,
            cached_container_gpu: HashMap::new(),
            effects: EffectManager::new(),
            claude_dashboard: ClaudeDashboard::new(),
            claude_data_path: find_claude_data_path(),
            last_claude_refresh: Instant::now() - Duration::from_secs(60),
            claude_refresh_interval: Duration::from_secs(10),
        };

        // Refresh system stats FIRST to populate GPU cache
        app.refresh_system_stats();
        app.refresh_containers().await?;
        app.update_filtered_indices();

        Ok(app)
    }

    /// Update filtered indices based on current filter and status filter
    pub fn update_filtered_indices(&mut self) {
        self.filtered_indices = self.containers
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                // Text filter
                if !self.filter.matches(&c.name) {
                    return false;
                }
                // Status filter
                match self.status_filter {
                    StatusFilter::All | StatusFilter::Groups => true,
                    StatusFilter::Running => c.status.is_running(),
                    StatusFilter::Stopped => !c.status.is_running(),
                }
            })
            .map(|(i, _)| i)
            .collect();

        // Adjust selection if needed
        if !self.filtered_indices.is_empty() {
            if let Some(selected) = self.container_list.selected() {
                if selected >= self.filtered_indices.len() {
                    self.container_list.state.select(Some(self.filtered_indices.len() - 1));
                }
            } else {
                self.container_list.state.select(Some(0));
            }
        } else {
            self.container_list.state.select(None);
        }
    }

    /// Get filtered containers
    pub fn filtered_containers(&self) -> Vec<&ContainerInfo> {
        self.filtered_indices
            .iter()
            .filter_map(|&i| self.containers.get(i))
            .collect()
    }

    pub async fn refresh_containers(&mut self) -> Result<()> {
        self.loading = true;
        self.last_container_refresh = Instant::now();

        let mut containers = self.docker.list_containers().await?;

        // Clone GPU cache to avoid borrow conflict
        let gpu_cache = self.cached_container_gpu.clone();

        for container in &mut containers {
            // Use is_active() to include paused containers (they still hold GPU memory)
            if container.status.is_active() {
                if let Ok(mut stats) = get_container_stats(self.docker.inner(), &container.name).await {
                    // Record history for sparklines
                    self.stats_history.record_cpu(&container.name, stats.cpu_percent);
                    self.stats_history.record_mem(&container.name, stats.memory_percent);
                    // Apply GPU usage if available
                    stats.vram_usage_mb = lookup_container_vram(&gpu_cache, &container.id);
                    container.stats = Some(stats);
                }
            }
        }

        self.containers = containers;
        self.update_filtered_indices();
        self.loading = false;

        Ok(())
    }

    pub async fn refresh_container_stats(&mut self) -> Result<()> {
        self.last_stats_refresh = Instant::now();

        // Clone GPU cache to avoid borrow conflict
        let gpu_cache = self.cached_container_gpu.clone();

        for container in &mut self.containers {
            // Use is_active() to include paused containers (they still hold GPU memory)
            if container.status.is_active() {
                if let Ok(mut stats) = get_container_stats(self.docker.inner(), &container.name).await {
                    // Record history for sparklines
                    self.stats_history.record_cpu(&container.name, stats.cpu_percent);
                    self.stats_history.record_mem(&container.name, stats.memory_percent);
                    // Apply GPU usage if available
                    stats.vram_usage_mb = lookup_container_vram(&gpu_cache, &container.id);
                    container.stats = Some(stats);
                }
            }
        }

        Ok(())
    }

    pub fn refresh_system_stats(&mut self) {
        self.sys.refresh_cpu_all();
        self.sys.refresh_memory();
        self.disks.refresh();

        let cpu_percent = self.sys.global_cpu_usage();
        let memory_used = self.sys.used_memory() as f32;
        let memory_total = self.sys.total_memory() as f32;
        let memory_percent = if memory_total > 0.0 {
            (memory_used / memory_total) * 100.0
        } else {
            0.0
        };

        // Sum all local physical disks (exclude tmpfs, overlay, network mounts, etc.)
        let (disk_used, disk_total) = self.disks.iter()
            .filter(|d| {
                let fs = d.file_system().to_string_lossy();
                let mount = d.mount_point().to_string_lossy();
                // Include real filesystems: ext4, xfs, btrfs, ntfs, etc.
                // Exclude: tmpfs, devtmpfs, overlay, squashfs, nfs, cifs, fuse
                let is_real_fs = matches!(fs.as_ref(),
                    "ext4" | "ext3" | "ext2" | "xfs" | "btrfs" | "ntfs" | "vfat" | "f2fs" | "zfs");
                // Exclude snap mounts and other system mounts
                let is_user_mount = !mount.starts_with("/snap") && !mount.starts_with("/boot");
                is_real_fs && is_user_mount
            })
            .fold((0.0f32, 0.0f32), |(used, total), d| {
                let d_total = d.total_space() as f32;
                let d_available = d.available_space() as f32;
                (used + (d_total - d_available), total + d_total)
            });

        let disk_percent = if disk_total > 0.0 {
            (disk_used / disk_total) * 100.0
        } else {
            0.0
        };

        // Throttle nvidia-smi calls - only refresh every 2 seconds
        let vram_percent = if self.last_vram_refresh.elapsed() >= self.vram_refresh_interval {
            self.last_vram_refresh = Instant::now();
            self.cached_vram = SystemStats::get_vram_percent();
            // Also refresh per-container GPU usage
            self.cached_container_gpu = get_container_gpu_usage();
            self.cached_vram
        } else {
            self.cached_vram
        };

        self.system_stats = SystemStats {
            cpu_percent,
            memory_percent,
            memory_used_gb: memory_used / 1024.0 / 1024.0 / 1024.0,
            memory_total_gb: memory_total / 1024.0 / 1024.0 / 1024.0,
            disk_percent,
            disk_used_gb: disk_used / 1024.0 / 1024.0 / 1024.0,
            disk_total_gb: disk_total / 1024.0 / 1024.0 / 1024.0,
            vram_percent,
        };
    }

    /// Refresh Claude sessions from JSONL files
    pub fn refresh_claude_sessions(&mut self) {
        self.last_claude_refresh = Instant::now();

        if let Some(ref data_path) = self.claude_data_path {
            // Load 720 hours (30 days) of session data
            if let Ok(entries) = load_usage_entries(data_path, 720) {
                let sessions = analyze_sessions(entries);
                self.claude_dashboard.update_sessions(sessions);
            }
        }
    }

    pub async fn load_logs(&mut self, container_name: &str) -> Result<()> {
        self.logs_container = container_name.to_string();
        self.logs = get_container_logs(self.docker.inner(), container_name, 500).await?;
        self.logs_view = LogsView::new();
        self.view_mode = ViewMode::Logs;
        Ok(())
    }

    pub async fn open_create_form(&mut self) -> Result<()> {
        self.create_form = CreateContainerForm::new();
        self.create_form.available_images = self.docker.list_images().await.unwrap_or_default();
        self.view_mode = ViewMode::Create;
        Ok(())
    }

    pub fn open_exec_modal(&mut self, container_name: String) {
        self.exec_modal = Some(ExecModal::new(container_name));
        self.view_mode = ViewMode::Exec;
    }

    pub async fn create_container_from_form(&mut self) -> Result<()> {
        let form = &self.create_form;

        if !form.is_valid() {
            return Ok(());
        }

        let port_host = form.port_host.parse::<u16>().ok();
        let port_container = form.port_container.parse::<u16>().ok();

        let env_vars: Vec<String> = if form.env_vars.is_empty() {
            Vec::new()
        } else {
            form.env_vars.split(',').map(|s| s.trim().to_string()).collect()
        };

        let volumes: Vec<String> = if form.volumes.is_empty() {
            Vec::new()
        } else {
            form.volumes.split(',').map(|s| s.trim().to_string()).collect()
        };

        let command = if form.command.is_empty() {
            None
        } else {
            Some(form.command.clone())
        };

        self.docker
            .create_container(
                &form.name,
                &form.image,
                port_host,
                port_container,
                env_vars,
                volumes,
                command,
            )
            .await?;

        self.view_mode = ViewMode::List;
        self.refresh_containers().await?;

        Ok(())
    }

    /// Get the currently selected container from filtered list
    pub fn selected_container(&self) -> Option<&ContainerInfo> {
        if self.status_filter == StatusFilter::Groups {
            // In groups mode, use the container index mapping
            self.container_list
                .selected_container_index()
                .and_then(|i| self.filtered_indices.get(i))
                .and_then(|&idx| self.containers.get(idx))
        } else {
            self.container_list
                .selected()
                .and_then(|i| self.filtered_indices.get(i))
                .and_then(|&idx| self.containers.get(idx))
        }
    }

    pub fn selected_container_name(&self) -> Option<String> {
        self.selected_container().map(|c| c.name.clone())
    }

    /// Get the item count for navigation (includes headers in groups mode)
    fn nav_item_count(&self) -> usize {
        if self.status_filter == StatusFilter::Groups {
            let list_count = self.container_list.item_count();
            if list_count > 0 {
                list_count
            } else {
                self.filtered_indices.len()
            }
        } else {
            self.filtered_indices.len()
        }
    }

    pub fn should_refresh_containers(&self) -> bool {
        self.last_container_refresh.elapsed() >= self.container_refresh_interval
    }

    pub fn should_refresh_stats(&self) -> bool {
        self.last_stats_refresh.elapsed() >= self.stats_refresh_interval
    }

    pub async fn tick(&mut self) -> Result<()> {
        if self.view_mode == ViewMode::Create || self.view_mode == ViewMode::Exec {
            return Ok(());
        }

        // Refresh system stats FIRST so GPU cache is populated before container stats
        self.refresh_system_stats();

        // Claude dashboard refresh
        if self.view_mode == ViewMode::ClaudeDashboard {
            if self.last_claude_refresh.elapsed() >= self.claude_refresh_interval {
                self.refresh_claude_sessions();
            }
            return Ok(());
        }

        if self.should_refresh_containers() {
            self.refresh_containers().await?;
        } else if self.should_refresh_stats() {
            self.refresh_container_stats().await?;
        }

        // Throttle log refreshes to every 2 seconds
        if self.view_mode == ViewMode::Logs && !self.logs_container.is_empty()
            && self.last_logs_refresh.elapsed() >= self.logs_refresh_interval {
            self.last_logs_refresh = Instant::now();
            if let Ok(logs) = get_container_logs(self.docker.inner(), &self.logs_container, 500).await {
                self.logs = logs;
            }
        }

        Ok(())
    }

    pub async fn handle_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                match self.view_mode {
                    ViewMode::Create => self.view_mode = ViewMode::List,
                    ViewMode::Filter => {
                        self.filter.deactivate();
                        self.update_filtered_indices();
                        self.view_mode = ViewMode::List;
                    }
                    ViewMode::Exec => {
                        self.exec_modal = None;
                        self.view_mode = ViewMode::List;
                    }
                    _ => self.should_quit = true,
                }
            }

            Action::Up => match self.view_mode {
                ViewMode::List | ViewMode::Filter => {
                    self.container_list.previous(self.nav_item_count())
                }
                ViewMode::ClaudeDashboard => {
                    self.claude_dashboard.select_prev();
                }
                ViewMode::Logs => self.logs_view.scroll_up(1),
                ViewMode::Create => {
                    if self.create_form.mode == CreateMode::ImageSelect {
                        self.create_form.prev_image();
                    } else {
                        self.create_form.prev_field();
                    }
                }
                ViewMode::Exec => {
                    if let Some(ref mut modal) = self.exec_modal {
                        modal.previous();
                    }
                }
                ViewMode::Info | ViewMode::Rename | ViewMode::CopyFiles => {} // No scrolling
                ViewMode::Processes => {
                    if let Some(ref mut modal) = self.processes_modal {
                        modal.scroll_up();
                    }
                }
            },

            Action::Down => match self.view_mode {
                ViewMode::List | ViewMode::Filter => {
                    self.container_list.next(self.nav_item_count())
                }
                ViewMode::ClaudeDashboard => {
                    self.claude_dashboard.select_next();
                }
                ViewMode::Logs => self.logs_view.scroll_down(1, self.logs.len()),
                ViewMode::Create => {
                    if self.create_form.mode == CreateMode::ImageSelect {
                        self.create_form.next_image();
                    } else {
                        self.create_form.next_field();
                    }
                }
                ViewMode::Exec => {
                    if let Some(ref mut modal) = self.exec_modal {
                        modal.next();
                    }
                }
                ViewMode::Info | ViewMode::Rename | ViewMode::CopyFiles => {} // No scrolling
                ViewMode::Processes => {
                    if let Some(ref mut modal) = self.processes_modal {
                        modal.scroll_down();
                    }
                }
            },

            Action::Top => match self.view_mode {
                ViewMode::List | ViewMode::Filter => self.container_list.top(),
                ViewMode::Logs => self.logs_view.top(),
                _ => {}
            },

            Action::Bottom => match self.view_mode {
                ViewMode::List | ViewMode::Filter => {
                    self.container_list.bottom(self.nav_item_count())
                }
                ViewMode::Logs => self.logs_view.bottom(self.logs.len()),
                _ => {}
            },

            Action::ViewLogs(name) => {
                self.load_logs(&name).await?;
            }

            Action::BackToList => {
                self.view_mode = ViewMode::List;
                self.logs.clear();
                self.logs_container.clear();
            }

            Action::ShowHelp => {
                self.modal = ModalState::Help;
            }

            Action::ShowConfirmDelete(name) => {
                self.modal = ModalState::Confirm(ConfirmAction::Delete(name));
            }

            Action::ShowConfirmStop(name) => {
                self.modal = ModalState::Confirm(ConfirmAction::Stop(name));
            }

            Action::CloseModal => {
                self.modal = ModalState::None;
            }

            Action::ConfirmAction => {
                if let ModalState::Confirm(ref confirm) = self.modal.clone() {
                    match confirm {
                        ConfirmAction::Delete(name) => {
                            self.docker.remove_container(name).await?;
                        }
                        ConfirmAction::Stop(name) => {
                            self.docker.stop_container(name).await?;
                        }
                    }
                    self.modal = ModalState::None;
                    self.refresh_containers().await?;
                }
            }

            Action::StartContainer(name) => {
                self.docker.start_container(&name).await?;
                self.effects.trigger_status_change(true);
                self.refresh_containers().await?;
            }

            Action::StopContainer(name) => {
                self.docker.stop_container(&name).await?;
                self.effects.trigger_status_change(false);
                self.refresh_containers().await?;
            }

            Action::RestartContainer(name) => {
                self.docker.restart_container(&name).await?;
                self.refresh_containers().await?;
            }

            Action::DeleteContainer(name) => {
                self.docker.remove_container(&name).await?;
                self.refresh_containers().await?;
            }

            Action::PauseContainer(name) => {
                self.docker.pause_container(&name).await?;
                self.refresh_containers().await?;
            }

            Action::UnpauseContainer(name) => {
                self.docker.unpause_container(&name).await?;
                self.refresh_containers().await?;
            }

            Action::RenameContainer(old_name, new_name) => {
                self.docker.rename_container(&old_name, &new_name).await?;
                self.refresh_containers().await?;
            }

            Action::ShowRename(name) => {
                self.rename_modal = Some(RenameModal::new(name));
                self.view_mode = ViewMode::Rename;
            }

            Action::ShowProcesses(name) => {
                if let Ok(processes) = self.docker.top_container(&name).await {
                    self.processes_modal = Some(ProcessesModal::new(name, processes));
                    self.view_mode = ViewMode::Processes;
                }
            }

            Action::ShowCopyFiles(name) => {
                self.copy_modal = Some(CopyFilesModal::new(name));
                self.view_mode = ViewMode::CopyFiles;
            }

            Action::CopyFromContainer(container, container_path, host_path) => {
                // Use docker cp command
                let _ = std::process::Command::new("docker")
                    .args(["cp", &format!("{}:{}", container, container_path), &host_path])
                    .status();
            }

            Action::CopyToContainer(container, host_path, container_path) => {
                // Use docker cp command
                let _ = std::process::Command::new("docker")
                    .args(["cp", &host_path, &format!("{}:{}", container, container_path)])
                    .status();
            }

            Action::Refresh => {
                self.refresh_containers().await?;
            }

            Action::CycleStatusFilter => {
                self.status_filter = self.status_filter.cycle();
                self.update_filtered_indices();
            }

            Action::ToggleClaudeDashboard => {
                if self.view_mode == ViewMode::ClaudeDashboard {
                    self.view_mode = ViewMode::List;
                } else {
                    self.view_mode = ViewMode::ClaudeDashboard;
                    // Refresh sessions if stale
                    if self.last_claude_refresh.elapsed() >= self.claude_refresh_interval {
                        self.refresh_claude_sessions();
                    }
                }
            }

            Action::RefreshClaudeSessions => {
                self.refresh_claude_sessions();
            }

            Action::ResumeClaudeSession => {
                if let Some(session) = self.claude_dashboard.selected_session() {
                    let session_id = session.session_id.clone();
                    let path = session.display_name().to_string();
                    resume_session(&session_id, &path);
                }
            }

            Action::Tick => {
                self.tick().await?;
            }

            Action::Left => {
                // Cycle list view mode backwards
                if self.view_mode == ViewMode::List {
                    self.list_view_mode = match self.list_view_mode {
                        ListViewMode::Stats => ListViewMode::Details,
                        ListViewMode::Network => ListViewMode::Stats,
                        ListViewMode::Details => ListViewMode::Network,
                    };
                }
            }

            Action::Right => {
                // Cycle list view mode forwards
                if self.view_mode == ViewMode::List {
                    self.list_view_mode = match self.list_view_mode {
                        ListViewMode::Stats => ListViewMode::Network,
                        ListViewMode::Network => ListViewMode::Details,
                        ListViewMode::Details => ListViewMode::Stats,
                    };
                }
            }

            _ => {}
        }

        Ok(())
    }

    pub fn render(&mut self, frame: &mut ratatui::Frame) {
        use crate::ui::layout::main_layout;
        use crate::ui::Theme;

        // Set background color
        let bg_block = ratatui::widgets::Block::default()
            .style(ratatui::prelude::Style::default().bg(Theme::BG));
        frame.render_widget(bg_block, frame.area());

        let (header_area, body, footer) = main_layout(frame.area());

        // Header with system stats
        Header::render(frame, header_area, &self.system_stats, self.system_stats.vram_percent, self.loading);

        // Main content area based on view mode
        match self.view_mode {
            ViewMode::List | ViewMode::Filter | ViewMode::Create | ViewMode::Exec | ViewMode::Info
            | ViewMode::Rename | ViewMode::Processes | ViewMode::CopyFiles => {
                // Full-width container list (with optional filter bar at bottom)
                let (list_area, filter_area) = if self.filter.active || self.view_mode == ViewMode::Filter {
                    let chunks = ratatui::prelude::Layout::default()
                        .direction(ratatui::prelude::Direction::Vertical)
                        .constraints([
                            ratatui::prelude::Constraint::Min(0),
                            ratatui::prelude::Constraint::Length(3),
                        ])
                        .split(body);
                    (chunks[0], Some(chunks[1]))
                } else {
                    (body, None)
                };

                // Container list (filtered) - full width with inline stats
                let filtered: Vec<ContainerInfo> = self.filtered_containers().into_iter().cloned().collect();
                let total_count = self.containers.len();
                self.container_list.render(frame, list_area, &filtered, self.list_view_mode, self.status_filter, total_count);

                // Filter bar
                if let Some(filter_rect) = filter_area {
                    self.filter.render(frame, filter_rect, self.filtered_indices.len(), self.containers.len());
                }
            }
            ViewMode::Logs => {
                // Full-screen logs view
                self.logs_view.focused = true;
                self.logs_view.render(frame, body, &self.logs, &self.logs_container);
            }
            ViewMode::ClaudeDashboard => {
                // Claude sessions dashboard
                self.claude_dashboard.render(frame, body);
            }
        }

        // Footer/Status bar
        let view_str = match self.view_mode {
            ViewMode::List => "list",
            ViewMode::Logs => "logs",
            ViewMode::Create => "create",
            ViewMode::Filter => "filter",
            ViewMode::Exec => "exec",
            ViewMode::Info => "info",
            ViewMode::ClaudeDashboard => "claude",
            ViewMode::Rename => "rename",
            ViewMode::Processes => "processes",
            ViewMode::CopyFiles => "copy",
        };
        StatusBar::render(frame, footer, view_str);

        // Modals (rendered last, on top)
        match &self.modal {
            ModalState::Help => HelpModal::render(frame, frame.area()),
            ModalState::Confirm(action) => ConfirmModal::render(frame, frame.area(), action),
            ModalState::None => {}
        }

        // Create modal
        if self.view_mode == ViewMode::Create {
            CreateModal::render(frame, frame.area(), &mut self.create_form);
        }

        // Exec modal
        if self.view_mode == ViewMode::Exec {
            if let Some(ref mut modal) = self.exec_modal {
                modal.render(frame, frame.area());
            }
        }

        // Info modal (network I/O)
        if self.view_mode == ViewMode::Info {
            InfoModal::render(frame, frame.area(), self.selected_container(), &self.stats_history);
        }

        // Rename modal
        if self.view_mode == ViewMode::Rename {
            if let Some(ref modal) = self.rename_modal {
                modal.render(frame, frame.area());
            }
        }

        // Processes modal
        if self.view_mode == ViewMode::Processes {
            if let Some(ref modal) = self.processes_modal {
                modal.render(frame, frame.area());
            }
        }

        // Copy files modal
        if self.view_mode == ViewMode::CopyFiles {
            if let Some(ref modal) = self.copy_modal {
                modal.render(frame, frame.area());
            }
        }
    }

    /// Render with visual effects
    pub fn render_with_effects(&mut self, frame: &mut ratatui::Frame, elapsed: Duration) {
        // First do the normal render
        self.render(frame);

        let area = frame.area();

        // Process startup fade-in effect (affects whole screen)
        self.effects.process(elapsed, frame.buffer_mut(), area);

        // Process loading effect on header area if loading
        if self.loading {
            let header_area = ratatui::prelude::Layout::default()
                .direction(ratatui::prelude::Direction::Vertical)
                .constraints([ratatui::prelude::Constraint::Length(3)])
                .split(area)[0];
            self.effects.process_loading(elapsed, frame.buffer_mut(), header_area, true);
        }

        // Use the same layout as render() to get correct body area
        let body_area = ratatui::prelude::Layout::default()
            .direction(ratatui::prelude::Direction::Vertical)
            .constraints([
                ratatui::prelude::Constraint::Length(1),  // header (1 line)
                ratatui::prelude::Constraint::Min(0),     // body
                ratatui::prelude::Constraint::Length(1),  // footer (1 line)
            ])
            .split(area)[1];

        self.effects.process_status(elapsed, frame.buffer_mut(), body_area);
    }
}

/// Lookup VRAM usage for a container from cached GPU metrics
fn lookup_container_vram(gpu_cache: &HashMap<String, f64>, container_id: &str) -> Option<f64> {
    // Try exact match first
    if let Some(&vram) = gpu_cache.get(container_id) {
        return Some(vram);
    }

    // Try prefix match (container IDs can be truncated)
    for (id, &vram) in gpu_cache {
        if id.starts_with(container_id) || container_id.starts_with(id) {
            return Some(vram);
        }
    }

    None
}
