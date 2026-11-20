use clust_tokio as tokio;
use std::{future::Future, sync::Arc};
use tokio::sync::{mpsc, Mutex as TokioMutex};
pub type Sender<T> = mpsc::Sender<T>;
pub type Receiver<T> = mpsc::Receiver<T>;
pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    mpsc::channel(capacity)
}
pub struct Task<T>(tokio::task::JoinHandle<T>);
impl<T> Drop for Task<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}
impl<T> Task<T> {
    pub async fn join(mut self) -> T {
        (&mut self.0).await.expect("task failed")
    }
    pub async fn cancel(mut self) {
        self.0.abort();
        let _ = (&mut self.0).await;
    }
}
pub fn spawn<F: Future + Send + 'static>(future: F) -> Task<F::Output>
where
    F::Output: Send + 'static,
{
    Task(tokio::spawn(future))
}
pub struct Mutex<T>(Arc<TokioMutex<T>>);
impl<T> Clone for Mutex<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
pub fn mutex<T>(value: T) -> Mutex<T> {
    Mutex(Arc::new(TokioMutex::new(value)))
}
impl<T: Copy> Mutex<T> {
    pub async fn read(&self) -> T {
        *self.0.lock().await
    }
    pub async fn update<M>(&self, message: M, f: fn(T, M) -> T) {
        let mut guard = self.0.lock().await;
        *guard = f(*guard, message);
    }
}
pub struct Actor<S, M> {
    mailbox: Sender<M>,
    task: Task<S>,
}
pub fn actor<S: Send + 'static, M: Send + 'static>(
    state: S,
    capacity: usize,
    handler: fn(S, M) -> S,
) -> Actor<S, M> {
    let (mailbox, mut inbox) = channel(capacity);
    let task = spawn(async move {
        let mut state = state;
        while let Some(message) = inbox.recv().await {
            state = handler(state, message);
        }
        state
    });
    Actor { mailbox, task }
}
impl<S, M> Actor<S, M> {
    pub fn sender(&self) -> Sender<M> {
        self.mailbox.clone()
    }
    pub async fn finish(self) -> S {
        drop(self.mailbox);
        self.task.join().await
    }
}
pub trait ExitStatus {
    fn code(self) -> i32;
}
impl ExitStatus for i32 {
    fn code(self) -> i32 {
        self
    }
}
impl ExitStatus for () {
    fn code(self) -> i32 {
        0
    }
}
