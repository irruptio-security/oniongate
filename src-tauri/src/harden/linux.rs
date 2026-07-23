//! Linux hardening deferred — UI shows coming-soon copy.

use super::HardenItem;

pub fn list() -> Vec<HardenItem> {
    Vec::new()
}

pub async fn apply(id: &str, enable: bool) -> Result<String, String> {
    let _ = (id, enable);
    Err("Linux hardening is coming next — not available in this build".into())
}
