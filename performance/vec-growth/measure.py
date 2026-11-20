"""Measure generated native programs. Pass the Clust executable as argv[1]."""
import os
from pathlib import Path
import statistics
import subprocess
import sys
import tempfile

COUNTER = """
struct Counter;
static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
#[global_allocator] static ALLOC: Counter = Counter;
unsafe impl std::alloc::GlobalAlloc for Counter {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        std::alloc::System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        std::alloc::System.dealloc(p, l)
    }
}
"""
with tempfile.TemporaryDirectory() as directory:
    root = Path(directory)
    for kind, value in [('u8', 'n as u8'), ('Item', 'Item.new(n)')]:
        source = root / 'main.cl'
        source.write_text(f'''use core.vec.{{Vec}};
class Item {{ pub n:i32; new(n:i32) {{self.n=n;}} drop() {{assert(self.n>=0);}} }}
fn main()->i32 {{
    var v=Vec<{kind}>.new(); var n=0;
    while n<32768 {{v.push({value}); n+=1;}}
    assert(v.len()==32768);
    return 0;
}}
''')
        rust = subprocess.check_output([sys.argv[1], 'emit-rust', str(source), '--release'], text=True)
        marker = 'fn main()'
        i = rust.rindex(marker)
        # The generated entry calls process::exit. Measure before its final exit.
        entry = rust[i:].replace('{', '{let start=std::time::Instant::now();let before=CALLS.load(std::sync::atomic::Ordering::Relaxed);', 1)
        marker = 'std::process::exit('
        assert marker in entry
        entry = entry.replace(marker, 'eprintln!("{} {}",CALLS.load(std::sync::atomic::Ordering::Relaxed)-before,start.elapsed().as_nanos());std::process::exit(')
        path = root / 'bench.rs'
        path.write_text(rust[:i] + COUNTER + entry)
        exe = root / 'bench'
        subprocess.run([os.getenv('CLUST_RUSTC','rustc'), '-Awarnings','-O',str(path),'-o',str(exe)],check=True)
        samples = [list(map(int, subprocess.run([str(exe)],check=True,capture_output=True,text=True).stderr.split())) for _ in range(9)]
        print(kind, 'allocation calls', statistics.median(s[0] for s in samples), 'median ns', statistics.median(s[1] for s in samples),flush=True)
