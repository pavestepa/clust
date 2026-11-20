use super::*;

pub(super) fn execute_native(
    source: &str,
    release: bool,
    arguments: &[String],
) -> Result<ExitCode, String> {
    let directory = env::temp_dir().join(format!("clust-run-{}", std::process::id()));
    fs::create_dir(&directory).map_err(|e| format!("cannot create run directory: {e}"))?;
    let directory = RunDirectory(directory);
    let executable = directory.0.join("program");
    build(source, &executable, false, release)?;
    execute_binary(&executable, arguments)
}

pub(super) fn execute_binary(executable: &Path, arguments: &[String]) -> Result<ExitCode, String> {
    let status = Command::new(executable)
        .args(arguments)
        .status()
        .map_err(|e| format!("cannot execute program: {e}"))?;
    Ok(exit_code(status))
}
pub(super) fn exit_code(status: std::process::ExitStatus) -> ExitCode {
    let code = status.code().unwrap_or_else(|| {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            128 + status.signal().unwrap_or(1)
        }
        #[cfg(not(unix))]
        {
            1
        }
    });
    ExitCode::from(code as u8)
}

pub(super) fn build(
    source: &str,
    output: &Path,
    object: bool,
    release: bool,
) -> Result<(), String> {
    let parent = output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let temporary = parent.join(format!(".clust-{}.output", std::process::id()));
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        let compiler = env::var_os("CLUST_RUSTC").unwrap_or_else(|| "rustc".into());
        let mut process = Command::new(compiler)
            .args([
                "--edition=2021",
                "--crate-name=clust_generated",
                "-C",
                if release {
                    "opt-level=3"
                } else {
                    "opt-level=0"
                },
            ])
            .args(if object {
                vec!["--crate-type=lib", "--emit=obj"]
            } else {
                vec![]
            })
            .args(["-", "-o"])
            .arg(&temporary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("cannot start Rust compiler: {e}"))?;
        let write = process.stdin.take().unwrap().write_all(source.as_bytes());
        let result = process.wait_with_output().map_err(|e| e.to_string())?;
        if !result.status.success() {
            return Err(format!(
                "Rust compilation failed: {}",
                String::from_utf8_lossy(&result.stderr)
            ));
        }
        write.map_err(|e| e.to_string())?;
        fs::rename(&temporary, output).map_err(|e| e.to_string())
    })();
    let _ = fs::remove_file(&temporary);
    result
}
