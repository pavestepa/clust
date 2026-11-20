use super::*;

pub(super) fn execute(
    command: &str,
    input: &Path,
    package: &str,
    project: Option<&project::Project>,
    program_arguments: &[String],
    options: async_command::BuildOptions,
) -> Result<ExitCode, String> {
    let async_command::BuildOptions {
        release,
        object,
        output,
        checks,
    } = options;
    if command == "syntax" {
        let text = fs::read_to_string(input).map_err(|e| format!("{}: {e}", input.display()))?;
        let source = Source::new(input.to_string_lossy(), text);
        let parsed = parse(SourceId(0), &source.text);
        if !parsed.diagnostics.is_empty() {
            return Err(render(&[source], &parsed.diagnostics));
        }
        println!("syntax passed (not a type or ownership check)");
        return Ok(ExitCode::SUCCESS);
    }
    let loaded = load(input, package);
    if !loaded.diagnostics.is_empty() {
        return Err(render(&loaded.sources, &loaded.diagnostics));
    }
    let module = loaded.module.unwrap();
    let program =
        clust_core::check::check_clust(&module).map_err(|d| render(&loaded.sources, &d))?;
    match command {
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
                return if let Some(project) = project {
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
                if let Some(project) = project {
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
