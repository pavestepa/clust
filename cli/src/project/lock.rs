//! Serialize writes to the generated package across Cargo invocations.
//! The OS releases this lock if a compiler process exits unexpectedly.
use std::{
    fs::{self, File, OpenOptions},
    io,
    path::Path,
};

pub(super) fn acquire(directory: &Path) -> io::Result<File> {
    fs::create_dir_all(directory)?;
    let path = directory.join("build.lock");
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        let file = options.open(path)?;
        rustix::fs::flock(&file, rustix::fs::FlockOperation::LockExclusive)?;
        Ok(file)
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
        loop {
            match options.open(&path) {
                Err(e) if e.raw_os_error() == Some(32) => {
                    std::thread::sleep(std::time::Duration::from_millis(25))
                }
                result => return result,
            }
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "mixed builds require filesystem locks on this host",
        ))
    }
}
