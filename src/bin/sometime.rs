use std::{
    env,
    io::{Error, Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    process,
};

fn main() {
    if let Some(cmd) = env::args()
        .nth(1)
        .filter(|arg| matches!(arg.as_str(), "clock" | "calendar"))
    {
        match send_command(&cmd) {
            Ok(response) => println!("{}", response),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    } else {
        eprintln!("Usage: sometime <clock|calendar>");
        process::exit(1);
    }
}

fn send_command(command: &str) -> Result<String, Error> {
    let xdg_runtime = env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set");
    let socket_path = PathBuf::from(xdg_runtime).join("sometime.sock");

    let mut stream = UnixStream::connect(&socket_path)?;
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response.trim().to_string())
}
