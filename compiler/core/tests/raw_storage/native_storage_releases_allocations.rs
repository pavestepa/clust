use super::*;

#[test]
fn native_storage_releases_allocations_on_return_panic_and_oom() {
    let mut cases = cases();
    let grow_case = cases.len();
    cases.push((
        r#"use core.storage.{OwnedBuffer};
        use core.intrinsic.{RawStorage, allocate};
        use compiler_fixtures.record_90.{Item};
        pub fn main() -> i32 {
            var b=OwnedBuffer<Item>.make(1);
            if trusted_region() { b.push(Item {payload: allocate<i32>(1)}); }
            b.reserve(8);
            return b.len() as i32 + 6;
        }"#
        .into(),
        Some(7),
    ));
    // A panic after the transfer must release both the new storage and each
    // element's nested allocation. The native harness checks the live count.
    cases.push((
        cases[grow_case]
            .0
            .replace("return b.len() as i32 + 6;", "assert(false);return 0;"),
        None,
    ));
    cases.push((source("pub fn main() -> i32 {if trusted_region() {let first=allocate<i32>(2);let second=allocate<i32>(3);return 7;}}"),Some(7)));
    let files = Fixture::new("");
    let counter = r#"
use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
struct Counting;
static LIVE:AtomicIsize=AtomicIsize::new(0);
static ALLOCATED:AtomicUsize=AtomicUsize::new(0);
static FAIL_AFTER:AtomicIsize=AtomicIsize::new(-1);
unsafe impl std::alloc::GlobalAlloc for Counting {
 unsafe fn alloc(&self,l:std::alloc::Layout)->*mut u8 {
  if FAIL_AFTER.load(Ordering::SeqCst)>=0 && FAIL_AFTER.fetch_sub(1,Ordering::SeqCst)==0 {return std::ptr::null_mut();}
  let p=std::alloc::GlobalAlloc::alloc(&std::alloc::System,l);
  if !p.is_null() {LIVE.fetch_add(1,Ordering::SeqCst);ALLOCATED.fetch_add(1,Ordering::SeqCst);}
  p
 }
 unsafe fn dealloc(&self,p:*mut u8,l:std::alloc::Layout) {
  LIVE.fetch_sub(1,Ordering::SeqCst);std::alloc::GlobalAlloc::dealloc(&std::alloc::System,p,l)
 }
}
#[global_allocator] static ALLOCATOR:Counting=Counting;
"#;
    for mode in [OverflowChecks::Checked, OverflowChecks::Wrapping] {
        let mut generated = counter.to_string();
        for (id, (source, expected)) in cases.iter().enumerate() {
            let fixture = Fixture::new(source);
            let p = fixture.checked().unwrap();
            // Keep allocation sites observable for fault injection: Rust may
            // otherwise elide unused allocations even with a counting allocator.
            let code = rust::emit(&p, mode)
                .unwrap()
                .replace(
                    "std::alloc::alloc(layout)",
                    "std::hint::black_box(std::alloc::alloc(layout))",
                )
                .replace("fn main() {", "fn unused_main() {");
            generated.push_str(&format!("mod case{id} {{\n{code}\n"));
            generated.push_str(&format!(
                r#"
pub fn check(fail:isize) {{
 let before=super::LIVE.load(super::Ordering::SeqCst);
 let allocated=super::ALLOCATED.load(super::Ordering::SeqCst);
 super::FAIL_AFTER.store(fail,super::Ordering::SeqCst);
 let result=unsafe {{f{}({})}};
 super::FAIL_AFTER.store(-1,super::Ordering::SeqCst);
 let after=super::LIVE.load(super::Ordering::SeqCst);
 let allocated_after=super::ALLOCATED.load(super::Ordering::SeqCst);
 assert_eq!(before,after,"allocation leak in case {id}");
 if fail>=0 {{assert!(result.is_err(),"expected OOM case {id}, fail={{fail}}");}} else {{ {} }}
 {}
}}
}}
"#,
                p.main.unwrap(),
                if code.contains("struct Context") {
                    "&mut Context{clock:0,panicking:false,weak:std::collections::BTreeMap::new()}"
                } else {
                    ""
                },
                expected.map_or(
                    format!("assert!(result.is_err(),\"expected panic case {id}\");"),
                    |n| format!("assert_eq!(result.unwrap(),{n});")
                ),
                if id == 11 || id == 12 {
                    "assert_eq!(allocated,allocated_after);"
                } else {
                    ""
                }
            ));
        }
        generated.push_str("fn main(){\n");
        for id in 0..cases.len() {
            generated.push_str(&format!("case{id}::check(-1);\n"));
        }
        generated.push_str(&format!("case{grow_case}::check(2);\n"));
        generated.push_str(&format!(
            "case{}::check(0);case{}::check(1);}}",
            cases.len() - 1,
            cases.len() - 1
        ));
        fs::write(files.0.join("native.rs"), generated).unwrap();
        for opt in ["opt-level=0", "opt-level=3"] {
            let output =
                Command::new(std::env::var("CLUST_RUSTC").unwrap_or_else(|_| "rustc".into()))
                    .args(["--edition=2021", "-C", opt])
                    .arg(files.0.join("native.rs"))
                    .arg("-o")
                    .arg(files.0.join("native"))
                    .output()
                    .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let output = Command::new(files.0.join("native")).output().unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
}
