pub mod audit;
mod control;
mod store;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

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
        "Created ephemeral {} onion service for loopback port {}",
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
        "Destroyed ephemeral onion service {}",
        project.hostname
    ))
}

pub async fn stop_all() {
    for project in store::list() {
        let _ = stop(&project.service_id).await;
    }
}

pub async fn audit(service_id: &str) -> Result<audit::OnionAudit, String> {
    let project =
        store::get(service_id).ok_or_else(|| "Onion service is not active".to_string())?;
    audit::audit(&project).await
}
