use std::future::Future;

/// This is a trait that abstracts a simple Async Runtime (e.g. Tokio Runtime) that can schedule
/// tasks and observe their completion.
pub trait AsyncRuntimeHandle {
    /// The error returned when a spawned task does not complete successfully.
    type JoinError;

    /// Spawns a task with the runtime.
    fn spawn<F>(&self, future: F) -> impl Future<Output = Result<F::Output, Self::JoinError>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    /// Blocks the current thread until the task has been completed
    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;
}

impl AsyncRuntimeHandle for tokio::runtime::Handle {
    type JoinError = tokio::task::JoinError;

    fn spawn<F>(&self, future: F) -> impl Future<Output = Result<F::Output, Self::JoinError>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::runtime::Handle::spawn(self, future)
    }

    fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::runtime::Handle::block_on(self, future)
    }
}
