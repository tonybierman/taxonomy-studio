/// Represents a pending action waiting for confirmation
#[derive(Debug, Clone)]
pub enum PendingAction {
    Open,
    New,
    Exit,
}

/// Represents an action for simple confirmation dialog
#[derive(Debug, Clone)]
pub enum SimpleConfirmationAction {
    Revert,
}

/// UI flow state management
/// This struct contains only UI-specific state (dialogs, pending actions)
#[derive(Debug)]
pub struct UiState {
    /// Pending action awaiting user confirmation
    pub pending_action: Option<PendingAction>,
    /// Simple confirmation action
    pub simple_confirmation_action: Option<SimpleConfirmationAction>,
}

#[allow(dead_code)]
impl UiState {
    pub fn new() -> Self {
        Self {
            pending_action: None,
            simple_confirmation_action: None,
        }
    }

    /// Set a pending action
    pub fn set_pending(&mut self, action: PendingAction) {
        self.pending_action = Some(action);
    }

    /// Take and consume the pending action
    pub fn take_pending(&mut self) -> Option<PendingAction> {
        self.pending_action.take()
    }

    /// Set a simple confirmation action
    pub fn set_simple_confirmation(&mut self, action: SimpleConfirmationAction) {
        self.simple_confirmation_action = Some(action);
    }

    /// Take and consume the simple confirmation action
    pub fn take_simple_confirmation(&mut self) -> Option<SimpleConfirmationAction> {
        self.simple_confirmation_action.take()
    }
}
