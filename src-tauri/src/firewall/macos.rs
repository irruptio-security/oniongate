use std::fs;
use std::process::Stdio;

use tokio::process::Command;

use super::FirewallStatus;

const ANCHOR: &str = "tor.socks.gui";

fn rules_path() -> Result<std::path::PathBuf, String> {
    let dir = crate::tor::process::ensure_data_dir()?;
    Ok(dir.join("pf-killswitch.conf"))
}

fn marker_path() -> Result<std::path::PathBuf, String> {
    let dir = crate::tor::process::ensure_data_dir()?;
    Ok(dir.join("killswitch.active"))
}

pub fn status() -> FirewallStatus {
    let marker_active =
        marker_path().ok().and_then(|p| fs::read_to_string(p).ok()) == Some("1".into());
    let live = std::process::Command::new("pfctl")
        .args(["-a", ANCHOR, "-sr"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            let rules = String::from_utf8_lossy(&out.stdout);
            rules.contains("proto udp") && rules.contains("block")
        });
    let active = live.unwrap_or(marker_active);
    let verified_live = live.is_some();
    FirewallStatus {
        supported: true,
        active,
        verified_live,
        marker_active,
        detail: if active {
            if verified_live {
                "Kill switch on: live pf anchor blocks UDP/QUIC; TUN strict_route handles TCP"
                    .into()
            } else {
                "Kill-switch marker exists; live pf inspection requires permission".into()
            }
        } else if marker_active {
            "Recovery marker exists, but the pf anchor is not active".into()
        } else {
            "Kill switch inactive (live pf anchor inspected)".into()
        },
    }
}

/// Block clearnet UDP (DNS/QUIC leaks). TCP fail-closed relies on sing-box strict_route in TUN mode.
fn pf_rules() -> String {
    "\
# Tor SOCKS GUI — UDP/QUIC leak protection
block drop out quick proto udp from any to any
pass out quick on lo0 proto udp all
pass out quick proto udp to 127.0.0.1
"
    .into()
}

pub async fn enable() -> Result<String, String> {
    let path = rules_path()?;
    fs::write(&path, pf_rules()).map_err(|e| e.to_string())?;

    // Prefer the installed privileged helper (no admin prompt); otherwise fall
    // back to the interactive osascript path.
    if crate::helper::client::available() {
        let via_helper = tokio::task::spawn_blocking(|| {
            crate::helper::client::request(&crate::helper::HelperRequest::KillSwitchEnable)
        })
        .await
        .map_err(|e| e.to_string())?;
        match via_helper {
            Ok(resp) if resp.ok => {}
            Ok(resp) => return Err(resp.message),
            Err(_) => {
                let script = format!(
                    "pfctl -a {ANCHOR} -f {} && pfctl -e || true",
                    shell_escape(&path.display().to_string())
                );
                run_admin(&script).await?;
            }
        }
    } else {
        let script = format!(
            "pfctl -a {ANCHOR} -f {} && pfctl -e || true",
            shell_escape(&path.display().to_string())
        );
        run_admin(&script).await?;
    }
    fs::write(marker_path()?, "1").map_err(|e| e.to_string())?;
    crate::logs::append("Kill switch enabled (macOS pf UDP block)");
    Ok("Kill switch enabled (UDP/QUIC blocked via pf)".into())
}

pub async fn disable() -> Result<String, String> {
    if crate::helper::client::available() {
        let via_helper = tokio::task::spawn_blocking(|| {
            crate::helper::client::request(&crate::helper::HelperRequest::KillSwitchDisable)
        })
        .await
        .map_err(|e| e.to_string())?;
        if let Ok(resp) = via_helper {
            if !resp.ok {
                return Err(resp.message);
            }
        } else {
            let script = format!("pfctl -a {ANCHOR} -F all || true");
            run_admin(&script).await?;
        }
    } else {
        let script = format!("pfctl -a {ANCHOR} -F all || true");
        run_admin(&script).await?;
    }
    let live = status();
    if live.verified_live && live.active {
        return Err("pf anchor still contains OnionGate rules after restore".into());
    }
    if let Ok(p) = marker_path() {
        let _ = fs::remove_file(p);
    }
    crate::logs::append("Kill switch disabled (macOS pf)");
    Ok("Kill switch disabled".into())
}

async fn run_admin(shell: &str) -> Result<(), String> {
    let status = Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "do shell script \"{}\" with administrator privileges",
            shell.replace('\\', "\\\\").replace('"', "\\\"")
        ))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("Administrator authorization failed or was cancelled".into())
    }
}

fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Clearnet UDP must be dropped while loopback stays usable for Tor's DNSPort.
    #[test]
    fn pf_rules_block_clearnet_udp_but_allow_loopback() {
        let rules = pf_rules();
        assert!(rules.contains("block drop out quick proto udp from any to any"));
        assert!(rules.contains("pass out quick on lo0 proto udp all"));
        assert!(rules.contains("pass out quick proto udp to 127.0.0.1"));

        let block = rules
            .lines()
            .position(|l| l.starts_with("block drop"))
            .unwrap();
        let pass = rules.lines().position(|l| l.starts_with("pass")).unwrap();
        assert!(block < pass, "loopback passes must follow the block rule");
    }

    /// These strings are interpolated into a root shell command.
    #[test]
    fn shell_escape_neutralises_quotes_and_metacharacters() {
        assert_eq!(shell_escape("/tmp/pf.conf"), "'/tmp/pf.conf'");
        assert_eq!(
            shell_escape("/tmp/a b/pf.conf"),
            "'/tmp/a b/pf.conf'",
            "spaces must stay inside the quotes"
        );
        assert_eq!(
            shell_escape("/tmp/'; rm -rf /; echo '"),
            "'/tmp/'\\''; rm -rf /; echo '\\'''"
        );
    }

    #[test]
    fn escaped_paths_survive_a_round_trip_through_sh() {
        let nasty = "/tmp/weird '$(id)` dir/pf.conf";
        let escaped = shell_escape(nasty);
        let out = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("printf %s {escaped}"))
            .output()
            .expect("sh runs");
        assert_eq!(String::from_utf8_lossy(&out.stdout), nasty);
    }
}
