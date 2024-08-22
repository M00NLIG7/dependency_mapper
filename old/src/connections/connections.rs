#![warn(missing_docs)]
use procfs::{
    net::{TcpNetEntry, TcpState, UdpNetEntry, UdpState},
    process::{FDTarget, Stat},
};
use crate::{implement_module, base::Response};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

/// NetworkConnection represents a network connection
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkConnection {
    pub(crate) local_address: String,
    pub(crate) remote_address: Option<String>,
    pub(crate) state: Option<ConnectionState>,
    pub(crate) protocol: String,
    pub(crate) process: Option<Process>,
}

/// Process represents a process
#[derive(Debug, Deserialize, Serialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    pub(crate) pid: u32,
    pub(crate) name: String,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionState {
    /// Established state
    Established,
    /// SynSent state
    SynSent,
    /// SynRecv state
    SynRecv,
    /// FinWait1 state
    FinWait1,
    /// FinWait2 state
    FinWait2,
    /// TimeWait state
    TimeWait,
    /// Closed state
    Close,
    /// CloseWait state
    CloseWait,
    /// LastAck state
    LastAck,
    /// Listen state
    Listen,
    /// Closing state
    Closing,
    /// Unknown state
    #[default]
    Unknown,
}

impl ConnectionState {
    /// Returns true if the state is Closed
    pub fn is_closed(&self) -> bool {
        matches!(self, ConnectionState::Close)
    }
}

macro_rules! impl_from_state {
    ($from_type:ty, $($variant:ident),* $(,)?) => {
        impl From<&$from_type> for ConnectionState {
            fn from(state: &$from_type) -> Self {
                match state {
                    $(
                        <$from_type>::$variant => ConnectionState::$variant,
                    )*
                    #[allow(unreachable_patterns)]
                    _ => ConnectionState::Unknown,
                }
            }
        }
    };
}

impl_from_state!(UdpState, Established, Close);
impl_from_state!(
    TcpState,
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    Close,
    FinWait2,
    TimeWait,
    CloseWait,
    LastAck,
    Listen,
    Closing
);

trait NetworkData {
    fn local_address(&self) -> String;
    fn remote_address(&self) -> String;
    fn inode(&self) -> u64;
    fn state(&self) -> ConnectionState;
    fn protocol(&self) -> String;
}

macro_rules! impl_network_data {
    ($type:ty, $protocol:expr) => {
        impl NetworkData for $type {
            fn local_address(&self) -> String {
                self.local_address.to_string()
            }

            fn remote_address(&self) -> String {
                self.remote_address.to_string()
            }

            fn inode(&self) -> u64 {
                self.inode
            }

            fn state(&self) -> ConnectionState {
                ConnectionState::from(&self.state)
            }

            fn protocol(&self) -> String {
                $protocol.into()
            }
        }
    };
}

impl_network_data!(UdpNetEntry, "UDP");
impl_network_data!(TcpNetEntry, "TCP");

fn process_network_entries<F, T>(
    fetch_entries: F,
    map: Arc<HashMap<u64, Stat>>,
) -> Vec<NetworkConnection>
where
    F: Fn() -> Result<Vec<T>, procfs::ProcError> + Send + 'static,
    T: NetworkData + Send + 'static, // Ensure T is Send
{
    // Spawn a thread to process the entries
    let handle = thread::spawn(move || {
        let mut connections = Vec::new();

        // Fetch the entries
        if let Ok(entries) = fetch_entries() {
            // Iterate over the entries
            for entry in entries {
                let state = entry.state(); // Get the state without consuming entry

                if state.is_closed() {
                    continue;
                }

                // Filter out CLOSE connections
                let connection = NetworkConnection {
                    local_address: entry.local_address(),
                    remote_address: Some(entry.remote_address()),
                    state: Some(state),
                    protocol: entry.protocol(),
                    process: map.get(&entry.inode()).map(|stat| Process {
                        pid: stat.pid as u32,
                        name: stat.comm.clone(),
                    }),
                    // .map(|stat| (stat.pid, stat.comm.clone())),
                };
                connections.push(connection);
            }
        }
        connections
    });

    handle.join().unwrap() // Join the thread and unwrap the result
}

/// conn_info returns a JSON string of all network connections
pub fn conn_info() -> serde_json::Value {
    let all_procs = procfs::process::all_processes().expect("Cant read /proc"); // handle errors appropriately
    let mut map: HashMap<u64, Stat> = HashMap::new();

    for process in all_procs.filter_map(Result::ok) {
        let stat = match process.stat() {
            Ok(s) => s,
            Err(_) => continue, // Skip processes where stat can't be obtained
        };

        let fds = match process.fd() {
            Ok(f) => f,
            Err(_) => continue, // Skip processes where fds can't be obtained
        };

        for fd in fds.filter_map(Result::ok) {
            if let FDTarget::Socket(inode) = fd.target {
                map.insert(inode, stat.clone());
            }
        }
    }

    let shared_map = Arc::new(map);

    let tcp_connections = process_network_entries(procfs::net::tcp, Arc::clone(&shared_map));
    let udp_connections = process_network_entries(procfs::net::udp, Arc::clone(&shared_map));
    let tcp6_connections = process_network_entries(procfs::net::tcp6, Arc::clone(&shared_map));
    let udp6_connections = process_network_entries(procfs::net::udp6, shared_map);

    // Combine TCP and UDP connections
    let mut all_connections = Vec::new();
    all_connections.extend(tcp_connections);
    all_connections.extend(udp_connections);
    all_connections.extend(tcp6_connections);
    all_connections.extend(udp6_connections);

    // Serialize the connections
    json!(all_connections)
}


#[derive(Deserialize, Default)]
pub struct ConnectionArgs {}

fn run_connections(args: ConnectionArgs) -> Result<Response, Box<dyn std::error::Error>> {
    let mut response = Response::new(format!("Hello, {}!", "c"), false, false);
    response.add_extra("data", &conn_info())?;
    Ok(response)
}

implement_module!(ConnectionModule, ConnectionArgs, run_connections);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_connection_info() {
        let conn_info = conn_info();

        dbg!(&conn_info);
        assert!(conn_info.is_array());
    }
}
