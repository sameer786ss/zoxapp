use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::agent::actor::AgentCommand;
use crate::agent::workspace::WorkspaceManager;

/// Channel for sending approval responses back to the agent
pub type ApprovalSender = oneshot::Sender<bool>;

/// Connection mode - cloud or offline
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    Cloud,
    Offline,
}

impl Default for ConnectionMode {
    fn default() -> Self {
        ConnectionMode::Cloud
    }
}

/// Holds the pending approval sender if one is waiting
pub struct ApprovalState {
    pub pending: Option<ApprovalSender>,
}

impl ApprovalState {
    pub fn new() -> Self {
        Self { pending: None }
    }
    
    /// Send approval response if there's a pending request
    /// Returns true if there was a pending request, false otherwise
    pub fn respond(&mut self, approved: bool) -> bool {
        if let Some(sender) = self.pending.take() {
            match sender.send(approved) {
                Ok(_) => {
                    println!("[ApprovalState] Sent approval response: {}", approved);
                    true
                }
                Err(_) => {
                    println!("[ApprovalState] Failed to send - receiver dropped");
                    false
                }
            }
        } else {
            println!("[ApprovalState] No pending approval to respond to");
            false
        }
    }
}

/// Global application state managed by Tauri
pub struct AppState {
    /// Channel to send commands to the agent actor
    pub agent_tx: Mutex<Sender<AgentCommand>>,
    
    /// Approval state for tool execution - SHARED between AppState and AgentActor
    pub approval_state: Arc<RwLock<ApprovalState>>,
    
    /// Workspace manager - shared instance to avoid recreation
    pub workspace: Arc<RwLock<Option<WorkspaceManager>>>,
    
    /// Current connection mode (cloud or offline)
    pub connection_mode: RwLock<ConnectionMode>,
}

impl AppState {
    /// Create new AppState with a shared approval state
    /// The same approval_state Arc must be passed to the AgentActor
    pub fn new(agent_tx: Sender<AgentCommand>, approval_state: Arc<RwLock<ApprovalState>>) -> Self {
        Self {
            agent_tx: Mutex::new(agent_tx),
            approval_state,
            workspace: Arc::new(RwLock::new(None)),
            connection_mode: RwLock::new(ConnectionMode::default()),
        }
    }
    
    /// Set the workspace manager (called during setup)
    pub fn set_workspace(&self, workspace: WorkspaceManager) {
        let mut ws = self.workspace.write();
        *ws = Some(workspace);
    }
    
    /// Get a clone of the workspace manager
    pub fn get_workspace(&self) -> Option<WorkspaceManager> {
        self.workspace.read().clone()
    }
    
    /// Send approval response directly to the waiting actor
    pub fn send_approval(&self, approved: bool) -> bool {
        let mut state = self.approval_state.write();
        state.respond(approved)
    }
    
    /// Get current connection mode
    pub fn get_connection_mode(&self) -> ConnectionMode {
        *self.connection_mode.read()
    }
    
    /// Check if in offline mode
    pub fn is_offline(&self) -> bool {
        matches!(*self.connection_mode.read(), ConnectionMode::Offline)
    }
}

