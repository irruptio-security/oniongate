use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::{SOCKS_HOST, SOCKS_PORT};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionConnectivityResult {
    pub host: String,
    pub port: u16,
    pub reachable: bool,
    pub remote_resolution: bool,
    pub latency_ms: Option<u64>,
    pub detail: String,
}

pub fn validate_v3_hostname(host: &str) -> Result<String, String> {
    let normalized = host
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    let label = normalized
        .strip_suffix(".onion")
        .ok_or_else(|| "Expected a .onion hostname".to_string())?;
    if label.len() != 56 || !label.chars().all(|c| matches!(c, 'a'..='z' | '2'..='7')) {
        return Err("Expected a 56-character v3 onion hostname".into());
    }
    Ok(normalized)
}

pub async fn test_connectivity(host: &str, port: u16) -> Result<OnionConnectivityResult, String> {
    let host = validate_v3_hostname(host)?;
    let started = Instant::now();
    let result = timeout(Duration::from_secs(20), socks_connect(&host, port)).await;
    match result {
        Ok(Ok(())) => Ok(OnionConnectivityResult {
            host,
            port,
            reachable: true,
            remote_resolution: true,
            latency_ms: Some(started.elapsed().as_millis() as u64),
            detail: "Tor accepted a SOCKS5 domain-name connection; no local DNS lookup was used"
                .into(),
        }),
        Ok(Err(error)) => Ok(OnionConnectivityResult {
            host,
            port,
            reachable: false,
            remote_resolution: true,
            latency_ms: None,
            detail: error,
        }),
        Err(_) => Ok(OnionConnectivityResult {
            host,
            port,
            reachable: false,
            remote_resolution: true,
            latency_ms: None,
            detail: "Timed out waiting for the onion service through Tor".into(),
        }),
    }
}

async fn socks_connect(host: &str, port: u16) -> Result<(), String> {
    let mut stream = TcpStream::connect((SOCKS_HOST, SOCKS_PORT))
        .await
        .map_err(|e| format!("Tor SOCKS is unavailable: {e}"))?;
    stream
        .write_all(&[5, 1, 0])
        .await
        .map_err(|e| e.to_string())?;
    let mut greeting = [0u8; 2];
    stream
        .read_exact(&mut greeting)
        .await
        .map_err(|e| e.to_string())?;
    if greeting != [5, 0] {
        return Err("Tor SOCKS rejected unauthenticated negotiation".into());
    }

    let mut request = vec![5, 1, 0, 3, host.len() as u8];
    request.extend_from_slice(host.as_bytes());
    request.extend_from_slice(&port.to_be_bytes());
    stream
        .write_all(&request)
        .await
        .map_err(|e| e.to_string())?;

    let mut reply = [0u8; 4];
    stream
        .read_exact(&mut reply)
        .await
        .map_err(|e| e.to_string())?;
    if reply[0] != 5 || reply[1] != 0 {
        return Err(format!(
            "Tor SOCKS connection failed with status {}",
            reply[1]
        ));
    }
    let address_len = match reply[3] {
        1 => 4,
        4 => 16,
        3 => {
            let mut len = [0u8; 1];
            stream
                .read_exact(&mut len)
                .await
                .map_err(|e| e.to_string())?;
            len[0] as usize
        }
        other => return Err(format!("Unexpected SOCKS address type {other}")),
    };
    let mut remainder = vec![0u8; address_len + 2];
    stream
        .read_exact(&mut remainder)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_v3_onion_names() {
        let valid = format!("{}.onion", "a".repeat(56));
        assert_eq!(validate_v3_hostname(&valid).unwrap(), valid);
        assert!(validate_v3_hostname("example.com").is_err());
        assert!(validate_v3_hostname("abcdefghijklmnop.onion").is_err());
        assert!(validate_v3_hostname(&format!("{}!.onion", "a".repeat(55))).is_err());
    }
}
