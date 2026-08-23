use std::net::{IpAddr, UdpSocket};
use std::process::Command;

/// Returns this machine's Tailscale IPv4 address, if Tailscale is running.
pub fn tailscale_ipv4() -> Option<String> {
    let output = Command::new("tailscale").args(["ip", "-4"]).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let ip = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if ip.is_empty() {
        None
    } else {
        Some(ip)
    }
}

/// Best-effort guess at a LAN IPv4 address (not loopback, not Tailscale CGNAT range).
pub fn lan_ipv4() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let local = socket.local_addr().ok()?.ip();

    match local {
        IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_unspecified() => Some(v4.to_string()),
        _ => None,
    }
}

pub struct ListenAddresses {
    pub bind: String,
    pub localhost: String,
    pub lan: Option<String>,
    pub tailscale: Option<String>,
}

pub fn listen_addresses(bind_addr: &str, port: u16) -> ListenAddresses {
    ListenAddresses {
        bind: bind_addr.to_string(),
        localhost: format!("127.0.0.1:{port}"),
        lan: lan_ipv4().map(|ip| format!("{ip}:{port}")),
        tailscale: tailscale_ipv4().map(|ip| format!("{ip}:{port}")),
    }
}
