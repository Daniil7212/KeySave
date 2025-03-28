use dns_lookup::lookup_host;

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
        .any(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                return false;
            }
            let remote = parts[2];
            let remote_parts: Vec<&str> = remote.split(':').collect();
            remote_parts.len() == 2
                && remote_parts[0] == ip
                && remote_parts[1].parse::<u16>().unwrap_or(0) == port
        })
}

// Проверка доступности сайта
pub fn is_site_open(url: &str) -> bool {
    if let Ok(ips) = lookup_host(url) {
        ips.into_iter().any(|ip| {
            let target_ip = ip.to_string();
            let ports = vec![80, 443]; // HTTP и HTTPS
            ports.into_iter().any(|port| check_connections(&target_ip, port))
        })
    } else {
        false
    }
}