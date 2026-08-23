pub mod cli;
pub mod history;
pub mod protocol;
pub mod server;
pub mod session;
pub mod style;
pub mod tailscale;
pub mod tui;

pub const DEFAULT_PORT: u16 = 8080;
pub const DEFAULT_BIND: &str = "0.0.0.0";
pub const DEFAULT_ADDR: &str = "127.0.0.1:8080";

pub fn default_bind_addr() -> String {
    format!("{DEFAULT_BIND}:{DEFAULT_PORT}")
}

pub fn normalize_addr(input: &str) -> String {
    let input = input.trim();
    if input.is_empty() {
        return DEFAULT_ADDR.to_string();
    }
    if input.contains(':') {
        input.to_string()
    } else {
        format!("{input}:{DEFAULT_PORT}")
    }
}
