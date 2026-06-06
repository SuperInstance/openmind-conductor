use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver, Sender};


/// A request sent from conductor to an agent via baton.
#[derive(Debug, Clone)]
pub struct FlexRequest {
    pub chord: String,
    pub args: Vec<String>,
}

/// A response from an agent back to the conductor.
#[derive(Debug, Clone)]
pub struct FlexResponse {
    pub agent_id: String,
    pub value: f64,
    pub success: bool,
}

/// In-process baton using tokio mpsc channels.
pub struct LocalBaton {
    senders: HashMap<String, Sender<FlexRequest>>,
    receiver: Receiver<FlexResponse>,
    resp_sender: Sender<FlexResponse>,
}

impl LocalBaton {
    pub fn new() -> Self {
        let (resp_tx, resp_rx) = channel::<FlexResponse>(256);
        LocalBaton {
            senders: HashMap::new(),
            receiver: resp_rx,
            resp_sender: resp_tx,
        }
    }

    /// Register an agent with its request channel.
    pub fn register(&mut self, agent_id: &str) -> Receiver<FlexRequest> {
        let (tx, rx) = channel::<FlexRequest>(256);
        self.senders.insert(agent_id.to_string(), tx);
        rx
    }

    /// Get a clone of the response sender (for agents to reply).
    pub fn response_sender(&self) -> Sender<FlexResponse> {
        self.resp_sender.clone()
    }

    /// Send a request to an agent.
    pub async fn send(&self, agent_id: &str, req: FlexRequest) -> Result<(), String> {
        let sender = self.senders.get(agent_id).ok_or(format!("Agent '{}' not registered", agent_id))?;
        sender.send(req).await.map_err(|e| e.to_string())
    }

    /// Receive a response (blocking).
    pub async fn recv(&mut self) -> Option<FlexResponse> {
        self.receiver.recv().await
    }
}

/// A mock baton for testing — no real channels, just stores requests/responses.
#[derive(Debug, Clone)]
pub struct MockBaton {
    pub sent: Arc<tokio::sync::Mutex<Vec<(String, FlexRequest)>>>,
    pub responses: Arc<tokio::sync::Mutex<HashMap<String, FlexResponse>>>,
}

impl MockBaton {
    pub fn new() -> Self {
        MockBaton {
            sent: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            responses: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub async fn send(&self, agent_id: &str, req: FlexRequest) -> Result<(), String> {
        self.sent.lock().await.push((agent_id.to_string(), req));
        Ok(())
    }

    pub async fn set_response(&self, agent_id: &str, resp: FlexResponse) {
        self.responses.lock().await.insert(agent_id.to_string(), resp);
    }

    pub async fn get_sent(&self) -> Vec<(String, FlexRequest)> {
        self.sent.lock().await.clone()
    }
}
