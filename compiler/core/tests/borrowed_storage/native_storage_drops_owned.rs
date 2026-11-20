use super::*;

#[test]
fn native_storage_drops_owned_fields_once_and_never_drops_views() {
    use std::{collections::BTreeSet, fs, process::Command};
    let cases = CASES.iter().chain(PANICS.iter()).collect::<Vec<_>>();
    let mut rust = String::new();
    let mut traces = Vec::new();
    for (i, (body, _)) in cases.iter().copied().enumerate() {
        let p = checked(&format!("{PREFIX}{body}")).unwrap();
        let (_, events) = interpret::execute_traced(
            &p,
            p.main.unwrap(),
            vec![],
            OverflowChecks::Checked,
            100_000,
        );
        let trace = events
            .iter()
            .map(|e| {
                let hir::Value::Aggregate(fields) = &e.value else {
                    panic!()
                };
                let hir::Value::Integer(n) = fields[0] else {
                    panic!()
                };
                format!("DROP:{}:{}", e.entry, n.signed_value().unwrap())
            })
            .collect::<Vec<_>>();
        traces.push(trace);
        let emitted = clust_core::rust::emit(&p, OverflowChecks::Checked)
            .unwrap()
            .replace("fn main()", "pub fn run()");
        let emitted = emitted
            .lines()
            .map(|line| {
                if let Some(id) = line.strip_prefix("// destructor-entry ") {
                    format!("eprintln!(\"DROP:{id}:{{}}\",(*p).f0);")
                } else if let Some(id) = line.strip_prefix("// class-destructor-entry ") {
                    format!("eprintln!(\"DROP:{id}:{{}}\",(*(*heap).data.as_ptr()).f0);")
                } else if line == "// class-allocation" {
                    "eprintln!(\"ALLOC:{:p}\",heap);".into()
                } else if line == "// class-deallocation" {
                    "eprintln!(\"FREE:{:p}\",heap);".into()
                } else {
                    line.to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        rust.push_str(&format!("mod case{i} {{\n{emitted}\n}}\n"));
    }
    rust.push_str(
        "fn main() {match std::env::args().nth(1).unwrap().parse::<usize>().unwrap() {\n",
    );
    for i in 0..cases.len() {
        rust.push_str(&format!("{i}=>case{i}::run(),\n"));
    }
    rust.push_str("_=>panic!(),}}\n");
    let dir = std::env::temp_dir().join(format!("clust-storage-native-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let source = dir.join("cases.rs");
    fs::write(&source, rust).unwrap();
    for level in ["0", "3"] {
        let binary = dir.join(format!("cases-{level}"));
        let result = Command::new(std::env::var("CLUST_RUSTC").unwrap_or_else(|_| "rustc".into()))
            .arg("--edition=2021")
            .args(["-C", &format!("opt-level={level}")])
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        for (i, (_, expected)) in cases.iter().copied().enumerate() {
            let result = Command::new(&binary).arg(i.to_string()).output().unwrap();
            let stderr = String::from_utf8_lossy(&result.stderr);
            assert_eq!(
                result.status.code(),
                Some(*expected as i32),
                "case {i} O{level}: {stderr}"
            );
            let trace = stderr
                .lines()
                .filter(|s| s.starts_with("DROP:"))
                .map(str::to_owned)
                .collect::<Vec<_>>();
            assert_eq!(trace, traces[i], "case {i} O{level}");
            let mut live = BTreeSet::new();
            let mut allocations = 0;
            for line in stderr.lines() {
                if let Some(p) = line.strip_prefix("ALLOC:") {
                    assert!(live.insert(p));
                    allocations += 1;
                } else if let Some(p) = line.strip_prefix("FREE:") {
                    assert!(live.remove(p));
                }
            }
            assert!(live.is_empty());
            assert_eq!(allocations, usize::from(cases[i].0.contains("class C")));
        }
    }
    fs::remove_dir_all(dir).unwrap();
}
