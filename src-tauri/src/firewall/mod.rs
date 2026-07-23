use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallStatus {
    pub supported: bool,
    pub active: bool,
    pub verified_live: bool,
    pub marker_active: bool,
    pub detail: String,
}

pub fn status() -> FirewallStatus {
    #[cfg(target_os = "macos")]
    {
        return macos::status();
    }
    #[cfg(target_os = "linux")]
    {
        return linux::status();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::status();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        FirewallStatus {
            supported: false,
            active: false,
            verified_live: false,
            marker_active: false,
            detail: "Kill switch not supported on this OS".into(),
        }
    }
}

/// Enable kill switch: block outbound except loopback + Tor SOCKS path necessities.
pub async fn enable_kill_switch() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::enable().await;
    }
    #[cfg(target_os = "linux")]
    {
        return linux::enable().await;
    }
    #[cfg(target_os = "windows")]
    {
        return windows::enable().await;
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Kill switch not supported on this OS".into())
    }
}

pub async fn disable_kill_switch() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::disable().await;
    }
    #[cfg(target_os = "linux")]
    {
        return linux::disable().await;
    }
    #[cfg(target_os = "windows")]
    {
        return windows::disable().await;
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Kill switch not supported on this OS".into())
    }
}
