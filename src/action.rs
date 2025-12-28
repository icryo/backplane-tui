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

    // Views
    ViewLogs(String),
    ViewDetails,
    BackToList,

    // Modals
    ShowHelp,
    ShowConfirmDelete(String),
    ShowConfirmStop(String),
    CloseModal,
    ConfirmAction,

    // App control
    Refresh,
    Quit,
    Tick, // Timer tick for stats refresh

    // No action
    None,
}
