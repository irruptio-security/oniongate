use std::process::Stdio;

use tokio::process::Command;

use super::FirewallStatus;

const RULE: &str = "OnionGate UDP Internet Guard";

pub fn status() -> FirewallStatus {
    let script = format!(
        "(Get-NetFirewallRule -DisplayName '{}' -ErrorAction SilentlyContinue | Where-Object Enabled -eq 'True').Count",
        RULE
    );
    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output();
    let live = output
        .ok()
        .filter(|result| result.status.success())
        .map(|result| {
            String::from_utf8_lossy(&result.stdout)
                .trim()
                .parse::<u32>()
                .unwrap_or(0)
                > 0
        });
    let active = live.unwrap_or(false);
    FirewallStatus {
        supported: true,
        active,
        verified_live: live.is_some(),
        marker_active: false,
        detail: if active {
            "Windows Defender Firewall blocks outbound UDP to Internet addresses".into()
        } else {
            "OnionGate Windows firewall rule is inactive".into()
        },
    }
}

pub async fn enable() -> Result<String, String> {
    if crate::helper::client::available() {
        let via = tokio::task::spawn_blocking(|| {
            crate::helper::client::request(&crate::helper::HelperRequest::KillSwitchEnable)
        })
        .await
        .map_err(|e| e.to_string())?;
        match via {
            Ok(resp) if resp.ok => return Ok(resp.message),
            Ok(resp) => return Err(resp.message),
            Err(_) => {}
        }
    }
    let script = format!(
        "Get-NetFirewallRule -DisplayName '{RULE}' -ErrorAction SilentlyContinue | Remove-NetFirewallRule; New-NetFirewallRule -DisplayName '{RULE}' -Direction Outbound -Action Block -Protocol UDP -RemoteAddress Internet -Profile Any | Out-Null"
    );
    run_admin(&script).await?;
    if !status().active {
        return Err("Windows firewall rule was not visible after elevation".into());
    }
    Ok("Windows UDP/QUIC Internet guard enabled".into())
}

pub async fn disable() -> Result<String, String> {
    if crate::helper::client::available() {
        let via = tokio::task::spawn_blocking(|| {
            crate::helper::client::request(&crate::helper::HelperRequest::KillSwitchDisable)
        })
        .await
        .map_err(|e| e.to_string())?;
        match via {
            Ok(resp) if resp.ok => return Ok(resp.message),
            Ok(resp) => return Err(resp.message),
            Err(_) => {}
        }
    }
    let script = format!(
        "Get-NetFirewallRule -DisplayName '{RULE}' -ErrorAction SilentlyContinue | Remove-NetFirewallRule"
    );
    run_admin(&script).await?;
    if status().active {
        return Err("Windows firewall rule remains active after restore".into());
    }
    Ok("Windows UDP/QUIC Internet guard disabled".into())
}

async fn run_admin(script: &str) -> Result<(), String> {
    let escaped = script.replace('\'', "''");
    let command = format!(
        "Start-Process powershell.exe -Verb RunAs -Wait -ArgumentList '-NoProfile','-NonInteractive','-Command','{escaped}'"
    );
    let status = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &command])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .map_err(|e| e.to_string())?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "Administrator authorization failed or was cancelled".into())
}
