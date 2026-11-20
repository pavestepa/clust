use clust::{async_program, project};
use clust_frontend::{
    modules::load,
    parser::parse,
    source::{Diagnostic, Source, SourceId},
};
mod async_command;
use clust_semantics::OverflowChecks;
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

fn main() -> ExitCode {
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
    if let Some(source) = async_program::compile(
        &fs::read_to_string(&input).map_err(|e| format!("{}: {e}", input.display()))?,
    )? {
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
    if args[0] == "syntax" {
        let text = fs::read_to_string(&input).map_err(|e| format!("{}: {e}", input.display()))?;
        let source = Source::new(input.to_string_lossy(), text);
        let parsed = parse(SourceId(0), &source.text);
        if !parsed.diagnostics.is_empty() {
            return Err(render(&[source], &parsed.diagnostics));
        }
        println!("syntax passed (not a type or ownership check)");
        return Ok(ExitCode::SUCCESS);
    }
    let loaded = load(&input, &package);
    if !loaded.diagnostics.is_empty() {
        return Err(render(&loaded.sources, &loaded.diagnostics));
    }
    let module = loaded.module.unwrap();
    let program =
        clust_core::check::check_clust(&module).map_err(|d| render(&loaded.sources, &d))?;
    match args[0].as_str() {
        "check" => println!("type check passed"),
        "eval" => {
            let main = program.main.ok_or("no main function")?;
            let value =
                clust_core::interpret::execute(&program, main, Vec::new(), checks, 1_000_000)
                    .map_err(|t| {
                        render(
                            &loaded.sources,
                            &[Diagnostic::new(
                                "R0001",
                                format!("reference execution failed: {:?}", t.kind),
                                t.span,
                            )],
                        )
                    })?;
            println!("{value:?}");
        }
        command => {
            let ir = clust_core::rust::emit_with_sources(&program, checks, &loaded.sources)
                .map_err(|d| render(&loaded.sources, &[d]))?;
            if command == "emit-rust" {
                print!("{ir}");
            } else if command == "run" {
                if program.main.is_none() {
                    return Err("executable requires main".into());
                }
                return if let Some(project) = &project {
                    execute_binary(&project.build(&ir, release)?, program_arguments)
                } else {
                    execute_native(&ir, release, program_arguments)
                };
            } else {
                if !object && program.main.is_none() {
                    return Err("executable requires main".into());
                }
                let requested_output = output.clone();
                let output =
                    output.unwrap_or_else(|| input.with_extension(if object { "o" } else { "" }));
                let canonical_output = output.canonicalize().ok();
                if output == input
                    || project.as_ref().is_some_and(|project| {
                        canonical_output.as_ref() == Some(&project.root.join("Clust.toml"))
                    })
                    || loaded.sources.iter().any(|source| {
                        canonical_output
                            .as_ref()
                            .is_some_and(|path| path == Path::new(&source.name))
                    })
                {
                    return Err("output would overwrite source".into());
                }
                if let Some(project) = &project {
                    let artifact = project.build(&ir, release)?;
                    if let Some(output) = requested_output {
                        if output.canonicalize().ok().as_ref() != Some(&artifact) {
                            fs::copy(&artifact, &output)
                                .map_err(|e| format!("{}: {e}", output.display()))?;
                        }
                        println!("built {}", output.display());
                    } else {
                        println!("built {}", artifact.display());
                    }
                } else {
                    build(&ir, &output, object, release)?;
                    println!("built {}", output.display());
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
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
