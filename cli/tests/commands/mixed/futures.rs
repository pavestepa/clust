use super::*;
#[test]
fn borrowed_future_must_not_outlive_owner() {
    let p = Project::new(
        "mod net;async fn main()->i32 {let s=String.from(\"abc\");return net.count(s).await;}",
    );
    fs::write(
        p.0.join("net.rs"),
        "pub struct Client(i32); pub fn make()->Client { Client(17) } impl Client {pub async fn fetch(&self)->i32{self.0}} pub async fn count(s:&String)->i32{s.len() as i32}",
    )
    .unwrap();
    expect(result(&p, "run", "main.cl"), 3);
    fs::write(p.0.join("main.cl"),"mod net;async fn main()->i32{let client=net.make();let pending=client.fetch();drop(pending);return client.fetch().await;}").unwrap();
    expect(result(&p, "run", "main.cl"), 17);
    fs::write(p.0.join("main.cl"),"mod net;async fn take_count(c:net.Client)->i32{return c.fetch().await;} async fn main()->i32{let c=net.make();return take_count(c).await+take_count(c).await;}").unwrap();
    expect(result(&p, "run", "main.cl"), 34);
    fs::write(p.0.join("main.cl"),"mod net;async fn main()->i32{let client=net.make();return spawn(client.fetch()).join().await;}").unwrap();
    rejects(result(&p, "check", "main.cl"), "does not live long enough");
    fs::write(p.0.join("main.cl"),"mod net; async fn main()->i32 {let s=String.from(\"abc\");let task=spawn(net.count(s));return task.join().await;}").unwrap();
    rejects(result(&p, "check", "main.cl"), "does not live long enough");
}

#[test]
fn cancelling_an_owned_future_drops_its_rust_argument() {
    let p = Project::new("mod jobs;async fn main()->i32{let h=jobs.make();let task=spawn(jobs.wait(h));task.cancel().await;return jobs.released();}");
    fs::write(
        p.0.join("jobs.rs"),
        r#"
        use std::sync::atomic::{AtomicI32,Ordering::SeqCst};
        static COUNT:AtomicI32=AtomicI32::new(0);
        pub struct Handle;
        impl Drop for Handle{fn drop(&mut self){COUNT.fetch_add(1,SeqCst);}}
        pub fn make()->Handle{Handle}
        pub async fn wait(h:Handle){std::future::pending::<()>().await;drop(h);}
        pub fn released()->i32{COUNT.load(SeqCst)}
    "#,
    )
    .unwrap();
    expect(result(&p, "run", "main.cl"), 1);
}
