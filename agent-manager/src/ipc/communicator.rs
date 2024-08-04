use super::IPC;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "request_type")]
pub enum MessageType {
    HealthCheck {
        status: String,
        cpu_usage: f64,
        memory_usage: f64,
    },
    Log {
        log_level: String,
        message: String,
        context: HashMap<String, String>,
    },
    Command {
        command: String,
        parameters: HashMap<String, String>,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Message {
    pub agent_id: String,
    pub timestamp: String,
    pub request: MessageType,
}

impl Message {
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

// Updated AgentHandler and Handler trait with async support
pub trait Handler {
    fn handle_msg(
        &self,
        message: Message,
    ) -> impl std::future::Future<Output = std::io::Result<()>> + Send;
}


pub struct DefaultAgentHandler;

impl Handler for DefaultAgentHandler {
    async fn handle_msg(&self, message: Message) -> std::io::Result<()> {
        match message.request {
            MessageType::HealthCheck {
                status,
                cpu_usage,
                memory_usage,
            } => {
                println!(
                    "Health Check - Status: {}, CPU: {}, Memory: {}",
                    status, cpu_usage, memory_usage
                );
            }
            MessageType::Log {
                log_level,
                message,
                context,
            } => {
                println!(
                    "Log - Level: {}, Message: {}, Context: {:?}",
                    log_level, message, context
                );
            }
            MessageType::Command {
                command,
                parameters,
            } => {
                println!(
                    "Command - Command: {}, Parameters: {:?}",
                    command, parameters
                );
            }
        };
        Ok(())
    }
}

// Updated AgentCommunicator with async receive_message
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentCommunicator<T: IPC, H: Handler> {
    ipc: T,
    handler: H,
}

impl<T: IPC, H: Handler> AgentCommunicator<T, H> {
    pub fn new(ipc: T, handler: H) -> Self {
        Self { ipc, handler }
    }

    pub async fn send_message(&self, message: &[u8]) -> std::io::Result<()> {
        self.ipc.send(message).await
    }

    pub async fn receive_message(&self) -> std::io::Result<()> {
        let data = self.ipc.receive().await?;
        let message = Message::try_from_bytes(&data)?;
        self.handler.handle_msg(message).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHandler;

    impl Handler for MockHandler {
        async fn handle_msg(&self, message: Message) -> std::io::Result<()> {
            // Simple mock behavior, e.g., just log the message
            println!("MockHandler received: {:?}", message);
            Ok(())
        }
    }

    struct MockIPC;

    impl IPC for MockIPC {
        fn send(&self, message: &[u8]) -> impl std::future::Future<Output = std::io::Result<()>> + Send {
            async move {
                // Mock sending the message
                println!("MockIPC sending message: {:?}", message);
                Ok(())
            }
        }

        fn receive(&self) -> impl std::future::Future<Output = std::io::Result<Vec<u8>>> + Send {
            // Provide some example JSON data
            let example_data = serde_json::to_vec(&Message {
                agent_id: "agent_001".to_string(),
                timestamp: "2024-08-04T12:34:56Z".to_string(),
                request: MessageType::HealthCheck {
                    status: "running".to_string(),
                    cpu_usage: 45.3,
                    memory_usage: 120.5,
                },
            }).unwrap();
            async move {
                // Mock receiving the message
                println!("MockIPC receiving message: {:?}", example_data);
                Ok(example_data)
            }
        }

    }

    #[tokio::test]
    async fn test_bytes_to_request() {
        // Example of creating a health check request
        let request = Message {
            agent_id: "agent_001".to_string(),
            timestamp: "2024-08-04T12:34:56Z".to_string(),
            request: MessageType::HealthCheck {
                status: "running".to_string(),
                cpu_usage: 45.3,
                memory_usage: 120.5,
            },
        };

        // Serialize the request to JSON
        let serialized = serde_json::to_vec(&request).unwrap();

        // Deserialize the JSON back to a Request struct
        let deserialized: Message = serde_json::from_slice(&serialized).unwrap();

        assert_eq!(request, deserialized);
    }

    #[tokio::test]
    async fn test_agent_communicator_with_mock_handler() {
        let ipc = MockIPC;
        let handler = MockHandler;
        let communicator = AgentCommunicator::new(ipc, handler);

        communicator.receive_message().await.unwrap();
    }
}
