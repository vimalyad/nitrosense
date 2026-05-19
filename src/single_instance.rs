use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::fd::AsRawFd;
use std::path::PathBuf;

pub struct SingleInstanceGuard {
    _lock_file: File,
}

pub fn acquire() -> io::Result<Option<SingleInstanceGuard>> {
    let path = lock_file_path();
    let mut lock_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)?;

    match try_lock(&lock_file) {
        Ok(()) => {
            lock_file.set_len(0)?;
            writeln!(lock_file, "{}", std::process::id())?;
            Ok(Some(SingleInstanceGuard {
                _lock_file: lock_file,
            }))
        }
        Err(error) if is_lock_contention(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

fn lock_file_path() -> PathBuf {
    if let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime_dir).join("nitrosense.lock");
    }

    let user = env::var("USER").unwrap_or_else(|_| "user".to_owned());
    env::temp_dir().join(format!("nitrosense-{user}.lock"))
}

fn try_lock(lock_file: &File) -> io::Result<()> {
    let result = unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };

    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

fn is_lock_contention(error: &io::Error) -> bool {
    matches!(
        error.raw_os_error(),
        Some(code) if code == libc::EWOULDBLOCK || code == libc::EAGAIN
    )
}
