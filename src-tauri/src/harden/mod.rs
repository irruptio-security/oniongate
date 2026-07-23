//! Opt-in OS privacy helpers (clean-room implementations inspired by common
//! macOS hardening guides — not a vendored copy of third-party scripts).
//! Credits: term7 MacOS Privacy Enhancements, privacy.sexy (see README).

use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
mod kill_siri;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
mod macports;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardenItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub active: bool,
    pub supported: bool,
    pub detail: String,
    /// `privacy` | `security` | `tools`
    pub group: String,
    /// `toggle` | `install` | `link` | `guide`
    pub control: String,
    pub risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KillSiriStatus {
    pub installed: bool,
    pub agent_loaded: bool,
    pub running: Vec<String>,
    pub total_watched: usize,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacPortsStatus {
    pub installed: bool,
    pub version: String,
    pub path: String,
    pub macos_version: String,
    pub macos_name: String,
    pub download_url: String,
    pub install_page: String,
    pub detail: String,
}

pub fn list() -> Vec<HardenItem> {
    #[cfg(target_os = "macos")]
    {
        return macos::list();
    }
    #[cfg(target_os = "linux")]
    {
        return linux::list();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Vec::new()
    }
}

pub async fn apply(id: &str, enable: bool) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::apply(id, enable).await;
    }
    #[cfg(target_os = "linux")]
    {
        return linux::apply(id, enable).await;
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (id, enable);
        Err("Hardening helpers are macOS/Linux only".into())
    }
}

pub fn kill_siri_status() -> KillSiriStatus {
    #[cfg(target_os = "macos")]
    {
        return kill_siri::status();
    }
    #[cfg(not(target_os = "macos"))]
    {
        KillSiriStatus {
            installed: false,
            agent_loaded: false,
            running: Vec::new(),
            total_watched: 0,
            detail: "Kill Siri is macOS only".into(),
        }
    }
}

pub fn macports_status() -> MacPortsStatus {
    #[cfg(target_os = "macos")]
    {
        return macports::status();
    }
    #[cfg(not(target_os = "macos"))]
    {
        MacPortsStatus {
            installed: false,
            version: String::new(),
            path: String::new(),
            macos_version: String::new(),
            macos_name: String::new(),
            download_url: String::new(),
            install_page: "https://www.macports.org/install.php".into(),
            detail: "MacPorts helpers are macOS only".into(),
        }
    }
}

pub fn open_macports_download() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macports::open_download();
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("MacPorts helpers are macOS only".into())
    }
}
