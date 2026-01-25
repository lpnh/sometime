use std::{
    env,
    fs::{self, File, OpenOptions},
    os::unix::io::AsRawFd,
    path::PathBuf,
};

pub struct Flock {
    _file: File, // holds the flock :3
    path: PathBuf,
}

impl Drop for Flock {
    fn drop(&mut self) {
        fs::remove_file(&self.path).unwrap();
    }
}

pub fn try_acquire_daemon_lock() -> Option<Flock> {
    let xdg_runtime = env::var("XDG_RUNTIME_DIR").unwrap();
    let path = PathBuf::from(xdg_runtime).join("sometime.lock");

    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .ok()?;

    let ret = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };

    if ret != 0 {
        eprintln!("sometime-daemon is already running");
        return None;
    }

    Some(Flock { _file: file, path })
}
