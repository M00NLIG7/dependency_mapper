#![warn(missing_docs)]
use procfs::{
    net::{TcpNetEntry, TcpState, UdpNetEntry, UdpState},
    process::{FDTarget, Stat},
};
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

/// ConnectionState represents the state of a network connection
/// It is a subset of the states in TcpState and UdpState
///
/// The states are:
/// - Established
/// - SynSent
/// - SynRecv
/// - FinWait1
/// - FinWait2
/// - TimeWait
/// - Closed
/// - CloseWait
/// - LastAck
/// - Listen
/// - Closing
/// - Unknown
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
    Closed,
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
        matches!(self, ConnectionState::Closed)
    }
}

// Converts UdpState to ConnectionState
impl From<&UdpState> for ConnectionState {
    fn from(udp_state: &UdpState) -> Self {
        match udp_state {
            UdpState::Established => ConnectionState::Established,
            UdpState::Close => ConnectionState::Closed,
        }
    }
}

impl From<&TcpState> for ConnectionState {
    fn from(tcp_state: &TcpState) -> Self {
        match tcp_state {
            TcpState::Established => ConnectionState::Established,
            TcpState::SynSent => ConnectionState::SynSent,
            TcpState::SynRecv => ConnectionState::SynRecv,
            TcpState::FinWait1 => ConnectionState::FinWait1,
            TcpState::FinWait2 => ConnectionState::FinWait2,
            TcpState::TimeWait => ConnectionState::TimeWait,
            TcpState::Close => ConnectionState::Closed,
            TcpState::CloseWait => ConnectionState::CloseWait,
            TcpState::LastAck => ConnectionState::LastAck,
            TcpState::Listen => ConnectionState::Listen,
            TcpState::Closing => ConnectionState::Closing,
            _ => ConnectionState::Unknown,
        }
    }
}

trait NetworkData {
    fn local_address(&self) -> String;
    fn remote_address(&self) -> String;
    fn inode(&self) -> u64;
    fn state(&self) -> ConnectionState;
    fn protocol(&self) -> String;
}

impl NetworkData for UdpNetEntry {
    fn local_address(&self) -> String {
        // Return the local_address from TcpNetEntry
        self.local_address.to_string()
    }

    fn remote_address(&self) -> String {
        // Return the remote_address from TcpNetEntry
        self.remote_address.to_string()
    }

    fn inode(&self) -> u64 {
        // Return the inode from TcpNetEntry
        self.inode
    }

    fn state(&self) -> ConnectionState {
        // Return the state from TcpNetEntry
        ConnectionState::from(&self.state)
    }

    fn protocol(&self) -> String {
        "UDP".into()
    }
}

impl NetworkData for TcpNetEntry {
    fn local_address(&self) -> String {
        // Return the local_address from TcpNetEntry
        self.local_address.to_string()
    }

    fn remote_address(&self) -> String {
        // Return the remote_address from TcpNetEntry
        self.remote_address.to_string()
    }

    fn inode(&self) -> u64 {
        // Return the inode from TcpNetEntry
        self.inode
    }

    fn state(&self) -> ConnectionState {
        // Return the state from TcpNetEntry
        ConnectionState::from(&self.state)
    }

    fn protocol(&self) -> String {
        "TCP".into()
    }
}

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

                // Filter out CLOSE connections
                if !state.is_closed() {
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
