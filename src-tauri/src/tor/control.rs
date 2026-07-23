use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::process::{ensure_data_dir, CONTROL_PORT, DNS_PORT, SOCKS_HOST};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlLine {
    pub code: u16,
    pub separator: char,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlReply {
    pub lines: Vec<ControlLine>,
    pub raw: String,
}

impl ControlReply {
    fn parse(raw_lines: Vec<String>) -> Result<Self, String> {
        let mut lines = Vec::with_capacity(raw_lines.len());
        for raw in &raw_lines {
            if raw == "." {
                lines.push(ControlLine {
                    code: 0,
                    separator: '.',
                    body: String::new(),
                });
                continue;
            }
            if raw.len() < 4 {
                return Err(format!("Malformed Tor control reply line: {raw:?}"));
            }
            let code = raw[..3]
                .parse::<u16>()
                .map_err(|_| format!("Malformed Tor control status: {raw:?}"))?;
            let separator = raw.as_bytes()[3] as char;
            if !matches!(separator, ' ' | '-' | '+') {
                return Err(format!("Malformed Tor control separator: {raw:?}"));
            }
            lines.push(ControlLine {
                code,
                separator,
                body: raw[4..].to_string(),
            });
        }
        Ok(Self {
            raw: raw_lines.join("\n"),
            lines,
        })
    }

    pub fn is_ok(&self) -> bool {
        self.lines
            .iter()
            .rev()
            .find(|line| line.code != 0)
            .map(|line| line.code == 250)
            .unwrap_or(false)
    }
}

fn cookie_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(app_dir) = ensure_data_dir() {
        paths.push(app_dir.join("tor-data").join("control_auth_cookie"));
    }

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".tor").join("control_auth_cookie"));
    }

    #[cfg(target_os = "macos")]
    {
        paths.push(PathBuf::from("/usr/local/var/lib/tor/control_auth_cookie"));
        paths.push(PathBuf::from(
            "/opt/homebrew/var/lib/tor/control_auth_cookie",
        ));
        paths.push(PathBuf::from("/usr/local/var/run/tor/control.authcookie"));
        paths.push(PathBuf::from(
            "/opt/homebrew/var/run/tor/control.authcookie",
        ));
    }

    #[cfg(target_os = "linux")]
    {
        paths.push(PathBuf::from("/var/run/tor/control.authcookie"));
        paths.push(PathBuf::from("/run/tor/control.authcookie"));
        paths.push(PathBuf::from("/var/lib/tor/control_auth_cookie"));
    }

    paths
}

fn read_cookie_hex() -> Option<String> {
    for path in cookie_candidates() {
        if let Ok(bytes) = fs::read(&path) {
            if !bytes.is_empty() {
                return Some(hex::encode(bytes));
            }
        }
    }
    None
}

async fn connect() -> Result<TcpStream, String> {
    let addr = format!("{SOCKS_HOST}:{CONTROL_PORT}");
    match timeout(Duration::from_secs(2), TcpStream::connect(&addr)).await {
        Ok(Ok(s)) => Ok(s),
        Ok(Err(e)) => Err(format!(
            "Cannot connect to Tor control port {addr}: {e}. Enable ControlPort 9051 or use the app-managed Tor process."
        )),
        Err(_) => Err(format!("Timed out connecting to Tor control port {addr}")),
    }
}

async fn roundtrip(stream: &mut TcpStream, line: &str) -> Result<ControlReply, String> {
    stream
        .write_all(format!("{line}\r\n").as_bytes())
        .await
        .map_err(|e| format!("Control port write failed: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut lines = Vec::new();
    loop {
        let mut buf = String::new();
        let n = reader
            .read_line(&mut buf)
            .await
            .map_err(|e| format!("Control port read failed: {e}"))?;
        if n == 0 {
            break;
        }
        let trimmed = buf.trim_end_matches(['\r', '\n']).to_string();
        let done = trimmed.len() >= 4 && &trimmed[3..4] == " ";
        lines.push(trimmed);
        if done {
            break;
        }
    }
    ControlReply::parse(lines)
}

async fn authenticate(stream: &mut TcpStream) -> Result<(), String> {
    let auth_cmds = match read_cookie_hex() {
        Some(cookie) => vec![
            format!("AUTHENTICATE {cookie}"),
            "AUTHENTICATE \"\"".to_string(),
        ],
        None => vec!["AUTHENTICATE \"\"".to_string()],
    };

    let mut last_err = String::new();
    for cmd in &auth_cmds {
        let reply = roundtrip(stream, cmd).await?;
        if reply.is_ok() {
            return Ok(());
        }
        last_err = reply.raw;
        // Failed auth often closes the connection; reconnect for next attempt.
        *stream = connect().await?;
    }
    Err(format!(
        "Control port authentication failed. Last reply: {last_err}"
    ))
}

pub async fn run_authenticated(commands: &[&str]) -> Result<ControlReply, String> {
    let mut stream = connect().await?;
    authenticate(&mut stream).await?;
    let mut last = ControlReply {
        lines: Vec::new(),
        raw: String::new(),
    };
    for cmd in commands {
        last = roundtrip(&mut stream, cmd).await?;
        if !last.is_ok() {
            let _ = roundtrip(&mut stream, "QUIT").await;
            return Err(format!("Control command failed ({cmd}): {}", last.raw));
        }
    }
    let _ = roundtrip(&mut stream, "QUIT").await;
    Ok(last)
}

pub async fn new_identity() -> Result<String, String> {
    run_authenticated(&["SIGNAL NEWNYM"]).await?;
    Ok("Requested new Tor identity (NEWNYM)".into())
}

/// Enable or disable Tor DNSPort for remote DNS resolution.
pub async fn apply_remote_dns(enabled: bool) -> Result<String, String> {
    if enabled {
        let dns = format!("SETCONF DNSPort=\"{SOCKS_HOST}:{DNS_PORT}\"");
        run_authenticated(&[dns.as_str(), "SETCONF AutomapHostsOnResolve=1"]).await?;
        // Brief wait for UDP listener.
        tokio::time::sleep(Duration::from_millis(400)).await;
        Ok(format!(
            "Remote DNS enabled (Tor DNSPort {SOCKS_HOST}:{DNS_PORT})"
        ))
    } else {
        run_authenticated(&["SETCONF DNSPort", "SETCONF AutomapHostsOnResolve=0"]).await?;
        Ok("Remote DNS disabled (DNSPort cleared)".into())
    }
}

/// Bootstrap progress 0–100 from GETINFO status/bootstrap-phase.
pub async fn bootstrap_progress() -> Result<u32, String> {
    let reply = run_authenticated(&["GETINFO status/bootstrap-phase"]).await?;
    for line in reply.raw.lines() {
        if let Some(idx) = line.find("PROGRESS=") {
            let rest = &line[idx + "PROGRESS=".len()..];
            let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = num.parse::<u32>() {
                return Ok(n.min(100));
            }
        }
    }
    // If SOCKS is up but we cannot parse, treat as unknown mid-progress.
    Err(format!("Could not parse bootstrap progress: {}", reply.raw))
}

/// Bytes read/written reported by Tor (`traffic/read`, `traffic/written`).
pub async fn traffic_counters() -> Result<(u64, u64), String> {
    let reply = run_authenticated(&["GETINFO traffic/read traffic/written"]).await?;
    let mut read = 0u64;
    let mut written = 0u64;
    for line in reply.raw.lines() {
        let line = line.trim();
        for (key, slot) in [
            ("traffic/read=", &mut read),
            ("traffic/written=", &mut written),
        ] {
            if let Some(idx) = line.find(key) {
                let v = &line[idx + key.len()..];
                let num: String = v.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(n) = num.parse() {
                    *slot = n;
                }
            }
        }
    }
    Ok((read, written))
}

/// Count circuits from `circuit-status`.
pub async fn circuit_count() -> Result<u32, String> {
    let reply = run_authenticated(&["GETINFO circuit-status"]).await?;
    let n = reply
        .raw
        .lines()
        .filter(|l| {
            let t = l
                .trim()
                .strip_prefix("250+circuit-status=")
                .or_else(|| l.trim().strip_prefix("250-circuit-status="))
                .unwrap_or(l.trim());
            if t == "." || t.is_empty() || t.starts_with("250 ") {
                return false;
            }
            t.split_whitespace()
                .next()
                .map(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or(false)
        })
        .count() as u32;
    Ok(n)
}

/// Pin exits to a country code (ISO 3166-1 alpha-2) or clear when empty.
pub async fn apply_exit_country(country: &str) -> Result<String, String> {
    let cc = country.trim().to_ascii_lowercase();
    if cc.is_empty() {
        run_authenticated(&["SETCONF ExitNodes", "SETCONF StrictNodes=0"]).await?;
        return Ok("Exit country pin cleared".into());
    }
    if cc.len() != 2 || !cc.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("Exit country must be a 2-letter ISO code (e.g. de, nl)".into());
    }
    let exit = format!("SETCONF ExitNodes={{{cc}}}");
    run_authenticated(&[exit.as_str(), "SETCONF StrictNodes=1", "SIGNAL NEWNYM"]).await?;
    Ok(format!("Exit nodes pinned to {{{cc}}} (NEWNYM requested)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiline_control_reply() {
        let reply = ControlReply::parse(vec![
            "250-status/bootstrap-phase=NOTICE BOOTSTRAP PROGRESS=80".into(),
            "250 OK".into(),
        ])
        .unwrap();
        assert!(reply.is_ok());
        assert_eq!(reply.lines[0].code, 250);
        assert_eq!(reply.lines[0].separator, '-');
    }

    #[test]
    fn rejects_malformed_control_reply() {
        assert!(ControlReply::parse(vec!["garbage".into()]).is_err());
        let denied = ControlReply::parse(vec!["515 Authentication failed".into()]).unwrap();
        assert!(!denied.is_ok());
    }
}
