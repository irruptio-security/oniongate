use std::process::Command;

use super::{MacosServiceSnapshot, ProxyStatus, SavedProxyState};
use crate::tor::{SOCKS_HOST, SOCKS_PORT};

fn run(args: &[&str]) -> Result<String, String> {
    let output = Command::new("networksetup")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run networksetup: {e}"))?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if err.is_empty() {
            format!("networksetup {:?} failed", args)
        } else {
            err
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn list_services() -> Result<Vec<String>, String> {
    let out = run(&["-listallnetworkservices"])?;
    Ok(out
        .lines()
        .skip(1)
        .filter(|l| !l.starts_with('*') && !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect())
}

fn hardware_port_for_device(device: &str) -> Option<String> {
    let out = Command::new("networksetup")
        .args(["-listallhardwareports"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut current_port = String::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Hardware Port: ") {
            current_port = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("Device: ") {
            if rest.trim() == device {
                return Some(current_port);
            }
        }
    }
    None
}

fn primary_services() -> Vec<String> {
    let mut preferred = Vec::new();

    if let Ok(out) = Command::new("route")
        .args(["-n", "get", "default"])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if let Some(iface) = line.trim().strip_prefix("interface:") {
                let device = iface.trim();
                if let Some(port) = hardware_port_for_device(device) {
                    preferred.push(port);
                }
            }
        }
    }

    if let Ok(services) = list_services() {
        for s in services {
            if !preferred.contains(&s) {
                preferred.push(s);
            }
        }
    }

    preferred
}

fn get_socks(service: &str) -> Result<MacosServiceSnapshot, String> {
    let out = run(&["-getsocksfirewallproxy", service])?;
    let mut enabled = false;
    let mut server = String::new();
    let mut port = String::new();
    for line in out.lines() {
        if let Some(v) = line.strip_prefix("Enabled: ") {
            enabled = v.trim().eq_ignore_ascii_case("Yes");
        } else if let Some(v) = line.strip_prefix("Server: ") {
            server = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("Port: ") {
            port = v.trim().to_string();
        }
    }
    Ok(MacosServiceSnapshot {
        service: service.to_string(),
        enabled,
        server,
        port,
    })
}

pub fn get_status() -> ProxyStatus {
    let services = primary_services();
    if services.is_empty() {
        return ProxyStatus {
            supported: true,
            enabled: false,
            detail: "No network services found".into(),
            host: SOCKS_HOST.into(),
            port: SOCKS_PORT,
        };
    }

    let mut enabled_on = Vec::new();
    for service in &services {
        if let Ok(snap) = get_socks(service) {
            if snap.enabled && snap.server == SOCKS_HOST && snap.port == SOCKS_PORT.to_string() {
                enabled_on.push(service.clone());
            }
        }
    }

    if !enabled_on.is_empty() {
        ProxyStatus {
            supported: true,
            enabled: true,
            detail: format!("SOCKS enabled on: {}", enabled_on.join(", ")),
            host: SOCKS_HOST.into(),
            port: SOCKS_PORT,
        }
    } else {
        ProxyStatus {
            supported: true,
            enabled: false,
            detail: format!("SOCKS off (checked {})", services.join(", ")),
            host: SOCKS_HOST.into(),
            port: SOCKS_PORT,
        }
    }
}

pub fn capture() -> Result<SavedProxyState, String> {
    let services = primary_services();
    if services.is_empty() {
        return Err("No network services available to configure".into());
    }

    let mut saved = SavedProxyState {
        platform: "macos".into(),
        ..SavedProxyState::default()
    };
    for service in services.iter().take(3) {
        saved.macos_services.push(get_socks(service)?);
    }
    Ok(saved)
}

pub fn enable(saved: &mut SavedProxyState) -> Result<String, String> {
    if saved.macos_services.is_empty() {
        *saved = capture()?;
    }

    let mut configured = Vec::new();
    for service in saved.macos_services.iter().map(|snap| &snap.service) {
        run(&[
            "-setsocksfirewallproxy",
            service,
            SOCKS_HOST,
            &SOCKS_PORT.to_string(),
        ])?;
        run(&["-setsocksfirewallproxystate", service, "on"])?;
        configured.push(service.clone());
    }

    Ok(format!(
        "System SOCKS proxy enabled on {}",
        configured.join(", ")
    ))
}

pub fn disable(saved: &mut SavedProxyState) -> Result<String, String> {
    if !saved.macos_services.is_empty() {
        for snap in &saved.macos_services {
            if snap.enabled && !snap.server.is_empty() {
                run(&[
                    "-setsocksfirewallproxy",
                    &snap.service,
                    &snap.server,
                    &snap.port,
                ])?;
                run(&["-setsocksfirewallproxystate", &snap.service, "on"])?;
            } else {
                run(&["-setsocksfirewallproxystate", &snap.service, "off"])?;
            }
        }
        saved.macos_services.clear();
        return Ok("Restored previous SOCKS proxy settings".into());
    }

    let services = primary_services();
    for service in services.iter().take(3) {
        let _ = run(&["-setsocksfirewallproxystate", service, "off"]);
    }
    Ok("Disabled system SOCKS proxy".into())
}
