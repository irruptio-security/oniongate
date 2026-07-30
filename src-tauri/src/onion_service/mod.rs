//! Onion Host: publish a loopback listener as a v3 onion service.
//!
//! Two tiers, with deliberately different key handling:
//!
//! - **Temporary** sites are created over the control port with `DiscardPK`.
//!   Tor throws the key away immediately, so the address can never be
//!   recreated, and the site is destroyed when it is stopped, when Tor stops,
//!   or when the app quits.
//! - **Permanent** sites live in [`persistent`], where Tor owns the key inside
//!   its own `HiddenServiceDir`. They survive restarts on purpose.

pub mod audit;
mod control;
pub mod persistent;
mod store;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A temporary onion site. Its key does not exist anywhere after creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnionProject {
    pub service_id: String,
    pub hostname: String,
    pub local_port: u16,
    pub virtual_port: u16,
    pub private: bool,
    pub client_credential: Option<String>,
    pub created_at_unix: u64,
}

pub async fn start(
    local_port: u16,
    virtual_port: u16,
    private: bool,
) -> Result<OnionProject, String> {
    if local_port == 0 || virtual_port == 0 {
        return Err("Ports must be between 1 and 65535".into());
    }
    if !crate::tor::control_reachable() {
        return Err("Start OnionGate's managed Tor before creating an onion service".into());
    }
    let listener = audit::inspect_listener(local_port);
    if !listener.reachable || !listener.loopback_only {
        return Err(listener.detail);
    }
    let created = control::add(local_port, virtual_port, private).await?;
    let project = OnionProject {
        hostname: format!("{}.onion", created.service_id),
        service_id: created.service_id,
        local_port,
        virtual_port,
        private,
        client_credential: created.client_credential,
        created_at_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    store::insert(project.clone())?;
    crate::logs::append(format!(
        "Created temporary {} onion site for loopback port {}",
        if private { "private" } else { "public" },
        local_port
    ));
    Ok(project)
}

pub fn list() -> Vec<OnionProject> {
    store::list()
}

pub async fn stop(service_id: &str) -> Result<String, String> {
    let project =
        store::get(service_id).ok_or_else(|| "Onion service is not active".to_string())?;
    if project.client_credential.is_some() {
        control::remove_client_authorization(service_id).await;
    }
    control::delete(service_id).await?;
    store::remove(service_id);
    Ok(format!(
        "Destroyed temporary onion site {}",
        project.hostname
    ))
}

/// Destroy every **temporary** site.
///
/// Permanent sites are intentionally untouched: they live in torrc rather than
/// the in-memory store, and surviving a session teardown is the whole point of
/// them. Deleting a permanent site is an explicit user action.
pub async fn stop_all_temporary() {
    for project in store::list() {
        let _ = stop(&project.service_id).await;
    }
}

/// Audit a temporary site.
pub async fn audit_temporary(service_id: &str) -> Result<audit::OnionAudit, String> {
    let project =
        store::get(service_id).ok_or_else(|| "Onion service is not active".to_string())?;
    audit::audit(&audit::AuditTarget {
        service_id: &project.service_id,
        hostname: &project.hostname,
        local_port: project.local_port,
        virtual_port: project.virtual_port,
        client_credential: project.client_credential.as_deref(),
        auth_required_without_credential: false,
    })
    .await
}

/// Audit a permanent site.
pub async fn audit_permanent(id: &str) -> Result<audit::OnionAudit, String> {
    let site = persistent::get(id).ok_or_else(|| "No permanent site with that id".to_string())?;
    let hostname = site
        .hostname
        .clone()
        .ok_or_else(|| "Tor has not published this site yet; try again shortly".to_string())?;
    let service_id = hostname.trim_end_matches(".onion").to_string();
    audit::audit(&audit::AuditTarget {
        service_id: &service_id,
        hostname: &hostname,
        local_port: site.local_port,
        virtual_port: site.virtual_port,
        client_credential: None,
        auth_required_without_credential: site.auth_enabled,
    })
    .await
}
