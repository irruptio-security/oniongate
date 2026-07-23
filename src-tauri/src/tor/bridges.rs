use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::pt::Transport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeLine {
    pub raw: String,
    pub transport: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeScanResult {
    pub raw: String,
    pub transport: String,
    pub endpoint: Option<String>,
    pub ok: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

/// Normalize a pasted/fetched line into a torrc `Bridge …` entry.
pub fn normalize_bridge_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let body = trimmed
        .strip_prefix("Bridge ")
        .or_else(|| trimmed.strip_prefix("bridge "))
        .unwrap_or(trimmed)
        .trim();
    if body.is_empty() {
        return None;
    }
    Some(format!("Bridge {body}"))
}

pub fn parse_bridge_lines(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in text.lines() {
        if let Some(n) = normalize_bridge_line(line) {
            if !out.contains(&n) {
                out.push(n);
            }
        }
    }
    out
}

pub fn describe_bridge(line: &str) -> BridgeLine {
    let raw = normalize_bridge_line(line).unwrap_or_else(|| line.trim().to_string());
    let body = raw.strip_prefix("Bridge ").unwrap_or(raw.as_str());
    let mut parts = body.split_whitespace();
    let first = parts.next().unwrap_or("");
    let (transport, endpoint) = if Transport::from_bridge_token(first).is_some() {
        let ep = parts.next().map(|s| s.to_string());
        (first.to_string(), ep)
    } else {
        ("vanilla".into(), Some(first.to_string()))
    };
    BridgeLine {
        raw,
        transport,
        endpoint,
    }
}

fn extract_host_port(endpoint: &str) -> Option<String> {
    if let Some(rest) = endpoint.strip_prefix('[') {
        let (host, port) = rest.split_once("]:")?;
        return Some(format!("[{host}]:{port}"));
    }
    if endpoint.matches(':').count() == 1 {
        return Some(endpoint.to_string());
    }
    None
}

fn tcp_probe(endpoint: &str, timeout: Duration) -> Result<u64, String> {
    let addr_str =
        extract_host_port(endpoint).ok_or_else(|| format!("Cannot parse endpoint: {endpoint}"))?;
    let addrs: Vec<SocketAddr> = addr_str
        .to_socket_addrs()
        .map_err(|e| format!("DNS/resolve failed for {addr_str}: {e}"))?
        .collect();
    if addrs.is_empty() {
        return Err(format!("No addresses for {addr_str}"));
    }
    let start = Instant::now();
    let mut last_err = String::new();
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => return Ok(start.elapsed().as_millis() as u64),
            Err(e) => last_err = e.to_string(),
        }
    }
    Err(last_err)
}

fn is_documentation_ip(host: &str) -> bool {
    host.starts_with("192.0.2.")
        || host.starts_with("198.51.100.")
        || host.starts_with("203.0.113.")
        || host == "0.0.3.0"
}

fn urlarg_host(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=");
    for part in line.split_whitespace() {
        if let Some(rest) = part.strip_prefix(&needle) {
            let host = rest
                .trim_matches('"')
                .split(['/', ':'])
                .next()
                .unwrap_or("")
                .to_string();
            if !host.is_empty() {
                return Some(host);
            }
        }
    }
    None
}

/// Fronted transports: probe broker/front :443 instead of dummy bridge IPs.
fn fronted_probe_target(info: &BridgeLine) -> Option<String> {
    let raw = &info.raw;
    if let Some(h) = urlarg_host(raw, "url") {
        return Some(format!("{h}:443"));
    }
    if let Some(h) = urlarg_host(raw, "front") {
        return Some(format!("{h}:443"));
    }
    if let Some(h) = urlarg_host(raw, "ampcache") {
        return Some(format!("{h}:443"));
    }
    // meek url=https://host/...
    for part in raw.split_whitespace() {
        if let Some(rest) = part.strip_prefix("url=") {
            let u = rest.trim_matches('"');
            if let Some(after) = u
                .strip_prefix("https://")
                .or_else(|| u.strip_prefix("http://"))
            {
                let hostport = after.split('/').next().unwrap_or(after);
                if hostport.contains(':') {
                    return Some(hostport.to_string());
                }
                if !hostport.is_empty() {
                    return Some(format!("{hostport}:443"));
                }
            }
        }
    }
    None
}

const SCAN_SAMPLE: usize = 40;

pub fn scan_bridges(lines: &[String]) -> Vec<BridgeScanResult> {
    let sample: Vec<&String> = if lines.len() > SCAN_SAMPLE {
        // Evenly sample large catalogs.
        let step = lines.len() as f64 / SCAN_SAMPLE as f64;
        (0..SCAN_SAMPLE)
            .map(|i| &lines[(i as f64 * step) as usize])
            .collect()
    } else {
        lines.iter().collect()
    };

    sample
        .into_iter()
        .map(|line| {
            let info = describe_bridge(line);
            let transport = Transport::from_bridge_token(&info.transport);
            let fronted = transport.map(|t| t.is_fronted()).unwrap_or(false);

            let probe = if fronted {
                if let Some(target) = fronted_probe_target(&info) {
                    Some(target)
                } else if let Some(ep) = &info.endpoint {
                    let host = ep.split(':').next().unwrap_or(ep);
                    if is_documentation_ip(host) {
                        // Fallback: Azure / snowflake fronts commonly used
                        Some("ajax.aspnetcdn.com:443".into())
                    } else {
                        extract_host_port(ep)
                    }
                } else {
                    None
                }
            } else {
                info.endpoint.as_ref().and_then(|ep| extract_host_port(ep))
            };

            match probe {
                Some(target) => match tcp_probe(&target, Duration::from_secs(4)) {
                    Ok(ms) => BridgeScanResult {
                        raw: info.raw,
                        transport: info.transport,
                        endpoint: Some(target),
                        ok: true,
                        latency_ms: Some(ms),
                        error: None,
                    },
                    Err(e) => BridgeScanResult {
                        raw: info.raw,
                        transport: info.transport,
                        endpoint: Some(target),
                        ok: false,
                        latency_ms: None,
                        error: Some(e),
                    },
                },
                None => BridgeScanResult {
                    raw: info.raw,
                    transport: info.transport,
                    endpoint: None,
                    ok: false,
                    latency_ms: None,
                    error: Some("No probe target".into()),
                },
            }
        })
        .collect()
}

/// Tor Browser–style fronted defaults (used offline / thin-set top-up).
pub fn bundled_defaults(transport: &str) -> Vec<String> {
    match transport {
        "snowflake" => parse_bridge_lines(
            r#"
Bridge snowflake 192.0.2.3:80 2B280B23E1107BB62ABFC40DDCC8824814F80A72 fingerprint=2B280B23E1107BB62ABFC40DDCC8824814F80A72 url=https://1098762253.rsc.cdn77.org/ fronts=app.datapacket.com,www.datapacket.com ice=stun:stun.epygi.com:3478,stun:stun.uls.co.za:3478,stun:stun.voipgate.com:3478,stun:stun.mixvoip.com:3478,stun:stun.telnyx.com:3478,stun:stun.hot-chilli.net:3478,stun:stun.fitauto.ru:3478,stun:stun.m-online.net:3478 utls-imitate=hellorandomizedalpn
Bridge snowflake 192.0.2.4:80 8838024498816A039FCBBAB14E6F40A0843051FA fingerprint=8838024498816A039FCBBAB14E6F40A0843051FA url=https://1098762253.rsc.cdn77.org/ fronts=app.datapacket.com,www.datapacket.com ice=stun:stun.epygi.com:3478,stun:stun.uls.co.za:3478,stun:stun.voipgate.com:3478,stun:stun.mixvoip.com:3478,stun:stun.telnyx.com:3478,stun:stun.hot-chilli.net:3478,stun:stun.fitauto.ru:3478,stun:stun.m-online.net:3478 utls-imitate=hellorandomizedalpn
"#,
        ),
        "meek" | "meek-azure" | "meek_lite" => parse_bridge_lines(
            r#"
Bridge meek_lite 192.0.2.20:80 url=https://1603026938.rsc.cdn77.org front=www.phpmyadmin.net utls=HelloRandomizedALPN
"#,
        ),
        "conjure" => parse_bridge_lines(
            r#"
Bridge conjure 99.198.105.162:443 3C5E804C0311CC39F4FDE516EBAE19B6DDC16DD9 url=https://registration.refraction.network/api
"#,
        ),
        _ => Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchBridgesResult {
    pub lines: Vec<String>,
    pub source: String,
    pub transport: String,
    pub from_cache: bool,
}

/// Return trusted built-in transport lines or the user's local cache.
///
/// OnionGate deliberately does not download bridge lines from third-party
/// collectors. Users can paste BridgeDB lines on the Bridges page.
pub async fn fetch_bridge_lines_for(transport: &str) -> Result<FetchBridgesResult, String> {
    let transport = if transport.is_empty() {
        "obfs4"
    } else {
        transport
    };

    if transport == "all" {
        let mut all = Vec::new();
        let mut sources = Vec::new();
        for t in [
            "obfs4",
            "webtunnel",
            "snowflake",
            "meek",
            "conjure",
            "vanilla",
        ] {
            match Box::pin(fetch_bridge_lines_for(t)).await {
                Ok(res) => {
                    for l in res.lines {
                        if !all.contains(&l) {
                            all.push(l);
                        }
                    }
                    sources.push(format!("{}:{}", t, res.source));
                }
                Err(_) => {}
            }
        }
        if !all.is_empty() {
            let _ = crate::db::cache_bridges("all", &all, &sources.join("+"));
            return Ok(FetchBridgesResult {
                lines: all,
                source: sources.join("+"),
                transport: "all".into(),
                from_cache: false,
            });
        }
    }

    // Offline cache
    if let Some((lines, source, _)) = crate::db::load_bridge_cache(transport) {
        if !lines.is_empty() {
            return Ok(FetchBridgesResult {
                lines,
                source: format!("{source} (cached)"),
                transport: transport.into(),
                from_cache: true,
            });
        }
    }

    let bundled = bundled_defaults(transport);
    if !bundled.is_empty() {
        return Ok(FetchBridgesResult {
            lines: bundled,
            source: "bundled-defaults".into(),
            transport: transport.into(),
            from_cache: false,
        });
    }

    Err(format!(
        "No trusted built-in {transport} lines are available. Paste bridges obtained from the Tor Project's BridgeDB."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_deduplicates_bridge_lines() {
        let lines = parse_bridge_lines(
            "\n# ignore\nobfs4 203.0.113.5:443 ABC cert=x\nBridge obfs4 203.0.113.5:443 ABC cert=x\n",
        );
        assert_eq!(lines, vec!["Bridge obfs4 203.0.113.5:443 ABC cert=x"]);
    }

    #[test]
    fn describes_transport_and_endpoint() {
        let info = describe_bridge("Bridge webtunnel 192.0.2.1:443 ABC url=https://example.com/");
        assert_eq!(info.transport, "webtunnel");
        assert_eq!(info.endpoint.as_deref(), Some("192.0.2.1:443"));

        let vanilla = describe_bridge("198.51.100.2:9001 ABC");
        assert_eq!(vanilla.transport, "vanilla");
        assert_eq!(vanilla.endpoint.as_deref(), Some("198.51.100.2:9001"));
    }

    #[test]
    fn parses_ipv4_and_ipv6_probe_targets() {
        assert_eq!(
            extract_host_port("203.0.113.1:443").as_deref(),
            Some("203.0.113.1:443")
        );
        assert_eq!(
            extract_host_port("[2001:db8::1]:443").as_deref(),
            Some("[2001:db8::1]:443")
        );
        assert!(extract_host_port("2001:db8::1:443").is_none());
    }
}
