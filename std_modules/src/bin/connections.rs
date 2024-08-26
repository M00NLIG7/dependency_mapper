use procfs::net::{TcpNetEntry, TcpState, UdpNetEntry, UdpState};
use serde::{Deserialize, Serialize};
use std::thread;
use std_modules::response::{Response, Dependency};
use std_modules::implement_module;
use thiserror::Error;

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq, Clone, Hash)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionState {
    Established,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    #[default]
    Unknown,
}

impl ConnectionState {
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

fn parse_ipv6_address(address: &str) -> Option<(&str, &str)> {
    let mut parts = address.strip_prefix("[")?.split("]:");
    let ip = parts.next()?;
    let port = parts.last()?;
    Some((ip, port))
}

fn parse_ipv4_address(address: &str) -> Option<(&str, &str)> {
    let mut parts = address.split(':');
    let ip = parts.next()?;
    let port = parts.last()?;
    Some((ip, port))
}

fn parse_address(address: &str) -> Option<(&str, &str)> {
    if address.contains('[') {
        parse_ipv6_address(address)
    } else {
        parse_ipv4_address(address)
    }
}

fn process_network_entries<F, T>(
    fetch_entries: F,
    omit_local_connections: bool,
) -> Vec<Dependency>
where
    F: Fn() -> Result<Vec<T>, procfs::ProcError> + Send + 'static,
    T: NetworkData + Send + 'static,
{
    thread::spawn(move || {
        let mut dependencies = Vec::new();

        if let Ok(entries) = fetch_entries() {
            for entry in entries {
                let state = entry.state();
                if state.is_closed() {
                    continue;
                }

                if let (Some((local_ip, local_port)), Some((remote_ip, remote_port))) = 
                    (parse_address(&entry.local_address()), parse_address(&entry.remote_address())) {
                    if omit_local_connections {
                        if local_ip.contains("127.0.0.") || remote_ip.contains("127.0.0.") || local_ip.contains("::1") || remote_ip.contains("::1") {
                            eprintln!("Omitting local connection: {}:{} -> {}:{}", local_ip, local_port, remote_ip, remote_port);
                            continue;
                        }
                    }

                    if let (Ok(local_port), Ok(remote_port)) = (local_port.parse::<i32>(), remote_port.parse::<i32>()) {
                        let dependency = Dependency {
                            module: "Connections".to_string(),
                            local_port,
                            local_ip: local_ip.to_string(),
                            local_os: "Linux".to_string(),
                            remote_port,
                            remote_ip: remote_ip.to_string(),
                            description: format!("{} connection", entry.protocol()),
                        };
                        dependencies.push(dependency);
                    }
                }
            }
        }
        dependencies
    }).join().unwrap()
}

pub fn conn_info(omit_local_connections: bool) -> Vec<Dependency> {

    // TODO: Do we need the PID?
    /*
    let all_procs = procfs::process::all_processes().expect("Can't read /proc");
    let mut map: HashMap<u64, Stat> = HashMap::new();

    for process in all_procs.filter_map(Result::ok) {
        if let Ok(stat) = process.stat() {
            if let Ok(fds) = process.fd() {
                for fd in fds.filter_map(Result::ok) {
                    if let FDTarget::Socket(inode) = fd.target {
                        map.insert(inode, stat.clone());
                    }
                }
            }
        }
    }

    let shared_map = Arc::new(map);
    */

    let tcp_dependencies = process_network_entries(procfs::net::tcp, omit_local_connections);
    let udp_dependencies = process_network_entries(procfs::net::udp , omit_local_connections);
    let tcp6_dependencies = process_network_entries(procfs::net::tcp6, omit_local_connections);
    let udp6_dependencies = process_network_entries(procfs::net::udp6, omit_local_connections);

    let mut all_dependencies = Vec::new();
    all_dependencies.extend(tcp_dependencies);
    all_dependencies.extend(udp_dependencies);
    all_dependencies.extend(tcp6_dependencies);
    all_dependencies.extend(udp6_dependencies);

    all_dependencies
}

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("Failed to get connection information: {0}")]
    ConnectionError(#[from] procfs::ProcError),
}

#[derive(Deserialize, Default)]
pub struct ConnectionArgs {
    omit_local_connections: bool,
}

fn run_connections(args: ConnectionArgs) -> Result<Response, ModuleError> {

    let conn_info = conn_info(args.omit_local_connections);
    let response = Response::new(conn_info, false, false);
    Ok(response)
}

implement_module!(ConnectionModule, ConnectionArgs, ModuleError, run_connections);


fn main() {
    std_modules::response::run_module::<ConnectionModule>();
}
