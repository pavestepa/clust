fn add(total: i32, value: i32) -> i32 {
    return total + value;
}

async fn snapshot(total: Mutex<i32>) -> i32 {
    return total.read().await;
}

async fn produce(output: Sender<i32>, total: Mutex<i32>) {
    total.update(4, add).await;
    output.send(7).await.is_ok();
}

async fn main() -> i32 {
    let (output, input) = channel(1);
    var inbox = input;
    let total = mutex(0);
    let worker = spawn(produce(output, total.clone()));
    let counter = actor(1, 2, add);
    let mailbox = counter.sender();
    mailbox.send(inbox.recv().await.unwrap_or(0)).await.is_ok();
    worker.join().await;
    drop(mailbox);
    let value = snapshot(total).await;
    drop(total);
    return counter.finish().await + value;
}
