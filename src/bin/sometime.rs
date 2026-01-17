use std::{
    env,
    io::{Error, Read, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    process,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: sometime <clock|calendar>");
        process::exit(1);
    }

    match args[1].as_str() {
        "clock" | "calendar" | "status" => match send_command(&args[1]) {
            Ok(response) => println!("{}", response),
            Err(e) => {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        _ => {
            eprintln!("Usage: sometime <clock|calendar>");
            process::exit(1);
        }
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
