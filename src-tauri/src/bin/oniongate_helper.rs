//! OnionGate privileged helper daemon.
//!
//! Runs with elevated privileges (installed once via launchd/systemd/SCM). It
//! performs ONLY the fixed, typed operations in `tor_socks_gui_lib::helper` —
//! never arbitrary shell — and the privileged rulesets are baked in here, not
//! supplied by the client. On Unix the connecting peer is authenticated by uid.
//!
//! NOTE (pre-ship hardening): this binary links the app library only for the
//! shared protocol types; before release it should be split into a minimal
//! crate so the privileged daemon does not carry the GUI dependency tree.

#[cfg(unix)]
fn main() {
    unix_daemon::run();
}

#[cfg(windows)]
fn main() {
    // Enter the Windows service control dispatcher; falls back to a console
    // run (for debugging) if not launched by the SCM.
    if windows_daemon::run_as_service().is_err() {
        windows_daemon::run_console();
    }
}

#[cfg(not(any(unix, windows)))]
fn main() {
    eprintln!("oniongate-helper is not supported on this platform");
    std::process::exit(1);
}

// ============================ Unix (macOS/Linux) ============================

#[cfg(unix)]
mod unix_daemon {
    use std::fs;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::os::unix::io::AsRawFd;
    use std::os::unix::net::{UnixListener, UnixStream};

    use tor_socks_gui_lib::helper::{decode, encode, HelperRequest, HelperResponse, SOCKET_PATH};

    #[cfg(target_os = "macos")]
    extern "C" {
        fn getpeereid(fd: i32, euid: *mut u32, egid: *mut u32) -> i32;
    }

    #[cfg(target_os = "macos")]
    fn peer_uid(stream: &UnixStream) -> Option<u32> {
        let mut euid: u32 = u32::MAX;
        let mut egid: u32 = u32::MAX;
        let rc = unsafe { getpeereid(stream.as_raw_fd(), &mut euid, &mut egid) };
        (rc == 0).then_some(euid)
    }

    #[cfg(target_os = "linux")]
    fn peer_uid(stream: &UnixStream) -> Option<u32> {
        let mut credentials = std::mem::MaybeUninit::<libc::ucred>::zeroed();
        let mut length = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        let rc = unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                credentials.as_mut_ptr().cast(),
                &mut length,
            )
        };
        if rc != 0 || length as usize != std::mem::size_of::<libc::ucred>() {
            return None;
        }
        Some(unsafe { credentials.assume_init() }.uid)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn peer_uid(_stream: &UnixStream) -> Option<u32> {
        None
    }

    fn allowed_uid() -> Option<u32> {
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg == "--allow-uid" {
                if let Some(uid) = args.next().and_then(|v| v.parse::<u32>().ok()) {
                    return Some(uid);
                }
            }
        }
        // Fall back to the console owner (the logged-in GUI user).
        fs::metadata("/dev/console").ok().map(|m| m.uid())
    }

    pub fn run() {
        let allow = allowed_uid();
        eprintln!("oniongate-helper starting; allowed uid = {allow:?}");
        let _ = fs::remove_file(SOCKET_PATH);
        let listener = match UnixListener::bind(SOCKET_PATH) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("failed to bind {SOCKET_PATH}: {e}");
                std::process::exit(1);
            }
        };
        // World-connectable socket; the peer-uid check below is the real gate.
        let _ = fs::set_permissions(SOCKET_PATH, fs::Permissions::from_mode(0o666));

        for incoming in listener.incoming() {
            match incoming {
                Ok(stream) => handle(stream, allow),
                Err(e) => eprintln!("accept error: {e}"),
            }
        }
    }

    fn handle(stream: UnixStream, allow: Option<u32>) {
        let authorized = match (peer_uid(&stream), allow) {
            (Some(p), Some(a)) => p == a,
            (Some(p), None) => p != 0, // require a non-root user if console unknown
            _ => false,
        };
        if !authorized {
            let _ = respond(&stream, &HelperResponse::err("unauthorized peer"));
            return;
        }
        let mut reader = match stream.try_clone() {
            Ok(s) => BufReader::new(s),
            Err(_) => return,
        };
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
            return;
        }
        let response = match decode::<HelperRequest>(&line) {
            Ok(req) => dispatch(req),
            Err(e) => HelperResponse::err(format!("bad request: {e}")),
        };
        let _ = respond(&stream, &response);
    }

    fn respond(mut stream: &UnixStream, response: &HelperResponse) -> std::io::Result<()> {
        let bytes = encode(response)
            .unwrap_or_else(|_| b"{\"ok\":false,\"message\":\"encode error\"}\n".to_vec());
        stream.write_all(&bytes)?;
        stream.flush()
    }

    fn dispatch(req: HelperRequest) -> HelperResponse {
        match req {
            HelperRequest::Ping => HelperResponse::ok("pong"),
            HelperRequest::KillSwitchEnable => super::executor::kill_switch_enable(),
            HelperRequest::KillSwitchDisable => super::executor::kill_switch_disable(),
        }
    }
}

// ---------------------- Unix privileged executors ----------------------

#[cfg(target_os = "macos")]
mod executor {
    use std::fs;
    use std::path::Path;
    use std::process::Command;
    use tor_socks_gui_lib::helper::HelperResponse;

    const PFCTL: &str = "/sbin/pfctl";
    const PF_ANCHOR: &str = "tor.socks.gui";
    const PF_RULES_PATH: &str = "/var/run/oniongate-pf.conf";
    // Baked, fixed policy — never supplied by the client.
    const PF_RULES: &str = "\
# OnionGate — UDP/QUIC leak protection
pass out quick on lo0 proto udp all
pass out quick proto udp to 127.0.0.1
block drop out quick proto udp from any to any
";

    pub fn kill_switch_enable() -> HelperResponse {
        if !Path::new(PFCTL).exists() {
            return HelperResponse::err("pfctl not found");
        }
        if let Err(e) = fs::write(PF_RULES_PATH, PF_RULES) {
            return HelperResponse::err(format!("write pf rules: {e}"));
        }
        match Command::new(PFCTL)
            .args(["-a", PF_ANCHOR, "-f", PF_RULES_PATH])
            .status()
        {
            Ok(s) if s.success() => {}
            Ok(s) => return HelperResponse::err(format!("pfctl load failed: {s}")),
            Err(e) => return HelperResponse::err(format!("pfctl load error: {e}")),
        }
        let _ = Command::new(PFCTL).arg("-e").status();
        HelperResponse::ok("Kill switch enabled (pf UDP/QUIC block)")
    }

    pub fn kill_switch_disable() -> HelperResponse {
        if !Path::new(PFCTL).exists() {
            return HelperResponse::err("pfctl not found");
        }
        match Command::new(PFCTL)
            .args(["-a", PF_ANCHOR, "-F", "all"])
            .status()
        {
            Ok(s) if s.success() => HelperResponse::ok("Kill switch disabled"),
            Ok(s) => HelperResponse::err(format!("pfctl flush failed: {s}")),
            Err(e) => HelperResponse::err(format!("pfctl flush error: {e}")),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::PF_RULES;

        #[test]
        fn loopback_exceptions_precede_the_quick_udp_block() {
            let block = PF_RULES
                .lines()
                .position(|line| line.starts_with("block drop"))
                .unwrap();
            for pass in PF_RULES
                .lines()
                .enumerate()
                .filter_map(|(index, line)| line.starts_with("pass").then_some(index))
            {
                assert!(pass < block);
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod executor {
    use std::process::Command;
    use tor_socks_gui_lib::helper::HelperResponse;

    const TABLE: &str = "tor_socks_gui_ks";

    fn run_script(script: &str) -> Result<(), String> {
        let status = Command::new("sh")
            .arg("-c")
            .arg(script)
            .status()
            .map_err(|e| e.to_string())?;
        status
            .success()
            .then_some(())
            .ok_or_else(|| format!("nft script failed: {status}"))
    }

    pub fn kill_switch_enable() -> HelperResponse {
        // Baked, fixed policy — never supplied by the client.
        let script = format!(
            "nft list table inet {TABLE} >/dev/null 2>&1 && nft delete table inet {TABLE} || true; \
             nft add table inet {TABLE} && \
             nft 'add chain inet {TABLE} output {{ type filter hook output priority 0; policy accept; }}' && \
             nft add rule inet {TABLE} output oif lo accept && \
             nft add rule inet {TABLE} output ip daddr 127.0.0.1 accept && \
             nft add rule inet {TABLE} output ip6 daddr ::1 accept && \
             nft add rule inet {TABLE} output udp dport 53 drop && \
             nft add rule inet {TABLE} output udp dport 443 drop && \
             nft add rule inet {TABLE} output meta l4proto udp drop"
        );
        match run_script(&script) {
            Ok(()) => HelperResponse::ok("Kill switch enabled (nftables UDP/QUIC block)"),
            Err(e) => HelperResponse::err(e),
        }
    }

    pub fn kill_switch_disable() -> HelperResponse {
        match run_script(&format!(
            "nft delete table inet {TABLE} 2>/dev/null || true"
        )) {
            Ok(()) => HelperResponse::ok("Kill switch disabled"),
            Err(e) => HelperResponse::err(e),
        }
    }
}

// ============================ Windows ============================

#[cfg(windows)]
mod executor {
    use std::process::Command;
    use tor_socks_gui_lib::helper::HelperResponse;

    const RULE: &str = "OnionGate UDP Internet Guard";

    fn powershell(script: &str) -> Result<(), String> {
        let status = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .status()
            .map_err(|e| e.to_string())?;
        status
            .success()
            .then_some(())
            .ok_or_else(|| format!("powershell failed: {status}"))
    }

    pub fn kill_switch_enable() -> HelperResponse {
        let script = format!(
            "Get-NetFirewallRule -DisplayName '{RULE}' -ErrorAction SilentlyContinue | Remove-NetFirewallRule; \
             New-NetFirewallRule -DisplayName '{RULE}' -Direction Outbound -Action Block -Protocol UDP -RemoteAddress Internet -Profile Any | Out-Null"
        );
        match powershell(&script) {
            Ok(()) => HelperResponse::ok("Windows UDP/QUIC Internet guard enabled"),
            Err(e) => HelperResponse::err(e),
        }
    }

    pub fn kill_switch_disable() -> HelperResponse {
        let script = format!(
            "Get-NetFirewallRule -DisplayName '{RULE}' -ErrorAction SilentlyContinue | Remove-NetFirewallRule"
        );
        match powershell(&script) {
            Ok(()) => HelperResponse::ok("Windows UDP/QUIC Internet guard disabled"),
            Err(e) => HelperResponse::err(e),
        }
    }
}

#[cfg(windows)]
mod windows_daemon {
    use std::ffi::OsString;
    use std::time::Duration;

    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::windows::named_pipe::ServerOptions;
    use tor_socks_gui_lib::helper::{decode, encode, HelperRequest, HelperResponse, PIPE_NAME};
    use windows_service::service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    };
    use windows_service::service_control_handler::{self, ServiceControlHandlerResult};
    use windows_service::{define_windows_service, service_dispatcher};

    const SERVICE_NAME: &str = "OnionGateHelper";

    define_windows_service!(ffi_service_main, service_main);

    pub fn run_as_service() -> Result<(), windows_service::Error> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    fn service_main(_args: Vec<OsString>) {
        let _ = run_service();
    }

    fn run_service() -> Result<(), Box<dyn std::error::Error>> {
        let status_handle =
            service_control_handler::register(SERVICE_NAME, move |control| match control {
                ServiceControl::Stop | ServiceControl::Interrogate => {
                    ServiceControlHandlerResult::NoError
                }
                _ => ServiceControlHandlerResult::NotImplemented,
            })?;
        let running = ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        };
        status_handle.set_service_status(running)?;
        serve_pipe();
        Ok(())
    }

    // Console fallback for debugging (not run under SCM).
    pub fn run_console() {
        serve_pipe();
    }

    fn serve_pipe() {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!("failed to start runtime: {e}");
                return;
            }
        };
        rt.block_on(async {
            loop {
                let server = match ServerOptions::new().create(PIPE_NAME) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("pipe create error: {e}");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                };
                if server.connect().await.is_err() {
                    continue;
                }
                let mut reader = BufReader::new(server);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() || line.trim().is_empty() {
                    continue;
                }
                let response = match decode::<HelperRequest>(&line) {
                    Ok(HelperRequest::Ping) => HelperResponse::ok("pong"),
                    Ok(HelperRequest::KillSwitchEnable) => super::executor::kill_switch_enable(),
                    Ok(HelperRequest::KillSwitchDisable) => super::executor::kill_switch_disable(),
                    Err(e) => HelperResponse::err(format!("bad request: {e}")),
                };
                let mut inner = reader.into_inner();
                if let Ok(bytes) = encode(&response) {
                    let _ = inner.write_all(&bytes).await;
                    let _ = inner.flush().await;
                }
            }
        });
    }
}
