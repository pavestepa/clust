//! Provide rustc its normal source directory layout, including sibling data for
//! include_str! and child modules of a file module.
use std::{fs, io, path::Path};

pub(super) fn stage(entry: &Path, destination: &Path) -> io::Result<()> {
    let entry = entry.canonicalize()?;
    let source = entry.parent().unwrap();
    for old in fs::read_dir(destination)? {
        let old = old?;
        if old.file_type()?.is_symlink() {
            let expected = source.join(old.file_name());
            if expected != fs::read_link(old.path())? || fs::symlink_metadata(&expected).is_err() {
                #[cfg(windows)]
                if old.path().is_dir() {
                    fs::remove_dir(old.path())?;
                    continue;
                }
                fs::remove_file(old.path())?;
            }
        }
    }
    for sibling in fs::read_dir(source)? {
        let sibling = sibling?;
        let name = sibling.file_name();
        if name == "main.rs" || name == "target" || name.to_string_lossy().starts_with('.') {
            continue;
        }
        let link = destination.join(&name);
        if fs::symlink_metadata(&link).is_ok() {
            continue;
        }
        let path = sibling.path();
        #[cfg(unix)]
        std::os::unix::fs::symlink(path, link)?;
        #[cfg(windows)]
        if path.is_dir() {
            std::os::windows::fs::symlink_dir(path, link)?;
        } else {
            std::os::windows::fs::symlink_file(path, link)?;
        }
    }
    Ok(())
}
