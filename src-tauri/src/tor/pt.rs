use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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

/// Tor splits `ClientTransportPlugin` lines on whitespace and never unquotes the
/// `exec` argument, so a quoted path is exec'd with the quote characters still
/// attached and a path containing whitespace is truncated at the first space.
/// Either way the managed proxy dies with status 1 and Tor retries forever.
fn is_torrc_exec_safe(path: &Path) -> bool {
    path.to_str()
        .is_some_and(|s| !s.is_empty() && !s.chars().any(char::is_whitespace))
}

fn exec_path_string(path: &Path) -> String {
    let raw = path.display().to_string();
    if cfg!(windows) {
        raw.replace('\\', "/")
    } else {
        raw
    }
}

/// Private, whitespace-free directory holding aliases for transport binaries
/// whose real path Tor cannot express.
fn alias_dir() -> Result<PathBuf, String> {
    #[cfg(unix)]
    let candidates = vec![
        std::env::temp_dir(),
        PathBuf::from("/tmp"),
        PathBuf::from("/var/tmp"),
    ];
    // `C:\Program Files\…` is the default install location, so the alias path is
    // the normal case on Windows rather than an edge case.
    #[cfg(not(unix))]
    let candidates = {
        let mut c = vec![std::env::temp_dir()];
        for key in ["ProgramData", "SystemDrive"] {
            if let Ok(value) = std::env::var(key) {
                if !value.is_empty() {
                    c.push(PathBuf::from(value));
                }
            }
        }
        c
    };

    let mut last_error = String::from("no candidate directory");
    for base in candidates {
        if !is_torrc_exec_safe(&base) {
            continue;
        }
        let dir = base.join("oniongate-pt");
        match prepare_private_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(e) => last_error = e,
        }
    }
    Err(format!(
        "No writable whitespace-free directory for transport aliases ({last_error})"
    ))
}

#[cfg(unix)]
fn prepare_private_dir(dir: &Path) -> Result<(), String> {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};

    if let Ok(meta) = fs::symlink_metadata(dir) {
        if !meta.is_dir() {
            return Err(format!("{} is not a directory", dir.display()));
        }
        if meta.permissions().mode() & 0o777 != 0o700 {
            return Err(format!("{} is not private to this user", dir.display()));
        }
        return Ok(());
    }
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .map_err(|e| format!("Failed to create {}: {e}", dir.display()))
}

#[cfg(not(unix))]
fn prepare_private_dir(dir: &Path) -> Result<(), String> {
    if let Ok(meta) = fs::symlink_metadata(dir) {
        if !meta.is_dir() {
            return Err(format!("{} is not a directory", dir.display()));
        }
        return Ok(());
    }
    fs::create_dir_all(dir).map_err(|e| format!("Failed to create {}: {e}", dir.display()))
}

#[cfg(unix)]
fn refresh_alias(target: &Path, alias: &Path) -> Result<(), String> {
    if fs::read_link(alias).is_ok_and(|current| current == target) {
        return Ok(());
    }
    if fs::symlink_metadata(alias).is_ok() {
        fs::remove_file(alias)
            .map_err(|e| format!("Failed to replace {}: {e}", alias.display()))?;
    }
    std::os::unix::fs::symlink(target, alias).map_err(|e| {
        format!(
            "Failed to link {} to {}: {e}",
            alias.display(),
            target.display()
        )
    })
}

#[cfg(not(unix))]
fn refresh_alias(target: &Path, alias: &Path) -> Result<(), String> {
    let current = match (fs::metadata(target), fs::metadata(alias)) {
        (Ok(t), Ok(a)) => t.len() == a.len(),
        _ => false,
    };
    if current {
        return Ok(());
    }
    if fs::symlink_metadata(alias).is_ok() {
        let _ = fs::remove_file(alias);
    }
    if fs::hard_link(target, alias).is_ok() {
        return Ok(());
    }
    fs::copy(target, alias).map(|_| ()).map_err(|e| {
        format!(
            "Failed to copy {} to {}: {e}",
            target.display(),
            alias.display()
        )
    })
}

/// Path for `binary` that survives Tor's torrc parser verbatim.
fn torrc_exec_path(binary: &Path) -> Result<PathBuf, String> {
    if is_torrc_exec_safe(binary) {
        return Ok(binary.to_path_buf());
    }
    let name = binary
        .file_name()
        .ok_or_else(|| format!("Transport binary has no file name: {}", binary.display()))?;
    let alias = alias_dir()?.join(name);
    if !is_torrc_exec_safe(&alias) {
        return Err(format!(
            "Cannot expose {} to Tor: alias path still contains whitespace",
            binary.display()
        ));
    }
    refresh_alias(binary, &alias)?;
    Ok(alias)
}

/// Render one torrc line. Split out from binary discovery so the exact syntax
/// Tor expects is unit-testable without a bundled transport on disk.
fn render_plugin_line(transports: &[&str], exec: &Path, is_conjure: bool) -> String {
    let mut names = transports.to_vec();
    names.sort_unstable();
    names.dedup();
    let joined = names.join(",");
    let path = exec_path_string(exec);
    if is_conjure {
        format!("ClientTransportPlugin {joined} exec {path} -registerURL {CONJURE_REGISTER_URL}")
    } else {
        format!("ClientTransportPlugin {joined} exec {path}")
    }
}

/// Build ClientTransportPlugin lines for transports present in `needed`.
pub fn client_transport_plugin_lines(needed: &[Transport]) -> Result<Vec<String>, String> {
    let mut by_bin: BTreeMap<PathBuf, Vec<&'static str>> = BTreeMap::new();
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
        let exec = torrc_exec_path(&path)?;
        let is_conjure = conjure_path.as_ref() == Some(&path);
        lines.push(render_plugin_line(&transports, &exec, is_conjure));
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Tor exec'd the literal `"` characters and the proxy died with status 1,
    /// which Tor retries forever. The path must appear bare.
    #[test]
    fn plugin_line_never_quotes_the_executable() {
        let line = render_plugin_line(&["obfs4"], Path::new("/opt/oniongate/lyrebird"), false);
        assert_eq!(
            line,
            "ClientTransportPlugin obfs4 exec /opt/oniongate/lyrebird"
        );
        assert!(!line.contains('"'), "torrc exec path must not be quoted");
    }

    #[test]
    fn plugin_line_sorts_and_dedups_transport_names() {
        let line = render_plugin_line(
            &["webtunnel", "obfs4", "meek_lite", "obfs4"],
            Path::new("/opt/lyrebird"),
            false,
        );
        assert_eq!(
            line,
            "ClientTransportPlugin meek_lite,obfs4,webtunnel exec /opt/lyrebird"
        );
    }

    #[test]
    fn conjure_line_carries_the_registration_url() {
        let line = render_plugin_line(&["conjure"], Path::new("/opt/conjure-client"), true);
        assert_eq!(
            line,
            format!("ClientTransportPlugin conjure exec /opt/conjure-client -registerURL {CONJURE_REGISTER_URL}")
        );
    }

    #[test]
    fn rejects_paths_tor_cannot_parse() {
        assert!(is_torrc_exec_safe(Path::new("/opt/oniongate/lyrebird")));
        assert!(!is_torrc_exec_safe(Path::new("/My Apps/lyrebird")));
        assert!(!is_torrc_exec_safe(Path::new("")));
    }

    /// A whitespace-free path must be used as-is, with no alias indirection.
    #[test]
    fn safe_paths_are_passed_through_untouched() {
        let path = Path::new("/opt/oniongate/lyrebird");
        assert_eq!(torrc_exec_path(path).unwrap(), path);
    }

    #[cfg(unix)]
    #[test]
    fn spaced_paths_are_aliased_to_a_parsable_location() {
        let base = std::env::temp_dir().join("oniongate-pt-test/My Apps");
        fs::create_dir_all(&base).unwrap();
        let real = base.join("lyrebird");
        fs::write(&real, b"#!/bin/sh\n").unwrap();

        let exec = torrc_exec_path(&real).unwrap();
        assert!(is_torrc_exec_safe(&exec), "alias must be parsable by Tor");
        assert_eq!(fs::read_link(&exec).unwrap(), real);
        assert_eq!(exec.file_name(), real.file_name());

        // Repeat runs must be idempotent rather than failing on an existing link.
        assert_eq!(torrc_exec_path(&real).unwrap(), exec);

        let line = render_plugin_line(&["obfs4"], &exec, false);
        assert_eq!(line.split_whitespace().count(), 4);

        let _ = fs::remove_file(&exec);
        let _ = fs::remove_dir_all(std::env::temp_dir().join("oniongate-pt-test"));
    }

    #[test]
    fn maps_bridge_tokens_to_transports() {
        let lines = vec![
            "Bridge obfs4 203.0.113.5:443 ABC cert=x".to_string(),
            "Bridge snowflake 192.0.2.3:80 ABC".to_string(),
            "Bridge meek_lite 192.0.2.20:80 url=https://example.com".to_string(),
            "198.51.100.2:9001 ABC".to_string(),
        ];
        assert_eq!(
            transports_from_bridge_lines(&lines),
            vec![
                Transport::Obfs4,
                Transport::Snowflake,
                Transport::Meek,
                Transport::Vanilla
            ]
        );
    }

    #[test]
    fn vanilla_bridges_need_no_transport_plugin() {
        assert!(client_transport_plugin_lines(&[Transport::Vanilla])
            .unwrap()
            .is_empty());
    }

    /// End-to-end against the bundled transport, when `npm run deps` has run.
    #[test]
    fn generated_line_is_launchable_by_tor() {
        if find_pt_binary(Transport::Snowflake).is_none() {
            return;
        }
        let lines = client_transport_plugin_lines(&[Transport::Snowflake]).unwrap();
        let line = lines.first().expect("one plugin line");
        assert!(!line.contains('"'), "{line}");
        let tokens: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(tokens.len(), 4, "Tor parses this line positionally: {line}");
        assert_eq!(
            &tokens[..3],
            &["ClientTransportPlugin", "snowflake", "exec"]
        );
        assert!(Path::new(tokens[3]).exists(), "exec path must resolve");
    }

    /// `meek` is spelled `meek_lite` in torrc; the rest match their bridge token.
    #[test]
    fn transport_ctp_names_match_torrc_spelling() {
        assert_eq!(Transport::Meek.ctp_name(), Some("meek_lite"));
        assert_eq!(Transport::Obfs4.ctp_name(), Some("obfs4"));
        assert_eq!(Transport::Snowflake.ctp_name(), Some("snowflake"));
        assert_eq!(Transport::Vanilla.ctp_name(), None);
    }
}
