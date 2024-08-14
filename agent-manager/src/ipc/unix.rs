use super::IPC;
use tokio::net::{UnixStream, UnixListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use std::path::Path;
use std::sync::Arc;
use log::{debug, error, info};

// UnixSocketIPC represents an IPC mechanism using Unix domain sockets
pub struct UnixSocketIPC {
    pub socket_path: String,
    // Arc<Mutex<Option<UnixListener>>> allows for thread-safe, mutable access to the listener
    listener: Arc<Mutex<Option<UnixListener>>>,
}

impl UnixSocketIPC {
    // Creates a new UnixSocketIPC instance
    pub async fn new(socket_path: String) -> std::io::Result<Self> {
        debug!("Creating new UnixSocketIPC at {}", socket_path);
        // Remove existing socket file if it exists
        if Path::new(&socket_path).exists() {
            debug!("Socket file already exists, removing it");
            std::fs::remove_file(&socket_path)?;
        }
        // Bind to the Unix domain socket
        let listener = UnixListener::bind(&socket_path)?;
        debug!("Listener bound successfully");
        Ok(Self {
            socket_path,
            listener: Arc::new(Mutex::new(Some(listener))),
        })
    }
}

// Implement Clone for UnixSocketIPC to allow multiple references
impl Clone for UnixSocketIPC {
    fn clone(&self) -> Self {
        debug!("Cloning UnixSocketIPC");
        Self {
            socket_path: self.socket_path.clone(),
            listener: Arc::clone(&self.listener),
        }
    }
}

// Implement the IPC trait for UnixSocketIPC
impl IPC for UnixSocketIPC {
    // Send a message through the Unix domain socket
    async fn send(&self, message: &[u8]) -> std::io::Result<()> {
        debug!("Attempting to send message");
        // Connect to the socket
        let mut stream = UnixStream::connect(&self.socket_path).await?;
        debug!("Connected to socket for sending");
        // Write the entire message
        stream.write_all(message).await?;
        debug!("Message sent successfully");
        Ok(())
    }

    // Receive a message from the Unix domain socket
    async fn receive(&self) -> std::io::Result<Vec<u8>> {
        debug!("Attempting to receive message");
        // Lock the listener to ensure thread-safe access
        let mut listener_guard = self.listener.lock().await;
        // Get a mutable reference to the listener, or return an error if it's been taken
        let listener = listener_guard.as_mut().ok_or_else(|| {
            error!("Listener has already been taken");
            std::io::Error::new(std::io::ErrorKind::Other, "Listener has already been taken")
        })?;

        debug!("Waiting for incoming connection");
        // Accept an incoming connection
        let (mut stream, _) = listener.accept().await?;
        debug!("Connection accepted");
        let mut buffer = Vec::new();
        // Read the entire message into the buffer
        stream.read_to_end(&mut buffer).await?;
        debug!("Message received: {:?}", String::from_utf8_lossy(&buffer));

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};
    use env_logger;

    // Initialize the logger for tests
    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // Test basic send and receive functionality
    #[tokio::test]
    async fn test_unix_socket_ipc() {
        init();
        info!("Starting test_unix_socket_ipc");
        let socket_path = "/tmp/test.sock".to_string();
        let ipc = UnixSocketIPC::new(socket_path.clone()).await.unwrap();
        let message = b"Hello, world!";
        let ipc2 = ipc.clone();
        
        // Spawn a task to receive the message
        let handle = tokio::spawn(async move {
            debug!("Receiver: Starting to receive");
            let received_message = ipc2.receive().await.unwrap();
            debug!("Receiver: Message received");
            assert_eq!(received_message, b"Hello, world!");
        });

        // Give the receiver a moment to start listening
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Send the message
        debug!("Sender: Starting to send");
        ipc.send(message).await.unwrap();
        debug!("Sender: Message sent");

        // Wait for the receive operation to complete with a timeout
        match timeout(Duration::from_secs(5), handle).await {
            Ok(result) => result.unwrap(),
            Err(_) => panic!("Test timed out"),
        }
        info!("test_unix_socket_ipc completed successfully");
    }

    // Test sending and receiving multiple messages
    #[tokio::test]
    async fn test_multiple_messages() {
        init();
        info!("Starting test_multiple_messages");
        let socket_path = "/tmp/test_multiple.sock".to_string();
        let ipc = UnixSocketIPC::new(socket_path.clone()).await.unwrap();
        let messages: Vec<&[u8]> = vec![b"First message", b"Second message", b"Third message"];

        for (i, message) in messages.iter().enumerate() {
            let ipc_clone = ipc.clone();
            let message_clone = message.to_vec();

            // Spawn a task to receive each message
            let receive_handle = tokio::spawn(async move {
                debug!("Receiver {}: Starting to receive", i);
                let received = ipc_clone.receive().await.unwrap();
                debug!("Receiver {}: Message received", i);
                assert_eq!(received, message_clone);
            });

            // Give the receiver a moment to start listening
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Send each message
            debug!("Sender {}: Starting to send", i);
            ipc.send(message).await.unwrap();
            debug!("Sender {}: Message sent", i);

            // Wait for the receive operation to complete with a timeout
            match timeout(Duration::from_secs(5), receive_handle).await {
                Ok(result) => result.unwrap(),
                Err(_) => panic!("Test timed out for message {}", i),
            }
        }
        info!("test_multiple_messages completed successfully");
    }
}
