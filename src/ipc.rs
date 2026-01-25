use std::{
    env,
    io::{BufRead, BufReader, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

use crate::View;

pub fn setup_ipc_listener() -> std::io::Result<UnixListener> {
    let socket_path = get_socket_path();

    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    UnixListener::bind(&socket_path)
}

pub fn get_socket_path() -> PathBuf {
    let xdg_runtime = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set");
    PathBuf::from(xdg_runtime).join("sometime.sock")
}

pub fn cleanup_socket() {
    let socket_path = get_socket_path();
    let _ = std::fs::remove_file(&socket_path);
}

/// Parse incoming client request and return a command if valid
pub fn handle_client(stream: UnixStream) -> Option<View> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
        return None;
    }

    let line = line.trim();

    let command = match line {
        "clock" => Some(View::Clock),
        "calendar" => Some(View::Calendar),
        _ => {
            let response = format!("Unknown command: {}", line);
            let mut stream = reader.into_inner();
            stream
                .write_all(response.as_bytes())
                .expect("Failed to write response");
            stream.flush().expect("Failed to flush response");
            return None;
        }
    };

    // Send acknowledgment
    let mut stream = reader.into_inner();
    stream
        .write_all(b"is happening")
        .expect("Failed to write response");
    stream.flush().expect("Failed to flush response");

    command
}
