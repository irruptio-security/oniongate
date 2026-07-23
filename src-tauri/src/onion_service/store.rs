use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use super::OnionProject;

static ACTIVE: LazyLock<Mutex<HashMap<String, OnionProject>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn insert(project: OnionProject) -> Result<(), String> {
    ACTIVE
        .lock()
        .map_err(|_| "Onion project store lock poisoned".to_string())?
        .insert(project.service_id.clone(), project);
    Ok(())
}

pub fn list() -> Vec<OnionProject> {
    ACTIVE
        .lock()
        .map(|projects| projects.values().cloned().collect())
        .unwrap_or_default()
}

pub fn get(service_id: &str) -> Option<OnionProject> {
    ACTIVE
        .lock()
        .ok()
        .and_then(|projects| projects.get(service_id).cloned())
}

pub fn remove(service_id: &str) -> Option<OnionProject> {
    ACTIVE
        .lock()
        .ok()
        .and_then(|mut projects| projects.remove(service_id))
}
