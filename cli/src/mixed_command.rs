use super::*;
pub(super) fn execute(
    command: &str,
    input: &Path,
    project: Option<project::Project>,
    mixed: clust::interop::Mixed,
    args: &[String],
    options: async_command::BuildOptions,
) -> Result<ExitCode, String> {
    if command == "eval" {
        return Err("foreign Rust code requires native execution with clust run".into());
    }
    if options.object {
        return Err("mixed crates build a Cargo executable".into());
    }
    if command == "syntax" {
        return Ok(ExitCode::SUCCESS);
    }
    let standalone = project.is_none();
    let _temp;
    let project = if let Some(p) = project {
        p
    } else {
        let dir = env::temp_dir().join(format!("clust-mixed-{}", std::process::id()));
        fs::create_dir(&dir).map_err(|e| e.to_string())?;
        _temp = RunDirectory(dir.clone());
        project::Project::standalone(dir, input.into())
    };
    let project = if mixed.asynchronous {
        project.with_tokio(options.checks == OverflowChecks::Checked)?
    } else {
        project
    };
    let project = project.with_overflow(options.checks == OverflowChecks::Checked);
    let (artifact, source, lock) = project.compile_mixed(&mixed, options.release, command)?;
    match command {
        "emit-rust" => print!("{source}"),
        "run" => {
            let mut child = Command::new(&artifact)
                .args(args)
                .spawn()
                .map_err(|e| format!("cannot execute program: {e}"))?;
            drop(lock);
            return child
                .wait()
                .map(native::exit_code)
                .map_err(|e| e.to_string());
        }
        "build" => {
            let dest = options.output.unwrap_or_else(|| {
                if standalone {
                    input.with_extension("")
                } else {
                    artifact.clone()
                }
            });
            if dest != artifact {
                if dest.canonicalize().ok().is_some_and(|p| {
                    mixed.sources.contains(&p)
                        || p.extension().is_some_and(|ext| {
                            matches!(ext.to_str(), Some("rs" | "cl" | "toml" | "lock"))
                        })
                }) {
                    return Err("output would overwrite source".into());
                }
                let staged =
                    dest.with_file_name(format!(".clust-mixed-{}.output", std::process::id()));
                fs::File::create_new(&staged).map_err(|e| e.to_string())?;
                let result = fs::copy(artifact, &staged).and_then(|_| fs::rename(&staged, &dest));
                if let Err(e) = result {
                    let _ = fs::remove_file(staged);
                    return Err(e.to_string());
                }
            }
            println!("built {}", dest.display());
        }
        _ => {}
    }
    Ok(ExitCode::SUCCESS)
}
