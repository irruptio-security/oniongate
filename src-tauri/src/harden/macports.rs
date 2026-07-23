//! Detect MacPorts and open the official pkg for the current macOS (term7-inspired).

use std::path::PathBuf;
use std::process::Command;

use super::MacPortsStatus;

const INSTALL_PAGE: &str = "https://www.macports.org/install.php";
/// Bump when MacPorts ships a newer stable; used to build a direct pkg URL.
const MACPORTS_VERSION: &str = "2.12.5";

fn sw_vers_product() -> (String, u32) {
    let out = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_default();
    let ver = out.trim().to_string();
    let major = ver
        .split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (ver, major)
}

fn macos_codename(major: u32) -> &'static str {
    match major {
        26 => "Tahoe",
        15 => "Sequoia",
        14 => "Sonoma",
        13 => "Ventura",
        12 => "Monterey",
        11 => "BigSur",
        _ => "Unknown",
    }
}

fn pkg_suffix(major: u32) -> Option<&'static str> {
    match major {
        26 => Some("26-Tahoe"),
        15 => Some("15-Sequoia"),
        14 => Some("14-Sonoma"),
        13 => Some("13-Ventura"),
        12 => Some("12-Monterey"),
        11 => Some("11-BigSur"),
        _ => None,
    }
}

fn find_port() -> Option<PathBuf> {
    if let Ok(p) = which::which("port") {
        return Some(p);
    }
    let p = PathBuf::from("/opt/local/bin/port");
    if p.is_file() {
        Some(p)
    } else {
        None
    }
}

fn port_version(path: &PathBuf) -> String {
    Command::new(path)
        .arg("version")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

pub fn status() -> MacPortsStatus {
    let (macos_version, major) = sw_vers_product();
    let macos_name = macos_codename(major).to_string();
    let download_url = pkg_suffix(major)
        .map(|suffix| {
            format!(
                "https://github.com/macports/macports-base/releases/download/v{MACPORTS_VERSION}/MacPorts-{MACPORTS_VERSION}-{suffix}.pkg"
            )
        })
        .unwrap_or_else(|| INSTALL_PAGE.into());

    if let Some(path) = find_port() {
        let version = port_version(&path);
        MacPortsStatus {
            installed: true,
            version: version.clone(),
            path: path.display().to_string(),
            macos_version,
            macos_name,
            download_url,
            install_page: INSTALL_PAGE.into(),
            detail: format!("MacPorts installed ({version}) at {}", path.display()),
        }
    } else {
        MacPortsStatus {
            installed: false,
            version: String::new(),
            path: String::new(),
            macos_version: macos_version.clone(),
            macos_name: macos_name.clone(),
            download_url,
            install_page: INSTALL_PAGE.into(),
            detail: format!(
                "MacPorts not found. macOS {macos_version} ({macos_name}) — install the official pkg yourself."
            ),
        }
    }
}

pub fn open_download() -> Result<String, String> {
    let st = status();
    let url = if st.installed {
        st.install_page
    } else {
        st.download_url
    };
    let status = Command::new("open")
        .arg(&url)
        .status()
        .map_err(|e| format!("Failed to open browser: {e}"))?;
    if status.success() {
        Ok(if st.installed {
            "Opened MacPorts install guide".into()
        } else {
            format!("Opened MacPorts download for macOS {}", st.macos_name)
        })
    } else {
        Err("Could not open MacPorts URL".into())
    }
}
