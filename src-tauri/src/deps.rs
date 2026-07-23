use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::tor::process::find_binary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepStatus {
    pub name: String,
    pub path: Option<String>,
    pub available: bool,
    pub hint: String,
}

pub fn find_singbox() -> Option<PathBuf> {
    find_binary("sing-box")
}

pub fn deps_status() -> Vec<DepStatus> {
    let sing = find_singbox();
    let available = sing.is_some();
    vec![DepStatus {
        name: "sing-box".into(),
        available,
        path: sing.as_ref().map(|p| p.display().to_string()),
        hint: if available {
            "Bundled or system sing-box available".into()
        } else {
            "Bundled sing-box missing — run npm run deps before building".into()
        },
    }]
}
