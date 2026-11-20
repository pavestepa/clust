use super::*;

#[test]
fn published_examples_and_benchmarks_use_current_clust_syntax() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let examples = fs::read_dir(root.join("examples"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join("main.cl"));
    let benches = fs::read_dir(root.join("performance/bench"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "cl"));
    for source in examples.chain(benches) {
        let output = Command::new(env!("CARGO_BIN_EXE_clust"))
            .arg("check")
            .arg(&source)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}: {}",
            source.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn obsolete_source_syntax_has_no_cli_mode() {
    let project = Project::new("class C { constructor() {} }");
    for command in ["syntax", "check", "eval", "emit-rust", "build", "run"] {
        let result = project.run(&[command, "main.cl"]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("E0200"));
    }
    let result = project.run(&["check", "main.cl", "--experimental"]);
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown option"));
}

#[test]
fn commands_load_imports_and_report_original_locations() {
    let project =
        Project::new("mod lib;use crate.lib.{answer};\nfn main()->i32 {return answer();}");
    fs::write(
        project.0.join("lib.cl"),
        "pub fn answer()->i32 {return 42;}",
    )
    .unwrap();
    assert!(project.run(&["check", "main.cl"]).status.success());
    let result = project.run(&["eval", "main.cl"]);
    assert!(result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("42"));
    fs::write(
        project.0.join("lib.cl"),
        "pub fn answer()->i32 {\nreturn true;\n}",
    )
    .unwrap();
    let result = project.run(&["check", "main.cl"]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("lib.cl:2:"));
}

#[test]
fn build_emits_host_object_without_overwriting_imported_sources() {
    let project =
        Project::new("mod lib;use crate.lib.{answer};\nfn main()->i32 {return answer();}");
    fs::write(
        project.0.join("lib.cl"),
        "pub fn answer()->i32 {return 42;}",
    )
    .unwrap();
    let result = project.run(&["build", "main.cl", "--emit=object", "-o", "main.o"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(fs::metadata(project.0.join("main.o")).unwrap().len() > 0);
    for output in ["main.cl", "lib.cl"] {
        let result = project.run(&["build", "main.cl", "--emit=object", "-o", output]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("overwrite source"));
    }
}

#[test]
fn release_overflow_policy_can_be_overridden() {
    let project = Project::new("fn main()->i32 {let x:i8=127;return (x+1) as i32;}");
    assert!(!project.run(&["eval", "main.cl"]).status.success());
    assert!(project
        .run(&["eval", "main.cl", "--release", "--overflow-checks=off"])
        .status
        .success());
    assert!(!project
        .run(&["eval", "main.cl", "--release", "--overflow-checks=on"])
        .status
        .success());
}

#[test]
fn rust_source_and_host_executables_are_available_without_clang() {
    let project = Project::new("fn main()->i32 {let x:i8=127;return (x+1) as i32;}");
    let source = project.run(&["emit-rust", "main.cl"]);
    assert!(source.status.success());
    assert!(String::from_utf8_lossy(&source.stdout).contains("fn main()"));
    assert!(!project.run(&["emit-llvm", "main.cl"]).status.success());
    for (options, expected) in [
        (vec!["build", "main.cl", "-o", "program"], 101),
        (
            vec![
                "build",
                "main.cl",
                "--release",
                "--overflow-checks=off",
                "-o",
                "program",
            ],
            128,
        ),
        (
            vec![
                "build",
                "main.cl",
                "--release",
                "--overflow-checks=on",
                "-o",
                "program",
            ],
            101,
        ),
    ] {
        let built = project.run(&options);
        assert!(
            built.status.success(),
            "{}",
            String::from_utf8_lossy(&built.stderr)
        );
        let result = Command::new(project.0.join("program")).output().unwrap();
        assert_eq!(result.status.code(), Some(expected));
    }
}

#[test]
fn failed_rust_compilation_preserves_existing_output() {
    let project = Project::new("fn main()->i32 {return 0;}");
    fs::write(project.0.join("program"), "previous output").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_clust"))
        .current_dir(&project.0)
        .args(["build", "main.cl", "-o", "program"])
        .env("CLUST_RUSTC", project.0.join("missing-rustc"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot start Rust compiler"));
    assert_eq!(
        fs::read_to_string(project.0.join("program")).unwrap(),
        "previous output"
    );
    assert!(!fs::read_dir(&project.0).unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .starts_with(".clust-")));
}
