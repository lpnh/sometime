use sometime::ipc;

fn main() {
    if let Some(cmd) = std::env::args().nth(1).and_then(|arg| arg.parse().ok()) {
        match ipc::invoke_daemon(cmd) {
            Ok(response) => println!("{}", response),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Usage: sometime <clock|calendar|dismiss>");
        std::process::exit(1);
    }
}
