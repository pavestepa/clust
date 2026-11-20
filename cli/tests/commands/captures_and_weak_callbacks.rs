use super::*;

#[test]
fn captures_and_weak_callbacks_enforce_access_and_escape_rules() {
    for (source, expected) in [
        ("fn main()->i32 {var count=0;var next=fn()->i32 {count+=1;return count;};count+=10;return next();}", "E0381"),
        ("fn main()->i32 {let count=0;let next=fn()->i32 {count+=1;return count;};return next();}", "E0405"),
        ("interface Read {call(self)->i32;} class Item {value:i32;new() {self.value=1;} pub read(self)->i32 {return self.value;}} fn make(item:Item)->dyn Read {return fn()->i32 {return item.read();};} fn main()->i32 {let item=Item.new();let callback=make(item);return callback()+item.read();}", "E038"),
        ("use core.weak.{Weak};use core.vec.{Vec};class Item {new() {}} fn consume(value:Item)->i32 {var values=Vec<Item>.new();values.push(value);return 1;} fn main()->i32 {let item=Item.new();let handle=Weak<Item>.new(item);handle.with<i32>(consume);return 0;}", "E0404"),
        ("use core.weak.{Weak};class Item {pub value:i32;new() {self.value=0;}} fn change(var value:Item)->i32 {value.value+=1;return value.value;} fn main()->i32 {let item=Item.new();let handle=Weak<Item>.new(item);handle.with<i32>(change);return 0;}", "E0320"),
    ] {
        let project = Project::new(source);
        let result = Command::new(env!("CARGO_BIN_EXE_clust"))
            .current_dir(&project.0)
            .args(["check", "main.cl"])
            .output().unwrap();
        let diagnostics = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success(), "invalid program accepted: {source}");
        assert!(diagnostics.contains(expected), "{source}: {diagnostics}");
        assert!(!diagnostics.contains("source 922337"), "synthetic source location leaked: {diagnostics}");
    }
}

#[test]
fn tokio_programs_transfer_ownership_and_close_channels() {
    let project = Project::new(include_str!("../../../examples/clust_async/main.cl"));
    let output = project.run(&["run", "main.cl", "--release"]);
    assert_eq!(
        output.status.code(),
        Some(12),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let source = "async fn wait(input: Receiver<i32>, output: Sender<i32>) {
        var inbox = input;
        output.send(inbox.recv().await.unwrap_or(0)).await.is_ok();
    }
    async fn main() -> i32 {
        let (hold, input) = channel(1);
        let (output, result) = channel(1);
        var inbox = result;
        let worker = spawn(wait(input, output));
        worker.cancel().await;
        return inbox.recv().await.unwrap_or(9);
    }";
    fs::write(project.0.join("main.cl"), source).unwrap();
    let output = project.run(&["run", "main.cl"]);
    assert_eq!(
        output.status.code(),
        Some(9),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(
        project.0.join("main.cl"),
        source.replace(
            "worker.cancel().await;",
            "drop(input); worker.cancel().await;",
        ),
    )
    .unwrap();
    let output = project.run(&["check", "main.cl"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("moved value"));
}

#[test]
fn async_call_contracts_follow_the_future_lifetime() {
    let declarations = "
        async fn read(value: Mutex<i32>) -> i32 { return value.read().await; }
        async fn forward(value: Mutex<i32>) -> i32 { return read(value).await; }
        async fn receive(var inbox: Receiver<i32>) -> i32 {
            return inbox.recv().await.unwrap_or(0);
        }
        async fn bump(var n: i32, value: Mutex<i32>) { n += read(value).await; }
        async fn consume(value: Mutex<i32>) { drop(value); }
    ";
    let main = "
        let value = mutex(3);
        let future = forward(value);
        var n = future.await;
        bump(n, value).await;
        let (tx, rx) = channel(2);
        var inbox = rx;
        tx.send(1).await.is_ok();
        tx.send(2).await.is_ok();
        let pending = receive(inbox);
        drop(pending);
        return n + receive(inbox).await + receive(inbox).await + spawn(read(value)).join().await;
    ";
    let project = Project::new("");
    let write = |body: &str| {
        fs::write(
            project.0.join("main.cl"),
            format!("{declarations} async fn main() -> i32 {{ {body} }}"),
        )
        .unwrap()
    };
    write(main);
    let output = project.run(&["run", "main.cl"]);
    assert_eq!(
        output.status.code(),
        Some(12),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for (body,message) in [
        ("let x=mutex(3); let f=read(x); drop(x); return f.await;", "borrow"),
        ("let x=mutex(3); consume(x).await; return read(x).await;", "moved"),
        ("let (tx,rx)=channel(1); var r=rx; let f=receive(r); let g=receive(r); return f.await+g.await;", "mutable"),
        ("let (tx,rx)=channel(1); var r=rx; return spawn(receive(r)).join().await;", "lifetime"),
    ]{
        write(body);
        let output=project.run(&["check","main.cl"]);
        let error=String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success() && (error.contains(message)||error.contains("does not live long enough")), "{body}: {error}");
    }
}
