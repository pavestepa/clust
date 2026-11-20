use crate::{execute_binary, project, RunDirectory};
use clust_semantics::OverflowChecks;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};
pub(crate) struct BuildOptions {
    pub release: bool,
    pub object: bool,
    pub output: Option<PathBuf>,
    pub checks: OverflowChecks,
}
pub(crate) fn execute(
    command: &str,
    input: &Path,
    file_input: bool,
    project: Option<project::Project>,
    source: &str,
    program_arguments: &[String],
    options: BuildOptions,
) -> Result<ExitCode, String> {
    let BuildOptions {
        release,
        object,
        output,
        checks,
    } = options;
    match command {
        "emit-rust" => {
            print!("{source}");
            return Ok(ExitCode::SUCCESS);
        }
        "syntax" => return Ok(ExitCode::SUCCESS),
        "eval" => return Err("use run for Tokio applications".into()),
        _ => (),
    }
    if object {
        return Err("object emission is unsupported for async applications".into());
    }
    let _temp;
    let app = if let Some(app) = project {
        app
    } else {
        let directory = env::temp_dir().join(format!("clust-async-{}", std::process::id()));
        fs::create_dir(&directory).map_err(|e| e.to_string())?;
        _temp = RunDirectory(directory.clone());
        project::Project::standalone(directory, input.to_path_buf())
    };
    let artifact = app
        .with_tokio(checks == OverflowChecks::Checked)?
        .build(source, release)?;
    if command == "run" {
        return execute_binary(&artifact, program_arguments);
    }
    if command == "build" {
        let dest = output.unwrap_or_else(|| {
            if file_input {
                input.with_extension("")
            } else {
                artifact.clone()
            }
        });
        if dest != artifact {
            if dest.canonicalize().ok().is_some_and(|path| {
                Some(&path) == input.canonicalize().ok().as_ref()
                    || path.file_name().is_some_and(|name| name == "Clust.toml")
            }) {
                return Err("output would overwrite source".into());
            }
            let staged = dest.with_file_name(format!(".clust-async-{}.output", std::process::id()));
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staged)
                .map_err(|e| e.to_string())?;
            let result = fs::copy(&artifact, &staged).and_then(|_| fs::rename(&staged, &dest));
            if let Err(error) = result {
                let _ = fs::remove_file(&staged);
                return Err(error.to_string());
            }
        }
        println!("built {}", dest.display());
    }
    Ok(ExitCode::SUCCESS)
}
