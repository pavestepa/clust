use clust::{async_program, project};
use clust_frontend::{
    modules::load,
    parser::parse,
    source::{Diagnostic, Source, SourceId},
};
mod async_command;
mod mixed_command;
mod sync_command;
use clust_semantics::OverflowChecks;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

fn main() -> ExitCode {
    if let Some(tool) = env::var_os("CLUST_METADATA_RUSTDOC") {
        return match clust::interop::metadata::rustdoc(&tool) {
            Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        };
    }
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}
fn render(sources: &[Source], errors: &[Diagnostic]) -> String {
    errors
        .iter()
        .map(|d| d.render(sources))
        .collect::<Vec<_>>()
        .join("\n")
}
fn run() -> Result<ExitCode, String> {
    let mut args: Vec<_> = env::args().skip(1).collect();
    if args.is_empty() || args == ["--help"] {
        println!("Clust 0.1 development\nUsage: clust <syntax|check|eval|emit-rust|build|run> [file.cl|project-directory|Clust.toml] [options]\nOptions: --release --overflow-checks=on|off --package-name=app\nBuild: --emit=object -o <output>\nRun: [-- <program arguments>]\nThe default input is the current project's Clust.toml.\nsyntax checks grammar only; eval is bounded reference execution.\nTarget: host (64-bit); set CLUST_RUSTC or CLUST_CARGO to select build tools.");
        return Ok(ExitCode::SUCCESS);
    }
    if args == ["--version"] {
        println!("clust 0.1.0 (development)");
        return Ok(ExitCode::SUCCESS);
    }
    if args.len() == 1 || args.get(1).is_some_and(|arg| arg.starts_with('-')) {
        args.insert(1, ".".into());
    }
    if args.len() < 2
        || !["syntax", "check", "eval", "emit-rust", "build", "run"].contains(&args[0].as_str())
    {
        return Err("invalid command; use --help".into());
    }
    let mut release = false;
    let mut override_checks = None;
    let mut object = false;
    let mut output = None;
    let mut package = "app".to_owned();
    let mut explicit_package = false;
    let mut index = 2;
    let mut program_arguments = &args[args.len()..];
    while index < args.len() {
        match args[index].as_str() {
            "--" if args[0] == "run" => {
                program_arguments = &args[index + 1..];
                break;
            }
            "--release" => release = true,
            "--overflow-checks=on" => override_checks = Some(OverflowChecks::Checked),
            "--overflow-checks=off" => override_checks = Some(OverflowChecks::Wrapping),
            "--emit=object" if args[0] == "build" => object = true,
            "-o" if args[0] == "build" => {
                index += 1;
                output = Some(PathBuf::from(args.get(index).ok_or("-o requires a path")?));
            }
            option if option.starts_with("--package-name=") => {
                explicit_package = true;
                package = option[15..].into();
                if package.is_empty() {
                    return Err("package name cannot be empty".into());
                }
            }
            option => return Err(format!("unknown option '{option}'")),
        }
        index += 1;
    }
    let project = project::Project::resolve(Path::new(&args[1]))?;
    let input = project
        .as_ref()
        .map_or_else(|| PathBuf::from(&args[1]), |project| project.entry.clone());
    if let Some(project) = &project {
        if !explicit_package {
            package = project.name.clone();
        }
        if object {
            return Err("--emit=object requires a source file input".into());
        }
    }
    let checks = override_checks.unwrap_or(OverflowChecks::Checked);
    let text = fs::read_to_string(&input).map_err(|e| format!("{}: {e}", input.display()))?;
    if let Some(mixed) = clust::interop::compile(&input, &text, project.as_ref(), checks)? {
        return mixed_command::execute(
            &args[0],
            &input,
            project,
            mixed,
            program_arguments,
            async_command::BuildOptions {
                release,
                object,
                output,
                checks,
            },
        );
    }
    if let Some(source) = async_program::compile(&text)? {
        return async_command::execute(
            &args[0],
            &input,
            args[1].ends_with(".cl"),
            project,
            &source,
            program_arguments,
            async_command::BuildOptions {
                release,
                object,
                output,
                checks,
            },
        );
    }
    sync_command::execute(
        &args[0],
        &input,
        &package,
        project.as_ref(),
        program_arguments,
        async_command::BuildOptions {
            release,
            object,
            output,
            checks,
        },
    )
}

/// Keep run artifacts separate from source files, even when compilation fails.
struct RunDirectory(PathBuf);
impl Drop for RunDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

mod native;
use self::native::build;
use self::native::execute_binary;
use self::native::execute_native;
