use std::fs;
use std::process::Stdio;

use tokio::process::Command;

use super::FirewallStatus;

const TABLE: &str = "tor_socks_gui_ks";

fn marker_path() -> Result<std::path::PathBuf, String> {
    let dir = crate::tor::process::ensure_data_dir()?;
    Ok(dir.join("killswitch.active"))
}

pub fn status() -> FirewallStatus {
    let marker_active =
        marker_path().ok().and_then(|p| fs::read_to_string(p).ok()) == Some("1".into());
    let live = std::process::Command::new("nft")
        .args(["list", "table", "inet", TABLE])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            let rules = String::from_utf8_lossy(&out.stdout);
            rules.contains("hook output") && rules.contains("udp") && rules.contains("drop")
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
                "Kill switch on: live nftables table blocks UDP/QUIC; TUN strict_route handles TCP"
                    .into()
            } else {
                "Kill-switch marker exists; live nftables inspection requires permission".into()
            }
        } else if marker_active {
            "Recovery marker exists, but the nftables table is not active".into()
        } else {
            "Kill switch inactive (live nftables table inspected)".into()
        },
    }
}

fn nft_script_enable() -> String {
    format!(
        "\
nft list table inet {TABLE} >/dev/null 2>&1 && nft delete table inet {TABLE} || true
nft add table inet {TABLE}
nft 'add chain inet {TABLE} output {{ type filter hook output priority 0; policy accept; }}'
nft add rule inet {TABLE} output oif lo accept
nft add rule inet {TABLE} output ip daddr 127.0.0.1 accept
nft add rule inet {TABLE} output ip6 daddr ::1 accept
nft add rule inet {TABLE} output udp dport 53 drop
nft add rule inet {TABLE} output udp dport 443 drop
nft add rule inet {TABLE} output meta l4proto udp drop
"
    )
}

fn nft_script_disable() -> String {
    format!("nft delete table inet {TABLE} 2>/dev/null || true\n")
}

pub async fn enable() -> Result<String, String> {
    if crate::helper::client::available() {
        let via = tokio::task::spawn_blocking(|| {
            crate::helper::client::request(&crate::helper::HelperRequest::KillSwitchEnable)
        })
        .await
        .map_err(|e| e.to_string())?;
        match via {
            Ok(resp) if resp.ok => {}
            Ok(resp) => return Err(resp.message),
            Err(_) => run_root(&nft_script_enable()).await?,
        }
    } else {
        run_root(&nft_script_enable()).await?;
    }
    fs::write(marker_path()?, "1").map_err(|e| e.to_string())?;
    crate::logs::append("Kill switch enabled (Linux nftables UDP block)");
    Ok("Kill switch enabled (UDP/QUIC blocked via nftables)".into())
}

pub async fn disable() -> Result<String, String> {
    if crate::helper::client::available() {
        let via = tokio::task::spawn_blocking(|| {
            crate::helper::client::request(&crate::helper::HelperRequest::KillSwitchDisable)
        })
        .await
        .map_err(|e| e.to_string())?;
        match via {
            Ok(resp) if resp.ok => {}
            Ok(resp) => return Err(resp.message),
            Err(_) => run_root(&nft_script_disable()).await?,
        }
    } else {
        run_root(&nft_script_disable()).await?;
    }
    let live = status();
    if live.verified_live && live.active {
        return Err("nftables table still exists after restore".into());
    }
    if let Ok(p) = marker_path() {
        let _ = fs::remove_file(p);
    }
    crate::logs::append("Kill switch disabled (Linux nftables)");
    Ok("Kill switch disabled".into())
}

async fn run_root(script: &str) -> Result<(), String> {
    if which::which("pkexec").is_ok() {
        let status = Command::new("pkexec")
            .args(["sh", "-c", script])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map_err(|e| e.to_string())?;
        if status.success() {
            return Ok(());
        }
        return Err("pkexec failed or was cancelled".into());
    }
    let status = Command::new("sudo")
        .args(["-n", "sh", "-c", script])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("Need pkexec or passwordless sudo to manage nftables kill switch".into())
    }
}
