use std::{
    io::{BufRead, BufReader, Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    path::PathBuf,
};

use crate::Command;

// === daemon ===

pub fn setup_listener() -> anyhow::Result<UnixListener> {
    let socket_path = socket_path()?;

    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    Ok(UnixListener::bind(&socket_path)?)
}

pub fn recv_cmd(stream: UnixStream) -> anyhow::Result<Command> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    reader.read_line(&mut line)?;

    let line = line.trim();

    match line.parse() {
        Ok(command) => {
            let mut stream = reader.into_inner();
            stream.write_all(b"is happening")?;
            stream.flush()?;
            Ok(command)
        }
        Err(e) => Err(anyhow::anyhow!("{}", e)),
    }
}

pub fn unlink_socket() -> anyhow::Result<()> {
    let socket_path = socket_path()?;
    std::fs::remove_file(&socket_path)?;
    Ok(())
}

// === cli ===

pub fn invoke_daemon(command: Command) -> anyhow::Result<String> {
    let socket_path = socket_path()?;
    let mut stream = UnixStream::connect(&socket_path)?;
    stream.write_all(command.to_string().as_bytes())?;
    stream.write_all(b"\n")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response.trim().to_string())
}

// === shared ===

fn socket_path() -> anyhow::Result<PathBuf> {
    let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")?;
    Ok(PathBuf::from(xdg_runtime).join("sometime.sock"))
}
