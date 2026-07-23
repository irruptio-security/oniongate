use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCheck {
    pub id: String,
    pub label: String,
    pub status: String,
    pub detail: String,
    /// How to fix a warn/fail result (shown in the UI when not passing).
    pub remediation: Option<String>,
}

/// Concrete "how to fix" guidance per check id, shown when the check warns/fails.
fn remediation_for(id: &str) -> Option<String> {
    let text = match id {
        "tor_ip" => {
            "Connect Tor from the Connect screen. If your network blocks Tor, enable a bridge \
             on the Bridges tab (Smart Connect will also try the bundled Snowflake transport)."
        }
        "ip_separation" => {
            "Make sure Tor is connected and your apps route through it (enable the system proxy \
             or TUN on the Routing tab), then run the verifier again."
        }
        "dns" => {
            "Turn on 'Resolve through Tor' in Settings, or use socks5h in proxy apps. In TUN mode \
             DNS is sent to Tor automatically."
        }
        "udp_quic" => {
            "Enable the kill switch on the Routing tab, or switch to TUN mode, to block clearnet \
             UDP/QUIC (this needs an administrator prompt)."
        }
        "ipv6" => {
            "Enable TUN with strict routing to contain IPv6, or disable IPv6 for this network in \
             System Settings."
        }
        "app_policy" => {
            "Enable TUN and add applications under the Apps tab. Isolated per-app routing requires \
             TUN to be active."
        }
        "session_guard" => {
            "Turn on Session Guard under Apps using the 'Only selected via Tor' policy so selected \
             apps are suspended if the Tor route drops."
        }
        "recovery" => {
            "Open the Connect screen and run Emergency Restore to reconcile firewall, TUN, and \
             proxy state with the crash-recovery journal."
        }
        _ => return None,
    };
    Some(text.into())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeakReport {
    pub created_at_unix: u64,
    pub passed: bool,
    pub checks: Vec<VerificationCheck>,
}

fn check(id: &str, label: &str, passed: bool, detail: impl Into<String>) -> VerificationCheck {
    VerificationCheck {
        id: id.into(),
        label: label.into(),
        status: if passed { "pass" } else { "fail" }.into(),
        detail: detail.into(),
        remediation: if passed { None } else { remediation_for(id) },
    }
}

fn warn(id: &str, label: &str, detail: impl Into<String>) -> VerificationCheck {
    VerificationCheck {
        id: id.into(),
        label: label.into(),
        status: "warn".into(),
        detail: detail.into(),
        remediation: remediation_for(id),
    }
}

fn ipv6_default_route() -> bool {
    #[cfg(target_os = "macos")]
    let result = Command::new("route")
        .args(["-n", "get", "-inet6", "default"])
        .output();
    #[cfg(target_os = "linux")]
    let result = Command::new("ip")
        .args(["-6", "route", "show", "default"])
        .output();
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let result: std::io::Result<std::process::Output> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "unsupported",
    ));
    result
        .map(|output| output.status.success() && !output.stdout.is_empty())
        .unwrap_or(false)
}

pub async fn run() -> LeakReport {
    let settings = crate::settings::load();
    let firewall = crate::firewall::status();
    let tun = crate::tun::process_seems_running();
    let direct = crate::ip::fetch_direct_for_verification().await;
    let tor = crate::ip::fetch_via_tor().await;
    let mut checks = Vec::new();

    checks.push(check(
        "tor_ip",
        "Tor egress",
        tor.is_ok(),
        if tor.is_ok() {
            "A remote IP check succeeded through SOCKS hostname resolution"
        } else {
            "Tor egress could not reach the remote verifier"
        },
    ));
    match (&direct, &tor) {
        (Ok(direct), Ok(tor)) => checks.push(check(
            "ip_separation",
            "Direct/Tor IP separation",
            direct != tor,
            if direct != tor {
                "Direct and Tor egress addresses differ (addresses are not stored)"
            } else {
                "Direct and Tor egress addresses unexpectedly match"
            },
        )),
        _ => checks.push(warn(
            "ip_separation",
            "Direct/Tor IP separation",
            "One path was unavailable, so separation could not be compared",
        )),
    }

    checks.push(check(
        "dns",
        "Resolve through Tor",
        !settings.remote_dns || crate::tor::dns_reachable(),
        if settings.remote_dns {
            "Tor's local UDP DNSPort responded"
        } else {
            "Disabled by user; proxy applications must manage DNS explicitly"
        },
    ));
    checks.push(check(
        "udp_quic",
        "UDP/QUIC containment",
        tun || firewall.active,
        if firewall.active && firewall.verified_live {
            "Live firewall rules were inspected"
        } else if tun {
            "TUN policy blocks UDP/QUIC"
        } else {
            "No active TUN or verified firewall rule"
        },
    ));

    let ipv6 = ipv6_default_route();
    checks.push(if !ipv6 || tun {
        check(
            "ipv6",
            "IPv6 route",
            true,
            if ipv6 {
                "An IPv6 default route exists and strict TUN routing is active"
            } else {
                "No IPv6 default route was detected"
            },
        )
    } else {
        warn(
            "ipv6",
            "IPv6 route",
            "IPv6 is available without active TUN containment",
        )
    });

    checks.push(check(
        "app_policy",
        "Selected-app policy",
        !settings.split_tunnel || (!settings.route_apps.is_empty() && tun),
        if settings.split_tunnel && tun {
            "Stable application identities are loaded into the active TUN policy"
        } else if settings.split_tunnel {
            "Application policy is configured but TUN is not active"
        } else {
            "No selected-app policy is enabled"
        },
    ));
    checks.push(
        if settings.session_guard && settings.app_routing_policy == "only" {
            check(
                "session_guard",
                "Session Guard",
                true,
                "Selected processes suspend if the Tor/TUN route disappears",
            )
        } else {
            warn(
                "session_guard",
                "Session Guard",
                "Fail-closed selected-app suspension is not enabled",
            )
        },
    );

    let recovery = crate::session::recovery_status();
    checks.push(check(
        "recovery",
        "Kill-switch recovery",
        !recovery.needed,
        "Live firewall/TUN/proxy state was compared with the crash-recovery journal",
    ));

    let passed = checks.iter().all(|item| item.status != "fail");
    let report = LeakReport {
        created_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        passed,
        checks,
    };
    let _ = crate::db::save_leak_report(&report);
    report
}
