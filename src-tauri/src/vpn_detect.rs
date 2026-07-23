//! Best-effort detection of an active VPN (warn when combined with Tor).
//! Prefer high-confidence signals only — macOS always has several system `utun`
//! interfaces, so counting those alone is a false positive.

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpnStatus {
    pub active: bool,
    pub detail: String,
    pub warning: String,
}

const WARNING: &str = "Running Tor with a VPN can weaken your threat model: the VPN operator sees that you use Tor, traffic may be easier to correlate, and misconfigured setups can leak or break connectivity. Prefer Tor alone unless you understand the tradeoffs.";

pub fn detect() -> VpnStatus {
    #[cfg(target_os = "macos")]
    {
        return detect_macos();
    }
    #[cfg(target_os = "linux")]
    {
        return detect_linux();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        VpnStatus {
            active: false,
            detail: "VPN detection not supported on this platform".into(),
            warning: WARNING.into(),
        }
    }
}

#[cfg(target_os = "macos")]
fn detect_macos() -> VpnStatus {
    let mut hints = Vec::new();

    // High confidence: System Settings / networkconfig VPN services marked Connected.
    if let Ok(out) = Command::new("scutil").args(["--nc", "list"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let lower = line.to_ascii_lowercase();
            if !lower.contains("(connected)") {
                continue;
            }
            // Prefer explicit VPN-ish service types; also accept any *connected* service
            // whose display name clearly looks like a VPN product.
            if lower.contains("vpn")
                || lower.contains("ipsec")
                || lower.contains("ikev2")
                || lower.contains("l2tp")
                || lower.contains("pptp")
                || lower.contains("wireguard")
                || lower.contains("openvpn")
                || lower.contains("cisco")
                || lower.contains("anyconnect")
            {
                hints.push(line.trim().to_string());
            }
        }
    }

    // Do NOT treat multiple utun* interfaces as a VPN — macOS creates several for
    // Continuity, Private Relay scaffolding, etc. even with no VPN connected.

    if hints.is_empty() {
        VpnStatus {
            active: false,
            detail: "No VPN connection detected".into(),
            warning: WARNING.into(),
        }
    } else {
        VpnStatus {
            active: true,
            detail: hints.join("; "),
            warning: WARNING.into(),
        }
    }
}

#[cfg(target_os = "linux")]
fn detect_linux() -> VpnStatus {
    let mut hints = Vec::new();
    if let Ok(out) = Command::new("ip").args(["-brief", "link"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let name = line.split_whitespace().next().unwrap_or("");
            // Generic tun/tap is noisy (containers). Prefer clear VPN names.
            if name.starts_with("wg")
                || name.contains("nordlynx")
                || name.contains("proton")
                || name.starts_with("tun0")
                || name.starts_with("ppp")
            {
                if name.contains("torsocks") {
                    continue;
                }
                if line.contains("UP") {
                    hints.push(name.to_string());
                }
            }
        }
    }
    if hints.is_empty() {
        VpnStatus {
            active: false,
            detail: "No obvious VPN interface detected".into(),
            warning: WARNING.into(),
        }
    } else {
        VpnStatus {
            active: true,
            detail: format!("Possible VPN interfaces: {}", hints.join(", ")),
            warning: WARNING.into(),
        }
    }
}
