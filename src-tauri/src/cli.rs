use crate::settings;
use crate::tor;

fn usage() {
    eprintln!(
        "\
tor-socks-cli — headless companion

Usage:
  tor-socks-cli status
  tor-socks-cli start
  tor-socks-cli stop
  tor-socks-cli newnym
  tor-socks-cli bridges
  tor-socks-cli settings
"
    );
}

pub async fn run(args: &[String]) -> i32 {
    let cmd = args.get(1).map(String::as_str).unwrap_or("status");
    match cmd {
        "help" | "-h" | "--help" => {
            usage();
            0
        }
        "status" => {
            let s = settings::load();
            println!("tor_installed={}", tor::find_tor_binary().is_some());
            println!("socks_up={}", tor::socks_reachable());
            println!("control_up={}", tor::control_reachable());
            println!("dns_up={}", tor::dns_reachable());
            println!("smart_connect={}", s.smart_connect);
            println!("bridges_enabled={}", s.bridges_enabled);
            println!("bridge_count={}", s.bridge_lines.len());
            println!("connection_mode={}", s.connection_mode);
            println!("kill_switch={}", s.kill_switch);
            println!("exit_country={}", s.exit_country);
            let recovery = crate::session::recovery_status();
            println!("session_phase={:?}", recovery.phase);
            println!("recovery_needed={}", recovery.needed);
            if tor::control_reachable() {
                match tor::bootstrap_progress().await {
                    Ok(p) => println!("bootstrap={p}"),
                    Err(e) => println!("bootstrap_error={e}"),
                }
            }
            0
        }
        "start" => {
            if let Err(e) = crate::session::begin_connect() {
                eprintln!("error: {e}");
                return 1;
            }
            let rt = tokio::runtime::Handle::current();
            let _ = rt;
            let mut managed = None;
            match if settings::load().smart_connect {
                tor::smart_connect(&mut managed).await.map(|r| r.message)
            } else {
                tor::start_tor(&mut managed).await
            } {
                Ok(msg) => {
                    let _ =
                        crate::session::set_phase(crate::session::SessionPhase::Protected, None);
                    println!("{msg}");
                    // Keep process alive briefly so managed child is not dropped immediately.
                    // For CLI, prefer system/managed restart path — leak child for session.
                    std::mem::forget(managed);
                    0
                }
                Err(e) => {
                    let _ = crate::session::set_phase(
                        crate::session::SessionPhase::Degraded,
                        Some(e.clone()),
                    );
                    eprintln!("error: {e}");
                    1
                }
            }
        }
        "stop" => {
            let journal = crate::session::load();
            let managed_tor = tokio::sync::Mutex::new(None);
            let managed_singbox = tokio::sync::Mutex::new(None);
            let managed_snowflake = tokio::sync::Mutex::new(None);
            let saved_proxy = std::sync::Mutex::new(journal.original_proxy.unwrap_or_default());
            match crate::cleanup::teardown_session(
                &managed_tor,
                &managed_singbox,
                &managed_snowflake,
                &saved_proxy,
            )
            .await
            {
                Ok(msg) => {
                    println!("{msg}");
                    0
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    1
                }
            }
        }
        "newnym" => match tor::new_identity().await {
            Ok(msg) => {
                println!("{msg}");
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        },
        "bridges" => {
            let s = settings::load();
            if s.bridge_lines.is_empty() {
                println!("(no bridges configured)");
            } else {
                for line in &s.bridge_lines {
                    println!("{line}");
                }
            }
            0
        }
        "settings" => match serde_json::to_string_pretty(&settings::load()) {
            Ok(j) => {
                println!("{j}");
                0
            }
            Err(e) => {
                eprintln!("error: {e}");
                1
            }
        },
        other => {
            eprintln!("Unknown command: {other}");
            usage();
            2
        }
    }
}
