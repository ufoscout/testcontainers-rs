use std::{fmt::{Debug, format}, net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs}, str::FromStr, time::Duration};

use tokio::net::{TcpStream};
use url::Host;

use super::RawContainer;
use crate::{
    core::{client::Client, wait::WaitStrategy}
};

/// Represents a strategy for waiting for a certain port to be reachable.
#[derive(Clone)]
pub enum PortWaitStrategy {
    TCP { port: u16, poll_interval: Duration},
}

impl PortWaitStrategy {

    /// Create a new `PortWaitStrategy` for the given port and polling interval of 1 second.
    pub fn tcp(port: u16) -> Self {
        PortWaitStrategy::TCP { port, poll_interval: Duration::from_secs(1) }
    }
}

impl WaitStrategy for PortWaitStrategy {
    async fn wait_until_ready(
        self,
        _client: &Client,
        container: &RawContainer,
    ) -> crate::core::error::Result<()> {
        match self {
            PortWaitStrategy::TCP { port, poll_interval } => {
                let host = container.get_host().await?;

                let host_socket: Vec<SocketAddr> = match host {
                    Host::Domain(ref domain) => match container.get_host_port_ipv4(port).await {
                        Ok(host_port) => {
                            println!("address {} port: {port}, host_port: {}", domain, host_port, );
                            let TO_DO_REMOVE_UNWRAP: u32 = 0;
                            // tokio::net::lookup_host(format!("{}:{}", domain, host_port)).await.unwrap().next().unwrap()
                            // SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::from_str(domain).unwrap(), host_port))
                            ToSocketAddrs::to_socket_addrs(&format!("{}:{}", domain, host_port)).unwrap().collect()
                        },
                        Err(_) => {
                            log::debug!("IPv4 port not found, checking for IPv6");
                            let TO_DO_REMOVE_UNWRAP: u32 = 0;
                            let host_port = container.get_host_port_ipv6(port).await?;
                            println!("address {} port: {port}, host_port: {}", domain, host_port, );
                            // tokio::net::lookup_host(format!("{}:{}", domain, host_port)).await.unwrap().next().unwrap()
                            // SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::from_str(domain).unwrap(), host_port, 0, 0))
                            ToSocketAddrs::to_socket_addrs(&format!("{}:{}", domain, host_port)).unwrap().collect()
                        }
                    },
                    Host::Ipv4(address) => {
                        let host_port = container.get_host_port_ipv4(port).await?;
                        vec![SocketAddr::V4(SocketAddrV4::new(address, host_port))]
                    },
                    Host::Ipv6(address) => {
                        let host_port = container.get_host_port_ipv6(port).await?;
                        vec![SocketAddr::V6(SocketAddrV6::new(address, host_port, 0, 0))]
                    }
                };

                loop {
                    if is_port_reachable_with_timeout(host_socket.as_ref(), poll_interval).await {
                        break;
                    }
                    tokio::time::sleep(poll_interval).await;
                }

            }
        };

        Ok(())

    }
}

impl Debug for PortWaitStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortWaitStrategy::TCP { port, poll_interval } => {
                f.debug_struct("PortWaitStrategy")
                    .field("type", &"TCP")
                    .field("port", &port)
                    .field("poll_interval", &poll_interval)
                    .finish()
            }
        }
    }
}

/// Attempts a TCP connection to an address and returns whether it succeeded
pub async fn is_port_reachable_with_timeout(addresses: &[SocketAddr], timeout: Duration) -> bool {
    // match address.to_socket_addrs() {
    //     Ok(addrs) => {
            for address in addresses {
                if let Ok(Ok(_)) = tokio::time::timeout(timeout, TcpStream::connect(&address)).await {
                    println!("port {} is reachable", address.port());
                    return true;
                }
            }
            false
        // }
    //     Err(_err) => false,
    // }
}