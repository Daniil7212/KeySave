use std::{
    net::{IpAddr, ToSocketAddrs},
    process::Command,
    str::FromStr,
};

// Проверка соединений
fn check_connections(ip: &str, port: u16) -> bool {
    use std::process::Command;

    let output = Command::new("netstat")
        .args(&["-n", "-a", "-p", "tcp"])
        .output()
        .expect("Failed to execute netstat");

    let output_str = String::from_utf8_lossy(&output.stdout);
    output_str
        .lines()
        .filter(|line| line.contains("ESTABLISHED"))
        .filter_map(|line| {
            let mut parts = line.split_whitespace().nth(2)?;
            let mut parts = parts.split(':');
            Some((parts.next()?, parts.next()?))
        })
        .any(|(addr, port_str)| {
            addr == ip && port_str.parse::<u16>().map_or(false, |p| p == port)
        })
}

// Проверка доступности сайта
pub fn is_site_open(url: &str) -> bool {
    if let Ok(ip) = IpAddr::from_str(url) {
        return [80, 443].iter().any(|&port| check_connections(&ip.to_string(), port));
    }

    url.to_socket_addrs()
        .map(|addrs| {
            addrs.filter_map(|addr| {
                match addr {
                    std::net::SocketAddr::V4(v4) => Some(v4.ip().to_string()),
                    std::net::SocketAddr::V6(v6) => Some(v6.ip().to_string()),
                }
            })
                .any(|ip| [80, 443].iter().any(|&port| check_connections(&ip, port)))
        })
        .unwrap_or(false)
}