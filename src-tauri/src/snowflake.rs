use std::process::Stdio;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};

use crate::tor::process::find_binary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowflakeStatus {
    pub binary: Option<String>,
    pub available: bool,
    pub running: bool,
    pub detail: String,
}

pub fn find_proxy_binary() -> Option<std::path::PathBuf> {
    find_binary("snowflake-proxy")
}

fn process_running() -> bool {
    std::process::Command::new("pgrep")
        .args(["-f", "snowflake-proxy"])
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
}

pub fn status(child: &Option<Child>) -> SnowflakeStatus {
    let bin = find_proxy_binary();
    let running = child.as_ref().and_then(|c| c.id()).is_some() || process_running();
    SnowflakeStatus {
        available: bin.is_some(),
        binary: bin.as_ref().map(|p| p.display().to_string()),
        running,
        detail: if bin.is_none() {
            "snowflake-proxy not found on PATH".into()
        } else if running {
            "Volunteering as a Snowflake proxy".into()
        } else {
            "Snowflake proxy idle".into()
        },
    }
}

pub async fn start(managed: &mut Option<Child>) -> Result<String, String> {
    if managed.as_ref().and_then(|c| c.id()).is_some() || process_running() {
        return Ok("Snowflake proxy already running".into());
    }
    let bin = find_proxy_binary().ok_or_else(|| {
        "snowflake-proxy not found. Build/install from https://gitlab.torproject.org/tpo/anti-censorship/pluggable-transports/snowflake".to_string()
    })?;
    let child = Command::new(&bin)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start snowflake-proxy: {e}"))?;
    *managed = Some(child);
    tokio::time::sleep(Duration::from_millis(400)).await;
    if let Some(child) = managed.as_mut() {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!("snowflake-proxy exited early ({status})"));
        }
    }
    crate::logs::append("Started snowflake-proxy volunteer");
    Ok("Started Snowflake proxy (volunteer)".into())
}

pub async fn stop(managed: &mut Option<Child>) -> Result<String, String> {
    if let Some(mut child) = managed.take() {
        let _ = child.kill().await;
        let _ = child.wait().await;
        crate::logs::append("Stopped snowflake-proxy");
        return Ok("Stopped Snowflake proxy".into());
    }
    let _ = Command::new("pkill")
        .args(["-f", "snowflake-proxy"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;
    Ok("Stopped Snowflake proxy (best-effort)".into())
}
