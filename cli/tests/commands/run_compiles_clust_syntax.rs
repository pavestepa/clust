use super::*;

#[test]
fn run_compiles_clust_syntax_and_preserves_program_exit_codes() {
    let project = Project::new(include_str!("../../../examples/clust_counter/main.cl"));
    for options in [
        vec!["run", "main.cl"],
        vec!["run", "main.cl", "--release", "--", "--user-option"],
    ] {
        let child = Command::new(env!("CARGO_BIN_EXE_clust"))
            .current_dir(&project.0)
            .args(options)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let directory = std::env::temp_dir().join(format!("clust-run-{}", child.id()));
        let result = child.wait_with_output().unwrap();
        assert_eq!(
            result.status.code(),
            Some(5),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stdout.is_empty());
        assert!(!directory.exists());
        assert!(!project.0.join("main").exists());
    }
}

#[test]
fn run_failures_clean_temporary_outputs_and_do_not_execute_stale_programs() {
    let project = Project::new("fn main()->i32 {return 0;}");
    let child = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["run", "main.cl"])
        .env("CLUST_RUSTC", project.0.join("missing-rustc"))
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let directory = std::env::temp_dir().join(format!("clust-run-{}", child.id()));
    let result = child.wait_with_output().unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("cannot start Rust compiler"));
    assert!(!directory.exists());
    assert!(!project
        .run(&["run", "main.cl", "-o", "main.cl"])
        .status
        .success());
    fs::write(project.0.join("main.cl"), "fn other() {}").unwrap();
    let result = project.run(&["run", "main.cl"]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("executable requires main"));
}

#[test]
fn manifest_projects_build_with_cargo_and_resolve_local_dependencies() {
    let project = Project::new(include_str!("../../../examples/clust_counter/main.cl"));
    fs::create_dir(project.0.join("rust support")).unwrap();
    fs::write(
        project.0.join("rust support/Cargo.toml"),
        "[package]\nname='support'\nversion='0.1.0'\nedition='2021'\n[lib]\npath='lib.rs'\n",
    )
    .unwrap();
    fs::write(
        project.0.join("rust support/lib.rs"),
        "pub fn answer()->i32 {5}\n",
    )
    .unwrap();
    fs::write(project.0.join("Clust.toml"),"[package]\nname='counter'\nentry='main.cl'\n[dependencies]\nsupport={path='rust support'}\n").unwrap();
    for args in [
        vec!["check"],
        vec!["build"],
        vec!["build", "Clust.toml", "--release", "-o", "counter"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_clust"))
            .current_dir(&project.0)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    assert!(project
        .0
        .join("target/clust/artifacts/debug/counter")
        .is_file());
    assert!(project
        .0
        .join("target/clust/generated/Cargo.lock")
        .is_file());
    assert_eq!(
        Command::new(project.0.join("counter"))
            .status()
            .unwrap()
            .code(),
        Some(5)
    );
    let output = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["run", "--release"])
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(5),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let original = fs::read(project.0.join("Clust.toml")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["build", ".", "-o", "Clust.toml"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(project.0.join("Clust.toml")).unwrap(), original);
}

#[test]
fn approved_cli_rejects_unsafe_blocks_and_checks_release_overflow() {
    let project = Project::new("fn main()->i32 {let x:i8=127;return (x+1) as i32;}");
    for args in [
        vec!["eval", "main.cl"],
        vec!["eval", "main.cl", "--release"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_clust"))
            .current_dir(&project.0)
            .args(args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("R0001"));
    }
    fs::write(project.0.join("main.cl"), "fn main() {unsafe {}} ").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["syntax", "main.cl"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Clust has no unsafe blocks"));
}

#[test]
fn standard_library_examples_execute() {
    for source in [
        include_str!("../../../examples/clust_results/main.cl"),
        include_str!("../../../examples/clust_defaults/main.cl"),
        "use core.option.{Option,unwrap_or,is_some};fn main()->i32 {let value:Option<i32>=Option.None;if is_some(value) {return 1;} return unwrap_or(value,7);}",
        "use core.result.{Result,is_ok};fn main()->i32 {let value:Result<i32,i32>=Result.Err(7);if is_ok(value) {return 1;} return match value {Result.Ok(n)=>n,Result.Err(e)=>e};}",
    ] {
        let project=Project::new(source);
        for release in [false,true] {
            let mut command=Command::new(env!("CARGO_BIN_EXE_clust"));
            command.current_dir(&project.0).args(["run","main.cl"]);
            if release {command.arg("--release");}
            let output=command.output().unwrap();
            assert_eq!(output.status.code(),Some(7),"{source}: {}",String::from_utf8_lossy(&output.stderr));
        }
    }
}
