//! App-side client for the privileged helper daemon.
//!
//! All calls are best-effort: if the daemon is not installed/running,
//! [`available`] returns false and callers fall back to the interactive
//! elevation prompt, so unsigned builds behave exactly as before.

use super::{HelperRequest, HelperResponse};

#[cfg(any(unix, windows))]
use super::{decode, encode};

/// True when the daemon appears reachable.
pub fn available() -> bool {
    #[cfg(unix)]
    {
        std::path::Path::new(super::SOCKET_PATH).exists()
    }
    #[cfg(windows)]
    {
        // A quick connect attempt is the only reliable existence check for a pipe.
        request(&HelperRequest::Ping).map(|r| r.ok).unwrap_or(false)
    }
    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

/// Send one request and read one response. Blocking; call via
/// `tokio::task::spawn_blocking` from async contexts.
#[cfg(unix)]
pub fn request(req: &HelperRequest) -> Result<HelperResponse, String> {
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    let stream =
        UnixStream::connect(super::SOCKET_PATH).map_err(|e| format!("helper unavailable: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .map_err(|e| e.to_string())?;
    stream
        .set_write_timeout(Some(Duration::from_secs(20)))
        .map_err(|e| e.to_string())?;

    let mut write_half = stream.try_clone().map_err(|e| e.to_string())?;
    write_half
        .write_all(&encode(req)?)
        .map_err(|e| format!("helper write failed: {e}"))?;
    write_half.flush().map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("helper read failed: {e}"))?;
    if line.trim().is_empty() {
        return Err("helper closed the connection without responding".into());
    }
    decode::<HelperResponse>(&line)
}

/// Windows named-pipe client. A pipe can be opened like a file for a single
/// request/response exchange.
#[cfg(windows)]
pub fn request(req: &HelperRequest) -> Result<HelperResponse, String> {
    use std::fs::OpenOptions;
    use std::io::{BufRead, BufReader, Write};

    let pipe = OpenOptions::new()
        .read(true)
        .write(true)
        .open(super::PIPE_NAME)
        .map_err(|e| format!("helper unavailable: {e}"))?;
    let mut writer = pipe.try_clone().map_err(|e| e.to_string())?;
    writer
        .write_all(&encode(req)?)
        .map_err(|e| format!("helper write failed: {e}"))?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut reader = BufReader::new(pipe);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("helper read failed: {e}"))?;
    if line.trim().is_empty() {
        return Err("helper closed the connection without responding".into());
    }
    decode::<HelperResponse>(&line)
}

#[cfg(not(any(unix, windows)))]
pub fn request(_req: &HelperRequest) -> Result<HelperResponse, String> {
    Err("privileged helper is not available on this platform".into())
}
