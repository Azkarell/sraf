use std::sync::Arc;



#[derive(Clone)]
pub struct Runtime(Arc<tokio::runtime::Runtime>);

impl Runtime {
    pub fn new() -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        Runtime(Arc::new(rt))
    }

    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.0.spawn(future)
    }

    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: std::future::Future,
    {
        self.0.block_on(future)
    }
}