use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use super::{ArtifactReport, HostTool, PersistenceEntry, PersistenceReport, PostureCheck};

fn output(program: &str, args: &[&str]) -> (bool, String) {
    Command::new(program)
        .args(args)
        .output()
        .map(|result| {
            let mut text = String::from_utf8_lossy(&result.stdout).to_string();
            text.push_str(&String::from_utf8_lossy(&result.stderr));
            (result.status.success(), text.trim().to_string())
        })
        .unwrap_or_else(|error| (false, error.to_string()))
}

fn posture_check(
    id: &str,
    title: &str,
    good: bool,
    detail: String,
    source: &str,
    remediation: Option<&str>,
) -> PostureCheck {
    PostureCheck {
        id: id.into(),
        title: title.into(),
        status: if good { "pass" } else { "warn" }.into(),
        detail,
        source: source.into(),
        remediation: remediation.map(str::to_string),
    }
}

/// Non-privileged Remote Login (SSH) detection.
///
/// `systemsetup -getremotelogin` requires root even to read status and prints
/// "You need administrator access to run this tool... exiting!" when run
/// un-elevated. Instead, check for a TCP :22 listener via `netstat -an`, which
/// works without privileges.
fn ssh_listening() -> bool {
    let (_, out) = output("netstat", &["-an"]);
    out.lines().any(|line| {
        line.contains("LISTEN")
            && line
                .split_whitespace()
                .any(|column| column.ends_with(".22"))
    })
}

pub fn posture() -> Vec<PostureCheck> {
    let (_, sip) = output("csrutil", &["status"]);
    let (_, filevault) = output("fdesetup", &["status"]);
    let (_, gatekeeper) = output("spctl", &["--status"]);
    let firewall_bin = "/usr/libexec/ApplicationFirewall/socketfilterfw";
    let (_, firewall) = output(firewall_bin, &["--getglobalstate"]);
    let remote_login_active = ssh_listening();
    let (_, sharing) = output("sharing", &["-l"]);
    let (_, listeners) = output("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN"]);
    let leaked = listeners
        .lines()
        .skip(1)
        .filter(|line| {
            line.contains("*:")
                || line.contains("0.0.0.0:")
                || line.contains("[::]:")
                || line.contains(":::")
        })
        .count();

    let xprotect =
        Path::new("/Library/Apple/System/Library/CoreServices/XProtect.bundle/Contents/Info.plist");
    let xprotect_days = fs::metadata(xprotect)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .map(|age| age.as_secs() / 86_400);

    vec![
        posture_check(
            "sip",
            "System Integrity Protection",
            sip.to_ascii_lowercase().contains("enabled"),
            sip,
            "csrutil status",
            Some("Enable SIP from macOS Recovery unless a documented workflow requires otherwise."),
        ),
        posture_check(
            "filevault",
            "FileVault",
            filevault.to_ascii_lowercase().contains("on"),
            filevault,
            "fdesetup status",
            Some("Enable FileVault in System Settings → Privacy & Security."),
        ),
        posture_check(
            "gatekeeper",
            "Gatekeeper",
            gatekeeper.to_ascii_lowercase().contains("enabled"),
            gatekeeper,
            "spctl --status",
            Some("Re-enable Gatekeeper with `sudo spctl --global-enable`."),
        ),
        posture_check(
            "xprotect",
            "XProtect freshness",
            xprotect_days.is_some_and(|days| days <= 30),
            xprotect_days
                .map(|days| format!("XProtect metadata modified {days} day(s) ago"))
                .unwrap_or_else(|| "XProtect metadata was not found".into()),
            "XProtect bundle metadata",
            Some("Install current macOS security data updates."),
        ),
        posture_check(
            "firewall",
            "Application Firewall",
            firewall.contains("State = 1"),
            firewall,
            "socketfilterfw --getglobalstate",
            Some("Enable the macOS firewall in System Settings → Network → Firewall."),
        ),
        posture_check(
            "remote_login",
            "Remote Login",
            !remote_login_active,
            if remote_login_active {
                "Remote Login (SSH) appears active — a service is listening on TCP port 22".into()
            } else {
                "No SSH listener detected on TCP port 22".into()
            },
            "netstat TCP :22 listener probe",
            Some("Disable Remote Login unless SSH access is required."),
        ),
        posture_check(
            "sharing",
            "File sharing",
            !sharing.contains("smb_shared"),
            if sharing.is_empty() {
                "No sharing records returned".into()
            } else {
                "Sharing configuration inspected".into()
            },
            "sharing -l",
            Some("Review Sharing services in System Settings → General → Sharing."),
        ),
        posture_check(
            "listeners",
            "Externally bound TCP listeners",
            leaked == 0,
            format!("{leaked} wildcard TCP listener(s) detected"),
            "lsof TCP LISTEN inventory",
            Some("Bind development services to 127.0.0.1 and disable unneeded listeners."),
        ),
        posture_check(
            "tor_route",
            "Active Tor route",
            crate::tor::socks_reachable(),
            if crate::tor::socks_reachable() {
                "Tor SOCKS is reachable".into()
            } else {
                "Tor SOCKS is not active".into()
            },
            "OnionGate live route probe",
            None,
        ),
    ]
}

fn fingerprint(path: &Path) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.display().to_string().as_bytes());
    if let Ok(data) = fs::read(path) {
        if data.len() <= 4 * 1024 * 1024 {
            hasher.update(&data);
        }
    }
    hex::encode(hasher.finalize())
}

fn signature(path: &Path) -> (Option<bool>, Option<String>) {
    if !path.exists() {
        return (None, None);
    }
    let (valid, text) = output(
        "codesign",
        &["-dv", "--verbose=4", &path.display().to_string()],
    );
    let team = text
        .lines()
        .find_map(|line| line.strip_prefix("TeamIdentifier="))
        .map(str::to_string)
        .filter(|team| team != "not set");
    (Some(valid), team)
}

fn modified(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn add_file(entries: &mut Vec<PersistenceEntry>, kind: &str, path: PathBuf) {
    #[cfg(unix)]
    let is_executable = {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(&path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    };
    let (signed, team_id) = if is_executable {
        signature(&path)
    } else {
        (None, None)
    };
    entries.push(PersistenceEntry {
        id: fingerprint(&path),
        kind: kind.into(),
        path: path.display().to_string(),
        signed,
        team_id,
        modified_unix: modified(&path),
    });
}

fn inventory() -> Vec<PersistenceEntry> {
    let mut entries = Vec::new();
    let home = dirs::home_dir().unwrap_or_default();
    for (kind, dir) in [
        ("User LaunchAgent", home.join("Library/LaunchAgents")),
        ("System LaunchAgent", PathBuf::from("/Library/LaunchAgents")),
        (
            "System LaunchDaemon",
            PathBuf::from("/Library/LaunchDaemons"),
        ),
    ] {
        if let Ok(files) = fs::read_dir(dir) {
            for file in files.flatten() {
                if file.path().extension().and_then(|ext| ext.to_str()) == Some("plist") {
                    let plist_path = file.path();
                    add_file(&mut entries, kind, plist_path.clone());
                    let (_, plist) = output("plutil", &["-p", &plist_path.display().to_string()]);
                    if let Some(executable) = plist
                        .split('"')
                        .find(|value| value.starts_with('/') && Path::new(value).is_file())
                    {
                        add_file(
                            &mut entries,
                            &format!("{kind} executable"),
                            PathBuf::from(executable),
                        );
                    }
                }
            }
        }
    }
    for profile in [
        ".zshrc",
        ".zprofile",
        ".bash_profile",
        ".bashrc",
        ".profile",
    ] {
        let path = home.join(profile);
        if path.is_file() {
            add_file(&mut entries, "Shell profile", path);
        }
    }
    let cron = output("crontab", &["-l"]).1;
    if !cron.is_empty() && !cron.contains("no crontab") {
        let mut hasher = Sha256::new();
        hasher.update(cron.as_bytes());
        entries.push(PersistenceEntry {
            id: hex::encode(hasher.finalize()),
            kind: "User crontab".into(),
            path: "crontab -l".into(),
            signed: None,
            team_id: None,
            modified_unix: 0,
        });
    }
    let (_, extensions) = output("systemextensionsctl", &["list"]);
    for line in extensions.lines().filter(|line| line.contains("activated")) {
        let mut hasher = Sha256::new();
        hasher.update(line.as_bytes());
        entries.push(PersistenceEntry {
            id: hex::encode(hasher.finalize()),
            kind: "System extension".into(),
            path: line.trim().to_string(),
            signed: Some(true),
            team_id: None,
            modified_unix: 0,
        });
    }
    // NOTE: `sfltool dumpbtm` (Background/Login Items) is intentionally excluded
    // from the routine inventory. It reads the Background Task Management store,
    // which is gated by Full Disk Access and raises a TCC prompt on every call.
    // Since the background monitor runs this inventory every 30s, including it
    // here re-prompted the user repeatedly. Login/Background Items are now scanned
    // only on explicit user request via `login_items_snapshot()`.
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries.dedup_by(|a, b| a.id == b.id);
    entries
}

fn baseline_path() -> Result<PathBuf, String> {
    Ok(crate::tor::process::ensure_data_dir()?.join("persistence-baseline.json"))
}

pub fn persistence() -> Result<PersistenceReport, String> {
    let entries = inventory();
    let baseline_path = baseline_path()?;
    let baseline: Vec<PersistenceEntry> = fs::read_to_string(&baseline_path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default();
    let added = entries
        .iter()
        .filter(|entry| !baseline.iter().any(|old| old.id == entry.id))
        .cloned()
        .collect();
    let removed_ids = baseline
        .iter()
        .filter(|old| !entries.iter().any(|entry| entry.id == old.id))
        .map(|entry| entry.id.clone())
        .collect();
    Ok(PersistenceReport {
        entries,
        baseline_exists: baseline_path.is_file(),
        added,
        removed_ids,
    })
}

pub fn save_persistence_baseline() -> Result<String, String> {
    let entries = inventory();
    let raw = serde_json::to_vec_pretty(&entries).map_err(|e| e.to_string())?;
    fs::write(baseline_path()?, raw).map_err(|e| e.to_string())?;
    Ok(format!(
        "Saved baseline with {} persistence entries",
        entries.len()
    ))
}

/// On-demand scan of Background/Login Items via `sfltool dumpbtm`.
///
/// This is deliberately NOT part of the routine inventory: reading the
/// Background Task Management store requires Full Disk Access and triggers a
/// TCC prompt. Calling it only when the user explicitly asks keeps the app from
/// re-prompting on the monitor's 30s loop.
pub fn login_items_snapshot() -> super::LoginItemsSnapshot {
    let (ok, out) = output("sfltool", &["dumpbtm"]);
    let lower = out.to_ascii_lowercase();
    let denied = !ok
        || out.is_empty()
        || lower.contains("full disk access")
        || lower.contains("not permitted")
        || lower.contains("operation not permitted")
        || lower.contains("requires");
    if denied {
        return super::LoginItemsSnapshot {
            available: false,
            items: Vec::new(),
            detail:
                "Could not read Background/Login Items. Grant OnionGate Full Disk Access in System \
                 Settings → Privacy & Security, then scan again."
                    .into(),
        };
    }
    // Light parse: collect each item's Name, annotated with Developer Name when present.
    let mut items: Vec<String> = Vec::new();
    let mut current: Option<String> = None;
    for line in out.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Name:") {
            if let Some(name) = current.take() {
                items.push(name);
            }
            current = Some(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("Developer Name:") {
            if let Some(name) = current.as_mut() {
                let developer = rest.trim();
                if !developer.is_empty() {
                    name.push_str(" — ");
                    name.push_str(developer);
                }
            }
        }
    }
    if let Some(name) = current.take() {
        items.push(name);
    }
    items.retain(|name| !name.is_empty());
    items.sort();
    items.dedup();
    let detail = if items.is_empty() {
        "No background/login items reported.".into()
    } else {
        format!("{} background/login item(s).", items.len())
    };
    super::LoginItemsSnapshot {
        available: true,
        items,
        detail,
    }
}

fn artifact_target(path: &Path) -> Result<PathBuf, String> {
    if path.extension().and_then(|ext| ext.to_str()) == Some("app") {
        let directory = path.join("Contents/MacOS");
        return fs::read_dir(&directory)
            .map_err(|e| format!("Cannot inspect application executable: {e}"))?
            .flatten()
            .map(|entry| entry.path())
            .find(|entry| entry.is_file())
            .ok_or_else(|| "Application has no executable in Contents/MacOS".into());
    }
    Ok(path.to_path_buf())
}

pub fn inspect_artifact(input: &str) -> Result<ArtifactReport, String> {
    let path = PathBuf::from(input);
    if !path.exists() {
        return Err("Selected artifact does not exist".into());
    }
    let target = artifact_target(&path)?;
    let bytes = fs::read(&target).map_err(|e| format!("Cannot hash artifact: {e}"))?;
    let sha256 = hex::encode(Sha256::digest(bytes));
    let (_, quarantine) = output(
        "xattr",
        &["-p", "com.apple.quarantine", &path.display().to_string()],
    );
    let (signature_valid, signature_text) = output(
        "codesign",
        &[
            "--verify",
            "--deep",
            "--strict",
            "--verbose=4",
            &path.display().to_string(),
        ],
    );
    let (_, details) = output(
        "codesign",
        &["-dv", "--verbose=4", &path.display().to_string()],
    );
    let team_id = details
        .lines()
        .find_map(|line| line.strip_prefix("TeamIdentifier="))
        .map(str::to_string)
        .filter(|value| value != "not set");
    let identifier = details
        .lines()
        .find_map(|line| line.strip_prefix("Identifier="))
        .map(str::to_string);
    let authorities = details
        .lines()
        .filter_map(|line| line.strip_prefix("Authority="))
        .map(str::to_string)
        .collect();
    let (_, entitlements_text) = output(
        "codesign",
        &["-d", "--entitlements", ":-", &path.display().to_string()],
    );
    let entitlements = entitlements_text
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("<key>")
                .and_then(|value| value.strip_suffix("</key>"))
                .map(str::to_string)
        })
        .collect();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let assessment_type = if extension == "pkg" {
        "install"
    } else {
        "execute"
    };
    let (notarized, assessment) = output(
        "spctl",
        &[
            "-a",
            "-vv",
            "--type",
            assessment_type,
            &path.display().to_string(),
        ],
    );
    Ok(ArtifactReport {
        path: path.display().to_string(),
        kind: if extension.is_empty() {
            "Mach-O or executable".into()
        } else {
            extension
        },
        sha256: sha256.clone(),
        quarantined: !quarantine.is_empty(),
        quarantine_value: (!quarantine.is_empty()).then_some(quarantine),
        signature_valid,
        notarized,
        identifier,
        team_id,
        authorities,
        entitlements,
        detail: format!("Signature: {signature_text} · Assessment: {assessment}"),
        reputation_url: format!("https://www.virustotal.com/gui/file/{sha256}"),
    })
}

pub fn host_tools() -> Vec<HostTool> {
    [
        (
            "lulu",
            "LuLu",
            "/Applications/LuLu.app",
            "https://objective-see.org/products/lulu.html",
            "Outbound application firewall",
        ),
        (
            "blockblock",
            "BlockBlock",
            "/Applications/BlockBlock Helper.app",
            "https://objective-see.org/products/blockblock.html",
            "Persistence change monitoring",
        ),
        (
            "oversight",
            "OverSight",
            "/Applications/OverSight.app",
            "https://objective-see.org/products/oversight.html",
            "Microphone and camera monitoring",
        ),
        (
            "knockknock",
            "KnockKnock",
            "/Applications/KnockKnock.app",
            "https://objective-see.org/products/knockknock.html",
            "On-demand persistence inventory",
        ),
    ]
    .into_iter()
    .map(|(id, name, path, url, purpose)| HostTool {
        id: id.into(),
        name: name.into(),
        installed: Path::new(path).exists(),
        path: Path::new(path).exists().then(|| path.to_string()),
        official_url: url.into(),
        purpose: purpose.into(),
    })
    .collect()
}
