use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[cfg(target_os = "macos")]
mod macos;

static PERSISTENCE_CHANGES: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostureCheck {
    pub id: String,
    pub title: String,
    pub status: String,
    pub detail: String,
    pub source: String,
    pub remediation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceEntry {
    pub id: String,
    pub kind: String,
    pub path: String,
    pub signed: Option<bool>,
    pub team_id: Option<String>,
    pub modified_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceReport {
    pub entries: Vec<PersistenceEntry>,
    pub baseline_exists: bool,
    pub added: Vec<PersistenceEntry>,
    pub removed_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostTool {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub path: Option<String>,
    pub official_url: String,
    pub purpose: String,
}

/// Result of an explicit, user-initiated Background/Login Items scan.
/// Separate from the routine persistence inventory because it requires Full
/// Disk Access and triggers a TCC prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginItemsSnapshot {
    pub available: bool,
    pub items: Vec<String>,
    pub detail: String,
}

pub fn posture() -> Vec<PostureCheck> {
    #[cfg(target_os = "macos")]
    {
        macos::posture()
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec![PostureCheck {
            id: "platform".into(),
            title: "Workstation audit".into(),
            status: "info".into(),
            detail: "The focused workstation audit is currently implemented for macOS".into(),
            source: "OnionGate platform adapter".into(),
            remediation: None,
        }]
    }
}

pub fn persistence() -> Result<PersistenceReport, String> {
    #[cfg(target_os = "macos")]
    {
        macos::persistence()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Persistence baseline is currently implemented for macOS".into())
    }
}

pub fn save_persistence_baseline() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        macos::save_persistence_baseline()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Persistence baseline is currently implemented for macOS".into())
    }
}

pub fn host_tools() -> Vec<HostTool> {
    #[cfg(target_os = "macos")]
    {
        macos::host_tools()
    }
    #[cfg(not(target_os = "macos"))]
    {
        Vec::new()
    }
}

/// Explicit, on-demand Background/Login Items scan. Only run when the user asks,
/// so the required Full Disk Access prompt appears once rather than repeatedly.
pub fn login_items_snapshot() -> LoginItemsSnapshot {
    #[cfg(target_os = "macos")]
    {
        macos::login_items_snapshot()
    }
    #[cfg(not(target_os = "macos"))]
    {
        LoginItemsSnapshot {
            available: false,
            items: Vec::new(),
            detail: "Background/Login Items scan is currently implemented for macOS".into(),
        }
    }
}

pub fn persistence_change_count() -> usize {
    PERSISTENCE_CHANGES.load(Ordering::Relaxed)
}

pub fn start_monitor() {
    tauri::async_runtime::spawn(async {
        loop {
            let protected = crate::session::load().phase == crate::session::SessionPhase::Protected;
            if protected {
                if let Ok(report) = persistence() {
                    let changes = if report.baseline_exists {
                        report.added.len() + report.removed_ids.len()
                    } else {
                        0
                    };
                    let previous = PERSISTENCE_CHANGES.swap(changes, Ordering::Relaxed);
                    if changes > 0 && changes != previous {
                        crate::logs::append(format!(
                            "Workstation baseline changed during protected session: {changes} item(s)"
                        ));
                    }
                }
            } else {
                PERSISTENCE_CHANGES.store(0, Ordering::Relaxed);
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}
