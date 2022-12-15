use tokio::task::JoinHandle;

pub type Future<T> = JoinHandle<T>;
