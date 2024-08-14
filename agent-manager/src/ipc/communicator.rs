use super::IPC;
use dashmap::DashMap;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq)]
pub enum OS {
    Windows,
    Linux,
    MacOS,
    Unknown,
}

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq)]
#[serde(tag = "request_type")]
pub enum MessageType {
    HealthCheck {
        status: String,
        cpu_usage: f64,
        memory_usage: f64,
        storage_usage: f64,
    },
    Dependency {
        module: String,
        local_port: u16,
        local_ip: String,
        local_os: OS,
        remote_port: u16,
        remote_ip: String,
        description: String,
    },
    Node {
        src_ip: String,
        os: OS,
    },
    Log {
        log_level: String,
        message: String,
        context: HashMap<String, String>,
    },
    Custom(serde_json::Value),
}

#[derive(Serialize, Clone, Deserialize, Debug, PartialEq)]
pub struct Message {
    pub agent_id: AgentID,
    pub timestamp: String,
    pub request: MessageType,
}

impl Message {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

pub trait Handler: Send + Sync + 'static {
    fn handle_msg(
        &self,
        message: Message,
    ) -> impl std::future::Future<Output = Result<(), AgentError>> + Send;
}

pub struct DefaultAgentHandler;

impl Handler for DefaultAgentHandler {
    async fn handle_msg(&self, message: Message) -> Result<(), AgentError> {
        match message.request {
            MessageType::HealthCheck {
                status,
                cpu_usage,
                memory_usage,
                ..
            } => {
                info!(
                    "Health Check - Agent: {}, Status: {}, CPU: {}, Memory: {}",
                    message.agent_id, status, cpu_usage, memory_usage
                );
            }
            MessageType::Log {
                log_level,
                message: log_message,
                context,
            } => {
                info!(
                    "Log - Agent: {}, Level: {}, Message: {}, Context: {:?}",
                    message.agent_id, log_level, log_message, context
                );
            }
            _ => {
                warn!("Unsupported message type from agent: {}", message.agent_id);
            }
        };
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct AgentID(String);

impl fmt::Display for AgentID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub enum AgentError {
    NotFound,
    Communication(String),
    Serialization(String),
    Timeout,
}

impl std::error::Error for AgentError {}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AgentError::NotFound => write!(f, "Agent not found"),
            AgentError::Communication(msg) => write!(f, "Communication error: {}", msg),
            AgentError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            AgentError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

pub struct AgentCommunicator<T: IPC, H: Handler> {
    handler: Arc<H>,
    agents: DashMap<AgentID, Arc<T>>,
    timeout: Duration,
    message_count: AtomicUsize,
}

impl<T: IPC + Clone, H: Handler> AgentCommunicator<T, H> {
    pub fn new(handler: H, timeout: Duration) -> Self {
        Self {
            handler: Arc::new(handler),
            agents: DashMap::new(),
            timeout,
            message_count: AtomicUsize::new(0),
        }
    }

    pub fn add_agent(&self, agent_id: AgentID, ipc: T) {
        self.agents.insert(agent_id, Arc::new(ipc));
    }

    pub async fn send_message(&self, agent_id: &AgentID, message: &[u8]) -> Result<(), AgentError> {
        let ipc = self.agents.get(agent_id).ok_or(AgentError::NotFound)?;
        tokio::select! {
            result = ipc.send(message) => {
                self.message_count.fetch_add(1, Ordering::Relaxed);
                result.map_err(|e| AgentError::Communication(e.to_string()))
            },
            _ = tokio::time::sleep(self.timeout) => Err(AgentError::Timeout),
        }
    }

    pub async fn receive_message(&self, agent_id: &AgentID) -> Result<(), AgentError> {
        let ipc = self.agents.get(agent_id).ok_or(AgentError::NotFound)?;
        let data = tokio::select! {
            result = ipc.receive() => result.map_err(|e| AgentError::Communication(e.to_string()))?,
            _ = tokio::time::sleep(self.timeout) => return Err(AgentError::Timeout),
        };
        let message =
            Message::try_from_bytes(&data).map_err(|e| AgentError::Serialization(e.to_string()))?;
        self.handler.handle_msg(message).await
    }

    pub async fn receive_all_messages(&self) -> Result<(), AgentError> {
        let agent_pairs: Vec<_> = self
            .agents
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        let tasks: Vec<_> = agent_pairs.into_iter().map(|(_, ipc)| {
            let handler = self.handler.clone();
            let timeout = self.timeout;
            task::spawn(async move {
                let data = tokio::select! {
                    result = ipc.receive() => result.map_err(|e| AgentError::Communication(e.to_string()))?,
                    _ = tokio::time::sleep(timeout) => return Err(AgentError::Timeout),
                };
                let message = Message::try_from_bytes(&data)
                    .map_err(|e| AgentError::Serialization(e.to_string()))?;
                handler.handle_msg(message).await?;
                Ok::<(), AgentError>(())
            })
        }).collect();

        for task in tasks {
            task.await
                .map_err(|e| AgentError::Communication(e.to_string()))??;
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), AgentError> {
        // Implement shutdown logic here
        Ok(())
    }

    pub fn get_message_count(&self) -> usize {
        self.message_count.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tokio::time::{sleep, Duration};

    use env_logger;

    #[test]
    fn setup() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[derive(Clone)]
    struct MockHandler {
        received_messages: Arc<Mutex<Vec<Message>>>,
    }

    impl MockHandler {
        fn new() -> Self {
            Self {
                received_messages: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Handler for MockHandler {
        async fn handle_msg(&self, message: Message) -> Result<(), AgentError> {
            match message.request {
                MessageType::HealthCheck {
                    ref status,
                    cpu_usage,
                    memory_usage,
                    ..
                } => {
                    info!(
                        "Health Check - Agent: {}, Status: {}, CPU: {}, Memory: {}",
                        message.agent_id, status, cpu_usage, memory_usage
                    );
                }
                MessageType::Log {
                    ref log_level,
                    message: ref log_message,
                    ref context,
                } => {
                    warn!(
                        "Log - Agent: {}, Level: {}, Message: {}, Context: {:?}",
                        message.agent_id, log_level, log_message, context
                    );
                }
                _ => {
                    info!("Unsupported message type from agent: {}", message.agent_id);
                }
            };

            self.received_messages.lock().unwrap().push(message);
            Ok(())
        }
    }

    #[derive(Clone)]
    struct MockIPC {
        send_count: Arc<Mutex<usize>>,
        receive_message: Arc<Mutex<Option<Message>>>,
        delay: Duration,
    }

    impl MockIPC {
        fn new(delay: Duration) -> Self {
            Self {
                send_count: Arc::new(Mutex::new(0)),
                receive_message: Arc::new(Mutex::new(None)),
                delay,
            }
        }

        fn set_receive_message(&self, message: Message) {
            *self.receive_message.lock().unwrap() = Some(message);
        }
    }

    impl IPC for MockIPC {
        async fn send(&self, _message: &[u8]) -> std::io::Result<()> {
            sleep(self.delay).await;
            let mut count = self.send_count.lock().unwrap();
            *count += 1;
            Ok(())
        }

        async fn receive(&self) -> std::io::Result<Vec<u8>> {
            sleep(self.delay).await;
            let message = self.receive_message.lock().unwrap().take();
            match message {
                Some(msg) => Ok(serde_json::to_vec(&msg).unwrap()),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No message set",
                )),
            }
        }
    }

    #[tokio::test]
    async fn test_send_message() {
        let handler = MockHandler::new();
        let communicator = AgentCommunicator::new(handler, Duration::from_millis(100));

        let agent_id = AgentID("agent_001".to_string());
        let mock_ipc = MockIPC::new(Duration::from_millis(10));
        communicator.add_agent(agent_id.clone(), mock_ipc.clone());

        let result = communicator.send_message(&agent_id, b"Test message").await;
        assert!(result.is_ok());

        let send_count = *mock_ipc.send_count.lock().unwrap();
        assert_eq!(send_count, 1, "Message should be sent once");
    }

    #[tokio::test]
    async fn test_receive_message() {
        let handler = MockHandler::new();
        let communicator = AgentCommunicator::new(handler.clone(), Duration::from_millis(100));

        let agent_id = AgentID("agent_001".to_string());
        let mock_ipc = MockIPC::new(Duration::from_millis(10));
        communicator.add_agent(agent_id.clone(), mock_ipc.clone());

        let test_message = Message {
            agent_id: agent_id.clone(),
            timestamp: "2024-08-04T12:34:56Z".to_string(),
            request: MessageType::HealthCheck {
                status: "running".to_string(),
                cpu_usage: 45.3,
                memory_usage: 120.5,
                storage_usage: 80.0,
            },
        };

        mock_ipc.set_receive_message(test_message.clone());

        let result = communicator.receive_message(&agent_id).await;
        assert!(result.is_ok(), "Receive message should succeed");

        let received_messages = handler.received_messages.lock().unwrap();
        assert_eq!(
            received_messages.len(),
            1,
            "Handler should receive one message"
        );
        assert_eq!(
            received_messages[0], test_message,
            "Received message should match sent message"
        );
    }

    #[tokio::test]
    async fn test_receive_all_messages() {
        let handler = MockHandler::new();
        let communicator = AgentCommunicator::new(handler.clone(), Duration::from_millis(100));

        let agent_id_1 = AgentID("agent_001".to_string());
        let agent_id_2 = AgentID("agent_002".to_string());
        let mock_ipc_1 = MockIPC::new(Duration::from_millis(10));
        let mock_ipc_2 = MockIPC::new(Duration::from_millis(10));

        communicator.add_agent(agent_id_1.clone(), mock_ipc_1.clone());
        communicator.add_agent(agent_id_2.clone(), mock_ipc_2.clone());

        let test_message_1 = Message {
            agent_id: agent_id_1.clone(),
            timestamp: "2024-08-04T12:34:56Z".to_string(),
            request: MessageType::HealthCheck {
                status: "running".to_string(),
                cpu_usage: 45.3,
                memory_usage: 120.5,
                storage_usage: 80.0,
            },
        };

        let test_message_2 = Message {
            agent_id: agent_id_2.clone(),
            timestamp: "2024-08-04T12:34:57Z".to_string(),
            request: MessageType::Log {
                log_level: "INFO".to_string(),
                message: "Test log".to_string(),
                context: HashMap::new(),
            },
        };

        mock_ipc_1.set_receive_message(test_message_1.clone());
        mock_ipc_2.set_receive_message(test_message_2.clone());

        let result = communicator.receive_all_messages().await;
        assert!(result.is_ok(), "Receive all messages should succeed");

        let received_messages = handler.received_messages.lock().unwrap();
        assert_eq!(
            received_messages.len(),
            2,
            "Handler should receive two messages"
        );
        assert!(
            received_messages.contains(&test_message_1),
            "Should receive message from agent 1"
        );
        assert!(
            received_messages.contains(&test_message_2),
            "Should receive message from agent 2"
        );
    }

    #[tokio::test]
    async fn test_non_existent_agent() {
        let handler = MockHandler::new();
        let communicator: AgentCommunicator<MockIPC, MockHandler> =
            AgentCommunicator::new(handler, Duration::from_millis(100));

        let non_existent_agent_id = AgentID("non_existent".to_string());

        let send_result = communicator
            .send_message(&non_existent_agent_id, b"Test message")
            .await;
        assert!(
            matches!(send_result, Err(AgentError::NotFound)),
            "Send to non-existent agent should fail with NotFound error"
        );

        let receive_result = communicator.receive_message(&non_existent_agent_id).await;
        assert!(
            matches!(receive_result, Err(AgentError::NotFound)),
            "Receive from non-existent agent should fail with NotFound error"
        );
    }

    #[tokio::test]
    async fn test_timeout() {
        let handler = MockHandler::new();
        let communicator = AgentCommunicator::new(handler, Duration::from_millis(50));

        let agent_id = AgentID("agent_001".to_string());
        let mock_ipc = MockIPC::new(Duration::from_millis(100)); // IPC delay > timeout
        communicator.add_agent(agent_id.clone(), mock_ipc);

        let send_result = communicator.send_message(&agent_id, b"Test message").await;
        assert!(
            matches!(send_result, Err(AgentError::Timeout)),
            "Send should timeout"
        );

        let receive_result = communicator.receive_message(&agent_id).await;
        assert!(
            matches!(receive_result, Err(AgentError::Timeout)),
            "Receive should timeout"
        );
    }

    #[tokio::test]
    async fn test_custom_message_type() {
        let handler = MockHandler::new();
        let communicator = AgentCommunicator::new(handler.clone(), Duration::from_millis(100));

        let agent_id = AgentID("agent_001".to_string());
        let mock_ipc = MockIPC::new(Duration::from_millis(10));
        communicator.add_agent(agent_id.clone(), mock_ipc.clone());

        let custom_data = serde_json::json!({
            "key1": "value1",
            "key2": 42
        });

        let test_message = Message {
            agent_id: agent_id.clone(),
            timestamp: "2024-08-04T12:34:56Z".to_string(),
            request: MessageType::Custom(custom_data.clone()),
        };

        mock_ipc.set_receive_message(test_message.clone());

        let result = communicator.receive_message(&agent_id).await;
        assert!(result.is_ok(), "Receive custom message should succeed");

        let received_messages = handler.received_messages.lock().unwrap();
        assert_eq!(
            received_messages.len(),
            1,
            "Handler should receive one message"
        );

        if let MessageType::Custom(received_data) = &received_messages[0].request {
            assert_eq!(
                received_data, &custom_data,
                "Received custom data should match sent data"
            );
        } else {
            panic!("Received message should be of Custom type");
        }
    }
}
