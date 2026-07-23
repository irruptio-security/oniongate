use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::process::find_binary;

/// Transport name as used in Bridge lines / ClientTransportPlugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    Obfs4,
    Snowflake,
    Webtunnel,
    Meek,
    Conjure,
    Dnstt,
    Vanilla,
}

impl Transport {
    /// Display / settings key.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Obfs4 => "obfs4",
            Self::Snowflake => "snowflake",
            Self::Webtunnel => "webtunnel",
            Self::Meek => "meek",
            Self::Conjure => "conjure",
            Self::Dnstt => "dnstt",
            Self::Vanilla => "vanilla",
        }
    }

    /// Name for ClientTransportPlugin (Tor Browser uses meek_lite).
    pub fn ctp_name(self) -> Option<&'static str> {
        match self {
            Self::Obfs4 => Some("obfs4"),
            Self::Snowflake => Some("snowflake"),
            Self::Webtunnel => Some("webtunnel"),
            Self::Meek => Some("meek_lite"),
            Self::Conjure => Some("conjure"),
            Self::Dnstt => Some("dnstt"),
            Self::Vanilla => None,
        }
    }

    pub fn from_bridge_token(token: &str) -> Option<Self> {
        match token.to_ascii_lowercase().as_str() {
            "obfs4" => Some(Self::Obfs4),
            "snowflake" => Some(Self::Snowflake),
            "webtunnel" => Some(Self::Webtunnel),
            "meek" | "meek_lite" => Some(Self::Meek),
            "conjure" => Some(Self::Conjure),
            "dnstt" => Some(Self::Dnstt),
            _ => None,
        }
    }

    pub fn is_fronted(self) -> bool {
        matches!(self, Self::Snowflake | Self::Meek | Self::Conjure)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtStatus {
    pub transport: String,
    pub binary: Option<String>,
    pub available: bool,
}

fn binary_names(transport: Transport) -> &'static [&'static str] {
    match transport {
        Transport::Obfs4 | Transport::Snowflake | Transport::Webtunnel | Transport::Meek => {
            &["lyrebird", "obfs4proxy"]
        }
        Transport::Conjure => &["conjure-client", "conjure"],
        Transport::Dnstt => &["dnstt-client", "dnstt"],
        Transport::Vanilla => &[],
    }
}

pub fn find_pt_binary(transport: Transport) -> Option<PathBuf> {
    if transport == Transport::Vanilla {
        return None;
    }
    for name in binary_names(transport) {
        if let Some(path) = find_binary(name) {
            return Some(path);
        }
    }
    // Dedicated snowflake-client / webtunnel still accepted if present.
    match transport {
        Transport::Snowflake => find_binary("snowflake-client"),
        Transport::Webtunnel => {
            find_binary("webtunnel-client").or_else(|| find_binary("webtunnel"))
        }
        Transport::Meek => find_binary("meek-client"),
        _ => None,
    }
}

pub fn pt_status_all() -> Vec<PtStatus> {
    [
        Transport::Obfs4,
        Transport::Snowflake,
        Transport::Webtunnel,
        Transport::Meek,
        Transport::Conjure,
        Transport::Dnstt,
    ]
    .into_iter()
    .map(|t| {
        let binary = find_pt_binary(t);
        PtStatus {
            transport: t.as_str().into(),
            available: binary.is_some(),
            binary: binary.map(|p| p.display().to_string()),
        }
    })
    .collect()
}

const CONJURE_REGISTER_URL: &str = "https://registration.refraction.network/api";

/// Build ClientTransportPlugin lines for transports present in `needed`.
pub fn client_transport_plugin_lines(needed: &[Transport]) -> Result<Vec<String>, String> {
    let mut by_bin: HashMap<PathBuf, Vec<&'static str>> = HashMap::new();
    let mut missing = Vec::new();
    let mut conjure_path: Option<PathBuf> = None;

    for t in needed {
        if *t == Transport::Vanilla {
            continue;
        }
        let Some(ctp) = t.ctp_name() else {
            continue;
        };
        match find_pt_binary(*t) {
            Some(path) => {
                if *t == Transport::Conjure {
                    conjure_path = Some(path.clone());
                }
                by_bin.entry(path).or_default().push(ctp);
            }
            None => missing.push(t.as_str()),
        }
    }

    if !missing.is_empty() {
        return Err(format!(
            "Missing pluggable transport binaries for: {}. Run `npm run deps` to bundle lyrebird.",
            missing.join(", ")
        ));
    }

    let mut lines = Vec::new();
    for (path, transports) in by_bin {
        let executable = format!(
            "\"{}\"",
            path.display()
                .to_string()
                .replace('\\', "/")
                .replace('"', "\\\"")
        );
        let mut names = transports;
        names.sort_unstable();
        names.dedup();
        let joined = names.join(",");
        if conjure_path.as_ref() == Some(&path) {
            lines.push(format!(
                "ClientTransportPlugin {joined} exec {} -registerURL {CONJURE_REGISTER_URL}",
                executable
            ));
        } else {
            lines.push(format!("ClientTransportPlugin {joined} exec {executable}",));
        }
    }
    Ok(lines)
}

pub fn transports_from_bridge_lines(lines: &[String]) -> Vec<Transport> {
    let mut out = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        let body = trimmed
            .strip_prefix("Bridge ")
            .or_else(|| trimmed.strip_prefix("bridge "))
            .unwrap_or(trimmed);
        let token = body.split_whitespace().next().unwrap_or("");
        if let Some(t) = Transport::from_bridge_token(token) {
            if !out.contains(&t) {
                out.push(t);
            }
        } else if !token.is_empty()
            && (token.contains('.') || token.chars().next().is_some_and(|c| c.is_ascii_digit()))
        {
            if !out.contains(&Transport::Vanilla) {
                out.push(Transport::Vanilla);
            }
        }
    }
    out
}
