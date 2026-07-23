#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SavedProxyState {
    pub platform: String,
    pub macos_services: Vec<MacosServiceSnapshot>,
    pub linux_mode: Option<String>,
    pub linux_socks_host: Option<String>,
    pub linux_socks_port: Option<i32>,
    pub windows_proxy_enable: Option<u32>,
    pub windows_proxy_server: Option<String>,
    pub windows_proxy_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacosServiceSnapshot {
    pub service: String,
    pub enabled: bool,
    pub server: String,
    pub port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatus {
    pub supported: bool,
    pub enabled: bool,
    pub detail: String,
    pub host: String,
    pub port: u16,
}

pub fn get_status() -> ProxyStatus {
    #[cfg(target_os = "macos")]
    {
        return macos::get_status();
    }
    #[cfg(target_os = "linux")]
    {
        return linux::get_status();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::get_status();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        ProxyStatus {
            supported: false,
            enabled: false,
            detail: "Unsupported platform".into(),
            host: crate::tor::SOCKS_HOST.into(),
            port: crate::tor::SOCKS_PORT,
        }
    }
}

pub fn capture() -> Result<SavedProxyState, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::capture();
    }
    #[cfg(target_os = "linux")]
    {
        return linux::capture();
    }
    #[cfg(target_os = "windows")]
    {
        return windows::capture();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        Err("Unsupported platform".into())
    }
}

pub fn enable(saved: &mut SavedProxyState) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::enable(saved);
    }
    #[cfg(target_os = "linux")]
    {
        return linux::enable(saved);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::enable(saved);
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = saved;
        Err("Unsupported platform".into())
    }
}

pub fn disable(saved: &mut SavedProxyState) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        return macos::disable(saved);
    }
    #[cfg(target_os = "linux")]
    {
        return linux::disable(saved);
    }
    #[cfg(target_os = "windows")]
    {
        return windows::disable(saved);
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = saved;
        Err("Unsupported platform".into())
    }
}
