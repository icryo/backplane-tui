/// Actions that can be performed in the application
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // Navigation
    Up,
    Down,
    Top,
    Bottom,
    Left,
    Right,

    // Container operations
    StartContainer(String),
    StopContainer(String),
    RestartContainer(String),
    DeleteContainer(String),
    PauseContainer(String),
    UnpauseContainer(String),
    RenameContainer(String, String), // (old_name, new_name)
    CopyFromContainer(String, String, String), // (container, container_path, host_path)
    CopyToContainer(String, String, String), // (container, host_path, container_path)

    // Views
    ViewLogs(String),
    ViewDetails,
    BackToList,

    // Modals
    ShowHelp,
    ShowConfirmDelete(String),
    ShowConfirmStop(String),
    ShowRename(String),
    ShowProcesses(String),
    ShowCopyFiles(String),
    CloseModal,
    ConfirmAction,

    // App control
    Refresh,
    Quit,
    Tick, // Timer tick for stats refresh
    CycleStatusFilter, // Cycle through All/Running/Stopped
    ToggleClaudeDashboard, // TAB to switch between containers and claude

    // Claude dashboard
    RefreshClaudeSessions,
    ResumeClaudeSession,

    // No action
    None,
}
