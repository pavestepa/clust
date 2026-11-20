use clust_runtime::tokio_support::{actor, mutex, spawn};
use std::{sync::mpsc, thread, time::Duration};

#[test]
fn worker_threads_make_progress_concurrently() {
    let rt = clust_tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    rt.block_on(async {
        let (started, ready) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let first = spawn(async move {
            started.send(thread::current().id()).unwrap();
            gate.recv_timeout(Duration::from_secs(10)).unwrap();
        });
        let waiting = ready.recv_timeout(Duration::from_secs(10)).unwrap();
        let second = spawn(async move {
            assert_ne!(waiting, thread::current().id());
            release.send(()).unwrap();
        });
        second.join().await;
        first.join().await;
    });
}

#[test]
fn concurrent_updates_are_exclusive_and_actor_drains_in_order() {
    let rt = clust_tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    rt.block_on(async {
        let total = mutex(0);
        let mut tasks = Vec::new();
        for _ in 0..8 {
            let shared = total.clone();
            tasks.push(spawn(async move {
                for _ in 0..128 {
                    shared.update(1, |a, b| a + b).await;
                }
            }));
        }
        for task in tasks {
            task.join().await;
        }
        assert_eq!(total.read().await, 1024);
        let digits = actor(0, 1, |a, b| 10 * a + b);
        let sender = digits.sender();
        for digit in 1..=3 {
            sender.send(digit).await.unwrap();
        }
        drop(sender);
        assert_eq!(digits.finish().await, 123);
    });
}
